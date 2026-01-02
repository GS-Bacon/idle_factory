#!/bin/bash
# Unified test report generator
# Collects results from all testing tools and generates a comprehensive report
# Usage: ./scripts/generate_test_report.sh [--html] [--json]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_DIR="$PROJECT_DIR/test_reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FORMAT="text"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --html)
            OUTPUT_FORMAT="html"
            shift
            ;;
        --json)
            OUTPUT_FORMAT="json"
            shift
            ;;
        *)
            shift
            ;;
    esac
done

mkdir -p "$REPORT_DIR"

echo "=== Generating Unified Test Report ===" >&2
echo "Timestamp: $TIMESTAMP" >&2

# Initialize report data
UNIT_TESTS_PASSED=0
UNIT_TESTS_FAILED=0
E2E_TESTS_PASSED=0
E2E_TESTS_FAILED=0
PROP_TESTS_PASSED=0
PROP_TESTS_FAILED=0
CLIPPY_WARNINGS=0
COVERAGE_PERCENT="N/A"
VLM_ISSUES=0
LOG_SEVERITY="none"
BUILD_STATUS="unknown"

# 1. Run cargo build check
echo "Checking build..." >&2
if cargo build --quiet 2>/dev/null; then
    BUILD_STATUS="pass"
else
    BUILD_STATUS="fail"
fi

# 2. Run unit tests
echo "Running unit tests..." >&2
UNIT_OUTPUT=$(cargo test --lib 2>&1 || true)
UNIT_TESTS_PASSED=$(echo "$UNIT_OUTPUT" | grep -oP '\d+(?= passed)' | tail -1 || echo "0")
UNIT_TESTS_FAILED=$(echo "$UNIT_OUTPUT" | grep -oP '\d+(?= failed)' | tail -1 || echo "0")
[ -z "$UNIT_TESTS_PASSED" ] && UNIT_TESTS_PASSED=0
[ -z "$UNIT_TESTS_FAILED" ] && UNIT_TESTS_FAILED=0

# 3. Run E2E tests
echo "Running E2E tests..." >&2
E2E_OUTPUT=$(cargo test --test e2e_test 2>&1 || true)
E2E_TESTS_PASSED=$(echo "$E2E_OUTPUT" | grep -oP '\d+(?= passed)' | tail -1 || echo "0")
E2E_TESTS_FAILED=$(echo "$E2E_OUTPUT" | grep -oP '\d+(?= failed)' | tail -1 || echo "0")
[ -z "$E2E_TESTS_PASSED" ] && E2E_TESTS_PASSED=0
[ -z "$E2E_TESTS_FAILED" ] && E2E_TESTS_FAILED=0

# 4. Run proptest if available
echo "Running property tests..." >&2
PROP_OUTPUT=$(cargo test --test proptest_invariants 2>&1 || true)
PROP_TESTS_PASSED=$(echo "$PROP_OUTPUT" | grep -oP '\d+(?= passed)' | tail -1 || echo "0")
PROP_TESTS_FAILED=$(echo "$PROP_OUTPUT" | grep -oP '\d+(?= failed)' | tail -1 || echo "0")
[ -z "$PROP_TESTS_PASSED" ] && PROP_TESTS_PASSED=0
[ -z "$PROP_TESTS_FAILED" ] && PROP_TESTS_FAILED=0

# 5. Run clippy
echo "Running clippy..." >&2
CLIPPY_OUTPUT=$(cargo clippy 2>&1 || true)
CLIPPY_WARNINGS=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:" | tr -d ' ' || echo "0")
[ -z "$CLIPPY_WARNINGS" ] && CLIPPY_WARNINGS=0

# 6. Check coverage report if exists
if [ -f "$REPORT_DIR/coverage/coverage.json" ]; then
    COVERAGE_PERCENT=$(jq -r '.coverage_percent // "N/A"' "$REPORT_DIR/coverage/coverage.json" 2>/dev/null || echo "N/A")
fi

# 7. Check VLM reports if exist
VLM_DIR="$REPORT_DIR/vlm"
if [ -d "$VLM_DIR" ]; then
    VLM_ISSUES=$(find "$VLM_DIR" -name "*.json" -exec jq -r 'select(.identical == false) | 1' {} \; 2>/dev/null | wc -l || echo "0")
