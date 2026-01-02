#!/bin/bash
# Scenario Test: 採掘機→コンベア→精錬炉→インゴット確認
# Usage: ./scripts/scenario_test.sh [timeout_seconds]

set -e

export DISPLAY=${DISPLAY:-:10}
TIMEOUT=${1:-120}
GAME_DIR="/home/bacon/idle_factory"
LOG_DIR="$GAME_DIR/logs/scenario"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="$LOG_DIR/scenario_$TIMESTAMP.log"

mkdir -p "$LOG_DIR"

log() { echo "[$(date +%H:%M:%S)] $1" | tee -a "$LOG_FILE"; }
err() { echo "[$(date +%H:%M:%S)] ERROR: $1" | tee -a "$LOG_FILE"; }

cleanup() {
    pkill -x idle_factory 2>/dev/null || true
    sleep 1
}

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
        ".") xdotool key period ;;
        ",") xdotool key comma ;;
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
}

log "=== Scenario Test: Production Line ==="
log "Timeout: ${TIMEOUT}s"

cleanup
cd "$GAME_DIR"

# ゲーム起動
log "Starting game..."
cargo run 2>&1 &
GAME_PID=$!

# ウィンドウ待機
for i in {1..30}; do
    if xdotool search --name "Idle Factory" >/dev/null 2>&1; then
        log "Game window found (${i}s)"
        sleep 2
        break
    fi
    sleep 1
done

WINDOW_ID=$(xdotool search --name "Idle Factory" 2>/dev/null | head -1)
if [ -z "$WINDOW_ID" ]; then
    err "Game window not found"
    cleanup
    exit 1
fi

xdotool windowactivate "$WINDOW_ID"
sleep 1
xdotool click 1  # ポインターロック取得
sleep 0.5

# シナリオ実行
log "--- Setting up production line ---"

# クリエイティブモード
send_cmd "/creative"
sleep 0.3
xdotool click 1
sleep 0.2

# 鉄鉱石がある場所にテレポート (y=8付近が地表)
send_cmd "/tp 5 12 5"
sleep 0.5
xdotool click 1
sleep 0.2

# 生産ラインを配置 (コマンドで確実に)
# Miner(0,8,0) -> Conveyor(1,8,0) -> Conveyor(2,8,0) -> Conveyor(3,8,0) -> Furnace(4,8,0)
log "Spawning production line..."
send_cmd "/test production"
sleep 1
xdotool click 1
sleep 0.3

# 配置確認のスクリーンショット
log "Taking setup screenshot..."
send_cmd "/tp 2 12 -3"
sleep 0.5
xdotool click 1
send_cmd "/look 30 0"
sleep 0.5
xdotool click 1
sleep 0.3
scrot "$LOG_DIR/setup_$TIMESTAMP.png" 2>/dev/null || true

# デバッグ出力
send_cmd "/debug_conveyor"
sleep 0.5
xdotool click 1
sleep 0.2

# 待機して生産を確認
log "--- Waiting for production (60s) ---"
START_TIME=$(date +%s)
INGOT_FOUND=0

while true; do
    CURRENT=$(date +%s)
    ELAPSED=$((CURRENT - START_TIME))
    
    if [ $ELAPSED -ge 60 ]; then
        log "60 seconds elapsed"
        break
    fi
    
    # ゲームが動いているか確認
    if ! kill -0 $GAME_PID 2>/dev/null; then
        err "Game crashed!"
        exit 1
    fi
    
    # 10秒ごとにスクリーンショット
    if [ $((ELAPSED % 10)) -eq 0 ] && [ $ELAPSED -gt 0 ]; then
        log "Progress: ${ELAPSED}s"
        scrot "$LOG_DIR/progress_${ELAPSED}s_$TIMESTAMP.png" 2>/dev/null || true
    fi
    
    sleep 1
done

# 最終確認
log "--- Final check ---"
send_cmd "/assert inventory iron_ingot 1"
sleep 0.5
xdotool click 1
sleep 0.2

# 最終スクリーンショット
scrot "$LOG_DIR/final_$TIMESTAMP.png" 2>/dev/null || true

# インベントリ確認
send_cmd "/tp 2 11 2"
sleep 0.3
xdotool click 1
xdotool key e  # インベントリ表示
sleep 0.5
scrot "$LOG_DIR/inventory_$TIMESTAMP.png" 2>/dev/null || true
xdotool key e  # 閉じる
sleep 0.2

cleanup

log ""
log "=== Scenario Test Complete ==="
log "Screenshots: $LOG_DIR"
ls -1 "$LOG_DIR"/*$TIMESTAMP* 2>/dev/null | while read f; do
    echo "  $(basename "$f")"
done

log "Check logs for /assert output"
