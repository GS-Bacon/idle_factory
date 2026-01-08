# 実装計画

> **ロードマップ**: `.specify/roadmap.md`
> **将来設計**: `.claude/architecture-future.md`
> **移行状況**: `./scripts/migration-status.sh`

## 現状サマリー (2026-01-08)

| 項目 | 値 |
|------|-----|
| バージョン | **0.3.103** |
| コード行数 | **36,413行** |
| テスト | **397件** 通過 |
| Clippy警告 | **0件** |
| 現在のマイルストーン | **M2: Core Mod基盤** |

---

## 完了済みタスク

| タスク | 状態 | 確認方法 |
|--------|------|----------|
| LocalPlayer Entity化 | ✅ | 47箇所で使用 |
| PlayerInventory Component化 | ✅ | `Res<PlayerInventory>` 0件 |
| MachineBundle使用 | ✅ | 23箇所で使用 |
| NetworkId定義 | ✅ | `components/network.rs` |
| GuardedEventWriter使用 | ✅ | 16箇所で使用 |
| WebSocket API (port 9877) | ✅ | 18メソッド実装済み |
| InteractingMachine統合 | ✅ | 旧Interacting* 0件 |
| レガシー機械削除 | ✅ | 旧struct 0件 |
| パニック防止 (P.0-P.4) | ✅ | フォールバック実装済み |
| セーブV2形式 | ✅ | 文字列ID化完了 |
| 固定Tick導入 | ✅ | FixedUpdate(20Hz) |
| 各Pluginモジュール化 | ✅ | D.6-D.14全て登録済み |
| Data Mod (TOML読み込み) | ✅ | 起動時ロード実装済み |

---

## M2: Core Mod基盤（現在のマイルストーン）

**目標**: WASMで新しいゲームロジックを追加できる

### W.1: Wasmtime統合

| タスク | 内容 | 状態 |
|--------|------|------|
| W.1.1 | `wasmtime` クレート追加 | ❌ |
| W.1.2 | WASMランタイム初期化 | ❌ |
| W.1.3 | `.wasm` ファイル読み込み | ❌ |
| W.1.4 | エラーハンドリング | ❌ |

