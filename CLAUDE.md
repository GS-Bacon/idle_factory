# Claude Code メモリ

## 最重要ルール（これだけ守れ）

| # | ルール | 詳細 |
|---|--------|------|
| 1 | **止まるな** | ビルドエラー/テスト失敗は自力で直す。確認を求めず完了まで進む |
| 2 | **テストなしの修正禁止** | バグ修正は必ずシナリオテストで再現してから |
| 3 | **並列実行** | 下記「並列実行フロー」参照 |
| 4 | **メタワーク禁止** | ドキュメント整理だけで終わるセッションは失敗 |
| 5 | **設計は architecture.md** | 新規実装は必ず参照、矛盾する実装は禁止 |

## 現在の状態（Single Source of Truth）

| 項目 | 値 |
|------|-----|
| バージョン | **0.3.167** |
| コード行数 | **34,624行** |
| テスト | **781件** 通過 |
| Clippy警告 | **0件** |
| 移行状態 | **✅ 新アーキ完全移行済み** |

## 止まって良い場面（これ以外は止まるな）

- 致命的エラーが3回連続で解決できない
- **新ゲーム要素、UI/UX変更、ゲームバランス変更**（ユーザー確認必要）
- pushの指示待ち
- 指定された全タスクが完了した

---

## 参照ドキュメント

| ファイル | 内容 |
|----------|------|
| **`.claude/architecture-future.md`** | **将来アーキテクチャ設計（権威ソース）** |
| `.claude/implementation-plan.md` | タスク一覧 |
| `.claude/bugs.md` | よくあるバグと対策 |

---

## 並列実行フロー

### 計画時（プランモード）

2ファイル以上変更する場合、以下の形式で計画:

```markdown
## Phase 1: 調査（並列）
| ID | 内容 | 依存 |
|----|------|------|
| R1 | xxx調査 | - |
| R2 | yyy調査 | - |

## Phase 2: 実装（並列、1ファイル1エージェント）
| ID | ファイル | 依存 |
|----|----------|------|
| I1 | src/a.rs | - |
| I2 | src/b.rs | I1 |
```

### 実行時

```
1. 調査フェーズ（worktree不要）
   Task(Explore) を複数同時起動

2. 実装フェーズ（worktree内でサブエージェント並列）
   ./scripts/parallel-run.sh start feature-x
   Task(general-purpose) を複数起動（1ファイル1エージェント）
   ※サブエージェントはビルドしない

3. 検証フェーズ（直列）
   cargo build && cargo test && cargo clippy

4. 完了
   ./scripts/parallel-run.sh finish feature-x
```

**判断基準**: 2ファイル以上変更 → worktree + サブエージェント並列

---

## コマンド・ツール

| コマンド | 用途 |
|----------|------|
| `cargo build` | ビルド（2秒） |
| `cargo test && cargo clippy` | テスト後に実行 |
| `./run.sh` | ゲーム起動 |
| `./scripts/parallel-run.sh` | worktree管理 |
| `node scripts/run-scenario.js <file>` | シナリオテスト |
| `gemini <cmd>` | Gemini連携（大規模モデル時のみ） |

---

## バグ修正フロー

```
1. tests/scenarios/bug_xxx.toml 作成
2. テスト失敗を確認
3. 修正
4. テスト成功を確認
```

シナリオテスト例:
```toml
name = "バグ再現: XXX"
[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }
[[steps]]
action = "wait"
params = { ms = 100 }
[[steps]]
action = "assert"
params = { condition = "ui_state == Inventory" }
```

---

## 禁止パターン（レガシー）

| 禁止 | 代替 |
|------|------|
| `PlayerInventory` (Resource) | `Inventory` (Component) + `LocalPlayer` |
| 個別機械ファイル (`furnace.rs`等) | `machines/generic.rs` |
| `InteractingFurnace/Crusher/Miner` | `InteractingMachine(Option<Entity>)` |

---

## 用語定義

| 用語 | 意味 | コード |
|------|------|--------|
| 収納 | コンベアで倉庫に入れる | `TransferTarget::Delivery` |
| 納品 | クエストでアイテム消費 | `quest_deliver_button` |

---

## 作業ログ

「ログを保存」と言われたら `WORK_LOG.md` に追記:

```markdown
## YYYY-MM-DD: タイトル
### 概要
1-2行
### 完了タスク
- 箇条書き
### 技術判断
- 決定と理由
```

---

## 作業終了時

以下を更新:
1. `CLAUDE.md` の「現在の状態」テーブル
2. `.claude/implementation-plan.md` の完了タスク

数値取得:
```bash
grep -E "^version" Cargo.toml | head -1
wc -l src/**/*.rs src/*.rs 2>/dev/null | tail -1
cargo test 2>&1 | grep -oP '\d+ passed' | awk -F' ' '{sum += $1} END {print sum}'
```

---

## バイブコーディング実験

このプロジェクトは「非エンジニアがAIだけで大規模ゲームを作れるか」の実験。

**観測すべき破綻パターン**:
1. コンテキスト限界 - AIが以前の決定を忘れる
2. メタワーク暴走 - ゲームが進まずドキュメントだけ増える
3. 修正ループ - AIがAIの書いたコードを直す無限ループ
4. 設計の不整合 - モジュール間で噛み合わない

**暴走検知**: AIが1時間作業したら → ゲームを起動して変化を見せる
