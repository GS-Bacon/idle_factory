#!/bin/bash
# E2E Quick Test - 高速スクリーンショットテスト
# 使い方: ./scripts/e2e-quick.sh [テスト名]

export DISPLAY=${DISPLAY:-:10}
SCREENSHOTS_DIR="/home/bacon/idle_factory/screenshots/verify"
GAME_DIR="/home/bacon/idle_factory"

# 色付き出力
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${GREEN}[E2E]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
err() { echo -e "${RED}[ERR]${NC} $1"; }

# スクリーンショットディレクトリ作成
mkdir -p "$SCREENSHOTS_DIR"

# 既存プロセス停止
cleanup() {
    pkill -9 -f "idle_factory" 2>/dev/null || true
    pkill -9 -f "target/debug/idle" 2>/dev/null || true
    sleep 1
}

# ゲーム起動して待機
start_game() {
    log "ゲーム起動中..."
    cleanup
    cd "$GAME_DIR"

    # バックグラウンドでゲーム起動
    cargo run 2>/dev/null &
    GAME_PID=$!

    # 起動待機（最大30秒）
    log "ウィンドウ待機中..."
    for i in {1..30}; do
        # プロセスが終了していないか確認
        if ! kill -0 $GAME_PID 2>/dev/null; then
            err "ゲームプロセスが終了しました"
            return 1
        fi

        # ウィンドウ検索（複数パターン）
        if xdotool search --name "Idle Factory" >/dev/null 2>&1; then
            log "ゲーム起動完了 (${i}秒)"
            sleep 2  # 描画完了待ち
            return 0
        fi

        if xdotool search --name "idle_factory" >/dev/null 2>&1; then
            log "ゲーム起動完了 (${i}秒)"
            sleep 2
            return 0
        fi

        sleep 1
    done

    warn "ゲーム起動タイムアウト（スクリーンショットで確認）"
    scrot "$SCREENSHOTS_DIR/timeout_check.png"
    return 1
}

# ウィンドウアクティブ化
activate_window() {
    local window_id
    window_id=$(xdotool search --name "Idle Factory" 2>/dev/null | head -1)
    [ -z "$window_id" ] && window_id=$(xdotool search --name "idle_factory" 2>/dev/null | head -1)

    if [ -n "$window_id" ]; then
        xdotool windowactivate --sync "$window_id" 2>/dev/null || true
        xdotool windowfocus "$window_id" 2>/dev/null || true
        sleep 0.5
        return 0
    fi
    warn "ウィンドウが見つかりません"
    return 1
}

# スクリーンショット撮影（連番付き）
SHOT_NUM=0
shot() {
    SHOT_NUM=$((SHOT_NUM + 1))
    local name="$1"
    local filepath="$SCREENSHOTS_DIR/$(printf '%02d' $SHOT_NUM)_${name}.png"
    scrot "$filepath"
    log "📸 $(printf '%02d' $SHOT_NUM)_$name"
}

# キー入力
key() {
    xdotool key "$1"
    sleep 0.15
}

# 複数キー連続入力
keys() {
    for k in "$@"; do
        key "$k"
    done
}

# マウス操作
click() {
    local x=${1:-640}
    local y=${2:-360}
    xdotool mousemove "$x" "$y"
    sleep 0.05
    xdotool click 1
    sleep 0.15
}

# 右クリック
rclick() {
    local x=${1:-640}
    local y=${2:-360}
    xdotool mousemove "$x" "$y"
    sleep 0.05
    xdotool click 3
    sleep 0.15
}

# テキスト入力
type_text() {
    xdotool type --delay 50 "$1"
    sleep 0.1
}

# =============================================================================
# テストケース
# =============================================================================

test_basic() {
    log "=== 基本テスト（6枚） ==="

    start_game || return 1

    shot "initial"
    activate_window
    click 640 360
    sleep 0.5
    shot "started"
    key "e"
    sleep 0.3
    shot "inventory"
    key "Escape"
    sleep 0.3
    shot "closed"
    key "F3"
    sleep 0.3
    shot "debug"
    key "F3"
    key "2"
    sleep 0.3
    shot "conveyor_mode"

    cleanup
    log "=== 完了: 6枚 ==="
}

test_conveyor() {
    log "=== コンベアテスト（8枚） ==="

    start_game || return 1

    activate_window
    click 640 360
    sleep 0.5

    # コンベアモード
    key "2"
    shot "cv_mode"

    # 直進コンベア x4
    click 450 350; click 500 350; click 550 350; click 600 350
    shot "cv_straight"

    # L字コンベア
    key "q"; click 650 350
    key "q"; click 700 350
    shot "cv_corners"

    # T字 + スプリッター
    key "q"; click 450 400
    key "q"; click 500 400
    shot "cv_t_splitter"

    # 回転して配置
    key "q"; key "r"; click 550 400
    shot "cv_rotated"

    # ズームイン
    for i in {1..12}; do xdotool click 4; sleep 0.03; done
    sleep 0.3
    shot "cv_zoomed"

    # 移動して近づく
    xdotool keydown d; sleep 0.3; xdotool keyup d
    xdotool keydown s; sleep 0.3; xdotool keyup s
    shot "cv_closeup"

    # デバッグ表示
    key "F3"
    shot "cv_debug"

    cleanup
    log "=== 完了: 8枚 ==="
}

test_machines() {
    log "=== 機械テスト（6枚） ==="

    start_game || return 1

    activate_window
    click 640 360
    sleep 0.5

    # 機械配置（採掘機、精錬炉、粉砕機）
    key "1"; click 500 350
    key "3"; click 550 350
    key "4"; click 600 350
    shot "mc_placed"

    # コンベア接続
    key "2"
    click 500 300; click 550 300; click 600 300
    shot "mc_connected"

    # 動作確認
    sleep 2
    shot "mc_working"

    # インベントリ
    key "e"
    sleep 0.3
    shot "mc_inventory"
    key "Escape"

    # 俯瞰
    for i in {1..12}; do xdotool click 5; sleep 0.03; done
    sleep 0.3
    shot "mc_overview"

    # デバッグ
    key "F3"
    shot "mc_debug"

    cleanup
    log "=== 完了: 6枚 ==="
}

test_full() {
    log "=== フルテスト（20枚） ==="
    test_basic
    SHOT_NUM=6  # リセット
    test_conveyor
    SHOT_NUM=14
    test_machines
    log "=== 全テスト完了: 20枚 ==="
}

# =============================================================================
# メイン
# =============================================================================

# 古いスクショを削除
rm -f "$SCREENSHOTS_DIR"/*.png 2>/dev/null || true

case "${1:-basic}" in
    basic|b)
        test_basic
        ;;
    conveyor|cv|c)
        test_conveyor
        ;;
    machines|mc|m)
        test_machines
        ;;
    full|all|f)
        test_full
        ;;
    *)
        echo "使い方: $0 [basic|conveyor|machines|full]"
        echo ""
        echo "  basic (b)    - 基本テスト（6枚）"
        echo "  conveyor (c) - コンベアテスト（8枚）"
        echo "  machines (m) - 機械テスト（6枚）"
        echo "  full (f)     - 全テスト（20枚）"
        exit 1
        ;;
esac

# 結果表示
echo ""
log "📂 $SCREENSHOTS_DIR"
echo "---"
ls -1 "$SCREENSHOTS_DIR"/*.png 2>/dev/null | while read f; do
    echo "  $(basename "$f")"
done
echo "---"
log "合計: $(ls -1 "$SCREENSHOTS_DIR"/*.png 2>/dev/null | wc -l) 枚"
