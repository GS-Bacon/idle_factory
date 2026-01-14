# I.1-I.5 入力隔離テスト 設計・実装計画

## 概要

**目的**: UI表示中にゲーム操作が効かないことを自動テストで保証する

**背景**: M3で電力UIを追加する前に、既存UIの入力隔離が確実に動作することを確認したい

## 現状分析

### シナリオテストシステム

```
WebSocket (port 9877)
├── test.get_state → { ui_state, player_position, cursor_locked }
├── test.send_input → GameAction注入
└── test.assert → 条件評価
```

### InputState設計

```rust
pub enum InputState {
    Gameplay,   // 全入力有効
    Inventory,  // インベントリ操作のみ
    MachineUI,  // 機械操作のみ
    Command,    // テキスト入力のみ
    Paused,     // クリックのみ
}

impl InputState {
    pub fn allows_movement(self) -> bool {
        matches!(self, InputState::Gameplay)  // Gameplay以外はfalse
    }
}
```

**既に実装済み**: `src/systems/player.rs:208` で `allows_movement()` チェック

### 課題

| 課題 | 詳細 | 対策 |
|------|------|------|
| 位置比較 | 初期位置を記録して比較する機能がない | run-scenario.jsで変数サポート |
| 機械UI | I.3, I.5は機械の存在が前提 | test.spawn_machine API追加 or スキップ |
| 位置変化検出 | 許容誤差0.01での比較が既存 | そのまま使用可能 |

---

## タスク詳細設計

### I.1: インベントリ中の移動キー無効

**テストフロー**:
```
1. get_state → initial_position記録
2. send_input: ToggleInventory
3. wait: 100ms
4. assert: ui_state == Inventory
5. send_input: MoveForward
6. wait: 500ms  # 移動が発生する十分な時間
7. get_state
8. assert: player_position == initial_position
9. send_input: ToggleInventory (閉じる)
```

**必要な拡張**:
- `run-scenario.js`: `save_state` アクションで変数保存
- または `assert_position_unchanged` 専用アクション

**実装案A: run-scenario.js拡張**

```javascript
// 新アクション: compare_position
case 'compare_position':
    const current = await send('test.get_state', {});
    const initial = variables._lastState;
    const tolerance = params.tolerance || 0.1;
    const unchanged =
        Math.abs(current.player_position[0] - initial.player_position[0]) < tolerance &&
        Math.abs(current.player_position[1] - initial.player_position[1]) < tolerance &&
        Math.abs(current.player_position[2] - initial.player_position[2]) < tolerance;
    if (unchanged === (params.expect === 'unchanged')) {
        console.log(`Position check: ${params.expect} ✓`);
        passed++;
    } else {
        console.log(`Position check: expected ${params.expect}, got ${unchanged ? 'unchanged' : 'changed'} ✗`);
        failed++;
    }
    break;
```

**シナリオTOML**:

```toml
# tests/scenarios/i1_inventory_move_isolation.toml
name = "I.1: インベントリ中の移動キー無効"
description = "インベントリを開いている間、WASDで移動しないことを確認"

[[steps]]
action = "get_state"  # _lastStateに保存

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == Inventory" }

[[steps]]
action = "send_input"
params = { action = "MoveForward" }

[[steps]]
action = "wait"
params = { ms = 500 }

[[steps]]
action = "compare_position"
params = { expect = "unchanged", tolerance = 0.1 }

# 後片付け
[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }
```

---

### I.2: ポーズ中のEキー・移動キー無効

**テストフロー**:
```
1. get_state → initial_position記録
2. send_input: TogglePause
3. wait: 100ms
4. assert: ui_state == PauseMenu
5. send_input: ToggleInventory (Eキー)
6. wait: 100ms
7. assert: ui_state == PauseMenu  # インベントリは開かない
8. send_input: MoveForward
9. wait: 500ms
10. compare_position: unchanged
11. send_input: TogglePause (閉じる)
```

**シナリオTOML**:

