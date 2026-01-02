#!/bin/bash
# テストシステムのヘルスチェック
# Usage: ./scripts/test_health.sh [--json]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

JSON_OUTPUT=false
[ "$1" = "--json" ] && JSON_OUTPUT=true

echo "=== テストシステムヘルスチェック ===" >&2
echo "日時: $(date)" >&2
echo "" >&2

# 1. テスト数
echo "--- テスト数 ---" >&2
LIB_COUNT=$(cargo test --lib -- --list 2>/dev/null | grep -c ": test$" || echo "0")
E2E_COUNT=$(cargo test --test e2e_test -- --list 2>/dev/null | grep -c ": test$" || echo "0")
PROP_COUNT=$(cargo test --test proptest_invariants -- --list 2>/dev/null | grep -c ": test$" || echo "0")
TOTAL_COUNT=$((LIB_COUNT + E2E_COUNT + PROP_COUNT))
echo "lib: $LIB_COUNT, e2e: $E2E_COUNT, proptest: $PROP_COUNT, 合計: $TOTAL_COUNT" >&2

# 2. 実行時間
echo "" >&2
echo "--- 実行時間 ---" >&2
START=$(date +%s.%N)
cargo test --lib --quiet >/dev/null 2>&1 || true
END=$(date +%s.%N)
LIB_TIME=$(echo "$END - $START" | bc)
echo "lib tests: ${LIB_TIME} sec" >&2

START=$(date +%s.%N)
cargo test --test e2e_test --quiet >/dev/null 2>&1 || true
END=$(date +%s.%N)
E2E_TIME=$(echo "$END - $START" | bc)
echo "e2e tests: ${E2E_TIME} sec" >&2

# 3. カバレッジ（最新）
echo "" >&2
echo "--- カバレッジ ---" >&2
COV_PERCENT="N/A"
if [ -f "$PROJECT_DIR/coverage/tarpaulin-report.json" ]; then
    COV_PERCENT=$(jq -r '.coverage_percentage // "N/A"' "$PROJECT_DIR/coverage/tarpaulin-report.json" 2>/dev/null || echo "N/A")
fi
echo "カバレッジ: $COV_PERCENT%" >&2

# 4. 誤検知（過去7日）
echo "" >&2
echo "--- 誤検知 ---" >&2
FP_COUNT=0
if [ -f "$PROJECT_DIR/test_reports/false_positives.log" ]; then
    WEEK_AGO=$(date -d "7 days ago" +%Y-%m-%d 2>/dev/null || date -v-7d +%Y-%m-%d 2>/dev/null || echo "2000-01-01")
    FP_COUNT=$(grep -v "^#" "$PROJECT_DIR/test_reports/false_positives.log" | awk -v d="$WEEK_AGO" '$1 >= d' | wc -l | tr -d ' ' || echo "0")
fi
echo "過去7日の誤検知: $FP_COUNT 件" >&2

# 5. 漏れバグ（今月）
echo "" >&2
echo "--- 漏れバグ ---" >&2
ESCAPED_COUNT=0
if [ -f "$PROJECT_DIR/.claude/escaped-bugs.md" ]; then
    THIS_MONTH=$(date +%Y-%m)
    ESCAPED_COUNT=$(grep -c "$THIS_MONTH" "$PROJECT_DIR/.claude/escaped-bugs.md" 2>/dev/null | tr -d ' ' || echo "0")
    [ -z "$ESCAPED_COUNT" ] && ESCAPED_COUNT=0
fi
echo "今月の漏れバグ: $ESCAPED_COUNT 件" >&2

# 6. コード品質
echo "" >&2
echo "--- コード品質 ---" >&2
UNWRAP_COUNT=$(grep -r "\.unwrap()" "$PROJECT_DIR/src/" --include="*.rs" 2>/dev/null | wc -l || echo "0")
EXPECT_COUNT=$(grep -r "\.expect(" "$PROJECT_DIR/src/" --include="*.rs" 2>/dev/null | wc -l || echo "0")
CODE_LINES=$(find "$PROJECT_DIR/src" -name "*.rs" -exec cat {} \; 2>/dev/null | wc -l || echo "0")
TEST_LINES=$(find "$PROJECT_DIR/tests" -name "*.rs" -exec cat {} \; 2>/dev/null | wc -l || echo "0")
echo "unwrap(): $UNWRAP_COUNT, expect(): $EXPECT_COUNT" >&2
echo "コード行数: $CODE_LINES, テスト行数: $TEST_LINES" >&2

# 7. Clippy警告
echo "" >&2
echo "--- Clippy ---" >&2
CLIPPY_WARNINGS=$(cargo clippy 2>&1 | grep -c "warning:" | tr -d ' ' || echo "0")
[ -z "$CLIPPY_WARNINGS" ] && CLIPPY_WARNINGS=0
echo "警告: $CLIPPY_WARNINGS 件" >&2

echo "" >&2
echo "=== チェック完了 ===" >&2

# ヘルス判定
HEALTH="good"
ISSUES=()

if [ "$CLIPPY_WARNINGS" -gt 0 ]; then
    HEALTH="warn"
    ISSUES+=("clippy警告あり")
fi

if [ "$UNWRAP_COUNT" -gt 5 ]; then
    HEALTH="warn"
    ISSUES+=("unwrap()が多い")
fi

LIB_TIME_INT=${LIB_TIME%.*}
if [ "${LIB_TIME_INT:-0}" -gt 5 ]; then
    HEALTH="warn"
    ISSUES+=("テストが遅い")
fi

if [ "$FP_COUNT" -gt 3 ]; then
    HEALTH="warn"
    ISSUES+=("誤検知が多い")
fi

if [ "$ESCAPED_COUNT" -gt 2 ]; then
    HEALTH="warn"
    ISSUES+=("漏れバグが多い")
fi

# JSON出力
if [ "$JSON_OUTPUT" = true ]; then
    cat <<EOF
{
  "timestamp": "$(date -Iseconds)",
  "health": "$HEALTH",
  "tests": {
    "lib": $LIB_COUNT,
    "e2e": $E2E_COUNT,
    "proptest": $PROP_COUNT,
    "total": $TOTAL_COUNT
  },
  "timing": {
    "lib_sec": $LIB_TIME,
    "e2e_sec": $E2E_TIME
  },
  "coverage_percent": "$COV_PERCENT",
  "false_positives_7d": $FP_COUNT,
  "escaped_bugs_month": $ESCAPED_COUNT,
  "code_quality": {
    "unwrap_count": $UNWRAP_COUNT,
    "expect_count": $EXPECT_COUNT,
    "clippy_warnings": $CLIPPY_WARNINGS,
    "code_lines": $CODE_LINES,
    "test_lines": $TEST_LINES
  },
  "issues": $(printf '%s\n' "${ISSUES[@]}" | jq -R . | jq -s .)
}
EOF
else
    echo ""
    echo "=== 総合評価: $HEALTH ==="
    if [ ${#ISSUES[@]} -gt 0 ]; then
        echo "問題点:"
        for issue in "${ISSUES[@]}"; do
            echo "  - $issue"
        done
    fi
fi
