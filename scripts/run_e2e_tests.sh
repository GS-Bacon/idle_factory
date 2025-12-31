#!/bin/bash
# E2E Test Runner Script
# RDPセッション内で実行してください
#
# 使い方:
#   ./run_e2e_tests.sh          # 全テスト実行
#   ./run_e2e_tests.sh quick    # クイックテスト（基本操作のみ）
#   ./run_e2e_tests.sh conveyor # コンベアテストのみ

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# ディスプレイを検出
if [ -z "$DISPLAY" ]; then
    # RDPセッションのディスプレイを探す
    RDP_DISPLAY=$(ps aux | grep "Xorg.*xrdp" | grep -v grep | awk '{for(i=1;i<=NF;i++) if($i ~ /^:[0-9]+$/) print $i}' | sort -t: -k2 -n | tail -1)
    if [ -n "$RDP_DISPLAY" ]; then
        export DISPLAY="$RDP_DISPLAY"
        echo "Using RDP display: $DISPLAY"
    else
        # Xvfbにフォールバック
        export DISPLAY=":10"
        echo "Using Xvfb display: $DISPLAY"
    fi
fi

# スクリーンショットディレクトリ
SCREENSHOT_DIR="$SCRIPT_DIR/screenshots/e2e"
mkdir -p "$SCREENSHOT_DIR"

# ログディレクトリ
LOG_DIR="$SCRIPT_DIR/logs/e2e"
mkdir -p "$LOG_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="$LOG_DIR/e2e_$TIMESTAMP.log"
REPORT_FILE="$SCREENSHOT_DIR/report_$TIMESTAMP.html"

echo "=== E2E Test Runner ===" | tee "$LOG_FILE"
echo "Display: $DISPLAY" | tee -a "$LOG_FILE"
echo "Timestamp: $TIMESTAMP" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# ゲームを起動
echo "Starting game..." | tee -a "$LOG_FILE"
pkill -f idle_factory 2>/dev/null || true
sleep 1

cargo run &
GAME_PID=$!
echo "Game PID: $GAME_PID" | tee -a "$LOG_FILE"

# ゲームの起動を待つ
echo "Waiting for game to start..." | tee -a "$LOG_FILE"
sleep 10

# ウィンドウIDを取得
WINDOW_ID=$(xdotool search --name "Idle Factory" | head -1)
if [ -z "$WINDOW_ID" ]; then
    echo "ERROR: Game window not found!" | tee -a "$LOG_FILE"
    kill $GAME_PID 2>/dev/null || true
    exit 1
fi
echo "Window ID: $WINDOW_ID" | tee -a "$LOG_FILE"

# テスト関数
run_test() {
    local TEST_NAME="$1"
    local DESCRIPTION="$2"
    shift 2

    echo "" | tee -a "$LOG_FILE"
    echo "=== Test: $TEST_NAME ===" | tee -a "$LOG_FILE"
    echo "Description: $DESCRIPTION" | tee -a "$LOG_FILE"

    # ウィンドウをフォーカス
    xdotool windowactivate $WINDOW_ID
    sleep 0.3

    # テストコマンドを実行
    "$@"
    local RESULT=$?

    # スクリーンショット撮影
    local SCREENSHOT="$SCREENSHOT_DIR/${TEST_NAME}_$TIMESTAMP.png"
    scrot "$SCREENSHOT"
    echo "Screenshot: $SCREENSHOT" | tee -a "$LOG_FILE"

    if [ $RESULT -eq 0 ]; then
        echo "Result: PASSED" | tee -a "$LOG_FILE"
    else
        echo "Result: FAILED" | tee -a "$LOG_FILE"
    fi

    return $RESULT
}

# テスト用関数
test_dismiss_tutorial() {
    xdotool key --window $WINDOW_ID space
    sleep 2
}

test_click_to_lock() {
    xdotool mousemove --window $WINDOW_ID 400 300
    xdotool click --window $WINDOW_ID 1
    sleep 0.5
}

test_select_slot() {
    local SLOT="$1"
    xdotool key --window $WINDOW_ID "$SLOT"
    sleep 0.3
}

test_place_block() {
    xdotool click --window $WINDOW_ID 3
    sleep 0.5
}

test_break_block() {
    xdotool click --window $WINDOW_ID 1
    sleep 0.5
}

