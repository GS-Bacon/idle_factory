# テストシステム自体の改善フロー

**作成日**: 2026-01-02

## 概要

テストシステムは「バグを見つける」だけでなく、「テストシステム自体を改善し続ける」仕組みが必要。

## メタテストの5つの柱

### 1. 検出漏れの追跡

バグが本番（手動プレイ）で見つかったとき:

```
バグ発見（本番）
    │
    ▼
なぜテストで検出できなかった？
    │
    ├── ユニットテストの穴？
    ├── E2Eシナリオの不足？
    ├── VLMプロンプトの見落とし？
    └── そもそもテスト不可能？
    │
    ▼
再発防止テストを追加
    │
    ▼
.claude/escaped-bugs.md に記録
```

**記録フォーマット**:
```markdown
## 漏れたバグログ

| 日付 | バグ内容 | 漏れた層 | 原因 | 追加したテスト |
|------|---------|---------|------|--------------|
| 2026-01-02 | コンベアが逆方向 | E2E | 方向テストなし | test_conveyor_direction |
| 2026-01-05 | UIが画面外 | VLM | プロンプトに含まず | VLM ui チェック追加 |
```

---

### 2. 誤検知（False Positive）の追跡

テストが問題ないのに失敗したとき:

| 種類 | 原因 | 対策 |
|------|------|------|
| Flaky test | 非決定論的実行 | シード固定、順序明示 |
| VLM誤検知 | プロンプトが曖昧 | プロンプト改善 |
| ピクセル比較誤検知 | 閾値が厳しすぎ | 閾値調整 |
| タイミング依存 | 処理時間の揺れ | リトライ or モック |

**記録先**: `test_reports/false_positives.log`

```
2026-01-02 VLM "chunk境界に隙間" → 実際は影の表現、プロンプト修正済み
2026-01-03 pixel_compare 1.2% diff → カメラ位置の微差、閾値1%→2%に変更
2026-01-04 test_conveyor_timing FLAKY → sleep追加で安定化
```

---

### 3. カバレッジギャップ分析

**頻度**: 週1回

```bash
# カバレッジ計測
cargo tarpaulin --out Html --output-dir coverage/

# 結果分析
./scripts/analyze_coverage.sh
```

**分析観点**:

| 状態 | アクション |
|------|----------|
| カバレッジ低い重要コード | テスト追加 |
| カバレッジ高いが価値低い | テスト削減検討 |
| デッドコード発見 | コード削除 |

---

### 4. テスト実行時間の監視

**目標時間**:

| 層 | 目標 | 超えたら |
|----|------|---------|
| pre-commit | < 5秒 | 遅いテスト特定 |
| cargo test | < 10秒 | 並列化検討 |
| E2E | < 2分 | モック化検討 |
| VLM | < 30秒 | キャッシュ活用 |

**監視スクリプト**:

```bash
#!/bin/bash
# scripts/test_timing.sh

echo "=== テスト実行時間 ==="
echo -n "lib tests: "
/usr/bin/time -f "%e sec" cargo test --lib 2>&1 | tail -1

echo -n "E2E: "
/usr/bin/time -f "%e sec" ./scripts/e2e-quick.sh 2>&1 | tail -1
```

---

### 5. テストシステムのレビュー

**頻度**: 月1回

**Geminiに聞く質問**:

```bash
./scripts/ask_gemini.sh "
現在のテスト構成:
- ユニットテスト: 142件
- proptest: 導入済み
- E2E: e2e_test.rs
- VLM: 6レベル
- ファズ: 基本版

質問:
1. この規模（11,500行）に対してテスト量は適切か？
2. 見落としている観点は？
3. 最新のRust/Bevyテスト手法で使えるものは？
4. テストの重複や無駄はないか？
"
```

---

## 改善サイクル図

```
         ┌──────────────────┐
         │   バグ発見       │
         └────────┬─────────┘
                  │
    ┌─────────────┴─────────────┐
    ▼                           ▼
┌────────┐                 ┌────────┐
│テストで│                 │本番で  │
│見つけた│                 │見つけた│
└───┬────┘                 └───┬────┘
    │                          │
    ▼                          ▼
┌────────┐                 ┌────────────┐
│修正のみ│                 │修正 +      │
│        │                 │テスト追加 +│
│        │                 │漏れ分析    │
└────────┘                 └────────────┘
                                │
                                ▼
                    ┌───────────────────┐
                    │テストシステム改善  │
                    │ ├── 新テスト追加   │
                    │ ├── プロンプト改善 │
                    │ ├── 閾値調整       │
                    │ └── escaped-bugs記録│
                    └───────────────────┘
```

---

## 必要なファイル・スクリプト

### 新規作成

| ファイル | 用途 |
|---------|------|
| `.claude/escaped-bugs.md` | 漏れたバグの記録 |
| `test_reports/false_positives.log` | 誤検知の記録 |
| `scripts/test_health.sh` | テストシステムヘルスチェック |
| `scripts/test_timing.sh` | 実行時間監視 |
| `scripts/analyze_coverage.sh` | カバレッジ分析 |

### test_health.sh

