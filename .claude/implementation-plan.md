# 実装計画

> **対象**: AI（タスク実行時の参照用）
> **人間向け**: `.specify/roadmap.md`（進捗確認、全体像）
> **設計詳細**: `.claude/architecture.md`

## ドキュメント役割分担

| ファイル | 対象 | 内容 |
|----------|------|------|
| `.specify/roadmap.md` | 人間 | マイルストーン概要、完了条件（簡潔） |
| **このファイル** | AI | タスク詳細、設計コード例、シナリオテスト例 |
| `.claude/architecture.md` | 両方 | 機能設計骨格、拡張ポイント |

---

## 現状サマリー (2026-01-11)

| 項目 | 値 |
|------|-----|
| バージョン | **0.3.136** |
| コード行数 | **29,253行** |
| テスト | **613件** 通過 |
| Clippy警告 | **0件** |
| 現在のマイルストーン | **M2.5: UIプレビュー & バグ修正** |

---

## マイルストーン一覧

| MS | 名前 | 状態 | 概要 |
|----|------|------|------|
| M1 | 基盤整備 | ✅ 完了 | Component化、動的ID、イベント |
| M2 | Core Mod基盤 | ✅ 完了 | WASM Mod対応 |
| **M2.5** | **UIプレビュー & バグ修正** | ❌ 進行中 | UI分離確認、バグ潰し |
| M3 | 電力システム | ❌ 未着手 | 発電機、電線、消費 |
| M4 | 液体・信号 | ❌ | パイプ、論理回路 |
| M4.5 | ツール・ビジュアル | ❌ | レシピエディタ、見た目強化 |
| M5 | ゲーム完成 | ❌ | 機械50種、マップ、ブループリント |
| M6 | マルチプレイ | ❌ | P2P、同期、権限 |

---

## M1: 基盤整備 ✅ 完了

<details>
<summary>完了済みタスク（クリックで展開）</summary>

| タスク | 確認方法 |
|--------|----------|
| LocalPlayer Entity化 | 47箇所で使用 |
| PlayerInventory Component化 | `Res<PlayerInventory>` 0件 |
| MachineBundle使用 | 23箇所で使用 |
| NetworkId定義 | `components/network.rs` |
| GuardedEventWriter使用 | 16箇所で使用 |
| WebSocket API (port 9877) | 18メソッド実装済み |
| InteractingMachine統合 | 旧Interacting* 0件 |
| レガシー機械削除 | 旧struct 0件 |
| パニック防止 (P.0-P.4) | フォールバック実装済み |
| セーブV2形式 | 文字列ID化完了 |
| 固定Tick導入 | FixedUpdate(20Hz) |
| 各Pluginモジュール化 | D.6-D.14全て登録済み |
| Data Mod (TOML読み込み) | 起動時ロード実装済み |
| BlockType→ItemId移行 | BlockType 0箇所、ItemId 469箇所 |

</details>

---

## M2: Core Mod基盤 ✅ 完了 (2026-01-09)

**目標**: WASMで新しいゲームロジックを追加できる

<details>
<summary>完了済みタスク（クリックで展開）</summary>

| セクション | 内容 | 状態 |
|-----------|------|------|
| W.1 | Wasmtime統合 | ✅ |
| W.2 | Mod API設計（ホスト関数） | ✅ |
| W.3 | サンプルCore Mod作成 | ✅ |
| W.4 | Mod間依存解決 | ✅ |
| W.5 | ホットリロード（開発用） | ✅ |
| W.6 | タグシステム | ✅ |
| W.7 | 特殊機械Core Mod化 | ✅ |

**実装ファイル**: `src/modding/wasm/` - 8ファイル

</details>

---

## M2.5: UIプレビュー & バグ修正

**目標**: UI・テクスチャ関連のバグを徹底的に潰してからM3へ進む

### 優先度順序

| 優先度 | セクション | 内容 | M3前に必須 |
|--------|-----------|------|-----------|
| 🔴 高 | **I** | 入力隔離テスト | ✅ 必須 |
| 🟡 中 | Q | テストAPI拡張 | 推奨 |
| 🟡 中 | T | UI要素検証テスト | 推奨 |
| 🟢 低 | U | UIプレビューモード | 任意 |
| 🟢 低 | B | バグ修正（UI表示系） | 任意 |
| 🟢 低 | R | ブロックテクスチャ | 任意 |

### U: UIプレビューモード

**背景**: テクスチャバグ調査中に沼った。UIとゲームロジックを分離して確認したい。

