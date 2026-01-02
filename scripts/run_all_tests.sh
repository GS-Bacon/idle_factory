#!/bin/bash
# Master test runner - runs all tests and generates comprehensive report
# Usage: ./scripts/run_all_tests.sh [--quick] [--full] [--vlm]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_DIR="$PROJECT_DIR/test_reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Default options
RUN_VLM=false
QUICK_MODE=false
FULL_MODE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --full)
            FULL_MODE=true
            shift
            ;;
        --vlm)
            RUN_VLM=true
            shift
            ;;
        *)
            shift
            ;;
    esac
done

mkdir -p "$REPORT_DIR"

echo "=============================================="
echo "  IDLE FACTORY - COMPREHENSIVE TEST SUITE"
echo "  Started: $(date)"
echo "=============================================="
echo ""

FAILURES=0

# Step 1: Build check
echo "[1/7] Checking build..."
if cargo build --quiet 2>/dev/null; then
    echo "  ✓ Build successful"
else
    echo "  ✗ Build failed"
    FAILURES=$((FAILURES + 1))
fi
echo ""

# Step 2: Clippy
echo "[2/7] Running clippy..."
CLIPPY_WARNINGS=$(cargo clippy 2>&1 | grep -c "warning:" | tr -d ' ' || echo "0")
[ -z "$CLIPPY_WARNINGS" ] && CLIPPY_WARNINGS=0
if [ "$CLIPPY_WARNINGS" -eq 0 ]; then
    echo "  ✓ No clippy warnings"
else
    echo "  ⚠ $CLIPPY_WARNINGS clippy warnings"
fi
echo ""

# Step 3: Unit tests
echo "[3/7] Running unit tests..."
if cargo test --lib --quiet 2>/dev/null; then
    echo "  ✓ Unit tests passed"
else
    echo "  ✗ Unit tests failed"
    FAILURES=$((FAILURES + 1))
fi
echo ""

# Step 4: E2E tests
echo "[4/7] Running E2E tests..."
if cargo test --test e2e_test --quiet 2>/dev/null; then
    echo "  ✓ E2E tests passed"
else
    echo "  ✗ E2E tests failed"
    FAILURES=$((FAILURES + 1))
fi
echo ""

# Step 5: Property tests
echo "[5/7] Running property tests..."
if cargo test --test proptest_invariants --quiet 2>/dev/null; then
    echo "  ✓ Property tests passed"
else
    echo "  ✗ Property tests failed"
    FAILURES=$((FAILURES + 1))
fi
echo ""

# Step 6: Log analysis (if logs exist)
echo "[6/7] Analyzing logs..."
if [ -d "$PROJECT_DIR/logs" ] && ls "$PROJECT_DIR/logs"/*.log >/dev/null 2>&1; then
    if "$SCRIPT_DIR/analyze_logs.sh" --json > "$REPORT_DIR/log_analysis_$TIMESTAMP.json" 2>/dev/null; then
        LOG_SEVERITY=$(jq -r '.severity // "none"' "$REPORT_DIR/log_analysis_$TIMESTAMP.json" 2>/dev/null || echo "none")
        echo "  ✓ Log analysis complete (severity: $LOG_SEVERITY)"
    else
        echo "  ⚠ Log analysis failed"
    fi
else
    echo "  - No logs to analyze"
fi
echo ""

# Step 7: Visual comparison (smart_compare)
echo "[7/7] Visual checks..."
BASELINE_DIR="$PROJECT_DIR/screenshots/baseline"
VERIFY_DIR="$PROJECT_DIR/screenshots/verify"
SMART_COMPARE="$SCRIPT_DIR/vlm_check/smart_compare.py"

if [ "$RUN_VLM" = true ] || [ "$FULL_MODE" = true ]; then
    if [ -d "$BASELINE_DIR" ] && [ -d "$VERIFY_DIR" ]; then
        echo "  Running smart_compare..."
        visual_passed=0
        visual_failed=0
        for baseline in "$BASELINE_DIR"/*.png; do
            [ -f "$baseline" ] || continue
            name=$(basename "$baseline")
            current="$VERIFY_DIR/$name"
            [ -f "$current" ] || continue

            result=$(python3 "$SMART_COMPARE" "$baseline" "$current" --json 2>/dev/null)
            severity=$(echo "$result" | jq -r '.severity // "error"')

            if [ "$severity" = "none" ] || [ "$severity" = "minor" ]; then
                visual_passed=$((visual_passed + 1))
            else
                visual_failed=$((visual_failed + 1))
                echo "    ❌ $name: $severity"
            fi
        done
        echo "  Visual: $visual_passed passed, $visual_failed failed"
        if [ $visual_failed -eq 0 ]; then
            echo "  ✓ Visual check passed"
        else
            echo "  ⚠ Visual check found issues"
        fi
    else
        echo "  - No baseline screenshots (run: ./scripts/e2e-quick.sh basic && ./scripts/e2e-quick.sh save-baseline)"
    fi
else
    echo "  - Skipped (use --vlm or --full to enable)"
fi
echo ""

# Generate final report
echo "Generating test report..."
"$SCRIPT_DIR/generate_test_report.sh" --json > "$REPORT_DIR/final_report_$TIMESTAMP.json" 2>/dev/null || true
echo ""

# Summary
echo "=============================================="
if [ "$FAILURES" -eq 0 ]; then
    echo "  ✓ ALL TESTS PASSED"
else
    echo "  ✗ $FAILURES TEST SUITE(S) FAILED"
fi
echo "  Report: $REPORT_DIR/final_report_$TIMESTAMP.json"
echo "  Completed: $(date)"
echo "=============================================="

exit $FAILURES
