# アーキテクチャ

## コード規模

総計: 約18,000行

| モジュール | 行数 | 責務 |
|------------|------|------|
| systems/ | 6,800 | ゲームロジック（Update系） |
| components/ | 1,600 | ECSコンポーネント定義 |
| save/ | 1,500 | セーブ/ロード |
| ui/ | 1,400 | UI生成・操作 |
| world/ | 1,050 | チャンク・地形生成 |
| game_spec/ | 1,050 | 仕様定義（Single Source of Truth） |
| machines/ | 1,050 | 機械システム |
| setup/ | 1,000 | 起動時初期化（Startup） |
| player/ | 600 | プレイヤー・インベントリ |
| logistics/ | 560 | コンベア |
| plugins/ | 440 | Bevyプラグイン |
| debug/ | 380 | デバッグHUD・ステートダンプ |
| core/ | 360 | 純粋ロジック（Bevy非依存） |
| src/*.rs | 3,000 | main, meshes, utils, vox_loader |

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
│   ├── machines.rs      # Direction, ConveyorShape等
│   ├── ui.rs            # UI関連
│   ├── input.rs         # InputState
│   └── player.rs        # カーソル状態
│
├── systems/             # Updateシステム
│   ├── block_operations/  # 設置・破壊
│   ├── targeting/         # レイキャスト・ハイライト
│   ├── command/           # コマンド入力
│   ├── player.rs          # 移動・視点
│   ├── inventory_ui.rs    # インベントリUI
│   ├── quest.rs           # クエスト進行
│   └── ...
│
├── machines/            # 機械固有ロジック
│   ├── miner.rs
│   ├── furnace.rs
│   └── crusher.rs
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
| 物流分離 | コンベアは `logistics/` で機械と分離 |
| UI分離 | UI定義は `ui/`、システムは `systems/` |

## 分割ルール

| 基準 | 目安 |
|------|------|
| 1ファイル | 500行以下推奨、800行超で検討、1000行超で分割 |
| 機械種別 | miner.rs, furnace.rs, crusher.rs |
| ライフサイクル | setup/（Startup）, systems/（Update） |

## テスト配置

| 種別 | 場所 |
|------|------|
| ユニット | 各モジュール内 `#[cfg(test)]` |
| 統合 | `src/main.rs` の `mod tests` |
| E2E | `tests/e2e_test.rs` |