| タスク | 内容 | 状態 |
|--------|------|------|
| U.1 | `src/bin/ui_preview.rs` 作成 | ❌ |
| U.2 | 最小限のプラグイン構成（UI系のみ） | ❌ |
| U.3 | ダミーインベントリ・ダミー機械データ | ❌ |
| U.4 | キー操作（E=インベントリ、Esc=ポーズ等） | ❌ |
| U.5 | 背景は単色または床のみ | ❌ |

**起動方法**:
```bash
cargo run --bin ui_preview
```

**含めるもの**:
- UIPlugin（インベントリ、ポーズメニュー、クエストUI等）
- InputManagerPlugin（キー入力）
- ダミーのPlayerInventory、CurrentQuest等

**含めないもの**:
- WorldData、ChunkMeshTasks（ワールド生成）
- MachineSystemsPlugin（機械処理）
- player_move、block_break等（ゲームプレイ）

### T: UI要素検証テスト（動的IDパターン）

**背景**: 現在のシナリオテストは`ui_state`しか検証できない。どのUI要素が表示されているかを自動テストしたい。

**設計方針**: 既存の`Id<T>`パターンに従い、Mod拡張可能な動的IDで実装

| タスク | 内容 | 状態 |
|--------|------|------|
| T.1 | `UIElementId = Id<UIElementCategory>` 定義 | ❌ |
| T.2 | `UIElementSpec` + レジストリ作成 | ❌ |
| T.3 | `mods/base/ui_elements.toml` で仕様定義 | ❌ |
| T.4 | `update_ui_visibility` システム実装 | ❌ |
| T.5 | `test.get_state`に`visible_elements`追加 | ❌ |
| T.6 | シナリオテストで検証 | ❌ |

**データ構造**:
```rust
// 動的UI要素ID（Modで追加可能）
pub type UIElementId = Id<UIElementCategory>;

// 仕様定義
pub struct UIElementSpec {
    pub id: String,           // "base:hotbar", "mymod:custom_hud"
    pub name: String,
    pub show_in: Vec<String>, // ["Gameplay", "Inventory"]
}
```

**TOML定義（Single Source of Truth）**:
```toml
# mods/base/ui_elements.toml
[[ui_elements]]
id = "base:hotbar"
name = "ホットバー"
show_in = ["Gameplay", "Inventory", "Machine"]

[[ui_elements]]
id = "base:inventory_panel"
name = "インベントリパネル"
show_in = ["Inventory"]

[[ui_elements]]
id = "base:pause_menu"
name = "ポーズメニュー"
show_in = ["PauseMenu"]

[[ui_elements]]
id = "base:crosshair"
name = "照準"
show_in = ["Gameplay"]

[[ui_elements]]
id = "base:quest_tracker"
name = "クエストトラッカー"
show_in = ["Gameplay"]
```

**システム**:
```rust
fn update_ui_visibility(
    ui_state: Res<UIState>,
    registry: Res<UIElementRegistry>,
    mut query: Query<(&UIElementId, &mut Visibility)>,
) {
    let current = ui_state.current().to_string();
    for (element_id, mut vis) in &mut query {
        let spec = registry.get(element_id);
        *vis = if spec.show_in.contains(&current) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
```

**メリット**:
- Modで新UIを追加可能（`mymod:custom_hud`等）
- 表示条件もTOMLで変更可能
- 既存の`Id<T>`パターンと一貫性
- コンパイル時にtypo検出（レジストリ経由）

### Q: テストAPI拡張（修正ループ防止）

**背景**: UIバグ修正で「直りました」→「直ってない」のループに陥った。根本原因は**証拠なしで完了宣言**していたこと。

**目標**: APIでUI要素の状態を取得し、シナリオテストで検証可能にする

| タスク | 内容 | 状態 |
|--------|------|------|
| Q.1 | `test.get_ui_elements` メソッド追加 | ❌ |
| Q.2 | 各UI要素に`UiElementTag`コンポーネント追加 | ❌ |
| Q.3 | 表示/非表示・インタラクト可能状態を取得 | ❌ |
| Q.4 | シナリオテストで検証可能に | ❌ |
| Q.5 | UI遷移パターン自動抽出スクリプト | ❌ |

**API設計**:
```json
// test.get_ui_elements レスポンス
{
  "elements": [
    {
      "id": "pause_title",
      "type": "text",
      "visible": true,
      "interactable": false
    },
    {
      "id": "resume_btn",
      "type": "button",
      "visible": true,
      "interactable": true
    },
    {
      "id": "settings_btn",
      "type": "button",
      "visible": true,
      "interactable": true
    },
    {
      "id": "quit_btn",
      "type": "button",
      "visible": true,
      "interactable": true
    }
  ],
  "ui_state": "PauseMenu"
}
```

