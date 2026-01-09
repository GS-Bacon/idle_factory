#!/bin/bash
# Bug Reproduction Test System
# 不具合報告時に自動的に再現確認を行うシステム
#
# 使い方:
#   ./scripts/bug-repro.sh add "左クリックで採掘できない"
#   ./scripts/bug-repro.sh run
#   ./scripts/bug-repro.sh list

set -e

export DISPLAY=${DISPLAY:-:10}
GAME_DIR="/home/bacon/idle_factory"
BUGS_DIR="$GAME_DIR/.claude/bug-tests"
SCREENSHOTS_DIR="$GAME_DIR/UIプレビュー/bug-repro"
GAME_LOG_FILE="/tmp/bug_repro_game_$$.log"
E2E_STATE_FILE="/home/bacon/idle_factory/e2e_state.json"
export E2E_EXPORT=1
export E2E_EXPORT_PATH="$E2E_STATE_FILE"

# 色付き出力
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${GREEN}[BUG-REPRO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
err() { echo -e "${RED}[ERR]${NC} $1"; }
info() { echo -e "${BLUE}[INFO]${NC} $1"; }

mkdir -p "$BUGS_DIR"
mkdir -p "$SCREENSHOTS_DIR"

# =============================================================================
# ゲーム操作ユーティリティ（e2e-quick.shから流用）
# =============================================================================

cleanup() {
    pkill -x idle_factory 2>/dev/null || true
    pkill -9 -f "target/debug/idle" 2>/dev/null || true

    # Check for UI bugs in log before cleanup
    if [ -f "$GAME_LOG_FILE" ]; then
        check_ui_bugs
    fi

    rm -f "$GAME_LOG_FILE" 2>/dev/null || true
    sleep 1
}

# Check for UI state bugs in log
check_ui_bugs() {
    if [ ! -f "$GAME_LOG_FILE" ]; then
        return 0
    fi

    local ui_bugs
    ui_bugs=$(grep -c "\[UI-BUG\]" "$GAME_LOG_FILE" 2>/dev/null) || ui_bugs=0

    if [ "$ui_bugs" -gt 0 ]; then
        err "=== UI State Bugs Detected: $ui_bugs ==="
        grep "\[UI-BUG\]" "$GAME_LOG_FILE" | while read line; do
            err "  $line"
        done
        err "========================================"
        return 1
    fi

    # Also show UI transitions for debugging
    local ui_transitions
    ui_transitions=$(grep -c "\[UI\]" "$GAME_LOG_FILE" 2>/dev/null) || ui_transitions=0
    if [ "$ui_transitions" -gt 0 ]; then
        log "UI State Transitions: $ui_transitions"
        grep "\[UI\]" "$GAME_LOG_FILE" | tail -10
    fi

    return 0
}

start_game() {
    log "ゲーム起動中..."
    cleanup
    cd "$GAME_DIR"

    cargo run --bin idle_factory 2>&1 > "$GAME_LOG_FILE" &
    GAME_PID=$!

    for i in {1..30}; do
        if ! kill -0 $GAME_PID 2>/dev/null; then
            err "ゲームプロセスが終了しました"
            cat "$GAME_LOG_FILE" | tail -30
            return 1
        fi

        if xdotool search --name "Idle Factory" >/dev/null 2>&1; then
            log "ゲーム起動完了 (${i}秒)"
            sleep 2
            return 0
        fi

        sleep 1
    done

    warn "ゲーム起動タイムアウト"
    return 1
}

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
    return 1
}

key() { xdotool key "$1"; sleep 0.15; }
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
    sleep 0.05
    xdotool click 3
    sleep 0.15
}

shot() {
    local name="$1"
    local filepath="$SCREENSHOTS_DIR/${name}.png"
    activate_window

    local window_id
    window_id=$(xdotool search --name "Idle Factory" 2>/dev/null | head -1)

    if [ -n "$window_id" ]; then
        import -window "$window_id" "$filepath" 2>/dev/null || scrot -u "$filepath" 2>/dev/null || scrot "$filepath"
    else
        scrot -u "$filepath" 2>/dev/null || scrot "$filepath"
    fi

    log "📸 $name"
    echo "$filepath"
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
    sleep 0.1
}

