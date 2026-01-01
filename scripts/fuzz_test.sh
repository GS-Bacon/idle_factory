#!/bin/bash
# B-1: Fuzzing test script
# Sends random key inputs to the game to detect crashes

set -e

# Check for required tools
command -v xdotool >/dev/null 2>&1 || { echo "xdotool required but not installed"; exit 1; }

# Configuration
ITERATIONS=${1:-100}
DELAY=${2:-0.1}
GAME_WINDOW="Idle Factory"

echo "=== Idle Factory Fuzzing Test ==="
echo "Iterations: $ITERATIONS"
echo "Delay: ${DELAY}s"

# Find game window
WINDOW_ID=$(DISPLAY=:10 xdotool search --name "$GAME_WINDOW" 2>/dev/null | head -1)
if [ -z "$WINDOW_ID" ]; then
    echo "Error: Game window '$GAME_WINDOW' not found"
    echo "Make sure the game is running"
    exit 1
fi

echo "Found game window: $WINDOW_ID"
DISPLAY=:10 xdotool windowactivate "$WINDOW_ID"
sleep 0.5

# Key pool for random input
KEYS=(w a s d e f3 1 2 3 4 5 6 7 8 9 space Return Escape r)

echo "Starting fuzzing..."
for i in $(seq 1 $ITERATIONS); do
    # Random key from pool
    KEY=${KEYS[$RANDOM % ${#KEYS[@]}]}

    # Random action: press, hold, or click
    ACTION=$((RANDOM % 3))

    case $ACTION in
        0)
            # Single key press
            DISPLAY=:10 xdotool key "$KEY"
            ;;
        1)
            # Key hold (short)
            DISPLAY=:10 xdotool keydown "$KEY"
            sleep 0.05
            DISPLAY=:10 xdotool keyup "$KEY"
            ;;
        2)
            # Mouse click (random button)
            BUTTON=$((RANDOM % 3 + 1))
            DISPLAY=:10 xdotool click "$BUTTON"
            ;;
    esac

    # Progress indicator
    if [ $((i % 10)) -eq 0 ]; then
        echo "Progress: $i/$ITERATIONS"
    fi

    sleep "$DELAY"

    # Check if game is still running
    if ! DISPLAY=:10 xdotool search --name "$GAME_WINDOW" >/dev/null 2>&1; then
        echo "CRASH DETECTED at iteration $i!"
        echo "Last input: $KEY (action: $ACTION)"
        exit 1
    fi
done

echo "=== Fuzzing Complete ==="
echo "No crashes detected in $ITERATIONS iterations"
