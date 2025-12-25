# Claude Code メモリ

## 作業ルール

| ルール | 詳細 |
|--------|------|
| ログ | 作業中は出力禁止、終了時に短く報告 |
| テスト | 実装/変更時はテスト作成、Clippy後にテスト実行 |
| コミット | 適宜実行、日本語メッセージ、プッシュは指示待ち |
| **Worktree** | **新機能は必ずWorktreeで作業（下記参照）** |
| 中断禁止 | 確認を求めず最後まで完了させる |
| 記録 | 完了/失敗時は`changelog.md`に追記 |
| **課題** | **未解決の問題は`issues.md`に追記** |
| パターン | 仕様変更/実装前に`patterns-compact.md`確認 |
| UI実装 | UI作成/修正時は`ui-design-rules.md`に従う |
| 動作確認 | `/e2e-test`スキルを使用、クラッシュ時はログ解析 |
| 3Dモデル | 「XXXのモデルを作成」指示でサブエージェント起動 |
| サブエージェント | **最大3-5個**に制限、過度な並列はトークン浪費 |

## トークン効率化（重要）

**リミット節約のため以下を厳守:**

| ルール | 詳細 |
|--------|------|
| タスク分割 | 1セッション=1機能、調査と実装は別セッション |
| 並列制限 | サブエージェントは同時3-5個まで |
| 参照最小化 | changelog全体を読まない、最新1週間分のみ |
| 計画優先 | 大きなタスクは先にspecs/に計画を書く→別セッションで実装 |
| 探索禁止 | 「全体を調査」より「XXXファイルを確認」と具体的に |

**セッション分割の例:**
```
❌ 「6ゲームを調査して9システム実装して」（トークン大量消費）
✅ セッション1:「Factorioのベルト仕様を調査」→結果をmdに保存
✅ セッション2:「specs/belt.mdに従って実装」→実装のみ
```

## 3Dモデル生成

「〇〇のモデルを作成して」という指示を受けたら:
1. `.specify/memory/modeling-rules.md` を参照
2. サブエージェント起動 (Task tool, subagent_type: general-purpose)
3. 完了後、結果を報告

## E2Eテスト

動作確認が必要な場合は `/e2e-test` スキルを使用:
- `/e2e-test` - フルテスト実行
- `/e2e-test interaction` - インタラクションテスト
- `/e2e-test report` - 最新レポート確認

結果確認の優先順位:
1. `screenshots/test_report.txt` - サマリ（最優先）
2. `screenshots/*_ui_dump_*.txt` - UIダンプ
3. `screenshots/*.png` - スクリーンショット（問題時のみ）

## 参照ファイル

| 優先度 | ファイル | 用途 |
|--------|----------|------|
| 必須 | `API_REFERENCE.md` | 関数・構造体一覧 |
| 状況 | `.specify/memory/constitution.md` | プロジェクト原則 |
| 状況 | `.specify/memory/changelog.md` | 開発履歴（**最新1週間のみ参照**） |
| 状況 | `.specify/memory/issues.md` | 未解決の課題 |
| 状況 | `.specify/memory/patterns-compact.md` | 52パターン |
| 状況 | `.specify/memory/ui-design-rules.md` | UIデザインルール |
| 3Dモデル | `.specify/memory/modeling-rules.md` | モデリング詳細ルール |
| 3Dモデル | `docs/style-guide.json` | 3Dモデルスタイルガイド |
| 3Dモデル | `tools/blender_scripts/_base.py` | Blender共通モジュール |
| 状況 | `.specify/specs/index-compact.md` | 全仕様集約 |
| 詳細時 | `.specify/specs/*.md`, `src/**/*.rs` | 個別レポート/ソース |
| アーカイブ | `.specify/memory/changelog-archive/` | 古い履歴（読まない） |

**原則**: 圧縮版優先、詳細は必要時のみ

**changelog管理**: 月が変わったら古い履歴を`changelog-archive/YYYY-MM.md`に移動

## Git Worktree ワークフロー（必須）

新機能実装時は**必ず**以下の手順で作業すること:

```bash
# 1. Worktree作成（機能名でブランチ作成）
git worktree add ../idle_factory_worktrees/機能名 -b feature/機能名

# 2. そのディレクトリで作業・コミット
cd ../idle_factory_worktrees/機能名
# ... 実装 ...
git add -A && git commit -m "feat: 機能説明"

# 3. masterにマージ（メインディレクトリで）
cd /home/bacon/github/idle_factory
git merge feature/機能名

# 4. Worktree削除
git worktree remove ../idle_factory_worktrees/機能名
git branch -d feature/機能名
```

**禁止事項**: masterブランチで直接新機能を実装しない

## README.md

**含める**: 概要、AI駆動開発の特徴、エディタ/MOD、実装済み機能、予定、技術スタック
**含めない**: ビルド手順、環境構築、構成詳細、操作方法、連絡先
**冒頭に記載**: 「🤖 AI開発プロジェクト」
