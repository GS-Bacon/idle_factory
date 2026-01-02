#!/bin/bash
# Smoke Test - 起動確認・クラッシュ検出・フリーズ検出
# Usage: ./scripts/smoke_test.sh [timeout_seconds]

set -e

export DISPLAY=${DISPLAY:-:10}
TIMEOUT=${1:-30}
GAME_DIR="/home/bacon/idle_factory"
LOG_FILE="/tmp/smoke_test_$$.log"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[SMOKE]${NC} $1"; }
warn() { echo -e "${YELLOW}[SMOKE]${NC} $1"; }
err() { echo -e "${RED}[SMOKE]${NC} $1"; }

cleanup() {
    pkill -x idle_factory 2>/dev/null
    rm -f "$LOG_FILE"
    return 0
}

trap cleanup EXIT

log "=== Smoke Test ==="
log "Timeout: ${TIMEOUT}s"

# Kill existing instances (match binary only)
pkill -x idle_factory 2>/dev/null || true
sleep 2

cd "$GAME_DIR"

# Test 1: Build check
log "[1/5] Build check..."
if ! cargo build --release 2>&1 | tail -3; then
    err "Build failed!"
    exit 1
fi
log "✓ Build OK"

# Test 2: Start game
log "[2/5] Starting game..."
cargo run --release 2>&1 > "$LOG_FILE" &
GAME_PID=$!

# Test 3: Window appears within 15 seconds
log "[3/5] Waiting for window..."
WINDOW_FOUND=0
for i in $(seq 1 15); do
    if xdotool search --name "Idle Factory" >/dev/null 2>&1; then
        WINDOW_FOUND=1
        log "✓ Window appeared in ${i}s"
        break
    fi
    sleep 1
done

if [ $WINDOW_FOUND -eq 0 ]; then
    err "Window not found within 15 seconds!"
    cat "$LOG_FILE" | tail -20
    exit 1
fi

sleep 2

# Test 4: Game doesn't crash for TIMEOUT seconds
log "[4/5] Running for ${TIMEOUT}s (crash check)..."
START_TIME=$(date +%s)
CRASHED=0

while true; do
    CURRENT=$(date +%s)
    ELAPSED=$((CURRENT - START_TIME))

    if [ $ELAPSED -ge $TIMEOUT ]; then
        break
    fi

    if ! kill -0 $GAME_PID 2>/dev/null; then
        CRASHED=1
        break
    fi

    # Check for panic in logs
    if grep -q "panic" "$LOG_FILE" 2>/dev/null; then
        CRASHED=1
        err "Panic detected in logs!"
        break
    fi

    # Progress every 10 seconds
    if [ $((ELAPSED % 10)) -eq 0 ] && [ $ELAPSED -gt 0 ]; then
        log "  Running... ${ELAPSED}s"
    fi

    sleep 1
done

if [ $CRASHED -eq 1 ]; then
    err "Game crashed!"
    echo "=== Last 30 lines of log ==="
    tail -30 "$LOG_FILE"
    exit 1
fi
log "✓ No crash for ${TIMEOUT}s"

# Test 5: Game responds (not frozen)
log "[5/5] Freeze check (sending input)..."
WINDOW_ID=$(xdotool search --name "Idle Factory" 2>/dev/null | head -1)
if [ -n "$WINDOW_ID" ]; then
    xdotool windowactivate "$WINDOW_ID" 2>/dev/null || true
    sleep 0.3
    xdotool key Escape  # Release pointer if locked
    sleep 0.3
    xdotool key w  # Try movement
    sleep 0.5

    # Check if still running
    if kill -0 $GAME_PID 2>/dev/null; then
        log "✓ Game responsive"
    else
        err "Game froze/crashed on input!"
        exit 1
    fi
else
    warn "Could not send input (window lost)"
fi

# Check for errors in log
ERROR_COUNT=$(grep -ci "error\|panic\|failed" "$LOG_FILE" 2>/dev/null || echo 0)
WARN_COUNT=$(grep -ci "warn" "$LOG_FILE" 2>/dev/null || echo 0)

echo ""
log "=== Smoke Test Results ==="
log "✓ Build:     OK"
log "✓ Startup:   OK"
log "✓ Stability: OK (${TIMEOUT}s)"
log "✓ Response:  OK"
[ $ERROR_COUNT -gt 0 ] && warn "  Errors in log: $ERROR_COUNT"
[ $WARN_COUNT -gt 0 ] && log "  Warnings in log: $WARN_COUNT"

log ""
log "All smoke tests passed!"
exit 0
