# /yolo - 自動改善フィードバックループ

全ての改善プロセスを自動的にループ実行するスキル。

## 使用方法

- `/yolo` - フル自動改善ループ（改善がなくなるまで継続）
- `/yolo once` - 1サイクルのみ実行
- `/yolo quick` - テストをスキップして高速実行
- `/yolo status` - 現在の状態を表示
- `/yolo stop` - 次のサイクルで停止（フラグ設定）

## 引数: $ARGUMENTS

## フィードバックループ概要

```
┌─────────────────────────────────────────────────────────────┐
│                    YOLO Mode Loop                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐             │
│   │ 1.Review │───▶│ 2.Record │───▶│ 3.Fix    │             │
│   │ 批判的   │    │ issues.md│    │ 問題解決 │             │
│   │ レビュー │    │ 更新     │    │          │             │
│   └──────────┘    └──────────┘    └──────────┘             │
│        ▲                               │                    │
│        │                               ▼                    │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐             │
│   │ 6.Check  │◀───│ 5.E2E    │◀───│ 4.Test   │             │
│   │ 改善あり?│    │ 動作確認 │    │ cargo    │             │
│   │          │    │          │    │ test     │             │
│   └──────────┘    └──────────┘    └──────────┘             │
│        │                                                    │
│        ▼                                                    │
│   改善あり → ループ継続                                      │
│   改善なし → 完了報告                                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 実行手順

### Phase 0: 初期化

```bash
# 現在の状態を記録
echo "$(date '+%Y-%m-%d %H:%M:%S') - YOLO開始" >> feedback/yolo_log.txt

# 初期計測
INITIAL_ISSUES=$(grep -c "未着手" .specify/memory/issues.md || echo 0)
INITIAL_WARNINGS=$(cargo clippy 2>&1 | grep -c "warning" || echo 0)
```

### Phase 1: 批判的レビュー生成

```markdown
## /review 実行

1. E2Eテスト実行して現状確認
   DISPLAY=:10 timeout 60 cargo run -- --e2e-test

2. 批判的レビュー生成
   - UI/UX評価
   - ゲームプレイ評価
   - コード品質評価
   - 競合比較

3. レビュー結果を保存
   feedback/reviews/$(date +%Y-%m-%d)_review.md
```

### Phase 2: issues.md更新

```markdown
## 問題をissues.mdに記録

1. レビューから致命的/重大な問題を抽出
2. 既存のissues.mdと重複チェック
3. 新規問題をタスクとして追加
   - 優先度設定 (critical/high/mid/low)
   - 関連レビュー番号を記録
```

### Phase 3: 問題解決

```markdown
## /fix-issues all 実行

1. issues.mdの未着手タスクを取得
2. 優先度順に解決
   - critical/high → 必ず解決
   - mid → 時間があれば
   - low → スキップ可

3. 各タスク完了後
   - issues.md更新
   - changelog記録
```

### Phase 4: テスト実行

```bash
# ユニットテスト
cargo test --lib 2>&1 | tee /tmp/yolo_test.log
TEST_RESULT=$?

# Clippy
cargo clippy 2>&1 | tee /tmp/yolo_clippy.log
CLIPPY_WARNINGS=$(grep -c "warning" /tmp/yolo_clippy.log || echo 0)

# 失敗時は修正を試みる
if [ $TEST_RESULT -ne 0 ]; then
    echo "テスト失敗 - 修正を試行"
    # エラー内容を解析して修正
fi
```

### Phase 5: E2E動作確認

```markdown
## /e2e-test 実行

1. ゲーム起動
   DISPLAY=:10 timeout 120 cargo run -- --e2e-test

2. 全タブをテスト
   - メインメニュー
   - ゲームプレイ
   - インベントリ
   - クラフト
   - 設定
   - etc.

3. 結果確認
   cat screenshots/test_report.txt
```

### Phase 6: 改善チェック

```markdown
## 改善があったか判定

比較項目:
- issues.md: 未着手タスク数
- Clippy: 警告数
- テスト: パス数
- E2E: 成功タブ数
- レビュー評価: 総合評価

