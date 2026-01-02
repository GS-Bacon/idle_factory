#!/bin/bash
# AI-powered game log analysis script
# Uses Gemini to detect problems in game logs
# Usage: ./scripts/analyze_logs.sh [log_file] [--json]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="${SCRIPT_DIR}/../logs"
REPORT_DIR="${SCRIPT_DIR}/../test_reports/log_analysis"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Parse arguments
LOG_FILE=""
JSON_OUTPUT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        *)
            LOG_FILE="$1"
            shift
            ;;
    esac
done

# Find log file to analyze
if [ -z "$LOG_FILE" ]; then
    # Use most recent log file
    LOG_FILE=$(ls -t "$LOG_DIR"/*.log 2>/dev/null | head -1)
    if [ -z "$LOG_FILE" ]; then
        echo "Error: No log files found in $LOG_DIR"
        exit 1
    fi
fi

if [ ! -f "$LOG_FILE" ]; then
    echo "Error: Log file not found: $LOG_FILE"
    exit 1
fi

mkdir -p "$REPORT_DIR"

# Truncate log if too large (keep last 500 lines for analysis)
LOG_CONTENT=$(tail -n 500 "$LOG_FILE")
LOG_LINES=$(wc -l < "$LOG_FILE")

echo "=== AI Log Analysis ===" >&2
echo "Analyzing: $LOG_FILE ($LOG_LINES lines total, using last 500)" >&2

# Create analysis prompt
PROMPT="以下のゲームログを分析して、問題を検出してください。

確認項目:
1. エラーパターン - ERROR, WARN, panic, failedなどのキーワード
2. 警告の頻度 - 同じ警告が繰り返されていないか
3. 異常な値 - 座標が範囲外、カウントが負数、NaN/Infなど
4. 繰り返しパターン - 無限ループの兆候、同じ処理の連続実行
5. タイミング異常 - 処理時間が異常に長い、フレーム落ちの兆候
6. リソース問題 - メモリ、ファイルハンドル、ネットワーク関連のエラー

出力形式 (JSON):
{
  \"summary\": \"全体の概要（1-2文）\",
  \"severity\": \"none|low|medium|high|critical\",
  \"issues\": [
    {
      \"type\": \"error|warning|anomaly|pattern\",
      \"description\": \"問題の説明\",
      \"line_examples\": [\"該当行の例\"],
      \"recommendation\": \"対処方法\"
    }
  ],
  \"statistics\": {
    \"total_lines\": $LOG_LINES,
    \"error_count\": 0,
    \"warning_count\": 0,
    \"analyzed_lines\": 500
  }
}

ログ内容:
$LOG_CONTENT"

# Check if Gemini is available
if command -v gemini >/dev/null 2>&1; then
    echo "Using Gemini for analysis..." >&2

    RESULT=$(echo "$PROMPT" | timeout 60 gemini 2>/dev/null || echo '{"error": "Gemini analysis failed or timed out"}')
else
    # Fallback: Basic pattern matching
    echo "Gemini not available, using basic pattern matching..." >&2

    ERROR_COUNT=$(echo "$LOG_CONTENT" | grep -ci "error\|panic\|failed" || true)
    WARN_COUNT=$(echo "$LOG_CONTENT" | grep -ci "warn" || true)

    if [ "$ERROR_COUNT" -gt 0 ]; then
        SEVERITY="high"
    elif [ "$WARN_COUNT" -gt 10 ]; then
        SEVERITY="medium"
    elif [ "$WARN_COUNT" -gt 0 ]; then
        SEVERITY="low"
    else
        SEVERITY="none"
    fi

    # Extract sample errors
    ERRORS=$(echo "$LOG_CONTENT" | grep -i "error\|panic\|failed" | head -5 | jq -R -s 'split("\n") | map(select(length > 0))')
    WARNINGS=$(echo "$LOG_CONTENT" | grep -i "warn" | head -5 | jq -R -s 'split("\n") | map(select(length > 0))')

    RESULT=$(cat <<EOF
{
  "summary": "Basic analysis: $ERROR_COUNT errors, $WARN_COUNT warnings detected",
  "severity": "$SEVERITY",
  "issues": [],
  "statistics": {
    "total_lines": $LOG_LINES,
    "error_count": $ERROR_COUNT,
    "warning_count": $WARN_COUNT,
    "analyzed_lines": 500
  },
  "sample_errors": $ERRORS,
  "sample_warnings": $WARNINGS,
  "note": "Full AI analysis requires Gemini CLI"
}
EOF
)
fi

# Save report
REPORT_FILE="$REPORT_DIR/analysis_$TIMESTAMP.json"
echo "$RESULT" > "$REPORT_FILE"
echo "Report saved: $REPORT_FILE" >&2

# Output result
if [ "$JSON_OUTPUT" = true ]; then
    echo "$RESULT"
else
    # Pretty print summary
    echo ""
    echo "=== Analysis Results ==="

    SUMMARY=$(echo "$RESULT" | jq -r '.summary // "No summary available"' 2>/dev/null || echo "Parse error")
    SEVERITY=$(echo "$RESULT" | jq -r '.severity // "unknown"' 2>/dev/null || echo "unknown")

    echo "Summary: $SUMMARY"
    echo "Severity: $SEVERITY"

    # Show issues if any
    ISSUE_COUNT=$(echo "$RESULT" | jq '.issues | length' 2>/dev/null || echo "0")
    if [ "$ISSUE_COUNT" -gt 0 ]; then
        echo ""
        echo "Issues found:"
        echo "$RESULT" | jq -r '.issues[] | "  [\(.type)] \(.description)"' 2>/dev/null || true
    fi

    # Show statistics
    echo ""
    echo "Statistics:"
    echo "$RESULT" | jq -r '.statistics | "  Lines: \(.total_lines), Errors: \(.error_count), Warnings: \(.warning_count)"' 2>/dev/null || true

    # Exit code based on severity
    case "$SEVERITY" in
        critical|high)
            exit 2
            ;;
        medium)
            exit 1
            ;;
        *)
            exit 0
            ;;
    esac
fi
