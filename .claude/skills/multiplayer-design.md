# Multiplayer Design Skill

マルチプレイヤー機能の設計・実装を支援するスキルです。

## 参照ドキュメント

- `.specify/specs/multiplayer-networking-best-practices.md`
- `.specify/specs/multiplayer-networking-antipatterns.md`
- `.specify/specs/security-anticheat-best-practices.md`
- `.specify/specs/security-anticheat-antipatterns.md`

---

## アーキテクチャ決定

### 推奨: サーバー権威モデル

```
クライアント → 入力送信 → サーバー
                            ↓
                      バリデーション
                            ↓
                      ゲームロジック実行
                            ↓
クライアント ← 状態同期 ← サーバー
```

### サーバーが管理すべきデータ

| データ | 必須 |
|--------|------|
| プレイヤー位置 | ✓ |
| HP/リソース | ✓ |
| ダメージ計算 | ✓ |
| 進行状態 | ✓ |
| インベントリ | ✓ |

### クライアント権威可能

| データ | 理由 |
|--------|------|
| コスメティック | バランスに影響なし |
| ローカルアニメーション | 見た目のみ |
| UIフィードバック | 即時性重視 |

---

## Bevy向けライブラリ選択

### Lightyear（推奨）

```toml
[dependencies]
lightyear = "0.x"
```

**特徴:**
- 予測/ロールバック内蔵
- WASM対応
- 帯域制限機能

### Renet/bevy_renet（シンプル）

```toml
[dependencies]
bevy_renet = "0.x"
```

**特徴:**
- 低レベル、柔軟
- 学習コスト低
- 自作予測が必要

---

## 実装パターン

### N1. サーバー権威

```rust
fn process_client_input(
    input: PlayerInput,
    client_id: ClientId,
) -> Result<(), ValidationError> {
    // 1. バリデーション
    validate_input(&input)?;

    // 2. サーバーで処理
    let result = apply_input(input);

    // 3. 結果を送信
    broadcast_state(result);
    Ok(())
}
```

### N2. クライアント予測

```rust
fn client_predict(input: &PlayerInput) {
    // ローカルで即時適用
    apply_input_locally(input);

    // サーバーに送信（シーケンス番号付き）
    send_to_server(seq_num, input);
    pending_inputs.push((seq_num, input.clone()));
}

fn reconcile(server_state: &State, last_seq: u64) {
    // サーバー状態を受け入れ
    current_state = server_state.clone();

    // 未処理の入力を再適用
    for (seq, input) in &pending_inputs {
        if *seq > last_seq {
            apply_input_locally(input);
        }
    }
}
```

### N3. 帯域最適化

```rust
// 差分のみ送信
fn send_delta(last: &State, current: &State) {
    let delta = calculate_diff(last, current);
    if !delta.is_empty() {
        send(delta);
    }
}

// Interest Management
fn get_relevant_entities(player_pos: Vec3) -> Vec<Entity> {
    entities.iter()
        .filter(|e| e.pos.distance(player_pos) < SYNC_RADIUS)
        .collect()
}
```

---

## セキュリティチェックリスト

### 必須

- [ ] サーバー権威アーキテクチャ
- [ ] 全入力のバリデーション
- [ ] レートリミティング
- [ ] セーブデータ暗号化

### 推奨

- [ ] 統計的異常検知
- [ ] 詳細ログ
- [ ] 段階的BAN対応

---

## 工場ゲーム特有の考慮

### 大量エンティティ同期

```rust
// イベントベースで同期
enum MachineEvent {
    Started(EntityId),
    Stopped(EntityId),
    Produced(EntityId, ItemId),
}

// 状態変化時のみ送信
fn on_machine_change(event: MachineEvent) {
    broadcast_event(event);
}
```

### ボクセル変更

```rust
// チャンク単位でバッチ
struct ChunkUpdate {
    chunk_pos: IVec3,
    changes: Vec<(IVec3, BlockType)>,
}
```

---

## アンチパターン回避

| 絶対に避ける | 対策 |
|-------------|------|
| クライアント信頼 | サーバー権威 |
| 同期ズレ放置 | 定期フル同期 |
| 入力バッファ暴走 | バッファ上限 |
| 再接続未考慮 | セッション維持 |

---

*このスキルはマルチプレイヤー実装の品質を確保するためのガイドです。*