```toml
# tests/scenarios/i2_pause_input_isolation.toml
name = "I.2: ポーズ中の入力隔離"
description = "ポーズ中にEキーでインベントリが開かず、移動もしない"

[[steps]]
action = "get_state"

[[steps]]
action = "send_input"
params = { action = "TogglePause" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == PauseMenu" }

# Eキーでインベントリが開かないことを確認
[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == PauseMenu" }

# 移動しないことを確認
[[steps]]
action = "send_input"
params = { action = "MoveForward" }

[[steps]]
action = "wait"
params = { ms = 500 }

[[steps]]
action = "compare_position"
params = { expect = "unchanged", tolerance = 0.1 }

# 後片付け
[[steps]]
action = "send_input"
params = { action = "TogglePause" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }
```

---

### I.3: 機械UI中のキー制御

**課題**: 機械UIを開くには、ワールドに機械があり、プレイヤーがその機械をターゲットしている必要がある

**選択肢**:

| 選択肢 | メリット | デメリット |
|--------|----------|------------|
| A: test.spawn_machine API追加 | 完全な自動テスト | 実装コスト高 |
| B: test.set_ui_state API追加 | 簡単 | 実際の遷移をテストしない |
| C: 手動で機械を配置した状態でテスト | 実装不要 | 再現性が低い |
| D: M3まで延期 | 今は実装不要 | リスクを後回し |

**推奨: 選択肢B（簡易版）+ 将来A（完全版）**

**簡易版シナリオ** (test.set_ui_state使用):

```toml
# tests/scenarios/i3_machine_ui_isolation.toml
name = "I.3: 機械UI中の移動キー無効"
description = "機械UIを開いている間、移動しないことを確認（UI状態強制設定）"

[[steps]]
action = "get_state"

[[steps]]
action = "set_ui_state"
params = { state = "MachineUI" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "send_input"
params = { action = "MoveForward" }

[[steps]]
action = "wait"
params = { ms = 500 }

[[steps]]
action = "compare_position"
params = { expect = "unchanged" }

[[steps]]
action = "set_ui_state"
params = { state = "Gameplay" }
```

**test.set_ui_state の実装**:

```rust
// src/modding/handlers/test.rs

pub fn handle_test_set_ui_state(
    request: &JsonRpcRequest,
) -> (JsonRpcResponse, Option<TestAction>) {
    let params: SetUiStateParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => return (JsonRpcResponse::error(...), None),
    };

    let action = match params.state.as_str() {
        "Gameplay" => TestAction::SetUiState(UiStateOverride::Gameplay),
        "Inventory" => TestAction::SetUiState(UiStateOverride::Inventory),
        "MachineUI" => TestAction::SetUiState(UiStateOverride::MachineUI),
        "PauseMenu" => TestAction::SetUiState(UiStateOverride::PauseMenu),
        _ => return (JsonRpcResponse::error(...), None),
    };

    (JsonRpcResponse::success(...), Some(action))
}
```

---

### I.4: ホットキー競合テスト

**テストフロー**:
```
1. ポーズ中にE → インベントリ開かない（I.2でカバー）
2. インベントリ中にESC → ポーズになる or インベントリが閉じる?
```

**仕様確認が必要**: インベントリ中のESCの挙動
- A案: インベントリを閉じてGameplayに戻る
- B案: ポーズメニューを開く

**現状の実装を確認して、その挙動をテストする**:

```toml
# tests/scenarios/i4_hotkey_conflict.toml
name = "I.4: ホットキー競合"
description = "UIが開いている状態で他のUIキーを押した時の挙動確認"

# Part 1: インベントリ中にESC
[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == Inventory" }

[[steps]]
action = "send_input"
params = { action = "TogglePause" }

[[steps]]
action = "wait"
params = { ms = 100 }

# 期待: Gameplayに戻る（ESCでUIを閉じる）
[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

# Part 2: ポーズ中にE（I.2でカバー済み、省略可）
```

---

### I.5: 機械削除時のUI復帰

**課題**: 機械の削除をシミュレートする必要がある

**選択肢**:
| 選択肢 | 実装コスト | 信頼性 |
|--------|-----------|--------|
| A: test.despawn_entity API | 中 | 高 |
| B: ユニットテストで確認 | 低 | 中 |
| C: 手動テスト | 0 | 低 |

**推奨: 選択肢B（ユニットテスト）**

