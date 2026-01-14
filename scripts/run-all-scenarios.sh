#!/bin/bash
# Run all scenario tests
# Usage: ./scripts/run-all-scenarios.sh [--verbose] [--keep-game]
#
# Options:
#   --verbose    Show detailed output for each scenario
#   --keep-game  Don't kill the game after tests (if we started it)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."
SCENARIO_DIR="$PROJECT_DIR/tests/scenarios"

VERBOSE=false
KEEP_GAME=false
STARTED_GAME=false

for arg in "$@"; do
    case $arg in
        --verbose) VERBOSE=true ;;
        --keep-game) KEEP_GAME=true ;;
    esac
done

echo "=============================================="
echo "  SCENARIO TEST RUNNER"
echo "  Started: $(date)"
echo "=============================================="
echo ""

# Check if game is running, start if not
if ! nc -z 127.0.0.1 9877 2>/dev/null; then
    echo "Game not running. Starting..."

    # Build first
    cargo build --manifest-path "$PROJECT_DIR/Cargo.toml" 2>&1 | tail -5

    # Start game in background
    DISPLAY=:10 cargo run --manifest-path "$PROJECT_DIR/Cargo.toml" --bin idle_factory >/dev/null 2>&1 &
    GAME_PID=$!
    STARTED_GAME=true

    # Wait for WebSocket port
    echo -n "Waiting for game to start"
    for i in {1..30}; do
        if nc -z 127.0.0.1 9877 2>/dev/null; then
            echo " ready!"
            break
        fi
        echo -n "."
        sleep 1
    done

    if ! nc -z 127.0.0.1 9877 2>/dev/null; then
        echo " TIMEOUT"
        echo "ERROR: Game failed to start within 30 seconds"
        kill $GAME_PID 2>/dev/null || true
        exit 1
    fi
    echo ""
fi

TOTAL=0
PASSED=0
FAILED=0
FAILED_LIST=""

for scenario in "$SCENARIO_DIR"/*.toml; do
    [ -f "$scenario" ] || continue
    TOTAL=$((TOTAL + 1))
    name=$(basename "$scenario" .toml)

    if [ "$VERBOSE" = true ]; then
        echo "Running: $name"
        if node "$SCRIPT_DIR/run-scenario.js" "$scenario"; then
            PASSED=$((PASSED + 1))
            echo "  PASSED"
        else
            FAILED=$((FAILED + 1))
            FAILED_LIST="$FAILED_LIST\n  - $name"
            echo "  FAILED"
        fi
        echo ""
    else
        printf "  %-45s " "$name"
        if node "$SCRIPT_DIR/run-scenario.js" "$scenario" >/dev/null 2>&1; then
            PASSED=$((PASSED + 1))
            echo "PASS"
        else
            FAILED=$((FAILED + 1))
            FAILED_LIST="$FAILED_LIST\n  - $name"
            echo "FAIL"
        fi
    fi
done

echo ""
echo "=============================================="
echo "  Results: $PASSED/$TOTAL passed, $FAILED failed"
if [ $FAILED -gt 0 ]; then
    echo -e "  Failed tests:$FAILED_LIST"
fi
echo "  Completed: $(date)"
echo "=============================================="

# Kill game if we started it and --keep-game not specified
if [ "$STARTED_GAME" = true ] && [ "$KEEP_GAME" = false ]; then
    echo ""
    echo "Stopping game..."
    pkill -f "idle_factory" 2>/dev/null || true
fi

exit $FAILED
