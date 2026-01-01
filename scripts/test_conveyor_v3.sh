#!/bin/bash
# E2E Test v3: さらに遅延増加 (100ms/char)
export DISPLAY=:10
SCREENSHOTS="/home/bacon/idle_factory/screenshots/verify"
mkdir -p "$SCREENSHOTS"

log() { echo "[TEST] $1"; }

WINDOW=$(xdotool search --name "Idle Factory" | head -1)
if [ -z "$WINDOW" ]; then
    log "ERROR: Game window not found"
    exit 1
fi
log "Window: $WINDOW"

xdotool windowactivate --sync "$WINDOW"
sleep 0.5
xdotool key space
sleep 0.3
xdotool click 1
sleep 0.3

# 100ms遅延版
type_very_slow() {
    local cmd="$1"
    for ((i=0; i<${#cmd}; i++)); do
        char="${cmd:$i:1}"
        case "$char" in
            [a-z]) xdotool key "$char" ;;
            [A-Z]) xdotool key "shift+${char,,}" ;;
            [0-9]) xdotool key "$char" ;;
            " ") xdotool key space ;;
            "/") xdotool key slash ;;
            "_") xdotool key shift+minus ;;
            "-") xdotool key minus ;;
        esac
        sleep 0.1  # 100ms delay
    done
}

send_cmd() {
    local cmd="$1"
    log "CMD: $cmd"
    xdotool key t
    sleep 0.5
    type_very_slow "$cmd"
    sleep 0.3
    xdotool key Return
    sleep 1.0
    xdotool click 1
    sleep 0.5
}

# Test
send_cmd "/creative"
scrot "$SCREENSHOTS/v3_1_creative.png"

send_cmd "/tp 0 12 0"
scrot "$SCREENSHOTS/v3_2_tp.png"

send_cmd "/look 60 0"
scrot "$SCREENSHOTS/v3_3_look.png"

send_cmd "/spawn 0 8 0 conveyor 0"
scrot "$SCREENSHOTS/v3_4_conveyor.png"

send_cmd "/setblock 0 9 0 stone"
scrot "$SCREENSHOTS/v3_5_block.png"

log "=== Test Complete ==="
