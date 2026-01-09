#!/bin/bash
# Run scenario-based E2E tests
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Start game in background
echo "Starting game..."
DISPLAY=:10 cargo run --release &
GAME_PID=$!

# Wait for game to start and WebSocket to be ready
echo "Waiting for game to start..."
for i in {1..30}; do
    if ss -tlnp 2>/dev/null | grep -q ':9877'; then
        echo "Game ready (port 9877 open)"
        break
    fi
    sleep 1
done

if ! ss -tlnp 2>/dev/null | grep -q ':9877'; then
    echo "ERROR: Game did not start (port 9877 not open)"
    kill $GAME_PID 2>/dev/null || true
    exit 1
fi

# Run scenarios
FAILED=0
for scenario in tests/scenarios/*.toml; do
    echo ""
    echo "=== Running $scenario ==="
    if node scripts/run-scenario.js "$scenario"; then
        echo "PASSED: $scenario"
    else
        echo "FAILED: $scenario"
        FAILED=1
    fi
done

# Cleanup
echo ""
echo "Stopping game..."
kill $GAME_PID 2>/dev/null || true

exit $FAILED
