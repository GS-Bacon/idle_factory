#!/bin/bash
# Visual Regression Test - スクリーンショット比較でUIバグを検出
# Usage: ./scripts/visual_regression.sh [update|check]

set -e

BASELINE_DIR="screenshots/baseline"
VERIFY_DIR="screenshots/verify"
DIFF_DIR="screenshots/diff"
THRESHOLD=5  # 許容する差分率（%）

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[VRT]${NC} $1"; }
warn() { echo -e "${YELLOW}[VRT]${NC} $1"; }
err() { echo -e "${RED}[VRT]${NC} $1"; }

mkdir -p "$BASELINE_DIR" "$DIFF_DIR"

MODE=${1:-check}

case "$MODE" in
    update)
        log "=== Updating baseline screenshots ==="
        if [ ! -d "$VERIFY_DIR" ] || [ -z "$(ls -A $VERIFY_DIR 2>/dev/null)" ]; then
            err "No screenshots in $VERIFY_DIR. Run E2E test first:"
            echo "  ./scripts/e2e-quick.sh"
            exit 1
        fi

        cp -v "$VERIFY_DIR"/*.png "$BASELINE_DIR/"
        log "Baseline updated with $(ls -1 $BASELINE_DIR/*.png 2>/dev/null | wc -l) screenshots"
        ;;

    check)
        log "=== Visual Regression Check ==="

        if [ ! -d "$BASELINE_DIR" ] || [ -z "$(ls -A $BASELINE_DIR 2>/dev/null)" ]; then
            warn "No baseline screenshots. Creating baseline first..."
            $0 update
            log "Baseline created. Run E2E again and then check."
            exit 0
        fi

        if [ ! -d "$VERIFY_DIR" ] || [ -z "$(ls -A $VERIFY_DIR 2>/dev/null)" ]; then
            err "No screenshots in $VERIFY_DIR. Run E2E test first."
            exit 1
        fi

        FAILED=0
        PASSED=0
        MISSING=0

        # Clear old diffs
        rm -f "$DIFF_DIR"/*.png

        for baseline in "$BASELINE_DIR"/*.png; do
            filename=$(basename "$baseline")
            verify="$VERIFY_DIR/$filename"
            diff_file="$DIFF_DIR/${filename%.png}_diff.png"

            if [ ! -f "$verify" ]; then
                warn "MISSING: $filename (no verify screenshot)"
                MISSING=$((MISSING + 1))
                continue
            fi

            # Compare images using ImageMagick
            # AE = Absolute Error (pixel count that differs)
            DIFF=$(compare -metric AE "$baseline" "$verify" "$diff_file" 2>&1 || true)

            # Get image dimensions to calculate percentage
            PIXELS=$(identify -format "%[fx:w*h]" "$baseline")

            if [ -n "$DIFF" ] && [ "$DIFF" != "0" ]; then
                PERCENT=$(echo "scale=2; $DIFF * 100 / $PIXELS" | bc)

                if [ "$(echo "$PERCENT > $THRESHOLD" | bc)" -eq 1 ]; then
                    err "FAIL: $filename - ${PERCENT}% different ($DIFF pixels)"
                    err "      Diff saved: $diff_file"
                    FAILED=$((FAILED + 1))
                else
                    log "PASS: $filename - ${PERCENT}% different (within threshold)"
                    rm -f "$diff_file"  # Remove diff if passed
                    PASSED=$((PASSED + 1))
                fi
            else
                log "PASS: $filename - identical"
                rm -f "$diff_file"
                PASSED=$((PASSED + 1))
            fi
        done

        # Check for new screenshots not in baseline
        for verify in "$VERIFY_DIR"/*.png; do
            filename=$(basename "$verify")
            baseline="$BASELINE_DIR/$filename"
            if [ ! -f "$baseline" ]; then
                warn "NEW: $filename (not in baseline)"
            fi
        done

        echo ""
        log "=== Results ==="
        log "Passed: $PASSED"
        [ $FAILED -gt 0 ] && err "Failed: $FAILED"
        [ $MISSING -gt 0 ] && warn "Missing: $MISSING"

        if [ $FAILED -gt 0 ]; then
            err "Visual regression detected! Check $DIFF_DIR for details."
            exit 1
        fi

        log "All visual checks passed!"
        ;;

    *)
        echo "Usage: $0 [update|check]"
        echo "  update - Save current screenshots as baseline"
        echo "  check  - Compare current screenshots against baseline"
        exit 1
        ;;
esac