```rust
// src/systems/player.rs or src/machines/generic.rs

#[cfg(test)]
mod tests {
    #[test]
    fn test_machine_despawn_closes_ui() {
        // App setup
        let mut app = App::new();
        // ... setup plugins

        // Spawn machine
        let machine = app.world_mut().spawn(MachineBundle::new(...)).id();

        // Open machine UI
        app.world_mut().resource_mut::<InteractingMachine>().0 = Some(machine);

        // Verify UI state
        assert_eq!(get_input_state(&app), InputState::MachineUI);

        // Despawn machine
        app.world_mut().despawn(machine);
        app.update();

        // Verify UI closed
        assert!(app.world().resource::<InteractingMachine>().0.is_none());
        assert_eq!(get_input_state(&app), InputState::Gameplay);
    }
}
```

---

## 実装計画

### Phase 1: テストランナー拡張 (30分)

1. `run-scenario.js` に `compare_position` アクション追加
2. 既存テストが壊れないことを確認

### Phase 2: 基本テスト作成 (1時間)

1. I.1: `i1_inventory_move_isolation.toml`
2. I.2: `i2_pause_input_isolation.toml`
3. I.4: `i4_hotkey_conflict.toml`
4. 全テスト実行して結果確認

### Phase 3: 機械UI関連 (1時間、オプション)

1. `test.set_ui_state` API追加
2. I.3: `i3_machine_ui_isolation.toml`
3. I.5: ユニットテスト追加

### Phase 4: CI統合 (オプション)

1. `scripts/run-all-scenarios.sh` 作成
2. GitHub Actions でシナリオテスト実行

---

## 完了条件

| タスク | 完了条件 |
|--------|----------|
| I.1 | `node scripts/run-scenario.js tests/scenarios/i1_*.toml` パス |
| I.2 | `node scripts/run-scenario.js tests/scenarios/i2_*.toml` パス |
| I.3 | `node scripts/run-scenario.js tests/scenarios/i3_*.toml` パス or ユニットテスト |
| I.4 | `node scripts/run-scenario.js tests/scenarios/i4_*.toml` パス |
| I.5 | ユニットテストでカバー |

---

## リスクと代替案

| リスク | 影響 | 代替案 |
|--------|------|--------|
| MoveForward入力が機能しない | テストが無意味 | キー入力のトレース追加 |
| 位置比較の許容誤差が不適切 | 誤判定 | toleranceパラメータ調整 |
| 機械UIテストの複雑化 | 実装遅延 | M3まで延期 |

---

## 依存関係

```
Phase 1 (compare_position)
    ↓
Phase 2 (I.1, I.2, I.4)
    ↓
Phase 3 (I.3, I.5) ← オプション
```

---

## Geminiレビュー結果 (2026-01-11)

### 評価

- **設計の妥当性**: ✅ 各テストは目的を達成できる
- **Phase分け**: ✅ 適切。基本UIから着手し複雑なものは後回し
- **test.set_ui_state案**: ✅ 入力隔離検証には有効（遷移ロジックは範囲外）
- **I.5ユニットテスト案**: ✅ 妥当。閉じたシステム内のロジックはユニットテスト向き

### 追加すべきテストケース

| 優先度 | ケース | 理由 |
|--------|--------|------|
| 高 | 入力の同時発生 | UIを開くキーと移動キーが同フレームで押された場合 |
| 中 | 高速な状態遷移 | UIを開いてすぐ閉じる・別UIへ遷移時の競合 |
| 中 | セーブ＆ロード堅牢性 | UI表示中にセーブ→ロード後の状態復元 |
| 低 | ウィンドウフォーカス | フォーカス喪失時の入力状態復帰 |
| 低 | キーバインド変更 | 設定変更後の追従 |

### 技術的注意点

1. **compare_position**
   - 許容誤差（tolerance）は必須
   - wait時間が短すぎると物理エンジン更新が間に合わない
   - → 500ms以上を推奨

2. **test.set_ui_state**
   - `InteractingMachine`等の関連リソースとの不整合リスク
   - テスト後のクリーンアップ必須

3. **WebSocket入力経路**
   - OSからのハードウェア入力経路は通らない
   - ゲームロジック以降は共通経路 → InputStateフィルタは検証可能

### 実装計画への反映

- I.4に「同フレーム入力」テストを追加検討
- compare_positionのwait時間を500ms以上に設定（既に設計済み）
- Phase 2完了後に「高速状態遷移」テストを検討

---

*作成日: 2026-01-11*
*Geminiレビュー: 2026-01-11*
