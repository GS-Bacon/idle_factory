#!/bin/bash
# Test JSON Report Generator
# Runs all tests and generates comprehensive JSON report with timing and trends
#
# Usage: ./scripts/test_json_report.sh [quick|full]

set -e

export DISPLAY=${DISPLAY:-:10}
MODE=${1:-quick}
GAME_DIR="/home/bacon/idle_factory"
REPORT_DIR="$GAME_DIR/test_reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$REPORT_DIR/test_report_$TIMESTAMP.json"
HISTORY_FILE="$REPORT_DIR/test_history.json"

mkdir -p "$REPORT_DIR"

cd "$GAME_DIR"

echo "═══════════════════════════════════════════════"
echo "  Generating Test JSON Report"
echo "  Mode: $MODE"
echo "  Date: $(date)"
echo "═══════════════════════════════════════════════"

# Kill any existing game
pkill -x idle_factory 2>/dev/null || true
sleep 1

# Initialize results array
declare -A TEST_RESULTS
declare -A TEST_TIMES
TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_SKIPPED=0

# Run a test and capture result
run_test() {
    local name="$1"
    local cmd="$2"
    local skip="${3:-false}"

    echo "Running: $name"

    if [ "$skip" = "true" ]; then
        TEST_RESULTS["$name"]="skipped"
        TEST_TIMES["$name"]=0
        TOTAL_SKIPPED=$((TOTAL_SKIPPED + 1))
        return 0
    fi

    local start_time=$(date +%s.%N)

    if eval "$cmd" >/dev/null 2>&1; then
        TEST_RESULTS["$name"]="passed"
        TOTAL_PASSED=$((TOTAL_PASSED + 1))
    else
        TEST_RESULTS["$name"]="failed"
        TOTAL_FAILED=$((TOTAL_FAILED + 1))
    fi

    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc)
    TEST_TIMES["$name"]=$duration

    echo "  Result: ${TEST_RESULTS[$name]} (${duration}s)"

    # Cleanup between tests
    pkill -x idle_factory 2>/dev/null || true
    sleep 0.5
}

# Run tests
START_TIME=$(date +%s)

# Test 1: Build
run_test "build" "cargo build 2>&1"

# Test 2: Unit Tests
run_test "unit_tests" "cargo test --lib 2>&1"

# Test 3: Integration Tests
run_test "integration_tests" "cargo test --test '*' 2>&1"

# Test 4: Clippy
run_test "clippy" "cargo clippy 2>&1"

# Test 5: SSIM Tests
run_test "ssim_tests" "cargo test ssim 2>&1"

# Test 6: Fuzz Tests
run_test "fuzz_tests" "cargo test fuzz_save 2>&1"

# Test 7: Smoke Test
run_test "smoke_test" "./scripts/smoke_test.sh 10 2>&1"

# Test 8: E2E Quick
run_test "e2e_quick" "./scripts/e2e-quick.sh 2>&1"

# Full mode tests
if [ "$MODE" = "full" ]; then
    # Test 9: Visual Regression
    run_test "visual_regression" "./scripts/visual_regression.sh check 2>&1"

    # Test 10: Fuzz Test (script)
    run_test "fuzz_script" "./scripts/fuzz_test.sh 30 0.1 2>&1"

    # Test 11: Scenario Test
    run_test "scenario_test" "./scripts/scenario_test.sh 60 2>&1"
else
    run_test "visual_regression" "" "true"
    run_test "fuzz_script" "" "true"
    run_test "scenario_test" "" "true"
fi

END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))

