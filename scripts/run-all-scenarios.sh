#!/bin/bash
# Run all scenario tests
# Usage: ./scripts/run-all-scenarios.sh [--visual]
#   --visual: Include visual (screenshot) tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SCENARIOS_DIR="$PROJECT_DIR/tests/scenarios"

# Parse arguments
INCLUDE_VISUAL=false
for arg in "$@"; do
    case $arg in
        --visual)
            INCLUDE_VISUAL=true
            ;;
    esac
done

# Check if game is running
if ! ss -tlnp 2>/dev/null | grep -q ":9877"; then
    echo "Error: Game not running. Start with: DISPLAY=:10 cargo run"
    echo "Waiting for game to start..."

    # Start game in background
    cd "$PROJECT_DIR"
    DISPLAY=:10 cargo run &
    GAME_PID=$!

    # Wait for WebSocket port
    for i in {1..30}; do
        if ss -tlnp 2>/dev/null | grep -q ":9877"; then
            echo "Game started (PID: $GAME_PID)"
            break
        fi
        sleep 1
    done

    if ! ss -tlnp 2>/dev/null | grep -q ":9877"; then
        echo "Error: Game failed to start"
        exit 1
    fi
fi

echo "=== Running Scenario Tests ==="
echo ""

PASSED=0
FAILED=0
SKIPPED=0

for scenario in "$SCENARIOS_DIR"/*.toml; do
    name=$(basename "$scenario" .toml)

    # Skip visual tests unless --visual flag
    if [[ "$name" == ui_visual_* ]] && [ "$INCLUDE_VISUAL" = false ]; then
        echo "SKIP: $name (use --visual to include)"
        ((SKIPPED++))
        continue
    fi

    echo -n "TEST: $name ... "

    if node "$SCRIPT_DIR/run-scenario.js" "$scenario" > /tmp/scenario_output.txt 2>&1; then
        echo "PASS"
        ((PASSED++))
    else
        echo "FAIL"
        cat /tmp/scenario_output.txt | grep -E "(Assert:|Error:|Compare:)" | head -5
        ((FAILED++))
    fi
done

echo ""
echo "=== Results ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Skipped: $SKIPPED"
echo ""

if [ $FAILED -gt 0 ]; then
    exit 1
fi