```bash
#!/bin/bash
# テストシステムのヘルスチェック

echo "=== テストシステムヘルスチェック ==="
echo "日時: $(date)"
echo ""

# 1. テスト数
echo "--- テスト数 ---"
TEST_COUNT=$(cargo test -- --list 2>/dev/null | grep -c "test$")
echo "総テスト数: $TEST_COUNT"

# 2. 実行時間
echo ""
echo "--- 実行時間 ---"
START=$(date +%s.%N)
cargo test --lib > /dev/null 2>&1
END=$(date +%s.%N)
echo "lib tests: $(echo "$END - $START" | bc) sec"

# 3. カバレッジ（最新）
echo ""
echo "--- カバレッジ ---"
if [ -f coverage/tarpaulin-report.json ]; then
    COV=$(jq '.coverage_percentage' coverage/tarpaulin-report.json)
    echo "カバレッジ: $COV%"
else
    echo "カバレッジ: 未計測"
fi

# 4. 誤検知（過去7日）
echo ""
echo "--- 誤検知 ---"
if [ -f test_reports/false_positives.log ]; then
    WEEK_AGO=$(date -d "7 days ago" +%Y-%m-%d)
    FP_COUNT=$(awk -v d="$WEEK_AGO" '$1 >= d' test_reports/false_positives.log | wc -l)
    echo "過去7日の誤検知: $FP_COUNT 件"
else
    echo "誤検知ログなし"
fi

# 5. 漏れバグ（今月）
echo ""
echo "--- 漏れバグ ---"
if [ -f .claude/escaped-bugs.md ]; then
    THIS_MONTH=$(date +%Y-%m)
    ESCAPED=$(grep -c "$THIS_MONTH" .claude/escaped-bugs.md || echo 0)
    echo "今月の漏れバグ: $ESCAPED 件"
else
    echo "漏れバグログなし"
fi

# 6. unwrap残数
echo ""
echo "--- コード品質 ---"
UNWRAP=$(grep -r "\.unwrap()" src/ --include="*.rs" | wc -l)
echo "unwrap()残数: $UNWRAP"

echo ""
echo "=== チェック完了 ==="
```

---

## 実行タイミング

| チェック | 頻度 | トリガー |
|---------|------|---------|
| 漏れバグ記録 | 都度 | 本番でバグ発見時 |
| 誤検知記録 | 都度 | テストが誤検知時 |
| test_health.sh | 週1回 | 月曜朝 or CI |
| カバレッジ計測 | 週1回 | CI or 手動 |
| Geminiレビュー | 月1回 | 月初 |

---

---

## 6. 不要なテストの検出と削除

テストも増えすぎると害になる:

| 害 | 説明 |
|----|------|
| 実行時間増加 | pre-commitが遅くなる |
| メンテナンスコスト | コード変更のたびに直す |
| 偽の安心感 | 意味のないテストが通っても無意味 |
| ノイズ | 重要なテストが埋もれる |

### 不要なテストの種類

| 種類 | 例 | 対処 |
|------|-----|------|
| **重複テスト** | 同じロジックを2箇所でテスト | 片方削除 |
| **トートロジー** | `assert!(true)` 同然のテスト | 削除 |
| **削除済み機能のテスト** | 使われないコードのテスト | 削除 |
| **過剰な境界値テスト** | 同じ分岐を何度もテスト | 代表例のみ残す |
| **実装詳細のテスト** | privateメソッドの細かいテスト | 公開APIテストに統合 |

### 検出方法

```bash
# 1. 実行されないテスト（デッドテスト）
cargo test -- --list 2>&1 | grep "0 tests"

# 2. 常に成功するテスト（mutation testing）
cargo mutants  # 変異テストで無意味なテスト発見

# 3. 遅いテスト
cargo test -- -Z unstable-options --report-time

# 4. 重複カバレッジ
# 同じ行を複数テストがカバー → 重複の可能性
```

### 判断基準

テストを削除していいか:

```
このテストがなくなったら...
  → 別のテストで同じバグを検出できる？ → 削除OK
  → バグが本番まで漏れる？ → 残す
  → リファクタリング時に壊れたことに気づける？ → 残す
```

### 削除プロセス

1. 候補を特定（上記の検出方法）
2. 「このテストは何を守っている？」を確認
3. 他のテストでカバーされていれば削除
4. 削除ログに記録

**記録先**: `.claude/deleted-tests.md`

```markdown
## 削除したテストログ

| 日付 | テスト名 | 削除理由 | 代替テスト |
|------|---------|---------|-----------|
| 2026-01-10 | test_add_one_item | test_add_multiple_itemsと重複 | test_add_multiple_items |
| 2026-01-12 | test_internal_helper | 実装詳細、APIテストでカバー | test_public_api |
```

### 定期レビュー

**頻度**: 月1回（テストシステムレビューと同時）

```bash
./scripts/ask_gemini.sh "
以下のテスト一覧を見て、削除候補を挙げて:

$(cargo test -- --list 2>&1)

判断基準:
- 重複しているもの
- 意味が薄いもの
- 実装詳細に依存しすぎているもの
"
```

---

## 成功指標

| 指標 | 目標 |
|------|------|
| 漏れバグ率 | < 10% (発見バグのうち本番で見つかる割合) |
| 誤検知率 | < 5% (テスト失敗のうち誤検知の割合) |
| Flaky test | 0件 |
| pre-commit時間 | < 5秒 |
| カバレッジ | 40%+ (重要ロジック70%+) |
| テスト/コード比率 | 適正範囲（テスト行数 < 本体行数の50%） |
