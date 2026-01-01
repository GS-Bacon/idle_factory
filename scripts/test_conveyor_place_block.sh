#!/bin/bash
# E2E Test: コンベア上にブロックを設置できるか検証
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

# Activate and unpause
xdotool windowactivate --sync "$WINDOW"
sleep 0.5
xdotool key space
sleep 0.3
xdotool click 1
sleep 0.3

# Helper: send command
send_cmd() {
    local cmd="$1"
    log "CMD: $cmd"
    xdotool key t
    sleep 0.2
    for ((i=0; i<${#cmd}; i++)); do
        char="${cmd:$i:1}"
        case "$char" in
            [a-z]) xdotool key "$char" ;;
            [0-9]) xdotool key "$char" ;;
            " ") xdotool key space ;;
            "/") xdotool key slash ;;
            "_") xdotool key shift+minus ;;
            "-") xdotool key minus ;;
        esac
        sleep 0.02
    done
    sleep 0.1
    xdotool key Return
    sleep 0.5
    xdotool click 1
    sleep 0.2
}

# Step 1: Creative mode
send_cmd "/creative"
scrot "$SCREENSHOTS/step1_creative.png"
log "Step 1: Creative mode enabled"

# Step 2: Teleport to test location
send_cmd "/tp 0 12 0"
scrot "$SCREENSHOTS/step2_teleport.png"
log "Step 2: Teleported to 0,12,0"

# Step 3: Look down
send_cmd "/look 60 0"
sleep 0.3
scrot "$SCREENSHOTS/step3_lookdown.png"
log "Step 3: Looking down"

# Step 4: Place conveyor at 0,8,0
send_cmd "/spawn 0 8 0 conveyor 0"
sleep 0.5
scrot "$SCREENSHOTS/step4_conveyor.png"
log "Step 4: Conveyor placed at 0,8,0"

# Step 5: Place stone block on top of conveyor (0,9,0)
send_cmd "/setblock 0 9 0 stone"
sleep 0.5
scrot "$SCREENSHOTS/step5_block.png"
log "Step 5: Stone block placed at 0,9,0"

# Step 6: Debug conveyor
send_cmd "/debug conveyor"
sleep 0.3
scrot "$SCREENSHOTS/step6_debug.png"
log "Step 6: Debug info"

# Step 7: Export state
send_cmd "/e2e export"
sleep 0.5

log "=== Test Complete ==="
log "Screenshots: $SCREENSHOTS/step*.png"
ls -la "$SCREENSHOTS"/step*.png 2>/dev/null

# Check e2e_state.json
if [ -f /home/bacon/idle_factory/e2e_state.json ]; then
    log "=== E2E State ==="
    cat /home/bacon/idle_factory/e2e_state.json
fi
