#!/bin/bash
# E2E Test v2: 遅延増加版
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

# Activate
xdotool windowactivate --sync "$WINDOW"
sleep 0.5
xdotool key space
sleep 0.3
xdotool click 1
sleep 0.3

# 遅延を増やした1文字入力 (50ms)
type_slow() {
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
        sleep 0.05  # 50ms delay per character
    done
}

send_cmd() {
    local cmd="$1"
    log "CMD: $cmd"
    xdotool key t
    sleep 0.3
    type_slow "$cmd"
    sleep 0.2
    xdotool key Return
    sleep 0.8
    xdotool click 1
    sleep 0.3
}

# Test
send_cmd "/creative"
sleep 0.5
scrot "$SCREENSHOTS/v2_1_creative.png"

send_cmd "/tp 0 12 0"
sleep 0.5
scrot "$SCREENSHOTS/v2_2_tp.png"

send_cmd "/look 60 0"
sleep 0.5
scrot "$SCREENSHOTS/v2_3_look.png"

send_cmd "/spawn 0 8 0 conveyor 0"
sleep 0.8
scrot "$SCREENSHOTS/v2_4_conveyor.png"

send_cmd "/setblock 0 9 0 stone"
sleep 0.8
scrot "$SCREENSHOTS/v2_5_block.png"

log "=== Test Complete ==="
ls -la "$SCREENSHOTS"/v2_*.png