type_text() {
    for ((i=0; i<${#1}; i++)); do
        type_char "${1:$i:1}"
    done
    sleep 0.2
}

send_command() {
    local cmd="$1"
    log "📤 コマンド: $cmd"
    key "t"
    sleep 0.3
    type_text "$cmd"
    sleep 0.2
    key "Return"
    sleep 0.5
}

get_state() {
    if [ -f "$E2E_STATE_FILE" ]; then
        cat "$E2E_STATE_FILE"
    else
        echo "{}"
    fi
}

# =============================================================================
# バグテスト定義
# =============================================================================

# バグIDを生成（タイムスタンプベース）
generate_bug_id() {
    echo "BUG_$(date +%Y%m%d_%H%M%S)"
}

# バグテストファイルを作成
create_bug_test() {
    local description="$1"
    local bug_id=$(generate_bug_id)
    local test_file="$BUGS_DIR/${bug_id}.sh"

    cat > "$test_file" << 'TEMPLATE'
#!/bin/bash
# Bug Test: BUG_ID
# Description: BUG_DESCRIPTION
# Created: BUG_DATE
# Status: pending (pending/reproduced/fixed/cannot-reproduce)

# Source common functions
source "$(dirname "$0")/../bug-repro.sh" --source-only 2>/dev/null || true

bug_setup() {
    # テスト前の準備
    start_game || return 1
    activate_window
    click 640 360  # チュートリアルdismiss
    sleep 1
}

bug_reproduce() {
    # 再現手順をここに記述
    # 例:
    # key "e"  # インベントリを開く
    # shot "step1_inventory_open"
    # key "Escape"
    # shot "step2_after_escape"

    echo "TODO: 再現手順を記述してください"
    return 1
}

bug_verify_broken() {
    # バグが発生していることを確認
    # 戻り値: 0=バグあり（期待通り壊れている）, 1=バグなし（正常動作）
    echo "TODO: バグ発生の確認ロジックを記述"
    return 1
}

bug_verify_fixed() {
    # バグが修正されていることを確認
    # 戻り値: 0=修正済み, 1=未修正
    echo "TODO: 修正確認ロジックを記述"
    return 1
}

bug_cleanup() {
    cleanup
}

# メイン実行
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "=== Bug Test: BUG_ID ==="
    echo "Description: BUG_DESCRIPTION"
    echo ""

    bug_setup || { echo "Setup failed"; exit 1; }

    if bug_reproduce; then
        echo "✅ 再現成功"
        if bug_verify_broken; then
            echo "🐛 バグ確認: 発生している"
        else
            echo "❓ バグ確認: 発生していない（修正済み？）"
        fi
    else
        echo "❌ 再現失敗"
    fi

    bug_cleanup
fi
TEMPLATE

    # テンプレートを実際の値で置換
    sed -i "s/BUG_ID/$bug_id/g" "$test_file"
    sed -i "s/BUG_DESCRIPTION/$description/g" "$test_file"
    sed -i "s/BUG_DATE/$(date '+%Y-%m-%d %H:%M:%S')/g" "$test_file"

    chmod +x "$test_file"

    log "バグテストを作成しました: $test_file"
    echo ""
    info "次のステップ:"
    info "1. $test_file を編集して再現手順を記述"
    info "2. ./scripts/bug-repro.sh run $bug_id で実行"
}

# =============================================================================
# 組み込みバグテスト（よくあるパターン）
# =============================================================================

# 左クリック採掘テスト
test_left_click_mining() {
    log "=== 左クリック採掘テスト ==="

    start_game || return 1
    activate_window

    # チュートリアルをスキップ
    click 640 360
    sleep 1
    shot "mining_01_initial"

    # カーソルをロック（画面クリック）
    click 640 360
    sleep 0.5

    # 下を向く
    send_command "/look 60 0"
    sleep 0.3
    shot "mining_02_looking_down"

    # 左クリックで採掘（長押し）
    log "左クリック長押しで採掘..."
    xdotool mousedown 1
    sleep 2
    xdotool mouseup 1
    sleep 0.5
    shot "mining_03_after_click"

    # 状態確認
    local state=$(get_state)
    log "ゲーム状態: $state"

    cleanup
    log "=== 左クリック採掘テスト完了 ==="
}

# Eキーインベントリ→ポーズメニュー表示テスト
test_inventory_pause_bug() {
    log "=== インベントリ→ポーズメニューバグテスト ==="

    start_game || return 1
    activate_window

    # チュートリアルをスキップ
    click 640 360
    sleep 1
    shot "inv_pause_01_initial"

    # カーソルロック
    click 640 360
    sleep 0.5

    # Eキーでインベントリを開く
    key "e"
    sleep 0.5
    shot "inv_pause_02_after_e"

    # ポーズメニューが表示されていないか確認
    # (スクリーンショットで目視確認)

    # ESCで閉じる
    key "Escape"
    sleep 0.5
    shot "inv_pause_03_after_esc"

    cleanup
    log "=== インベントリ→ポーズメニューバグテスト完了 ==="
}

# ESCポーズ中の背景操作テスト
test_pause_background_control() {
    log "=== ポーズ中背景操作テスト ==="

    start_game || return 1
    activate_window

    # チュートリアルをスキップ
    click 640 360
    sleep 1

    # カーソルロック
    click 640 360
    sleep 0.5
    shot "pause_bg_01_playing"

    # ESCでポーズ
    key "Escape"
    sleep 0.5
    shot "pause_bg_02_paused"

    # ポーズ中にクリック
    click 640 360
    sleep 0.5
    shot "pause_bg_03_click_in_pause"

    # ポーズメニューが消えていないか確認

    cleanup
    log "=== ポーズ中背景操作テスト完了 ==="
}

# チュートリアル中のクエスト表示テスト
test_tutorial_quest_visibility() {
    log "=== チュートリアル中クエスト表示テスト ==="

    start_game || return 1
    activate_window
    sleep 2

    # チュートリアルポップアップ表示中にスクショ
    shot "tut_quest_01_tutorial_showing"

    # 右上にクエストUIが表示されていないか確認

    # チュートリアルをdismiss
    click 640 360
    sleep 1
    shot "tut_quest_02_tutorial_dismissed"

    # チュートリアル完了後にクエストUIが表示されるか
    # (チュートリアル全ステップを進める必要あり)

    cleanup
    log "=== チュートリアル中クエスト表示テスト完了 ==="
}

# =============================================================================
# メインコマンド
# =============================================================================

list_bugs() {
    log "=== 登録済みバグテスト ==="
    echo ""

    if [ -z "$(ls -A "$BUGS_DIR"/*.sh 2>/dev/null)" ]; then
        info "登録済みバグテストはありません"
        echo ""
        info "組み込みテスト:"
        echo "  mining      - 左クリック採掘テスト"
        echo "  inv-pause   - インベントリ→ポーズメニューバグ"
        echo "  pause-bg    - ポーズ中背景操作"
        echo "  tut-quest   - チュートリアル中クエスト表示"
        return
    fi

    for f in "$BUGS_DIR"/*.sh; do
        [ -f "$f" ] || continue
        local bug_id=$(basename "$f" .sh)
        local desc=$(grep "^# Description:" "$f" | cut -d: -f2- | xargs)
        local status=$(grep "^# Status:" "$f" | cut -d: -f2- | xargs)

        case "$status" in
            *fixed*) echo -e "${GREEN}✅ $bug_id${NC}: $desc" ;;
            *reproduced*) echo -e "${RED}🐛 $bug_id${NC}: $desc" ;;
            *cannot*) echo -e "${YELLOW}❓ $bug_id${NC}: $desc" ;;
            *) echo -e "${BLUE}📝 $bug_id${NC}: $desc" ;;
        esac
    done

    echo ""
    info "組み込みテスト:"
    echo "  mining      - 左クリック採掘テスト"
    echo "  inv-pause   - インベントリ→ポーズメニューバグ"
    echo "  pause-bg    - ポーズ中背景操作"
    echo "  tut-quest   - チュートリアル中クエスト表示"
}

run_bug_test() {
    local test_name="$1"

    # 組み込みテスト
    case "$test_name" in
        mining)
            test_left_click_mining
            return $?
            ;;
        inv-pause)
            test_inventory_pause_bug
            return $?
            ;;
        pause-bg)
            test_pause_background_control
            return $?
            ;;
        tut-quest)
            test_tutorial_quest_visibility
            return $?
            ;;
    esac

    # カスタムテスト
    local test_file="$BUGS_DIR/${test_name}.sh"
    if [ -f "$test_file" ]; then
        bash "$test_file"
        return $?
    fi

    err "テストが見つかりません: $test_name"
    return 1
}

run_all_tests() {
    log "=== 全バグテスト実行 ==="

    local passed=0
    local failed=0

    # 組み込みテスト
    for test in mining inv-pause pause-bg tut-quest; do
        echo ""
        if run_bug_test "$test"; then
            passed=$((passed + 1))
        else
            failed=$((failed + 1))
        fi
    done

    # カスタムテスト
    for f in "$BUGS_DIR"/*.sh; do
        [ -f "$f" ] || continue
        local bug_id=$(basename "$f" .sh)
        echo ""
        if bash "$f"; then
            passed=$((passed + 1))
        else
            failed=$((failed + 1))
        fi
    done

    echo ""
    log "=== 結果: ✅ $passed 成功, ❌ $failed 失敗 ==="
}

show_help() {
    echo "Bug Reproduction Test System"
    echo ""
    echo "使い方: $0 <command> [args]"
    echo ""
    echo "コマンド:"
    echo "  add <description>  - 新しいバグテストを作成"
    echo "  list               - 登録済みバグテスト一覧"
    echo "  run <test>         - 特定のテストを実行"
    echo "  run-all            - 全テスト実行"
    echo ""
    echo "組み込みテスト:"
    echo "  mining      - 左クリック採掘テスト"
    echo "  inv-pause   - インベントリ→ポーズメニューバグ"
    echo "  pause-bg    - ポーズ中背景操作"
    echo "  tut-quest   - チュートリアル中クエスト表示"
    echo ""
    echo "ワークフロー:"
    echo "  1. バグ報告を受ける"
    echo "  2. ./scripts/bug-repro.sh run <組み込みテスト> で再現確認"
    echo "  3. スクリーンショットで問題を確認"
    echo "  4. 修正を実装"
    echo "  5. 再度テスト実行で修正確認"
    echo ""
    echo "スクリーンショット保存先: $SCREENSHOTS_DIR"
}

# =============================================================================
# エントリポイント
# =============================================================================

# --source-only オプションで関数だけをエクスポート
if [[ "$1" == "--source-only" ]]; then
    return 0 2>/dev/null || exit 0
fi

case "${1:-help}" in
    add)
        if [ -z "$2" ]; then
            err "説明を指定してください: $0 add \"バグの説明\""
            exit 1
        fi
        create_bug_test "$2"
        ;;
    list|ls)
        list_bugs
        ;;
    run)
        if [ -z "$2" ]; then
            err "テスト名を指定してください: $0 run <test>"
            exit 1
        fi
        rm -f "$SCREENSHOTS_DIR"/*.png 2>/dev/null || true
        run_bug_test "$2"
        echo ""
        log "📂 スクリーンショット: $SCREENSHOTS_DIR"
        ls -1 "$SCREENSHOTS_DIR"/*.png 2>/dev/null || true
        ;;
    run-all|all)
        rm -f "$SCREENSHOTS_DIR"/*.png 2>/dev/null || true
        run_all_tests
        ;;
    help|-h|--help)
        show_help
        ;;
    *)
        show_help
        exit 1
        ;;
esac
