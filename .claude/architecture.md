# アーキテクチャ

## コード規模 (2026-01-07更新)

総計: 約19,000行 (リファクタリングで-3,500行削減)

| モジュール | 行数 | 責務 |
|------------|------|------|
| systems/ | 5,500 | ゲームロジック（Update系） |
| components/ | 1,400 | ECSコンポーネント定義 |
| save/ | 1,500 | セーブ/ロード |
| ui/ | 800 | UI生成・操作（データ駆動化で削減） |
| world/ | 1,200 | チャンク・地形生成（Greedy meshing含む） |
| game_spec/ | 1,100 | 仕様定義（Single Source of Truth） |
| machines/ | 600 | 機械システム（generic.rsに統合） |
| setup/ | 1,000 | 起動時初期化（Startup） |
| player/ | 600 | プレイヤー・インベントリ |
| logistics/ | 560 | コンベア |
| plugins/ | 440 | Bevyプラグイン |
| debug/ | 380 | デバッグHUD・ステートダンプ |
| core/ | 360 | 純粋ロジック（Bevy非依存） |
| src/*.rs | 2,500 | main, meshes, utils, vox_loader |

## ディレクトリ構造

```
src/
├── main.rs              # エントリ、プラグイン登録
├── lib.rs               # ライブラリエクスポート
├── block_type.rs        # BlockType enum
├── constants.rs         # 定数
├── meshes.rs            # メッシュ生成
├── vox_loader.rs        # .voxファイル読み込み
├── utils.rs             # ユーティリティ
│
├── components/          # ECSコンポーネント
│   ├── mod.rs           # 共通型
│   ├── machines.rs      # Direction, ConveyorShape, Machine等
│   ├── ui.rs            # UI関連（InteractingMachine等）
│   ├── ui_state.rs      # UIState, UIContext, UIAction
│   ├── input.rs         # InputState（統合版）
│   └── player.rs        # カーソル状態
│
├── systems/             # Updateシステム
│   ├── block_operations/  # 設置・破壊
│   ├── targeting/         # レイキャスト・ハイライト
│   ├── command/           # コマンド入力
│   ├── player.rs          # 移動・視点
│   ├── inventory_ui.rs    # インベントリUI
│   ├── ui_navigation.rs   # UIステート管理・入力ハンドリング
│   ├── quest.rs           # クエスト進行
│   └── ...
│
├── machines/            # 機械システム（統合）
│   ├── mod.rs           # エクスポート
│   └── generic.rs       # 全機械の統一tick/UI/インタラクション
│
├── logistics/           # 物流インフラ
│   └── conveyor.rs
│
├── ui/                  # UI定義・生成
│   ├── machine_ui.rs
│   ├── storage_ui.rs
│   └── ...
│
├── setup/               # Startupシステム
│   ├── ui/              # UI初期化
│   ├── player.rs
│   └── lighting.rs
│
├── game_spec/           # 仕様定義
│   ├── mod.rs           # 定数、初期装備、クエスト
│   ├── machines.rs      # 機械仕様
│   └── recipes.rs       # レシピ定義
│
├── save/                # セーブシステム
│   ├── format.rs        # データ構造
│   └── systems.rs       # 保存/読込
│
├── world/               # ワールド
│   ├── mod.rs           # チャンク管理
│   └── terrain.rs       # 地形生成
│
├── player/              # プレイヤー
│   └── inventory.rs
│
├── core/                # 純粋ロジック（Bevy非依存）
├── plugins/             # Bevyプラグイン
├── events/              # ゲームイベント
├── debug/               # デバッグ機能
└── updater/             # 自動更新
```

## 依存関係

```
main.rs
  └── plugins/ (GamePlugin, MachineSystemsPlugin, DebugPlugin)
        ├── components/ (全システムから参照)
        ├── game_spec/ (仕様参照)
        ├── systems/ (ゲームロジック)
        ├── setup/ (初期化)
        └── ui/ (UI生成)
```

## 設計原則

| 原則 | 詳細 |
|------|------|
| ECSコンポジション | コンポーネントの組み合わせで機能を構成 |
| Single Source of Truth | 仕様は `game_spec/` に集約 |
| **データ駆動設計** | 機械は `MachineSpec` で定義、UIは自動生成 |
| 物流分離 | コンベアは `logistics/` で機械と分離 |
| UI分離 | UI定義は `ui/`、システムは `systems/` |
| **UIステート統合** | `UIState` + `UIAction` でUI操作を一元管理 |

## 分割ルール

| 基準 | 目安 |
|------|------|
| 1ファイル | 500行以下推奨、800行超で検討、1000行超で分割 |
| **機械統合** | `generic.rs` に全機械を統合（個別ファイル廃止） |
| ライフサイクル | setup/（Startup）, systems/（Update） |

## テスト配置

| 種別 | 場所 |
|------|------|
| ユニット | 各モジュール内 `#[cfg(test)]` |
| 統合 | `src/main.rs` の `mod tests` |
| E2E | `tests/e2e_test.rs` |
