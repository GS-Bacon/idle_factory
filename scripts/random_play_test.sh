#!/bin/bash
# Random Play Test - Extended random testing with anomaly detection
# Runs 1000+ random operations and monitors for crashes, hangs, and anomalies
#
# Usage: ./scripts/random_play_test.sh [iterations] [timeout_minutes]

set -e

command -v xdotool >/dev/null 2>&1 || { echo "xdotool required"; exit 1; }

ITERATIONS=${1:-1000}
TIMEOUT_MINUTES=${2:-30}
DELAY=0.05
GAME_WINDOW="Idle Factory"
LOG_DIR="logs/random_play"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="$LOG_DIR/random_$TIMESTAMP.log"
REPORT_FILE="$LOG_DIR/report_$TIMESTAMP.json"

export DISPLAY=${DISPLAY:-:10}

mkdir -p "$LOG_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[RANDOM]${NC} $1" | tee -a "$LOG_FILE"; }
warn() { echo -e "${YELLOW}[RANDOM]${NC} $1" | tee -a "$LOG_FILE"; }
err() { echo -e "${RED}[RANDOM]${NC} $1" | tee -a "$LOG_FILE"; }

echo "═══════════════════════════════════════════════" | tee "$LOG_FILE"
echo "  Random Play Test" | tee -a "$LOG_FILE"
echo "  Iterations: $ITERATIONS" | tee -a "$LOG_FILE"
echo "  Timeout: ${TIMEOUT_MINUTES}m" | tee -a "$LOG_FILE"
echo "  Date: $(date)" | tee -a "$LOG_FILE"
echo "═══════════════════════════════════════════════" | tee -a "$LOG_FILE"

# Start game if not running
WINDOW_ID=$(xdotool search --name "$GAME_WINDOW" 2>/dev/null | head -1 || true)
if [ -z "$WINDOW_ID" ]; then
    log "Starting game..."
    cd /home/bacon/idle_factory
    cargo run &
    GAME_PID=$!
    sleep 15
    WINDOW_ID=$(xdotool search --name "$GAME_WINDOW" 2>/dev/null | head -1)
    if [ -z "$WINDOW_ID" ]; then
        err "Failed to start game"
        exit 1
    fi
    log "Game started (PID: $GAME_PID, Window: $WINDOW_ID)"
else
    log "Found existing game window: $WINDOW_ID"
    GAME_PID=""
fi

xdotool windowactivate "$WINDOW_ID"
sleep 0.5

# Key pool - movement, actions, UI
KEYS=(w a s d e f f3 1 2 3 4 5 6 7 8 9 space Escape q shift)

# Command pool - various game commands
COMMANDS=(
    "/creative" "/survival" "/clear"
    "/give iron 64" "/give coal 32" "/give stone 64"
    "/give miner 5" "/give conveyor 10" "/give furnace 3"
    "/tp 0 12 0" "/tp 50 12 50" "/tp -30 15 -30" "/tp 100 20 100"
    "/look 0 0" "/look 45 90" "/look -30 180" "/look -60 0"
    "/spawn 0 8 0 miner" "/spawn 5 8 5 conveyor 1" "/spawn 10 8 10 furnace"
    "/spawn_line 0 0 1 5 conveyor"
    "/test production" "/debug_conveyor" "/debug_machine"
    "/save random_test" "/load random_test"
    "/screenshot random_test"
)

# Anomaly counters
CRASHES=0
HANGS=0
ERRORS=0
CMD_COUNT=0
KEY_COUNT=0
CLICK_COUNT=0
START_TIME=$(date +%s)
LAST_CHECK_TIME=$START_TIME

# Helper function to type a command character by character
type_char() {
    local char="$1"
    case "$char" in
        [a-z]) xdotool key "$char" ;;
        [A-Z]) xdotool key "shift+${char,,}" ;;
        [0-9]) xdotool key "$char" ;;
        " ") xdotool key space ;;
        "/") xdotool key slash ;;
        "_") xdotool key "shift+minus" ;;
        "-") xdotool key minus ;;
        *) xdotool key "$char" 2>/dev/null || true ;;
    esac
    sleep 0.05
}