test_toggle_inventory() {
    xdotool key --window $WINDOW_ID e
    sleep 0.5
}

test_toggle_command() {
    xdotool key --window $WINDOW_ID t
    sleep 0.5
}

test_toggle_debug() {
    xdotool key --window $WINDOW_ID F3
    sleep 0.3
}

test_move_camera() {
    # マウスを少し動かす
    xdotool mousemove_relative 50 0
    sleep 0.1
    xdotool mousemove_relative -50 0
    sleep 0.1
}

test_enter_command() {
    local CMD="$1"
    xdotool type --window $WINDOW_ID "$CMD"
    sleep 0.2
    xdotool key --window $WINDOW_ID Return
    sleep 0.5
}

# テスト実行モード
TEST_MODE="${1:-full}"

echo "" | tee -a "$LOG_FILE"
echo "Test mode: $TEST_MODE" | tee -a "$LOG_FILE"

PASSED=0
FAILED=0

# チュートリアル閉じる
run_test "tutorial_dismiss" "チュートリアルを閉じる" test_dismiss_tutorial && ((PASSED++)) || ((FAILED++))

# ポインターロック取得
run_test "pointer_lock" "クリックでポインターロック取得" test_click_to_lock && ((PASSED++)) || ((FAILED++))

case $TEST_MODE in
    quick)
        # クイックテスト：基本操作のみ
        run_test "select_miner" "採掘機選択（キー1）" test_select_slot 1 && ((PASSED++)) || ((FAILED++))
        run_test "place_miner" "採掘機設置" test_place_block && ((PASSED++)) || ((FAILED++))
        run_test "select_conveyor" "コンベア選択（キー2）" test_select_slot 2 && ((PASSED++)) || ((FAILED++))
        run_test "place_conveyor" "コンベア設置" test_place_block && ((PASSED++)) || ((FAILED++))
        run_test "inventory_open" "インベントリを開く" test_toggle_inventory && ((PASSED++)) || ((FAILED++))
        run_test "inventory_close" "インベントリを閉じる" test_toggle_inventory && ((PASSED++)) || ((FAILED++))
        ;;

    conveyor)
        # コンベアテスト
        run_test "select_conveyor" "コンベア選択" test_select_slot 2 && ((PASSED++)) || ((FAILED++))
        run_test "place_conveyor_1" "コンベア設置1" test_place_block && ((PASSED++)) || ((FAILED++))
        run_test "move_for_conveyor_2" "移動" test_move_camera && ((PASSED++)) || ((FAILED++))
        run_test "place_conveyor_2" "コンベア設置2" test_place_block && ((PASSED++)) || ((FAILED++))
        run_test "move_for_conveyor_3" "移動" test_move_camera && ((PASSED++)) || ((FAILED++))
        run_test "place_conveyor_3" "コンベア設置3" test_place_block && ((PASSED++)) || ((FAILED++))
        ;;

    full|*)
        # フルテスト

        # ブロック設置テスト
        echo "" | tee -a "$LOG_FILE"
        echo "=== Block Placement Tests ===" | tee -a "$LOG_FILE"

        run_test "select_miner" "採掘機選択（キー1）" test_select_slot 1 && ((PASSED++)) || ((FAILED++))
        run_test "place_miner" "採掘機設置" test_place_block && ((PASSED++)) || ((FAILED++))

        run_test "select_conveyor" "コンベア選択（キー2）" test_select_slot 2 && ((PASSED++)) || ((FAILED++))
        run_test "place_conveyor" "コンベア設置" test_place_block && ((PASSED++)) || ((FAILED++))

        run_test "select_crusher" "粉砕機選択（キー3）" test_select_slot 3 && ((PASSED++)) || ((FAILED++))
        run_test "place_crusher" "粉砕機設置" test_place_block && ((PASSED++)) || ((FAILED++))

        run_test "select_furnace" "精錬炉選択（キー4）" test_select_slot 4 && ((PASSED++)) || ((FAILED++))
        run_test "place_furnace" "精錬炉設置" test_place_block && ((PASSED++)) || ((FAILED++))

        # UI表示テスト
        echo "" | tee -a "$LOG_FILE"
        echo "=== UI Display Tests ===" | tee -a "$LOG_FILE"

        run_test "inventory_open" "インベントリを開く" test_toggle_inventory && ((PASSED++)) || ((FAILED++))
        run_test "inventory_close" "インベントリを閉じる" test_toggle_inventory && ((PASSED++)) || ((FAILED++))

        run_test "command_open" "コマンド入力を開く" test_toggle_command && ((PASSED++)) || ((FAILED++))
        # ESCで閉じる
        xdotool key --window $WINDOW_ID Escape
        sleep 0.3

        run_test "debug_hud" "デバッグHUD表示" test_toggle_debug && ((PASSED++)) || ((FAILED++))
        run_test "debug_hud_off" "デバッグHUD非表示" test_toggle_debug && ((PASSED++)) || ((FAILED++))

        # ホットバー選択テスト
        echo "" | tee -a "$LOG_FILE"
        echo "=== Hotbar Selection Tests ===" | tee -a "$LOG_FILE"

        for i in 1 2 3 4 5 6 7 8 9; do
            run_test "hotbar_$i" "ホットバースロット$i選択" test_select_slot $i && ((PASSED++)) || ((FAILED++))
        done

        # ゲームモードテスト
        echo "" | tee -a "$LOG_FILE"
        echo "=== Game Mode Tests ===" | tee -a "$LOG_FILE"

        test_toggle_command
        run_test "creative_mode" "クリエイティブモードに変更" test_enter_command "/creative" && ((PASSED++)) || ((FAILED++))

        test_toggle_command
        run_test "survival_mode" "サバイバルモードに変更" test_enter_command "/survival" && ((PASSED++)) || ((FAILED++))
        ;;