**参考**: [Wasmtime Embedding](https://docs.wasmtime.dev/lang-rust.html)

### W.2: Mod API設計（ホスト関数）

| タスク | 内容 | 状態 |
|--------|------|------|
| W.2.1 | ホスト関数一覧設計 | ❌ |
| W.2.2 | メモリ共有方式決定 | ❌ |
| W.2.3 | 読み取りAPI実装 | ❌ |
| W.2.4 | 書き込みAPI実装 | ❌ |
| W.2.5 | イベントフック連携 | ❌ |

**ファイル構成（カテゴリ別集約）**:
```
src/modding/
├── api/
│   ├── mod.rs         ← 全カテゴリを集約
│   ├── inventory.rs   ← インベントリ系API
│   ├── machine.rs     ← 機械系API
│   ├── world.rs       ← ワールド・ブロック系API
│   ├── event.rs       ← イベント購読系API
│   └── log.rs         ← ログ出力API
├── hooks/
│   ├── mod.rs         ← 全フックを集約
│   ├── machine.rs     ← 機械関連フック
│   ├── inventory.rs   ← インベントリ関連フック
│   └── world.rs       ← ワールド関連フック
└── registry.rs        ← 共通の登録機構
```

**設計方針**:
- 最初からカテゴリ別ファイル構造で開始
- API/フック追加 = 該当カテゴリに追記
- 新カテゴリ追加 = 新ファイル + mod.rsに1行
- マクロ方式は採用しない（デバッグ難易度を避ける）

**将来追加予定のカテゴリ**:
- `power.rs` - M3（電力）で追加
- `fluid.rs` - M4（液体）で追加
- `signal.rs` - M4（信号）で追加
- `physics.rs` - 簡易物理API（Create風Mod用、必要時）

**ホスト関数案**:
```rust
// 読み取り
fn get_machine_state(entity: u64) -> i32;
fn get_inventory_slot(entity: u64, slot: u32) -> (u32, u32); // item_id, count
fn get_power_level(entity: u64) -> i32;

// 書き込み
fn set_machine_enabled(entity: u64, enabled: i32);
fn transfer_item(from: u64, to: u64, item_id: u32, count: u32) -> i32;

// イベント
fn subscribe_event(event_type: u32);
fn emit_event(event_type: u32, data_ptr: u32, data_len: u32);

// ログ
fn log_info(msg_ptr: u32, msg_len: u32);
fn log_error(msg_ptr: u32, msg_len: u32);
```

### W.3: サンプルCore Mod作成

| タスク | 内容 | 状態 |
|--------|------|------|
| W.3.1 | Mod用Rustプロジェクト雛形 | ❌ |
| W.3.2 | ビルドスクリプト | ❌ |
| W.3.3 | 「Hello World」Mod | ❌ |
| W.3.4 | 機械状態変更Mod | ❌ |

**ディレクトリ構造案**:
```
mods/
├── base/              # Data Mod（TOML）
│   ├── items.toml
│   ├── machines.toml
│   └── recipes.toml
└── sample_core_mod/   # Core Mod（WASM）
    ├── Cargo.toml
    ├── src/
    │   └── lib.rs
    └── build.sh
```

### W.4: Mod間依存解決

| タスク | 内容 | 状態 |
|--------|------|------|
| W.4.1 | 依存関係宣言形式 | ❌ |
| W.4.2 | ロード順序決定 | ❌ |
| W.4.3 | 循環依存検出 | ❌ |

### W.5: ホットリロード（開発用）

| タスク | 内容 | 状態 |
|--------|------|------|
| W.5.1 | ファイル監視 | ❌ |
| W.5.2 | 再コンパイルトリガー | ❌ |
| W.5.3 | 状態保持リロード | ❌ |

### W.6: タグシステム（Forge Ore Dictionary相当）

**目標**: Mod間でアイテムを共有可能にする

### W.7: 特殊機械のCore Mod化（再実装）

**目標**: ハードコードされた特殊ロジックをCore Modで再定義

| タスク | 内容 | 状態 |
|--------|------|------|
| W.7.1 | 納品プラットフォーム再実装 | ❌ |
| W.7.2 | クエスト進行ロジック | ❌ |
| W.7.3 | 手動クラフトトリガー | ❌ |

**理由**: 現在の実装はプロトタイプ。Core Mod基盤完成後に正式実装する。

---

| タスク | 内容 | 状態 |
|--------|------|------|
| W.6.1 | アイテムにタグ定義追加 | ❌ |
| W.6.2 | タグ → アイテム一覧の逆引き | ❌ |
| W.6.3 | レシピでタグ指定対応 (`#ingot/copper`) | ❌ |
| W.6.4 | クエストでタグ指定対応 | ❌ |

**定義例**:
```toml
# items.toml
[copper_ingot]
name = "銅インゴット"
tags = ["ingot", "ingot/copper", "metal"]

[fancy_copper]
name = "高純度銅"
tags = ["ingot", "ingot/copper", "metal"]  # 同じタグ→互換性あり
```

**使用例**:
```toml
# recipes.toml
[copper_wire]
input = "#ingot/copper"   # タグ指定: 銅インゴットなら何でもOK
output = "base:copper_wire"
```

**メリット**:
- ModAの銅とModBの銅が互換
- レシピの柔軟性向上
- 「金属全部」「インゴット全部」のような指定が可能

---

## 残りの移行タスク（並行作業可）

### M.1: BlockType→ItemId移行

**ステータス**: 🔄 移行中（新機能追加時に順次）

| カテゴリ | 箇所数 | 対応 |
|----------|--------|------|
| 定義ファイル (`block_type.rs`, `id.rs`) | ~190 | **維持** |
| ワールド・描画層 | ~60 | **維持** (ブロックはBlockType) |
| ゲーム仕様 (`game_spec/`) | ~150 | **移行対象** |
| コンポーネント・システム | ~150 | **移行対象** |
| その他 | ~120 | **移行対象** |

**方針**: 一括移行ではなく、新機能実装時に関連箇所を移行

---

## 将来のマイルストーン（参考）

### M3: 電力システム

| タスク | 内容 |
|--------|------|
| P.1 | 電力グリッド計算 |
| P.2 | 発電機（水車、石炭発電） |
| P.3 | 電線ブロック |
| P.4 | 電力消費機械 |
| P.5 | 電力UI |

---

## M2→M3の間: リソースネットワーク設計

**⚠️ ユーザーと相談しながら進めること**

**目標**: 電力・液体・信号を統一的に扱える汎用基盤

### N: リソースネットワーク仕様策定

| タスク | 内容 | 状態 |
|--------|------|------|
| N.1 | リソース種別の抽象定義（電力/液体/信号の共通点） | ❌ |
| N.2 | ネットワーク接続の仕組み（電線/パイプ/ワイヤー） | ❌ |
| N.3 | ノード種別（Producer/Consumer/Storage/Conduit） | ❌ |
| N.4 | 計算方式（供給/需要/優先度） | ❌ |
| N.5 | Mod拡張ポイント（新リソース種類の追加方法） | ❌ |

**決めるべき仕様例**:
```toml
[power]
unit = "kW"
decay = false
max_transfer = 1000

[fluid.water]
unit = "L"
viscosity = 1.0
```

**これにより可能になること**:
- Modで「魔力」「蒸気」など新リソース追加
- 電力/液体/信号が同じ基盤で動く
- M3-M4の実装がスムーズに

**信号対応の方針**:
- M3: 案1（統一・シンプル）で電力のみ実装
- M4以降: 必要に応じて案3（信号フラグ追加）に拡張

```rust
// 案1: 最初の実装
struct ResourceType {
    name: String,
    unit: String,
    decay: bool,
    max_transfer: f32,
}

// 案3: 信号対応時に追加（既存コード変更なし）
struct ResourceType {
    name: String,
    unit: String,
    decay: bool,
    max_transfer: f32,
    is_signal: bool,         // 追加: trueならコピー伝播
    propagation_delay: u32,  // 追加: 信号遅延tick
}
```

| 特性 | 電力・液体 | 信号 |
|------|-----------|------|
| 値の型 | 数値 | ON/OFF or 0-15 |
| 分岐時 | 分割される | コピーされる |
| 消費 | 使うと減る | 減らない |

---

### M4: 液体・信号

| タスク | 内容 |
|--------|------|
| F.1 | 液体スロット・パイプ |
| F.2 | ポンプ・タンク |
| S.1 | 信号ワイヤー |
| S.2 | センサー |
| S.3 | 論理ゲート |

### M4.5: 調整ツール & ビジュアル強化

#### E: レシピエディタ（WebUI）

**目標**: 非エンジニアがブラウザからレシピ調整できる

| タスク | 内容 | 状態 |
|--------|------|------|
| E.1 | WebSocket APIにTOML読み書き追加 | ❌ |
| E.2 | 簡易HTML+JSエディタUI | ❌ |
| E.3 | バリデーション（不正レシピ検出） | ❌ |
| E.4 | ゲーム再起動なしリロード（stretch） | ❌ |

**構成**:
```
ブラウザ (localhost:9877/editor)
    ↓ WebSocket
既存WebSocket API
    ↓ ファイル
mods/base/*.toml
    ↓ 起動時読み込み
ゲーム本体
```

#### V: ビジュアル強化

**目標**: 見た目のインパクト向上（一般受け）

| タスク | 内容 | 状態 |
|--------|------|------|
| V.1 | ライティング改善（光源追加、影品質） | ❌ |
| V.2 | ポストプロセス（ブルーム、トーンマッピング） | ❌ |
| V.3 | 機械アニメーション（稼働時の動き） | ❌ |
| V.4 | パーティクル（煙、火花、光） | ❌ |
| V.5 | サウンド（SE、BGM） | ❌ |

---

### M5: ゲーム完成

- 機械50種類以上
- レシピ100種類以上
- マップ機能
- ブループリント

### M6: マルチプレイ

- P2P接続
- 状態同期
- 権限管理

---

## 新コンテンツ追加フロー

### 現在（Data Mod）

```
1. mods/base/items.toml に追加（3行）
2. mods/base/machines.toml に追加（10行）
3. mods/base/recipes.toml に追加（3行）
4. assets/models/ に3Dモデル配置
5. 完了（Rustコード変更なし）
```

### M2完了後（Core Mod）

```
1. mods/my_mod/Cargo.toml 作成
2. mods/my_mod/src/lib.rs に新ロジック
3. cargo build --target wasm32-unknown-unknown
4. mods/my_mod/mod.toml で依存宣言
5. 完了（本体コード変更なし）
```

---

## M2完了時: API Wiki自動生成

**目標**: コードのドキュメントコメントから自動でWiki更新

| タスク | 内容 | 状態 |
|--------|------|------|
| A.1 | Mod APIにドキュメントコメント追加 | ❌ |
| A.2 | ドキュメント抽出スクリプト作成 | ❌ |
| A.3 | GitHub Actions設定（push時に自動実行） | ❌ |
| A.4 | GitHub Wiki連携 or `/docs` 出力 | ❌ |

**トリガー**: `src/modding/**` への変更時

---

## M2完了時: 自動バグ取りシステム

**目標**: 龍が如く7式の自動バグ発見・報告システム

### B: 操作記録 & 再生

| タスク | 内容 | 状態 |
|--------|------|------|
| B.1 | InputRecorder（操作記録） | ❌ |
| B.2 | ReplayMode（再生システム） | ❌ |
| B.3 | バグ検出ロジック（パニック、NaN、フリーズ等） | ❌ |
| B.4 | 自動レポート（GitHub Issue連携） | ❌ |

### S: シナリオテスト

| タスク | 内容 | 状態 |
|--------|------|------|
| S.1 | シナリオ定義形式（TOML） | ❌ |
| S.2 | 抽象アクション実装（find, mine, place等） | ❌ |
| S.3 | シナリオ実行エンジン | ❌ |
| S.4 | プリセットシナリオ作成 | ❌ |

**シナリオ例**:
```toml
[[steps]]
action = "mine_until"
target = "base:iron_ore"
count = 10

[[steps]]
action = "place_machine"
machine = "base:furnace"
```

**バグ発生時の自動出力**:
- スクリーンショット
- 操作ログ（再現用）
- ゲーム状態ダンプ
- GitHub Issue自動作成

---

## M5完了時: リポジトリ名変更

**ゲーム名決定: RisoFactory**（理想ファクトリー）

| タスク | 内容 | 状態 |
|--------|------|------|
| R.1 | GitHubリポジトリ名を `riso-factory` に変更 | ❌ |
| R.2 | `Cargo.toml` の `name` を `riso_factory` に変更 | ❌ |
| R.3 | コード内の `idle_factory` 参照を更新 | ❌ |
| R.4 | ウィンドウタイトルを「RisoFactory」に変更 | ❌ |
| R.5 | README、ドキュメント更新 | ❌ |

**名前の由来**:
- 日本語: 理想（りそう）+ Factory
- イタリア語: Riso = 米（risottoの語源）& 笑い
- 「笑いの工場」というダブルミーニング

---

*最終更新: 2026-01-08*
