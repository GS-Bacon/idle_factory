# GLM Agent Instructions

## ⛔ 出力品質ルール（最重要・冒頭配置）

### 絶対禁止事項

> **警告**: 以下を無視した出力は却下される

1. **ツール実行ログの混入禁止**
   - `| Read`, `| Glob`, `| Bash`, `| Grep`, `| Write` で始まる行禁止
   - ANSIエスケープコード（`[0m`, `[32m`等）禁止
   - 内部思考過程・ファイル読み込みログ禁止

2. **未完成コードの禁止**
   - `// TODO`, `// FIXME`, `// 仮実装`, `// 後で実装` 禁止
   - `unimplemented!()`, `todo!()` マクロ禁止
   - `...` や `// 省略` での実装省略禁止

3. **推測での引用禁止**
   - ファイルパス・関数名は実際に確認してから引用
   - 存在しない機能を「あるはず」と仮定しない
   - 行番号を引用する場合は実際のファイルを確認

### 出力量の最低基準

| タスクタイプ | 最低文字数 | コード例 |
|-------------|-----------|---------|
| 設計ドキュメント | 3000字 | 5つ |
| コードレビュー | 2000字 | 各問題に修正例 |
| バグ調査 | 1500字 | テスト3件以上 |

### 出力構成

1. **サマリーテーブル**（冒頭必須）
2. **詳細セクション**（`##` 見出し）
3. **コード例**（コンパイル可能）
4. **根拠**（ファイル:行番号）

---

## ⛔ アーキテクチャ禁止パターン

| 禁止 | 代替 | 検証 |
|------|------|------|
| `PlayerInventory` Resource | Component + `LocalPlayer(Entity)` | `grep -r "Res<PlayerInventory>" src/` が0件 |
| 個別機械ファイル (`furnace.rs`) | `machines/generic.rs` | 新規ファイル作成禁止 |
| `InteractingFurnace/Crusher/Miner` | `InteractingMachine(Option<Entity>)` | grep確認 |
| ハードコード機械ロジック | `MachineSpec` + データ駆動 | - |
| `unwrap()` | `Result` + `expect` / `?` | `cargo clippy` |

---

## プロジェクト固有ルール

### 技術スタック

- Rust + Bevy 0.15
- ECSアーキテクチャ（Component/System分離）
- WASM Mod API

### コーディング規約

1. **O(n²)禁止** - HashMap/空間ハッシュ使用
2. **unwrap禁止** - Result + expect使用
3. **テスト必須** - 新機能にはユニットテスト

### 既存パターン参照

| パターン | ファイル |
|---------|----------|
| ID管理 | `src/core/id.rs` |
| イベント | `src/events/game_events.rs` |
| 機械定義 | `src/game_spec/machines.rs` |
| レシピ | `src/game_spec/recipes.rs` |

---

## 基本ルール

| ルール | 詳細 |
|--------|------|
| **ビルド確認** | `cargo build` を通してから提出 |
| テスト | `cargo test && cargo clippy` |
| コミット | 日本語、短く。pushは指示待ち |
| ゲーム起動 | `./run.sh` |
| バグ修正 | **再現テストなしの修正禁止** |
| アーキテクチャ | `.claude/architecture.md` が権威ソース |

---

## バグ修正ルール

1. **シナリオテストで再現**（`tests/scenarios/` にTOML作成）
2. テストが失敗することを確認
3. 修正する
4. テストがパスすることを確認

### 「直りました」宣言ルール

**禁止**: コード変更だけで「直りました」と言う

**必須**: 以下のいずれかの証拠を示すこと
1. シナリオテストがパス
2. スクショで修正後の状態を見せる
3. ログ出力で正常動作を確認

---

## 品質チェックリスト

出力前に確認:

- [ ] ツールログ混入なし
- [ ] TODOコメントなし
- [ ] コードがコンパイル可能
- [ ] 禁止パターン違反なし
- [ ] 最低文字数を満たす
- [ ] `cargo clippy` 警告0件

---

## 用語定義（ユビキタス言語）

| 用語 | 意味 | コード上の対応 |
|------|------|---------------|
| **手持ちインベントリ** | プレイヤーが持ち歩くアイテム | `PlayerInventory` (Component) |
| **プラットフォームインベントリ** | 納品プラットフォームに紐づいた倉庫 | `PlatformInventory` (Component) |
| **収納** | コンベア経由でプラットフォームインベントリにアイテムが入ること | `TransferTarget::Delivery` |
| **納品** | クエストのためにアイテムを消費すること | `quest_deliver_button` |

---

## 参照ドキュメント

| ファイル | 内容 |
|----------|------|
| **`.claude/architecture.md`** | **将来アーキテクチャ設計（権威ソース）** |
| `.claude/implementation-plan.md` | タスク一覧 |
| `docs/scenario-test-guide.md` | シナリオテスト詳細 |

---

## 特殊操作

### Windowsへビルド送信

**トリガー**: 「windowsに送信」「windowsに送って」等

```bash
./scripts/build-packages.sh --windows-only
tailscale file cp dist/idle_factory_*_windows.zip baconrogx13: || \
tailscale file cp dist/idle_factory_*_windows.zip smz-mousebook:
```

### 作業ログ保存

「ログを保存」と言われたら `WORK_LOG.md` に追記。

---

## 現在の状態

| 項目 | 値 |
|------|-----|
| バージョン | **0.3.184** |
| コード行数 | **29,253行** |
| テスト | **613件** 通過 |
| 移行状態 | **✅ 新アーキ完全移行済み** |
