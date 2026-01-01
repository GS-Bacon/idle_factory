# アーキテクチャ

## モジュール構成

```
src/
├── main.rs              # エントリポイント、プラグイン登録、テスト
├── block_type.rs        # ブロック種別定義
├── constants.rs         # 定数（BLOCK_SIZE, HOTBAR_SLOTS等）
├── game_spec.rs         # ゲーム仕様（初期装備、クエスト定義）
│
├── components/          # ECSコンポーネント定義
│   ├── mod.rs
│   ├── machines.rs      # Miner, Conveyor, Furnace, Crusher, Direction
│   ├── ui.rs            # UI関連コンポーネント
│   ├── input.rs         # InputState, InputStateResources
│   └── player.rs        # CursorLockState
│
├── events/              # ゲームイベント（マルチプレイ準備）
│   └── mod.rs
│
├── player/              # プレイヤー関連
│   ├── mod.rs
│   └── inventory.rs     # Inventory構造体
│
├── world/               # ワールド・チャンク
│   ├── mod.rs
│   ├── chunk.rs         # ChunkData, ChunkMesh
│   └── terrain.rs       # 地形生成
│
├── plugins/             # Bevyプラグイン
│   ├── mod.rs
│   └── machines.rs      # MachineSystemsPlugin
│
├── systems/             # Bevyシステム（Update毎に実行）
│   ├── mod.rs
│   ├── block_operations.rs  # ブロック設置・破壊
│   ├── chunk.rs             # チャンク生成・アンロード
│   ├── player.rs            # 移動・視点操作
│   ├── furnace.rs           # 精錬炉システム
│   ├── crusher.rs           # 粉砕機システム
│   ├── miner.rs             # 採掘機システム
│   ├── conveyor.rs          # コンベアシステム
│   ├── hotbar.rs            # ホットバーUI
│   ├── inventory_ui.rs      # インベントリUI
│   ├── command_ui.rs        # コマンド入力
│   ├── debug_ui.rs          # デバッグHUD
│   ├── quest.rs             # クエスト・納品
│   ├── targeting.rs         # ターゲットブロック・ハイライト
│   └── save_systems.rs      # セーブ・ロード
│
├── setup/               # 起動時セットアップ（Startup）
│   ├── mod.rs
│   ├── lighting.rs      # ライティング設定
│   ├── player.rs        # プレイヤーエンティティ生成
│   ├── ui_setup.rs      # UI生成（ホットバー、機械UI等）
│   └── initial_items.rs # 初期装備・配置
│
├── ui/                  # UIコンポーネント
│   ├── mod.rs
│   └── components.rs
│
├── meshes/              # メッシュ生成
│   └── mod.rs
│
├── utils/               # ユーティリティ
│   └── mod.rs           # ray_aabb_intersection等
│
├── save/                # セーブデータ
│   └── mod.rs
│
└── logging/             # ロギング
    └── mod.rs
```

## 依存関係

```
main.rs
  ├── components (全システムから参照)
  ├── events (ゲームイベント)
  ├── player/inventory
  ├── world/chunk, terrain
  ├── machines/components, conveyor
  ├── systems/* (全システム)
  ├── setup/* (起動時)
  └── ui/components
```

## 分割ルール

### 1. 行数目安
- 1ファイル500行以下を目指す
- 800行を超えたら分割を検討
- 1000行以上は即座に分割

### 2. 分割基準
| 基準 | 例 |
|------|-----|
| 機械種別 | furnace.rs, crusher.rs, miner.rs |
| 機能 | hotbar.rs, inventory_ui.rs |
| ライフサイクル | setup/ (Startup), systems/ (Update) |

### 3. モジュール構成パターン

**ディレクトリモジュール** (複数ファイル):
```
machines/
├── mod.rs          # pub use で再エクスポート
├── components.rs   # 共通コンポーネント
├── furnace.rs
└── crusher.rs
```

**単一ファイルモジュール** (小規模):
```
mod utils;  // utils.rs
```

### 4. インポートルール

```rust
// 良い例: 明示的なインポート
use crate::components::{Furnace, FurnaceSlot};
use crate::player::Inventory;

// 避ける: ワイルドカード（テスト以外）
use crate::components::*;  // テストでは許可
```

### 5. 公開範囲

| スコープ | 使い分け |
|----------|----------|
| `pub` | クレート外から使用（WASM連携等） |
| `pub(crate)` | クレート内のみ |
| なし | モジュール内のみ |

## システム登録パターン

```rust
// main.rs
app
    .add_systems(Startup, (setup_lighting, setup_player, setup_ui))
    .add_systems(Update, (
        // グループ化して登録
        player_move,
        player_look,
    ))
    .add_systems(Update, (
        furnace_interact,
        furnace_smelting,
        furnace_output,
    ));
```

## テストの配置

| テスト種別 | 場所 |
|-----------|------|
| ユニットテスト | 各モジュール内 `#[cfg(test)]` |
| 統合テスト | `src/main.rs` の `mod tests` |
| E2Eテスト | `tests/e2e_test.rs` |