判定基準:
- 1つでも改善 → 次サイクルへ
- 全て同じ → 完了

最大サイクル数: 10回（無限ループ防止）
```

### Phase 7: ループ継続 or 完了

```markdown
## 改善ありの場合

「Phase 1に戻る」

## 改善なしの場合

1. 最終レポート生成
   - 実行サイクル数
   - 改善した項目一覧
   - 残存問題
   - 実行時間

2. feedback/yolo_log.txt に記録

3. changelog.md に追記
```

## 安全機構

### 無限ループ防止
```
MAX_CYCLES=10
STAGNATION_LIMIT=3  # 改善なし連続回数
```

### 破壊的変更の禁止
以下は自動実行しない:
- アーキテクチャ変更
- 破壊的API変更
- 大規模リファクタリング
- セキュリティ関連

### ロールバック
各サイクル開始時にgit stashで状態保存:
```bash
git stash push -m "yolo-cycle-$(date +%s)"
```
失敗時はロールバック可能

## 実行例

### 例1: フル自動改善
```
> /yolo

=== YOLO Mode 開始 ===

--- サイクル 1/10 ---
[Phase 1] レビュー生成中...
  総合評価: B
  致命的問題: 0件
  重大問題: 2件

[Phase 2] issues.md更新
  新規追加: 2件
  未着手合計: 10件

[Phase 3] 問題解決中...
  #8 GETTING_STARTED.md作成 → ✅完了
  #9 unwrap修正 → ✅完了
  解決: 2/10件

[Phase 4] テスト実行
  cargo test: 177 passed
  cargo clippy: 0 warnings

[Phase 5] E2E確認
  7/9 タブ成功

[Phase 6] 改善チェック
  未着手: 10 → 8 (改善)
  継続判定: YES

--- サイクル 2/10 ---
...

--- サイクル 5/10 ---
[Phase 6] 改善チェック
  改善なし（3回連続）
  終了判定: YES

=== YOLO Mode 完了 ===

## 最終レポート
- 実行サイクル: 5回
- 解決した問題: 12件
- 残存問題: 3件（要手動）
- 総実行時間: 23分
- 最終評価: B → A
```

### 例2: 1サイクルのみ
```
> /yolo once

=== YOLO Mode (1サイクル) ===

--- サイクル 1/1 ---
[Phase 1] レビュー生成中...
...

=== 完了 ===
次回: `/yolo` で継続
```

### 例3: 状態確認
```
> /yolo status

## YOLO Status

最終実行: 2025-12-25 15:30
サイクル数: 5
解決済み: 12件
残存: 3件

### 未解決問題 (要手動)
- #13 e2e_test.rs分割
- #14 spec-impl-gap消化
- #17 .specify/整理

### 次回実行で対応予定
- #10 モックアセット作成
- #11 CI clippy設定
```

## ログファイル

```
feedback/
├── yolo_log.txt           # 実行履歴
├── sessions/
│   └── yolo_YYYYMMDD_HHMMSS/
│       ├── cycle_1.md     # サイクル1の詳細
│       ├── cycle_2.md     # サイクル2の詳細
│       └── final_report.md
└── reviews/
    └── ...
```

## 設定

`feedback/config/yolo_config.yaml`:
```yaml
max_cycles: 10
stagnation_limit: 3
skip_phases:
  - []  # 全実行
  # - [e2e]  # E2Eスキップ
  # - [review]  # レビュースキップ
auto_commit: false
auto_push: false
notification:
  on_complete: true
  on_error: true
```

## 注意事項

- **監視推奨**: 初回実行は監視しながら実行
- **時間**: フルサイクルは1時間以上かかる可能性あり
- **リソース**: E2Eテストは画面が必要（DISPLAY=:10）
- **コミット**: 自動コミットはしない（明示的に設定した場合のみ）
- **中断**: Ctrl+C で安全に中断可能

## 他スキルとの関係

```
/yolo
  ├── /review      (Phase 1)
  ├── /fix-issues  (Phase 3)
  ├── /e2e-test    (Phase 5)
  └── /evaluate    (Phase 6 補助)
```

YOLOモードは他スキルを内部的に呼び出すメタスキル。
