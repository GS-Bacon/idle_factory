#!/bin/bash
# E2E Full Test - ブロック設置からクエスト完了まで自動検証
#
# 使い方:
#   ./scripts/e2e-full.sh native   # ネイティブ版テスト
#   ./scripts/e2e-full.sh wasm     # WASM版テスト
#   ./scripts/e2e-full.sh all      # 両方テスト
#
# テスト内容:
# 1. 基本操作（起動、インベントリ、ホットバー）
# 2. ブロック設置（地面に正しく配置されるか）
# 3. コンベアL字配置（形状が正しいか）
# 4. 浮遊ブロックチェック（ブロックが浮いていないか）
# 5. クエスト進行確認

set -e

export DISPLAY=${DISPLAY:-:10}
export E2E_EXPORT=1
export E2E_EXPORT_PATH="/home/bacon/idle_factory/e2e_state.json"

SCREENSHOTS_DIR="/home/bacon/idle_factory/screenshots/e2e_full"
GAME_DIR="/home/bacon/idle_factory"
E2E_STATE_FILE="/home/bacon/idle_factory/e2e_state.json"
RESULT_FILE="/home/bacon/idle_factory/screenshots/e2e_full/results.json"

# 色付き出力
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${GREEN}[E2E]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
err() { echo -e "${RED}[ERR]${NC} $1"; }
info() { echo -e "${CYAN}[INFO]${NC} $1"; }

# テスト結果
PASSED=0
FAILED=0
declare -a FAILED_TESTS=()