fi

# 8. Check log analysis if exists
LOG_ANALYSIS_DIR="$REPORT_DIR/log_analysis"
if [ -d "$LOG_ANALYSIS_DIR" ]; then
    LATEST_LOG=$(ls -t "$LOG_ANALYSIS_DIR"/*.json 2>/dev/null | head -1)
    if [ -n "$LATEST_LOG" ]; then
        LOG_SEVERITY=$(jq -r '.severity // "none"' "$LATEST_LOG" 2>/dev/null || echo "none")
    fi
fi

# Calculate overall status
TOTAL_PASSED=$((UNIT_TESTS_PASSED + E2E_TESTS_PASSED + PROP_TESTS_PASSED))
TOTAL_FAILED=$((UNIT_TESTS_FAILED + E2E_TESTS_FAILED + PROP_TESTS_FAILED))

if [ "$BUILD_STATUS" = "fail" ]; then
    OVERALL_STATUS="FAIL"
elif [ "$TOTAL_FAILED" -gt 0 ]; then
    OVERALL_STATUS="FAIL"
elif [ "$CLIPPY_WARNINGS" -gt 0 ]; then
    OVERALL_STATUS="WARN"
elif [ "$VLM_ISSUES" -gt 0 ]; then
    OVERALL_STATUS="WARN"
else
    OVERALL_STATUS="PASS"
fi

# Generate report based on format
case "$OUTPUT_FORMAT" in
    json)
        REPORT=$(cat <<EOF
{
  "timestamp": "$TIMESTAMP",
  "overall_status": "$OVERALL_STATUS",
  "build": {
    "status": "$BUILD_STATUS"
  },
  "tests": {
    "unit": {
      "passed": $UNIT_TESTS_PASSED,
      "failed": $UNIT_TESTS_FAILED
    },
    "e2e": {
      "passed": $E2E_TESTS_PASSED,
      "failed": $E2E_TESTS_FAILED
    },
    "proptest": {
      "passed": $PROP_TESTS_PASSED,
      "failed": $PROP_TESTS_FAILED
    },
    "total_passed": $TOTAL_PASSED,
    "total_failed": $TOTAL_FAILED
  },
  "quality": {
    "clippy_warnings": $CLIPPY_WARNINGS,
    "coverage_percent": "$COVERAGE_PERCENT",
    "vlm_issues": $VLM_ISSUES,
    "log_severity": "$LOG_SEVERITY"
  }
}
EOF
)
        REPORT_FILE="$REPORT_DIR/report_$TIMESTAMP.json"
        echo "$REPORT" > "$REPORT_FILE"
        echo "$REPORT"
        ;;
    html)
        STATUS_COLOR="green"
        [ "$OVERALL_STATUS" = "WARN" ] && STATUS_COLOR="orange"
        [ "$OVERALL_STATUS" = "FAIL" ] && STATUS_COLOR="red"

        REPORT=$(cat <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>Test Report - $TIMESTAMP</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; max-width: 800px; margin: 40px auto; padding: 20px; }
        h1 { border-bottom: 2px solid #333; padding-bottom: 10px; }
        .status { display: inline-block; padding: 8px 16px; border-radius: 4px; font-weight: bold; }
        .pass { background: #d4edda; color: #155724; }
        .warn { background: #fff3cd; color: #856404; }
        .fail { background: #f8d7da; color: #721c24; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background: #f5f5f5; }
        .section { margin: 30px 0; }
        .metric { font-size: 24px; font-weight: bold; }
    </style>
</head>
<body>
    <h1>Idle Factory Test Report</h1>
    <p>Generated: $TIMESTAMP</p>
    <p>Overall Status: <span class="status ${OVERALL_STATUS,,}">$OVERALL_STATUS</span></p>

    <div class="section">
        <h2>Build Status</h2>
        <p class="metric">$BUILD_STATUS</p>
    </div>

    <div class="section">
        <h2>Test Results</h2>
        <table>
            <tr><th>Category</th><th>Passed</th><th>Failed</th></tr>
            <tr><td>Unit Tests</td><td>$UNIT_TESTS_PASSED</td><td>$UNIT_TESTS_FAILED</td></tr>
            <tr><td>E2E Tests</td><td>$E2E_TESTS_PASSED</td><td>$E2E_TESTS_FAILED</td></tr>
            <tr><td>Property Tests</td><td>$PROP_TESTS_PASSED</td><td>$PROP_TESTS_FAILED</td></tr>
            <tr><th>Total</th><th>$TOTAL_PASSED</th><th>$TOTAL_FAILED</th></tr>
        </table>
    </div>

    <div class="section">
        <h2>Code Quality</h2>
        <table>
            <tr><th>Metric</th><th>Value</th></tr>
            <tr><td>Clippy Warnings</td><td>$CLIPPY_WARNINGS</td></tr>
            <tr><td>Code Coverage</td><td>$COVERAGE_PERCENT</td></tr>
            <tr><td>VLM Issues</td><td>$VLM_ISSUES</td></tr>
            <tr><td>Log Analysis Severity</td><td>$LOG_SEVERITY</td></tr>
        </table>
    </div>
</body>
</html>
EOF
)
        REPORT_FILE="$REPORT_DIR/report_$TIMESTAMP.html"
        echo "$REPORT" > "$REPORT_FILE"
        echo "HTML report saved: $REPORT_FILE" >&2
        echo "$REPORT_FILE"
        ;;
    *)
        # Text format
        echo ""
        echo "=============================================="
        echo "  IDLE FACTORY TEST REPORT"
        echo "  $TIMESTAMP"
        echo "=============================================="
        echo ""
        echo "Overall Status: $OVERALL_STATUS"
        echo ""
        echo "--- Build ---"
        echo "  Status: $BUILD_STATUS"
        echo ""
        echo "--- Tests ---"
        printf "  %-20s %5s passed, %5s failed\n" "Unit Tests:" "$UNIT_TESTS_PASSED" "$UNIT_TESTS_FAILED"
        printf "  %-20s %5s passed, %5s failed\n" "E2E Tests:" "$E2E_TESTS_PASSED" "$E2E_TESTS_FAILED"
        printf "  %-20s %5s passed, %5s failed\n" "Property Tests:" "$PROP_TESTS_PASSED" "$PROP_TESTS_FAILED"
        echo "  ----------------------------------------"
        printf "  %-20s %5s passed, %5s failed\n" "TOTAL:" "$TOTAL_PASSED" "$TOTAL_FAILED"
        echo ""
        echo "--- Code Quality ---"
        echo "  Clippy Warnings: $CLIPPY_WARNINGS"
        echo "  Code Coverage: $COVERAGE_PERCENT"
        echo "  VLM Issues: $VLM_ISSUES"
        echo "  Log Severity: $LOG_SEVERITY"
        echo ""
        echo "=============================================="

        # Save text report
        REPORT_FILE="$REPORT_DIR/report_$TIMESTAMP.txt"
        {
            echo "IDLE FACTORY TEST REPORT"
            echo "Timestamp: $TIMESTAMP"
            echo "Overall Status: $OVERALL_STATUS"
            echo ""
            echo "Build: $BUILD_STATUS"
            echo "Unit Tests: $UNIT_TESTS_PASSED passed, $UNIT_TESTS_FAILED failed"
            echo "E2E Tests: $E2E_TESTS_PASSED passed, $E2E_TESTS_FAILED failed"
            echo "Property Tests: $PROP_TESTS_PASSED passed, $PROP_TESTS_FAILED failed"
            echo "Clippy Warnings: $CLIPPY_WARNINGS"
            echo "Coverage: $COVERAGE_PERCENT"
            echo "VLM Issues: $VLM_ISSUES"
            echo "Log Severity: $LOG_SEVERITY"
        } > "$REPORT_FILE"
        echo "Report saved: $REPORT_FILE" >&2
        ;;
esac

# Exit with appropriate code
case "$OVERALL_STATUS" in
    FAIL) exit 1 ;;
    WARN) exit 0 ;;
    PASS) exit 0 ;;
esac