esac

# 結果サマリー
echo "" | tee -a "$LOG_FILE"
echo "=== Test Summary ===" | tee -a "$LOG_FILE"
echo "Passed: $PASSED" | tee -a "$LOG_FILE"
echo "Failed: $FAILED" | tee -a "$LOG_FILE"
echo "Total: $((PASSED + FAILED))" | tee -a "$LOG_FILE"

# ゲーム終了
echo "" | tee -a "$LOG_FILE"
echo "Stopping game..." | tee -a "$LOG_FILE"
kill $GAME_PID 2>/dev/null || true

# HTMLレポート生成
echo "" | tee -a "$LOG_FILE"
echo "Generating HTML report..." | tee -a "$LOG_FILE"

cat > "$REPORT_FILE" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>E2E Test Report - $TIMESTAMP</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #1a1a2e; color: #eee; }
        h1 { color: #0f0; }
        .summary { font-size: 1.2em; margin: 20px 0; }
        .passed { color: #0f0; }
        .failed { color: #f00; }
        .test { margin: 15px 0; padding: 15px; background: #16213e; border-radius: 8px; }
        .test h3 { margin: 0 0 10px 0; }
        img { max-width: 600px; border: 2px solid #333; margin: 10px 0; }
        .gallery { display: flex; flex-wrap: wrap; gap: 10px; }
        .gallery img { max-width: 300px; }
    </style>
</head>
<body>
    <h1>E2E Test Report</h1>
    <p>Generated: $(date)</p>
    <p>Display: $DISPLAY</p>
    <p>Mode: $TEST_MODE</p>

    <div class="summary">
        <span class="passed">Passed: $PASSED</span> |
        <span class="failed">Failed: $FAILED</span> |
        Total: $((PASSED + FAILED))
    </div>

    <h2>Screenshots</h2>
    <div class="gallery">
EOF

# スクリーンショットを追加
for img in "$SCREENSHOT_DIR"/*_$TIMESTAMP.png; do
    if [ -f "$img" ]; then
        NAME=$(basename "$img" .png | sed "s/_$TIMESTAMP//")
        echo "        <div class='test'><h3>$NAME</h3><img src='$(basename "$img")' alt='$NAME'></div>" >> "$REPORT_FILE"
    fi
done

cat >> "$REPORT_FILE" << EOF
    </div>

    <h2>Log</h2>
    <pre>$(cat "$LOG_FILE")</pre>
</body>
</html>
EOF

echo "Report: $REPORT_FILE" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"
echo "=== E2E Tests Complete ===" | tee -a "$LOG_FILE"

# 終了コード
if [ $FAILED -gt 0 ]; then
    exit 1
else
    exit 0
fi
