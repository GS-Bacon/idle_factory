#!/bin/bash
# 統合テストスクリプト - 全自動バグ検出
# Usage: ./scripts/test_all.sh [quick|full]

# Don't exit on error - we handle failures ourselves
set +e

export DISPLAY=${DISPLAY:-:10}
MODE=${1:-quick}
GAME_DIR="/home/bacon/idle_factory"
REPORT_DIR="$GAME_DIR/test_reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$REPORT_DIR/report_$TIMESTAMP.txt"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$REPORT_DIR"

log() { echo -e "${GREEN}[TEST]${NC} $1" | tee -a "$REPORT_FILE"; }
warn() { echo -e "${YELLOW}[TEST]${NC} $1" | tee -a "$REPORT_FILE"; }
err() { echo -e "${RED}[TEST]${NC} $1" | tee -a "$REPORT_FILE"; }
header() { echo -e "${BLUE}[TEST]${NC} $1" | tee -a "$REPORT_FILE"; }

PASSED=0
FAILED=0
SKIPPED=0

run_test() {
    local name="$1"
    local cmd="$2"
    local skip="${3:-false}"

    header "━━━ $name ━━━"

    if [ "$skip" = "true" ]; then
        warn "SKIPPED: $name"
        SKIPPED=$((SKIPPED + 1))
        return 0
    fi

    # Clean up before each test (match binary only)
    pkill -x idle_factory 2>/dev/null || true
    sleep 1

    if eval "$cmd" >> "$REPORT_FILE" 2>&1; then
        log "✓ $name: PASSED"
        PASSED=$((PASSED + 1))
    else
        err "✗ $name: FAILED"
        FAILED=$((FAILED + 1))
    fi

    # Clean up after each test
    pkill -x idle_factory 2>/dev/null || true
    sleep 1
    echo "" | tee -a "$REPORT_FILE"
}

cd "$GAME_DIR"

echo "═══════════════════════════════════════════════" | tee "$REPORT_FILE"
echo "  Idle Factory 統合テストレポート" | tee -a "$REPORT_FILE"
echo "  Mode: $MODE" | tee -a "$REPORT_FILE"
echo "  Date: $(date)" | tee -a "$REPORT_FILE"
echo "═══════════════════════════════════════════════" | tee -a "$REPORT_FILE"
echo "" | tee -a "$REPORT_FILE"

# Kill any existing game (match binary, not scripts)
pkill -x idle_factory 2>/dev/null || true
sleep 1

# Test 1: Unit Tests
run_test "Unit Tests (cargo test)" "cargo test 2>&1 | tail -20"

# Test 2: Clippy
run_test "Clippy (lint check)" "cargo clippy 2>&1 | grep -E '(warning|error)' || echo 'No warnings'"

# Test 3: Smoke Test
run_test "Smoke Test (startup)" "./scripts/smoke_test.sh 15"

# Test 4: E2E Quick Test
run_test "E2E Quick Test" "./scripts/e2e-quick.sh"

# Test 5: Visual Regression
run_test "Visual Regression" "./scripts/visual_regression.sh check"

# Full mode additional tests
if [ "$MODE" = "full" ]; then
    # Test 6: Fuzz Test (short)
    run_test "Fuzz Test (50 iterations)" "./scripts/fuzz_test.sh 50 0.1"

    # Test 7: Scenario Test
    run_test "Scenario Test (production line)" "./scripts/scenario_test.sh 90"

    # Test 8: VLM Visual Check (full suite)
    if [ -n "$ANTHROPIC_API_KEY" ]; then
        run_test "VLM Visual Check (full)" "./scripts/vlm_check.sh --full-suite"
    else
        warn "VLM test skipped: ANTHROPIC_API_KEY not set"
        run_test "VLM Visual Check" "" "true"
    fi
else
    run_test "Fuzz Test" "" "true"
    run_test "Scenario Test" "" "true"

    # Quick mode: VLM quick check if API key available
    if [ -n "$ANTHROPIC_API_KEY" ]; then
        run_test "VLM Visual Check (quick)" "./scripts/vlm_check.sh --quick"
    else
        run_test "VLM Visual Check" "" "true"
    fi
fi

# Summary
echo "" | tee -a "$REPORT_FILE"
echo "═══════════════════════════════════════════════" | tee -a "$REPORT_FILE"
echo "  テスト結果サマリー" | tee -a "$REPORT_FILE"
echo "═══════════════════════════════════════════════" | tee -a "$REPORT_FILE"
log "Passed:  $PASSED"
[ $FAILED -gt 0 ] && err "Failed:  $FAILED"
[ $SKIPPED -gt 0 ] && warn "Skipped: $SKIPPED"
echo "" | tee -a "$REPORT_FILE"
log "Report: $REPORT_FILE"

if [ $FAILED -gt 0 ]; then
    err "Some tests failed!"
    exit 1
fi

log "All tests passed!"