**シナリオテスト例**:
```toml
# tests/scenarios/pause_menu_isolation.toml
name = "ポーズメニューの分離テスト"
description = "ポーズメニュー中は指定ボタンのみ操作可能"

[[steps]]
action = "send_input"
params = { action = "TogglePause" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == PauseMenu" }

[[steps]]
action = "assert_ui"
params = {
  visible = ["pause_title", "resume_btn", "settings_btn", "quit_btn"],
  interactable = ["resume_btn", "settings_btn", "quit_btn"]
}

[[steps]]
# ポーズ中にインベントリキーを押しても開かない
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "assert"
params = { condition = "ui_state == PauseMenu" }
```

**UI遷移パターン自動抽出**:
```bash
# コードからUI遷移を抽出してシナリオを自動生成
./scripts/generate-ui-tests.sh

# 出力例:
# tests/scenarios/generated/
#   ui_gameplay_to_inventory.toml
#   ui_gameplay_to_pause.toml
#   ui_pause_to_settings.toml
#   ui_pause_isolation.toml  # 他のキーが効かないことを確認
```

### I: 入力隔離テスト（🔴 高優先度 - M3前に必須）

**背景**: UI表示中にゲーム操作が効いてしまう可能性がある。M3で電力UIを追加する前に、既存UIの入力隔離を確実にする。

| タスク | 内容 | 状態 |
|--------|------|------|
| I.1 | `ui_input_isolation.toml` - インベントリ中に移動キー無効確認 | ❌ |
| I.2 | `pause_input_isolation.toml` - ポーズ中にEキー・移動キー無効確認 | ❌ |
| I.3 | `machine_ui_isolation.toml` - 機械UI中のキー制御確認 | ❌ |
| I.4 | `hotkey_conflict.toml` - E+Tab, E+ESC同時押しテスト | ❌ |
| I.5 | `machine_despawn_ui.toml` - 機械削除時のUI復帰確認 | ❌ |

**シナリオテスト例（I.1）**:
```toml
# tests/scenarios/ui_input_isolation.toml
name = "UI入力隔離: インベントリ中に移動キー無効"
description = "インベントリを開いている間、WASDで移動しないことを確認"

[[steps]]
action = "get_state"  # 初期位置を記録

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
action = "assert"
params = { condition = "player_position == initial_position" }  # 移動していない
```

**シナリオテスト例（I.4）**:
```toml
# tests/scenarios/hotkey_conflict.toml
name = "ホットキー競合: E+ESC同時押し"
description = "ESCを押しながらEを押してもインベントリが開かない"

[[steps]]
action = "send_input"
params = { action = "TogglePause" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == PauseMenu" }

[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == PauseMenu" }  # インベントリは開かない
```

**完了条件**:
- [ ] 全5件のシナリオテストがパス
- [ ] `node scripts/run-scenario.js tests/scenarios/ui_input_isolation.toml` 成功
- [ ] `node scripts/run-scenario.js tests/scenarios/hotkey_conflict.toml` 成功

### B: バグ修正

| タスク | 内容 | 状態 |
|--------|------|------|
| B.1 | UIプレビューで問題を洗い出し | ❌ |
| B.2 | テクスチャ表示バグ修正 | ❌ |
| B.3 | UI配置・レイアウトバグ修正 | ❌ |

### R: ブロックテクスチャ

**背景**: ワールドのブロックは頂点カラーのみで描画されている。テクスチャを貼りたい。

**現状**:
- ブロック: `ItemDescriptor.color` → 頂点カラー（テクスチャなし）
- `BlockTextureAtlas`は定義済みだが未使用

**目標**: ブロックにテクスチャが表示される

| タスク | 内容 | 状態 |
|--------|------|------|
| R.1 | テクスチャアトラス作成（16x16 × N種類） | ❌ |
| R.2 | `items.toml`にアトラス座標を追加 | ❌ |
| R.3 | メッシュ生成にUV座標を追加 | ❌ |
| R.4 | チャンクマテリアルにアトラステクスチャ適用 | ❌ |
| R.5 | フォールバック（テクスチャなし→頂点カラー） | ❌ |

**items.toml拡張案**:
```toml
[iron_ore]
name = "鉄鉱石"
color = "#8B7355"           # フォールバック用
atlas_index = 3             # アトラス内の位置（0始まり）
```

**アトラス構成**:
```
assets/textures/block_atlas.png (256x256, 16x16が16種類)
┌──┬──┬──┬──┐
│0 │1 │2 │3 │  ← stone, dirt, grass_top, iron_ore...
├──┼──┼──┼──┤
│4 │5 │6 │7 │
...
```