# Get code metrics
echo "Gathering metrics..."
CODE_LINES=$(find src -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
TEST_LINES=$(find tests -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
UNWRAP_COUNT=$(grep -r "\.unwrap()" src/ 2>/dev/null | wc -l || echo 0)
CLIPPY_WARNINGS=$(cargo clippy 2>&1 | grep -c "warning:" || echo 0)

# Test counts
UNIT_TEST_COUNT=$(cargo test --lib 2>&1 | grep -oP '\d+ passed' | head -1 | grep -oP '\d+' || echo 0)
INTEGRATION_TEST_COUNT=$(cargo test --test '*' 2>&1 | grep -oP '\d+ passed' | head -1 | grep -oP '\d+' || echo 0)

# Generate JSON report
cat << EOF > "$REPORT_FILE"
{
    "timestamp": "$TIMESTAMP",
    "mode": "$MODE",
    "duration_seconds": $TOTAL_TIME,
    "summary": {
        "passed": $TOTAL_PASSED,
        "failed": $TOTAL_FAILED,
        "skipped": $TOTAL_SKIPPED,
        "total": $((TOTAL_PASSED + TOTAL_FAILED + TOTAL_SKIPPED))
    },
    "tests": {
$(for name in "${!TEST_RESULTS[@]}"; do
    echo "        \"$name\": {"
    echo "            \"status\": \"${TEST_RESULTS[$name]}\","
    echo "            \"duration\": ${TEST_TIMES[$name]}"
    echo "        },"
done | sed '$ s/,$//')
    },
    "metrics": {
        "code_lines": $CODE_LINES,
        "test_lines": $TEST_LINES,
        "unit_tests": $UNIT_TEST_COUNT,
        "integration_tests": $INTEGRATION_TEST_COUNT,
        "unwrap_count": $UNWRAP_COUNT,
        "clippy_warnings": $CLIPPY_WARNINGS
    },
    "result": "$([ $TOTAL_FAILED -eq 0 ] && echo "PASS" || echo "FAIL")"
}
EOF

echo ""
echo "═══════════════════════════════════════════════"
echo "  Test Report Generated"
echo "═══════════════════════════════════════════════"
echo "Passed:  $TOTAL_PASSED"
echo "Failed:  $TOTAL_FAILED"
echo "Skipped: $TOTAL_SKIPPED"
echo "Duration: ${TOTAL_TIME}s"
echo ""
echo "Report: $REPORT_FILE"

# Update history file for trends
if [ -f "$HISTORY_FILE" ]; then
    # Read existing history and append new entry
    python3 << PYEOF
import json

try:
    with open("$HISTORY_FILE", "r") as f:
        history = json.load(f)
except:
    history = {"reports": []}

# Load new report
with open("$REPORT_FILE", "r") as f:
    new_report = json.load(f)

# Add to history (keep last 100)
history["reports"].append({
    "timestamp": new_report["timestamp"],
    "passed": new_report["summary"]["passed"],
    "failed": new_report["summary"]["failed"],
    "duration": new_report["duration_seconds"],
    "code_lines": new_report["metrics"]["code_lines"],
    "unit_tests": new_report["metrics"]["unit_tests"]
})
history["reports"] = history["reports"][-100:]

with open("$HISTORY_FILE", "w") as f:
    json.dump(history, f, indent=2)

print("History updated with {} entries".format(len(history["reports"])))
PYEOF
else
    # Create new history file
    python3 << PYEOF
import json

with open("$REPORT_FILE", "r") as f:
    new_report = json.load(f)

history = {
    "reports": [{
        "timestamp": new_report["timestamp"],
        "passed": new_report["summary"]["passed"],
        "failed": new_report["summary"]["failed"],
        "duration": new_report["duration_seconds"],
        "code_lines": new_report["metrics"]["code_lines"],
        "unit_tests": new_report["metrics"]["unit_tests"]
    }]
}

with open("$HISTORY_FILE", "w") as f:
    json.dump(history, f, indent=2)

print("History file created")
PYEOF
fi

# Show trends if history exists
if [ -f "$HISTORY_FILE" ]; then
    echo ""
    echo "=== Trends (last 5 runs) ==="
    python3 << PYEOF
import json

with open("$HISTORY_FILE", "r") as f:
    history = json.load(f)

reports = history["reports"][-5:]
for r in reports:
    status = "PASS" if r["failed"] == 0 else "FAIL"
    print(f"  {r['timestamp']}: {status} ({r['passed']}/{r['passed']+r['failed']}) in {r['duration']}s")
PYEOF
fi

if [ $TOTAL_FAILED -gt 0 ]; then
    exit 1
fi

exit 0
