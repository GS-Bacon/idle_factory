# Claude Code メモリ

## 基本ルール

| ルール | 詳細 |
|--------|------|
| テスト | 実装後は `cargo test` と `cargo clippy` |
| コミット | 日本語メッセージ、pushは指示待ち |
| 中断禁止 | 確認を求めず最後まで完了 |
| 動作確認 | 実際に `cargo run` で遊んで確認 |

## 禁止事項

- 仕様だけのセッション2回連続
- パターン集の事前確認（問題が起きたら見る）
- レビュー・評価スキルの使用
- ドキュメント整理・圧縮作業

## マイルストーン駆動

```
仕様（1ページ）→ 実装 → 動かして確認 → 次へ
```

「done」= テストが通る、ではなく「遊べる」

## 3Dモデル生成

「〇〇のモデルを作成」指示で:
1. `.specify/memory/modeling-rules.md` 参照
2. サブエージェント起動
3. 完了報告

## 参照ファイル

| ファイル | 用途 |
|----------|------|
| `API_REFERENCE.md` | 関数・構造体 |
| `.specify/specs/` | 仕様（実装対象） |
| `.specify/roadmap.md` | 将来機能（実装しない） |
| `.specify/memory/modeling-rules.md` | 3Dモデル |
| `.specify/memory/ui-design-rules.md` | UI実装時 |
