# 実装計画

> **設計詳細**: `.claude/architecture.md`
> **ロードマップ**: `.specify/roadmap.md`

## 現状サマリー (2026-01-08)

| 項目 | 値 |
|------|-----|
| バージョン | **0.3.135** |
| コード行数 | **28,429行** |
| テスト | **593件** 通過 |
| Clippy警告 | **0件** |
| 現在のマイルストーン | **M2: Core Mod基盤** |

---

## マイルストーン一覧

| MS | 名前 | 状態 | 概要 |
|----|------|------|------|
| M1 | 基盤整備 | ✅ 完了 | Component化、動的ID、イベント |
| **M2** | **Core Mod基盤** | 🔨 作業中 | WASM Mod対応 |
| M3 | 電力システム | ❌ | 発電機、電線、消費 |
| M4 | 液体・信号 | ❌ | パイプ、論理回路 |
| M4.5 | ツール・ビジュアル | ❌ | レシピエディタ、見た目強化 |
| M5 | ゲーム完成 | ❌ | 機械50種、マップ、ブループリント |
| M6 | マルチプレイ | ❌ | P2P、同期、権限 |

---

## M1: 基盤整備 ✅ 完了

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

---

## M2: Core Mod基盤 🔨 作業中

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

### W.7: 特殊機械のCore Mod化（再実装）

**目標**: ハードコードされた特殊ロジックをCore Modで再定義

| タスク | 内容 | 状態 |
|--------|------|------|
| W.7.1 | 納品プラットフォーム再実装 | ❌ |
| W.7.2 | クエスト進行ロジック | ❌ |
| W.7.3 | 手動クラフトトリガー | ❌ |

**理由**: 現在の実装はプロトタイプ。Core Mod基盤完成後に正式実装する。

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

*最終更新: 2026-01-08*