**並列開発**: U/Tと並列実装可能（UI依存なし）

**将来拡張（後付け可能）**: リソースパック切り替え、複数アトラス対応

---

## M2→M3の間: リソースネットワーク設計

**⚠️ ユーザーと相談しながら進めること**

**目標**: 電力・液体・信号を統一的に扱える汎用基盤

| タスク | 内容 | 状態 |
|--------|------|------|
| N.1 | リソース種別の抽象定義 | ❌ |
| N.2 | ネットワーク接続の仕組み | ❌ |
| N.3 | ノード種別（Producer/Consumer/Storage/Conduit） | ❌ |
| N.4 | 計算方式（供給/需要/優先度） | ❌ |
| N.5 | Mod拡張ポイント | ❌ |

**設計案**:
```rust
struct ResourceType {
    name: String,
    unit: String,
    decay: bool,
    max_transfer: f32,
    // M4で追加: is_signal, propagation_delay
}
```

| 特性 | 電力・液体 | 信号 |
|------|-----------|------|
| 値の型 | 数値 | ON/OFF or 0-15 |
| 分岐時 | 分割される | コピーされる |
| 消費 | 使うと減る | 減らない |

---

## M3: 電力システム

| タスク | 内容 |
|--------|------|
| P.1 | 電力グリッド計算 |
| P.2 | 発電機（水車、石炭発電） |
| P.3 | 電線ブロック |
| P.4 | 電力消費機械 |
| P.5 | 電力UI |

---

## M4: 液体・信号

| タスク | 内容 |
|--------|------|
| F.1 | 液体スロット・パイプ |
| F.2 | ポンプ・タンク |
| S.1 | 信号ワイヤー |
| S.2 | センサー |
| S.3 | 論理ゲート |

---

## M4.5: 調整ツール & ビジュアル強化

### E: レシピエディタ（WebUI）

**目標**: 非エンジニアがブラウザからレシピ調整できる

| タスク | 内容 | 状態 |
|--------|------|------|
| E.1 | WebSocket APIにTOML読み書き追加 | ❌ |
| E.2 | 簡易HTML+JSエディタUI | ❌ |
| E.3 | バリデーション（不正レシピ検出） | ❌ |
| E.4 | ゲーム再起動なしリロード（stretch） | ❌ |

### V: ビジュアル強化

**目標**: 見た目のインパクト向上

| タスク | 内容 | 状態 |
|--------|------|------|
| V.1 | ライティング改善 | ❌ |
| V.2 | ポストプロセス（ブルーム等） | ❌ |
| V.3 | 機械アニメーション | ❌ |
| V.4 | パーティクル | ❌ |
| V.5 | サウンド | ❌ |

---

## M5: ゲーム完成

- 機械50種類以上
- レシピ100種類以上
- マップ機能
- ブループリント

---

## M6: マルチプレイ

- P2P接続
- 状態同期
- 権限管理

---

## 付録

### 新コンテンツ追加フロー

**現在（Data Mod）**:
```
1. mods/base/items.toml に追加（3行）
2. mods/base/machines.toml に追加（10行）
3. mods/base/recipes.toml に追加（3行）
4. assets/models/ に3Dモデル配置
5. 完了（Rustコード変更なし）
```

**M2完了後（Core Mod）**:
```
1. mods/my_mod/Cargo.toml 作成
2. mods/my_mod/src/lib.rs に新ロジック
3. cargo build --target wasm32-unknown-unknown
4. mods/my_mod/mod.toml で依存宣言
5. 完了（本体コード変更なし）
```

### M2完了時: 追加タスク

**API Wiki自動生成**:
- ドキュメントコメントから自動でWiki更新
- GitHub Actions で `src/modding/**` 変更時にトリガー

**自動バグ取りシステム**:
- 操作記録 & 再生
- シナリオテスト（TOML定義）
- バグ検出→GitHub Issue自動作成

### M5完了時: リポジトリ名変更

**ゲーム名: RisoFactory**（理想ファクトリー）

| タスク | 内容 |
|--------|------|
| R.1 | GitHubリポジトリ名を `riso-factory` に変更 |
| R.2 | `Cargo.toml` の `name` を `riso_factory` に変更 |
| R.3 | コード内の `idle_factory` 参照を更新 |
| R.4 | ウィンドウタイトルを「RisoFactory」に変更 |
| R.5 | README、ドキュメント更新 |

**名前の由来**:
- 日本語: 理想（りそう）+ Factory
- イタリア語: Riso = 米 & 笑い
- 「笑いの工場」というダブルミーニング

---

*最終更新: 2026-01-11*
