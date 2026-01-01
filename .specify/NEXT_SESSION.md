# 次セッション用メモ

## 現在の状態（2026-01-01）

### プロジェクト状態

| 項目 | 状態 |
|------|------|
| v0.1 MVP | 完了 |
| テスト | 91件パス |
| ビルド | 2秒（dev） |
| ブランチ | master |

### 最近の作業

- アーキテクチャレビュー実施 → `.claude/refactoring-tasks.md` に25タスク記録
- ゲーム評価: 技術7/10、ゲーム体験6/10
- 廃止仕様アーカイブ削除（コンテキスト23%削減）

### 直近の課題

| 優先度 | タスク | 詳細 |
|--------|--------|------|
| 🔴高 | リファクタリング | block_operations.rs分割、ui_setup.rs分割、Plugin化 |
| 🟠バグ | unwrap()安全化 | 72箇所のunwrap()をOptionハンドリングに |
| 🔵テスト | E2E改善 | /test, /assert, /spawn_lineコマンド追加 |

## 解決済みの問題

- UIテキスト表示問題 → BUG-UI-1〜4で修正済み
- カメラ入力遅延 → PipelinedRenderingPlugin無効化で解決
- コンベア方向問題 → 一部修正済み（左右逆問題は残存）

## 次にやること

1. **技術基盤固め**を優先（ユーザー希望）
   - `.claude/refactoring-tasks.md` のタスク消化
   - 並列作業可能: CI/CD改善、Cargo.toml最適化

2. **ゲーム体験改善**は後回し
   - オンボーディング、バランス調整はプレイフィードバック待ち

## 参照ファイル

| ファイル | 内容 |
|----------|------|
| `CLAUDE.md` | AI向けメモリ（必読） |
| `.claude/refactoring-tasks.md` | 全タスクリスト |
| `.claude/architecture.md` | モジュール構成 |
| `src/game_spec.rs` | ゲーム仕様（Single Source of Truth） |

## 操作方法

- WASD/Space/Shift: 移動
- マウス: 視点操作
- 左クリック: ブロック破壊
- 右クリック: ブロック設置/機械操作
- E: インベントリ
- T or /: コマンド入力
- F3: デバッグHUD
