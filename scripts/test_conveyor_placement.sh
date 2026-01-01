#!/bin/bash
# Test: コンベア上にブロック設置

export DISPLAY=${DISPLAY:-:10}
SCREENSHOTS_DIR="/home/bacon/idle_factory/screenshots/verify"
mkdir -p "$SCREENSHOTS_DIR"

log() { echo "[E2E] $1"; }

# 1文字ずつ入力
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
    sleep 0.1  # 100ms delay to prevent character scrambling
}

send_cmd() {
    local cmd="$1"
    log "CMD: $cmd"
    xdotool key t
    sleep 0.5
    for ((i=0; i<${#cmd}; i++)); do
        type_char "${cmd:$i:1}"
    done
    sleep 0.1
    xdotool key Return
    sleep 0.5
    xdotool click 1
    sleep 0.2
}

# ウィンドウ検索
WINDOW=$(xdotool search --name "Idle Factory" | head -1)
if [ -z "$WINDOW" ]; then
    log "ERROR: Game window not found"
    exit 1
fi
log "Window: $WINDOW"

# ウィンドウアクティブ化
xdotool windowactivate "$WINDOW"
sleep 0.5
xdotool click 1
sleep 0.3

# クリエイティブモード
send_cmd "/creative"

# テレポート
send_cmd "/tp 0 12 0"
sleep 0.3

# 下を向く
send_cmd "/look 60 0"
sleep 0.3

# コンベアを配置 (y=8)
send_cmd "/spawn 0 8 0 conveyor 0"
sleep 0.5

# スクリーンショット1: コンベア配置後
scrot "$SCREENSHOTS_DIR/01_conveyor_placed.png"
log "Screenshot: 01_conveyor_placed.png"

# コンベアの上にブロックを配置 (y=9)
send_cmd "/setblock 0 9 0 stone"
sleep 0.5

# スクリーンショット2: ブロック配置後
scrot "$SCREENSHOTS_DIR/02_block_on_conveyor.png"
log "Screenshot: 02_block_on_conveyor.png"

# 状態エクスポート
send_cmd "/e2e_export"
sleep 0.3

# 結果確認
log "=== Test Complete ==="
log "Screenshots saved to: $SCREENSHOTS_DIR"
ls -la "$SCREENSHOTS_DIR"/*.png 2>/dev/null | tail -5

# e2e_state.json確認
if [ -f "/home/bacon/idle_factory/e2e_state.json" ]; then
    log "=== E2E State ==="
    cat /home/bacon/idle_factory/e2e_state.json | head -30
fi
