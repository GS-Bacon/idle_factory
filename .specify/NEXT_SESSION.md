# 次セッションへの引き継ぎ

## 完了したこと

### プロジェクト進行方針の刷新
- CLAUDE.mdをシンプル化（批判的姿勢を追加）
- 不要ファイル削除（-4,285行）
  - メタワークスキル（review, evaluate, fix-issues, yolo）
  - パターン集、changelog、issues、constitution
  - AIフィードバックループ（src/core/feedback/, feedback/）
- roadmap.md作成（将来機能の概要リスト）
- specs/index.mdをシンプル化

### 新しいルール
- 仕様だけのセッション2回連続禁止
- 「done」= 遊べる状態
- ユーザー提案にも批判的に意見する

---

## 次にやること

### 1. 最初のマイルストーン定義
具体的なゴールを決める。例:
```
タイトル画面 → ワールド生成 → 移動・採掘 →
インベントリ → クラフト → 機械設置 → 動作確認
```
これが動けばマイルストーン1完了。

### 2. 必要な仕様だけ確認
mechanics/から該当ファイルのみ読む:
- player.md
- worldgen.md
- interaction.md
- miner.md（or 採掘関連）
詳細が足りなければ実装時に決める。

### 3. 実装開始
仕様セッションは1回で終わり。すぐ実装に入る。

---

## 注意事項

### 批判的に見るべきポイント
- 「仕様を詳細化しよう」→ 仕様地獄の入口
- 「レビューしよう」→ メタワーク
- 「パターンに従おう」→ 問題が起きてから見ればいい
- 「done」と言いながらテスト通るだけ → 実際に遊べ

### 削除したもの（復活させない）
- /review, /evaluate, /fix-issues スキル
- patterns-compact.md（52パターン）
- constitution.md
- changelog.md, issues.md
- AIフィードバックループ（1400行）

---

## ファイル構成

```
.specify/
├── roadmap.md          # 将来機能（詳細は実装時）
├── NEXT_SESSION.md     # このファイル（読んだら削除可）
├── memory/
│   ├── modeling-*.md   # モデリング用
│   └── ui-design-rules.md
└── specs/
    ├── index.md        # 仕様インデックス
    ├── core-concept.md
    ├── editor.md       # エディタ仕様（確定）
    ├── ui.md
    ├── first-30-minutes.md
    └── mechanics/      # 個別メカニクス

.claude/commands/
├── e2e-test.md
└── generate-model.md
```
