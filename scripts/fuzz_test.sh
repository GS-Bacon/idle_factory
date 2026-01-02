#!/bin/bash
# Fuzzing test script - random inputs + commands to detect crashes
# Usage: ./scripts/fuzz_test.sh [iterations] [delay]

set -e

command -v xdotool >/dev/null 2>&1 || { echo "xdotool required"; exit 1; }

ITERATIONS=${1:-100}
DELAY=${2:-0.1}
GAME_WINDOW="Idle Factory"
LOG_DIR="logs/fuzz"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="$LOG_DIR/fuzz_$TIMESTAMP.log"

mkdir -p "$LOG_DIR"

echo "=== Idle Factory Fuzzing Test ===" | tee "$LOG_FILE"
echo "Iterations: $ITERATIONS, Delay: ${DELAY}s" | tee -a "$LOG_FILE"

# Find game window
WINDOW_ID=$(DISPLAY=:10 xdotool search --name "$GAME_WINDOW" 2>/dev/null | head -1)
if [ -z "$WINDOW_ID" ]; then
    echo "Error: Game window not found. Start the game first." | tee -a "$LOG_FILE"
    exit 1
fi

echo "Found window: $WINDOW_ID" | tee -a "$LOG_FILE"
DISPLAY=:10 xdotool windowactivate "$WINDOW_ID"
sleep 0.5

# Key pool
KEYS=(w a s d e f3 1 2 3 4 5 6 7 8 9 space Escape q shift)

# Command pool - Basic
COMMANDS=(
    "/creative" "/survival" "/clear"
    "/give iron 64" "/give coal 32" "/give miner 5" "/give conveyor 10"
    "/tp 0 12 0" "/tp 50 12 50" "/tp -30 15 -30"
    "/look 0 0" "/look 45 90" "/look -30 180"
    "/spawn 0 8 0 miner" "/spawn 5 8 5 conveyor 1" "/spawn 10 8 10 furnace"
    "/spawn_line 0 0 1 5 conveyor"
    "/test production" "/debug_conveyor"
    "/assert inventory iron 0"
    "/save fuzz" "/load fuzz"

    # Boundary value tests
    "/tp 0 0 0"
    "/tp 999999 999999 999999"
    "/tp -999999 -999999 -999999"
    "/give iron 0"
    "/give iron 999999"
    "/give iron 1"
    "/look 90 0"
    "/look -90 360"

    # Rapid operations
    "/save test1"
    "/load test1"
    "/save test2"
    "/load test2"

    # Stress tests
    "/spawn 0 8 0 miner"
    "/spawn 1 8 0 miner"
    "/spawn 2 8 0 miner"
    "/spawn 0 8 0 conveyor 0"
    "/spawn 1 8 0 conveyor 0"
    "/spawn 2 8 0 conveyor 0"

    # Invalid input tests (should be handled gracefully)
    "/give invaliditem 1"
    "/tp a b c"
    "/spawn 0 8 0 invalidmachine"
    "/look nan inf"
    "/give iron -1"
)

CMD_COUNT=0
KEY_COUNT=0

type_char() {
    local char="$1"
    case "$char" in
        [a-z]) DISPLAY=:10 xdotool key "$char" ;;
        [A-Z]) DISPLAY=:10 xdotool key "shift+${char,,}" ;;
        [0-9]) DISPLAY=:10 xdotool key "$char" ;;
        " ") DISPLAY=:10 xdotool key space ;;
        "/") DISPLAY=:10 xdotool key slash ;;
        "_") DISPLAY=:10 xdotool key "shift+minus" ;;
        "-") DISPLAY=:10 xdotool key minus ;;
        *) DISPLAY=:10 xdotool key "$char" 2>/dev/null || true ;;
    esac
    sleep 0.1  # 100ms delay to prevent character scrambling
}

send_cmd_chars() {
    local cmd="$1"
    DISPLAY=:10 xdotool key t
    sleep 0.5
    for ((i=0; i<${#cmd}; i++)); do
        type_char "${cmd:$i:1}"
    done
    sleep 0.1
    DISPLAY=:10 xdotool key Return
    sleep 0.2
}

echo "Starting fuzz..." | tee -a "$LOG_FILE"
for i in $(seq 1 $ITERATIONS); do
    # 20% commands, 80% keys/clicks
    if [ $((RANDOM % 5)) -eq 0 ]; then
        CMD="${COMMANDS[$RANDOM % ${#COMMANDS[@]}]}"
        echo "[$i] CMD: $CMD" >> "$LOG_FILE"
        send_cmd_chars "$CMD"
        CMD_COUNT=$((CMD_COUNT + 1))
    else
        KEY=${KEYS[$RANDOM % ${#KEYS[@]}]}
        ACTION=$((RANDOM % 3))

        case $ACTION in
            0) DISPLAY=:10 xdotool key "$KEY" ;;
            1) DISPLAY=:10 xdotool keydown "$KEY"; sleep 0.05; DISPLAY=:10 xdotool keyup "$KEY" ;;
            2) DISPLAY=:10 xdotool click $((RANDOM % 3 + 1)) ;;
        esac
        KEY_COUNT=$((KEY_COUNT + 1))
    fi

    [ $((i % 20)) -eq 0 ] && echo "Progress: $i/$ITERATIONS"
    sleep "$DELAY"

    # Crash check
    if ! DISPLAY=:10 xdotool search --name "$GAME_WINDOW" >/dev/null 2>&1; then
        echo "CRASH at iteration $i!" | tee -a "$LOG_FILE"
        echo "Log: $LOG_FILE"
        exit 1
    fi
done

echo "" | tee -a "$LOG_FILE"
echo "=== Fuzz Complete ===" | tee -a "$LOG_FILE"
echo "Keys: $KEY_COUNT, Commands: $CMD_COUNT" | tee -a "$LOG_FILE"
echo "No crashes detected" | tee -a "$LOG_FILE"