send_command() {
    local cmd="$1"
    xdotool key t
    sleep 0.3
    for ((i=0; i<${#cmd}; i++)); do
        type_char "${cmd:$i:1}"
    done
    sleep 0.1
    xdotool key Return
    sleep 0.2
}

# Check if game is responsive
check_game_alive() {
    if ! xdotool search --name "$GAME_WINDOW" >/dev/null 2>&1; then
        return 1
    fi
    return 0
}

# Check for errors in log (last 10 lines)
check_for_errors() {
    if [ -f "logs/game.log" ]; then
        local recent_errors=$(tail -100 logs/game.log 2>/dev/null | grep -c "ERROR\|PANIC\|panic" || true)
        if [ "$recent_errors" -gt 0 ]; then
            return 1
        fi
    fi
    return 0
}

# Periodic health check
health_check() {
    local current_time=$(date +%s)
    local elapsed=$((current_time - LAST_CHECK_TIME))

    # Check every 30 seconds
    if [ $elapsed -ge 30 ]; then
        LAST_CHECK_TIME=$current_time

        # Check if game is alive
        if ! check_game_alive; then
            warn "Game appears to have crashed!"
            CRASHES=$((CRASHES + 1))
            return 1
        fi

        # Check for errors
        if ! check_for_errors; then
            warn "Errors detected in game log"
            ERRORS=$((ERRORS + 1))
        fi
    fi
    return 0
}

log "Starting random play test..."

# Timeout mechanism
TIMEOUT_SECONDS=$((TIMEOUT_MINUTES * 60))

for i in $(seq 1 $ITERATIONS); do
    # Check timeout
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))
    if [ $ELAPSED -ge $TIMEOUT_SECONDS ]; then
        warn "Timeout reached ($TIMEOUT_MINUTES minutes)"
        break
    fi

    # Health check
    if ! health_check; then
        err "Game crashed at iteration $i"
        break
    fi

    # Random action: 15% commands, 60% keys, 25% clicks
    RAND=$((RANDOM % 100))

    if [ $RAND -lt 15 ]; then
        # Send command
        CMD="${COMMANDS[$RANDOM % ${#COMMANDS[@]}]}"
        echo "[$i] CMD: $CMD" >> "$LOG_FILE"
        send_command "$CMD"
        CMD_COUNT=$((CMD_COUNT + 1))
    elif [ $RAND -lt 75 ]; then
        # Press key
        KEY=${KEYS[$RANDOM % ${#KEYS[@]}]}
        ACTION=$((RANDOM % 3))

        case $ACTION in
            0) xdotool key "$KEY" ;;
            1) xdotool keydown "$KEY"; sleep 0.05; xdotool keyup "$KEY" ;;
            2) xdotool keydown "$KEY"; sleep 0.2; xdotool keyup "$KEY" ;;
        esac
        KEY_COUNT=$((KEY_COUNT + 1))
    else
        # Click
        BUTTON=$((RANDOM % 3 + 1))
        xdotool click $BUTTON
        CLICK_COUNT=$((CLICK_COUNT + 1))
    fi

    # Progress report every 100 iterations
    if [ $((i % 100)) -eq 0 ]; then
        log "Progress: $i/$ITERATIONS (elapsed: ${ELAPSED}s)"
    fi

    sleep "$DELAY"
done

# Final stats
END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))
ACTIONS=$((CMD_COUNT + KEY_COUNT + CLICK_COUNT))

log ""
log "═══════════════════════════════════════════════"
log "  Test Complete"
log "═══════════════════════════════════════════════"
log "Duration: ${TOTAL_TIME}s"
log "Actions: $ACTIONS (keys: $KEY_COUNT, commands: $CMD_COUNT, clicks: $CLICK_COUNT)"
log "Crashes: $CRASHES"
log "Hangs: $HANGS"
log "Errors: $ERRORS"

# Generate JSON report
cat << EOF > "$REPORT_FILE"
{
    "test": "random_play",
    "timestamp": "$TIMESTAMP",
    "iterations": $ITERATIONS,
    "actual_iterations": $i,
    "duration_seconds": $TOTAL_TIME,
    "actions": {
        "total": $ACTIONS,
        "keys": $KEY_COUNT,
        "commands": $CMD_COUNT,
        "clicks": $CLICK_COUNT
    },
    "anomalies": {
        "crashes": $CRASHES,
        "hangs": $HANGS,
        "errors": $ERRORS
    },
    "result": $([ $CRASHES -eq 0 ] && [ $HANGS -eq 0 ] && echo '"PASS"' || echo '"FAIL"')
}
EOF

log "Report saved: $REPORT_FILE"

# Cleanup if we started the game
if [ -n "$GAME_PID" ]; then
    log "Stopping game..."
    kill $GAME_PID 2>/dev/null || true
    wait $GAME_PID 2>/dev/null || true
fi

if [ $CRASHES -gt 0 ] || [ $HANGS -gt 0 ]; then
    err "Test FAILED: $CRASHES crashes, $HANGS hangs"
    exit 1
fi

log "Test PASSED: No crashes or hangs detected"
exit 0