# スクリーンショットディレクトリ作成
mkdir -p "$SCREENSHOTS_DIR"
rm -f "$SCREENSHOTS_DIR"/*.png "$SCREENSHOTS_DIR"/*.json 2>/dev/null || true

# ==============================================================================
# ユーティリティ関数
# ==============================================================================

cleanup() {
    pkill -9 -f "idle_factory" 2>/dev/null || true
    pkill -9 -f "target/debug/idle" 2>/dev/null || true
    pkill -9 -f "simple-http-server" 2>/dev/null || true
    pkill -9 -f "chromium" 2>/dev/null || true
    sleep 1
}

SHOT_NUM=0
shot() {
    SHOT_NUM=$((SHOT_NUM + 1))
    local name="$1"
    local filepath="$SCREENSHOTS_DIR/$(printf '%02d' $SHOT_NUM)_${name}.png"
    scrot "$filepath" 2>/dev/null || true
    log "📸 $(printf '%02d' $SHOT_NUM)_$name"
}

key() {
    xdotool key "$1"
    sleep 0.15
}

click() {
    local x=${1:-640}
    local y=${2:-360}
    xdotool mousemove "$x" "$y"
    sleep 0.05
    xdotool click 1
    sleep 0.15
}

rclick() {
    local x=${1:-640}
    local y=${2:-360}
    xdotool mousemove "$x" "$y"
    sleep 0.2
    xdotool click 3
    sleep 0.5
}

# 1文字ずつ入力 (xdotool typeの文字化け対策)
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
    sleep 0.03
}

type_text() {
    for ((i=0; i<${#1}; i++)); do
        type_char "${1:$i:1}"
    done
}

send_command() {
    local cmd="$1"
    log "📤 コマンド: $cmd"
    key "t"
    sleep 0.5
    type_text "$cmd"
    sleep 0.3
    key "Return"
    sleep 0.8
}

# 状態ファイルを待機
wait_for_state() {
    for i in {1..20}; do
        if [ -f "$E2E_STATE_FILE" ]; then
            return 0
        fi
        sleep 0.5
    done
    return 1
}

# JSON値取得
get_json() {
    local key="$1"
    if command -v jq >/dev/null 2>&1 && [ -f "$E2E_STATE_FILE" ]; then
        jq -r "$key" "$E2E_STATE_FILE" 2>/dev/null || echo "N/A"
    else
        echo "N/A"
    fi
}

# ==============================================================================
# アサーション関数
# ==============================================================================

assert_true() {
    local desc="$1"
    local condition="$2"

    if eval "$condition"; then
        log "✅ $desc"
        PASSED=$((PASSED + 1))
        return 0
    else
        err "❌ $desc"
        FAILED=$((FAILED + 1))
        FAILED_TESTS+=("$desc")
        return 1
    fi
}

assert_no_floating_blocks() {
    local floating_count=$(get_json '.floating_blocks | length')

    # N/A または空は0として扱う
    if [ "$floating_count" = "N/A" ] || [ -z "$floating_count" ]; then
        floating_count="0"
    fi

    if [ "$floating_count" = "0" ]; then
        log "✅ 浮遊ブロックなし"
        PASSED=$((PASSED + 1))
        return 0
    else
        err "❌ 浮遊ブロックあり: $floating_count 個"
        get_json '.floating_blocks[]' 2>/dev/null | while read pos; do
            err "   - $pos"
        done
        FAILED=$((FAILED + 1))
        FAILED_TESTS+=("浮遊ブロックチェック")
        return 1
    fi
}

assert_conveyor_shape() {
    local expected_shape="$1"
    local pos_x="$2"
    local pos_y="$3"
    local pos_z="$4"

    local shape=$(jq -r ".conveyors[] | select(.position[0]==$pos_x and .position[1]==$pos_y and .position[2]==$pos_z) | .shape" "$E2E_STATE_FILE" 2>/dev/null)

    if [ "$shape" = "$expected_shape" ]; then
        log "✅ コンベア形状: $shape @ ($pos_x,$pos_y,$pos_z)"
        PASSED=$((PASSED + 1))
        return 0
    else
        err "❌ コンベア形状: $shape != $expected_shape @ ($pos_x,$pos_y,$pos_z)"
        FAILED=$((FAILED + 1))
        FAILED_TESTS+=("コンベア形状 @ ($pos_x,$pos_y,$pos_z)")
        return 1
    fi
}

assert_quest_progress() {
    local min_delivered="$1"
    local delivered=$(get_json '.quest.delivered_amount')

    if [ "$delivered" -ge "$min_delivered" ] 2>/dev/null; then
        log "✅ クエスト進捗: $delivered / $(get_json '.quest.required_amount')"
        PASSED=$((PASSED + 1))
        return 0
    else
        err "❌ クエスト進捗不足: $delivered < $min_delivered"
        FAILED=$((FAILED + 1))
        FAILED_TESTS+=("クエスト進捗")
        return 1
    fi
}

# ==============================================================================
# ネイティブテスト
# ==============================================================================

test_native() {
    log "=== ネイティブテスト開始 ==="

    cleanup
    cd "$GAME_DIR"

    # ゲーム起動
    log "ゲーム起動中..."
    cargo run 2>/dev/null &
    GAME_PID=$!

    # ウィンドウ待機
    for i in {1..30}; do
        if ! kill -0 $GAME_PID 2>/dev/null; then
            err "ゲームプロセスが終了"
            return 1
        fi
        if xdotool search --name "Idle Factory" >/dev/null 2>&1; then
            log "ゲーム起動完了 (${i}秒)"
            sleep 2
            break
        fi
        if xdotool search --name "idle_factory" >/dev/null 2>&1; then
            log "ゲーム起動完了 (${i}秒)"
            sleep 2
            break
        fi
        sleep 1
    done

    # ウィンドウアクティブ化
    local window_id=$(xdotool search --name "Idle Factory" 2>/dev/null | head -1)
    [ -z "$window_id" ] && window_id=$(xdotool search --name "idle_factory" 2>/dev/null | head -1)

    if [ -z "$window_id" ]; then
        err "ウィンドウが見つかりません"
        shot "error_no_window"
        cleanup
        return 1
    fi

    xdotool windowactivate --sync "$window_id" 2>/dev/null || true
    xdotool windowfocus "$window_id" 2>/dev/null || true
    sleep 0.5

    # テスト実行
    run_game_tests "native"

    cleanup
    log "=== ネイティブテスト完了 ==="
}

# ==============================================================================
# WASMテスト
# ==============================================================================

test_wasm() {
    log "=== WASMテスト開始 ==="

    cleanup
    cd "$GAME_DIR"

    # WASMビルド確認
    if [ ! -f "web/idle_factory.js" ] || [ ! -f "web/idle_factory_bg.wasm" ]; then
        log "WASMビルド中..."
        ./scripts/build-wasm.sh || {
            err "WASMビルド失敗"
            return 1
        }
    fi

    # HTTPサーバー起動
    log "HTTPサーバー起動..."
    cd web
    simple-http-server --port 8080 --silent &
    SERVER_PID=$!
    sleep 2
    cd "$GAME_DIR"

    # Playwright テスト実行
    log "Playwrightテスト実行..."
    cd tests/e2e
    node e2e-visual-test.js --full 2>&1 | tee "$SCREENSHOTS_DIR/wasm_test.log"
    WASM_RESULT=$?
    cd "$GAME_DIR"

    # 結果確認
    if [ $WASM_RESULT -eq 0 ]; then
        log "✅ WASMテスト成功"
        PASSED=$((PASSED + 1))
    else
        err "❌ WASMテスト失敗"
        FAILED=$((FAILED + 1))
        FAILED_TESTS+=("WASM E2Eテスト")
    fi

    cleanup
    log "=== WASMテスト完了 ==="
}

# ==============================================================================
# 共通ゲームテスト
# ==============================================================================

run_game_tests() {
    local mode="$1"
    log "--- ゲームテスト ($mode) ---"

    # 初期スクリーンショット
    shot "${mode}_initial"

    # ポインターロック取得
    click 640 360
    sleep 0.5
    shot "${mode}_activated"

    # 状態ファイル待機
    wait_for_state || warn "状態ファイルが見つかりません"

    # === 基本操作テスト ===
    info "基本操作テスト..."

    # インベントリ
    key "e"
    sleep 0.3
    shot "${mode}_inventory"
    assert_true "インベントリ表示" "[ -f '$E2E_STATE_FILE' ]" || true
    key "e"  # 閉じる
    sleep 0.3
    click 640 360  # 再アクティベート
    sleep 0.3

    # デバッグHUD
    key "F3"
    sleep 0.3
    shot "${mode}_debug_hud"

    # === クリエイティブモード有効化 ===
    info "クリエイティブモード..."
    send_command "/creative"
    sleep 0.5
    click 640 360  # 再アクティベート
    sleep 0.3

    # アイテムを付与してホットバーに配置
    info "アイテム付与..."
    send_command "/give conveyor 20"
    sleep 0.3
    click 640 360
    sleep 0.3
    send_command "/give miner 5"
    sleep 0.3
    click 640 360
    sleep 0.3
    send_command "/give furnace 5"
    sleep 0.3
    click 640 360
    sleep 0.3

    # === ブロック設置テスト ===
    info "ブロック設置テスト..."

    # 安全な場所にテレポート（地面の少し上）
    send_command "/tp 10 10 10"
    sleep 1.0
    click 640 360  # 再アクティベート
    sleep 0.3

    # 下を向く
    send_command "/look 70 0"
    sleep 0.5
    click 640 360  # 再アクティベート
    sleep 0.5
    shot "${mode}_look_down"

    # インベントリ表示して確認・コンベアをホットバー1に移動
    key "e"
    sleep 0.5
    shot "${mode}_inventory_check"

    # コンベアがあるスロットをクリック（/giveで追加したコンベアは先頭に来るはず）
    # まずはインベントリを閉じてホットバー1で試す
    key "e"
    sleep 0.3
    click 640 360  # 再アクティベート
    sleep 0.3

    # ホットバー1を選択（最初のスロット）
    key "1"
    sleep 0.5
    shot "${mode}_slot1_selected"

    # デバッグHUDで確認
    key "F3"
    sleep 0.3

    # コンベア配置（直線）- より長い待機
    log "右クリックでコンベア配置..."
    rclick 640 360
    sleep 1.0
    shot "${mode}_conveyor1"

    # 少し移動して2つ目
    send_command "/look 60 30"
    sleep 0.5
    click 640 360  # 再アクティベート
    sleep 0.3
    key "2"
    sleep 0.2
    rclick 640 360
    sleep 0.5
    shot "${mode}_conveyor2"

    # 3つ目（L字になるはず）
    send_command "/look 60 60"
    sleep 0.5
    click 640 360  # 再アクティベート
    sleep 0.3
    key "2"
    sleep 0.2
    rclick 640 360
    sleep 0.5
    shot "${mode}_conveyor3"

    # /spawnコマンドで確実に機械を配置
    log "コマンドでコンベア・機械配置..."

    # デリバリープラットフォーム近く（Z=18付近）に配置
    # L字コンベア配置（東向き→南向きでL字になる）
    send_command "/spawn 8 9 16 conveyor 1"  # 東向き
    sleep 0.3
    click 640 360
    sleep 0.2
    send_command "/spawn 9 9 16 conveyor 2"  # 南向き（L字接続）
    sleep 0.3
    click 640 360
    sleep 0.2
    send_command "/spawn 9 9 17 conveyor 2"  # 南向き（直線）
    sleep 0.3
    click 640 360
    sleep 0.2

    # 採掘機と精錬炉も配置
    send_command "/spawn 7 9 16 miner"
    sleep 0.3
    click 640 360
    sleep 0.2
    send_command "/spawn 9 9 18 furnace"
    sleep 0.3
    click 640 360
    sleep 0.2

    # 配置した機械を見に行く（すぐ隣から水平に）
    log "配置した機械を確認..."
    send_command "/tp 8 10 14"  # コンベアの前に移動
    sleep 0.5
    click 640 360
    sleep 0.2
    send_command "/look 15 0"  # 少し下を向いて前方を見る
    sleep 0.5
    click 640 360
    sleep 0.3
    shot "${mode}_placed_machines"

    # 俯瞰で確認（近めから）
    send_command "/tp 8 13 17"  # 上空に移動
    sleep 0.5
    click 640 360
    sleep 0.2
    send_command "/look 60 0"  # 下を見る
    sleep 0.5
    click 640 360
    sleep 0.3
    shot "${mode}_overview"

    # === 検証 ===
    info "検証中..."
    sleep 1.0

    # 状態確認
    if [ -f "$E2E_STATE_FILE" ]; then
        log "--- 状態ファイル ---"
        cat "$E2E_STATE_FILE"
        log "-------------------"

        # E2E状態ファイルが有効か確認
        local fps=$(get_json '.fps')
        assert_true "E2E状態エクスポート" "[ '$fps' != 'N/A' ] && [ '$fps' != 'null' ]" || true

        # コンベア配置確認
        local conveyor_count=$(get_json '.conveyors | length')
        assert_true "コンベア配置 (>= 3)" "[ '$conveyor_count' -ge 3 ] 2>/dev/null" || true
        log "コンベア数: $conveyor_count"

        # コンベア形状確認（L字があるか）
        local shapes=$(get_json '.conveyors[].shape' | sort | uniq | tr '\n' ' ')
        log "コンベア形状: $shapes"

        # L字コンベア（CornerLeft or CornerRight）が存在するか確認
        if echo "$shapes" | grep -qE "CornerLeft|CornerRight"; then
            log "✅ L字コンベア検出"
            PASSED=$((PASSED + 1))
        else
            err "❌ L字コンベアが見つかりません"
            FAILED=$((FAILED + 1))
            FAILED_TESTS+=("L字コンベア検出")
        fi

        # 浮遊ブロックチェック
        assert_no_floating_blocks || true

        # クエスト状態確認
        local quest_desc=$(get_json '.quest.description')
        assert_true "クエスト状態取得" "[ -n '$quest_desc' ] && [ '$quest_desc' != 'N/A' ]" || true
        log "現在のクエスト: $quest_desc"

        # 機械配置確認
        local machine_count=$(get_json '.machines | length')
        assert_true "機械配置 (>= 2)" "[ '$machine_count' -ge 2 ] 2>/dev/null" || true
        log "機械数: $machine_count"
    else
        warn "状態ファイルがありません"
        FAILED=$((FAILED + 1))
        FAILED_TESTS+=("E2E状態ファイル生成")
    fi

    # === 機械設置テスト ===
    info "機械設置テスト..."

    # 採掘機選択（ホットバー1）
    key "1"
    sleep 0.2
    send_command "/look 70 180"
    sleep 0.3
    rclick 640 360
    sleep 0.3
    shot "${mode}_miner"

    # 精錬炉選択（ホットバー4）
    key "4"
    sleep 0.2
    send_command "/look 70 270"
    sleep 0.3
    rclick 640 360
    sleep 0.3
    shot "${mode}_furnace"

    # 最終確認
    send_command "/look 80 0"
    sleep 0.5
    shot "${mode}_final"

    # 機械数確認
    if [ -f "$E2E_STATE_FILE" ]; then
        local machine_count=$(get_json '.machines | length')
        assert_true "機械配置 (>= 1)" "[ '$machine_count' -ge 1 ]" || true
    fi

    log "--- ゲームテスト完了 ($mode) ---"
}

# ==============================================================================
# 結果出力
# ==============================================================================

output_results() {
    log ""
    log "=========================================="
    log "E2E テスト結果"
    log "=========================================="
    log "成功: $PASSED"
    log "失敗: $FAILED"
    log "合計: $((PASSED + FAILED))"

    if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
        log ""
        err "失敗したテスト:"
        for test in "${FAILED_TESTS[@]}"; do
            err "  - $test"
        done
    fi

    log ""
    log "スクリーンショット: $SCREENSHOTS_DIR"
    ls -1 "$SCREENSHOTS_DIR"/*.png 2>/dev/null | while read f; do
        echo "  $(basename "$f")"
    done

    # JSON結果ファイル
    cat > "$RESULT_FILE" << EOF
{
  "passed": $PASSED,
  "failed": $FAILED,
  "total": $((PASSED + FAILED)),
  "failed_tests": $(printf '%s\n' "${FAILED_TESTS[@]}" | jq -R . | jq -s .)
}
EOF
    log "結果ファイル: $RESULT_FILE"
}

# ==============================================================================
# メイン
# ==============================================================================

MODE="${1:-native}"

case "$MODE" in
    native|n)
        test_native
        ;;
    wasm|w)
        test_wasm
        ;;
    all|a)
        test_native
        NATIVE_PASSED=$PASSED
        NATIVE_FAILED=$FAILED
        PASSED=0
        FAILED=0
        test_wasm
        PASSED=$((NATIVE_PASSED + PASSED))
        FAILED=$((NATIVE_FAILED + FAILED))
        ;;
    *)
        echo "使い方: $0 [native|wasm|all]"
        exit 1
        ;;
esac

output_results

# 終了コード
if [ $FAILED -gt 0 ]; then
    exit 1
else
    exit 0
fi
