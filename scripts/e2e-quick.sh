#!/bin/bash
# E2E Quick Test - 高速スクリーンショットテスト
# 使い方: ./scripts/e2e-quick.sh [テスト名]

export DISPLAY=${DISPLAY:-:10}
export E2E_EXPORT=1
export E2E_EXPORT_PATH="/home/bacon/idle_factory/e2e_state.json"
SCREENSHOTS_DIR="/home/bacon/idle_factory/screenshots/verify"
GAME_DIR="/home/bacon/idle_factory"
E2E_STATE_FILE="/home/bacon/idle_factory/e2e_state.json"
GAME_LOG_FILE="/tmp/e2e_game_$$.log"
PANIC_DETECTED=0

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
    pkill -x idle_factory 2>/dev/null || true
    pkill -9 -f "target/debug/idle" 2>/dev/null || true
    rm -f "$GAME_LOG_FILE" 2>/dev/null || true
    sleep 1
}

# panic検出
check_panic() {
    if [ -f "$GAME_LOG_FILE" ]; then
        if grep -qi "panic\|thread.*panicked\|RUST_BACKTRACE" "$GAME_LOG_FILE" 2>/dev/null; then
            PANIC_DETECTED=1
            err "🚨 PANIC検出!"
            err "=== ログ最後の30行 ==="
            tail -30 "$GAME_LOG_FILE"
            err "======================"
            return 1
        fi
    fi
    return 0
}

# ゲーム起動して待機
start_game() {
    log "ゲーム起動中..."
    cleanup
    cd "$GAME_DIR"

    # バックグラウンドでゲーム起動（ログを記録）
    cargo run --bin idle_factory 2>&1 > "$GAME_LOG_FILE" &
    GAME_PID=$!

    # 起動待機（最大30秒）
    log "ウィンドウ待機中..."
    for i in {1..30}; do
        # プロセスが終了していないか確認
        if ! kill -0 $GAME_PID 2>/dev/null; then
            err "ゲームプロセスが終了しました"
            check_panic
            return 1
        fi

        # panic検出
        if ! check_panic; then
            cleanup
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

# スクリーンショット撮影（連番付き、ウィンドウ指定）
SHOT_NUM=0
shot() {
    SHOT_NUM=$((SHOT_NUM + 1))
    local name="$1"
    local filepath="$SCREENSHOTS_DIR/$(printf '%02d' $SHOT_NUM)_${name}.png"

    # 撮影前にウィンドウをアクティブ化して確認
    activate_window

    # panic検出
    if ! check_panic; then
        err "📸 $(printf '%02d' $SHOT_NUM)_$name - PANIC検出のためスキップ"
        return 1
    fi

    # ウィンドウIDを取得してウィンドウ指定撮影
    local window_id
    window_id=$(xdotool search --name "Idle Factory" 2>/dev/null | head -1)
    [ -z "$window_id" ] && window_id=$(xdotool search --name "idle_factory" 2>/dev/null | head -1)

    if [ -n "$window_id" ]; then
        # ImageMagickのimportでウィンドウ指定撮影
        import -window "$window_id" "$filepath" 2>/dev/null || scrot -u "$filepath" 2>/dev/null || scrot "$filepath"
    else
        # フォールバック：フォーカスウィンドウを撮影
        scrot -u "$filepath" 2>/dev/null || scrot "$filepath"
        warn "ウィンドウID取得失敗、フォールバック撮影"
    fi

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
    sleep 0.1  # 100ms delay to prevent character scrambling
}

# テキスト入力
type_text() {
    for ((i=0; i<${#1}; i++)); do
        type_char "${1:$i:1}"
    done
    sleep 0.2
}

# =============================================================================
# ゲーム状態取得・検証 (E2E_EXPORT連携)
# =============================================================================

# ゲーム状態を取得（JSON）
get_state() {
    if [ -f "$E2E_STATE_FILE" ]; then
        cat "$E2E_STATE_FILE"
    else
        echo "{}"
    fi
}

# JSONから値を取得 (jqが必要)
get_value() {
    local key="$1"
    if command -v jq >/dev/null 2>&1; then
        get_state | jq -r "$key"
    else
        warn "jqがインストールされていません"
        echo "N/A"
    fi
}

# プレイヤー位置を取得
get_player_pos() {
    get_value '.player_pos | "\(.[0]|round),\(.[1]|round),\(.[2]|round)"'
}

# カメラ向きを取得（度）
get_camera_dir() {
    get_value '(.camera_pitch * 57.3 | round | tostring) + "," + (.camera_yaw * 57.3 | round | tostring)'
}

# ターゲットブロック座標を取得
get_target_break() {
    get_value '.target_break | if . then "\(.[0]),\(.[1]),\(.[2])" else "None" end'
}

# ターゲット配置座標を取得
get_target_place() {
    get_value '.target_place | if . then "\(.[0]),\(.[1]),\(.[2])" else "None" end'
}

# 状態ログ出力
log_state() {
    local label="$1"
    sleep 0.2  # 状態更新待ち
    log "[$label] Pos:$(get_player_pos) Dir:$(get_camera_dir) Target:$(get_target_break) Place:$(get_target_place)"
}

# 位置検証（許容誤差付き）
assert_near_pos() {
    local expected_x="$1"
    local expected_y="$2"
    local expected_z="$3"
    local tolerance="${4:-2}"

    if ! command -v jq >/dev/null 2>&1; then
        warn "jqがないため位置検証をスキップ"
        return 0
    fi

    local state=$(get_state)
    local actual_x=$(echo "$state" | jq -r '.player_pos[0] | round')
    local actual_y=$(echo "$state" | jq -r '.player_pos[1] | round')
    local actual_z=$(echo "$state" | jq -r '.player_pos[2] | round')

    local dx=$(( actual_x - expected_x ))
    local dy=$(( actual_y - expected_y ))
    local dz=$(( actual_z - expected_z ))

    # 絶対値
    [ $dx -lt 0 ] && dx=$(( -dx ))
    [ $dy -lt 0 ] && dy=$(( -dy ))
    [ $dz -lt 0 ] && dz=$(( -dz ))

    if [ $dx -le $tolerance ] && [ $dy -le $tolerance ] && [ $dz -le $tolerance ]; then
        log "✅ 位置OK: ($actual_x,$actual_y,$actual_z) ≈ ($expected_x,$expected_y,$expected_z)"
        return 0
    else
        err "❌ 位置NG: ($actual_x,$actual_y,$actual_z) != ($expected_x,$expected_y,$expected_z)"
        return 1
    fi
}

# ターゲットブロック検証
assert_target() {
    local expected="$1"
    local actual=$(get_target_break)

    if [ "$actual" = "$expected" ]; then
        log "✅ ターゲットOK: $actual"
        return 0
    else
        err "❌ ターゲットNG: $actual != $expected"
        return 1
    fi
}

# カメラが下を向いているか検証
assert_looking_down() {
    if ! command -v jq >/dev/null 2>&1; then
        warn "jqがないため向き検証をスキップ"
        return 0
    fi

    local pitch=$(get_state | jq -r '.camera_pitch')
    local pitch_deg=$(echo "$pitch * 57.3" | bc -l 2>/dev/null || echo "0")

    # pitch > 30度 で下を向いていると判定
    if [ $(echo "$pitch_deg > 30" | bc -l 2>/dev/null || echo "0") -eq 1 ]; then
        log "✅ カメラは下向き (pitch=${pitch_deg}°)"
        return 0
    else
        warn "⚠ カメラが下を向いていない (pitch=${pitch_deg}°)"
        return 1
    fi
}

# =============================================================================
# 状態待機・検証 (中期タスク)
# =============================================================================

# 条件が満たされるまで待機 (jqクエリ)
# 使用例: wait_for '.creative_mode == true' 5
wait_for() {
    local jq_query="$1"
    local timeout="${2:-10}"
    local interval="${3:-0.5}"

    if ! command -v jq >/dev/null 2>&1; then
        warn "jqがないためwait_forをスキップ"
        sleep 1
        return 0
    fi

    local elapsed=0
    while [ $(echo "$elapsed < $timeout" | bc -l) -eq 1 ]; do
        local result=$(get_state | jq -r "$jq_query" 2>/dev/null)
        if [ "$result" = "true" ]; then
            log "✅ 条件成立: $jq_query (${elapsed}s)"
            return 0
        fi
        sleep "$interval"
        elapsed=$(echo "$elapsed + $interval" | bc -l)
    done

    warn "⚠ タイムアウト: $jq_query (${timeout}s)"
    return 1
}

# 状態変化を検証（変化前と変化後を比較）
# 使用例: assert_changed '.player_pos[0]'
assert_changed() {
    local jq_query="$1"
    local before=$(get_value "$jq_query")
    sleep 0.3
    local after=$(get_value "$jq_query")

    if [ "$before" != "$after" ]; then
        log "✅ 変化確認: $jq_query ($before → $after)"
        return 0
    else
        warn "⚠ 変化なし: $jq_query = $before"
        return 1
    fi
}

# =============================================================================
# ゲームコマンド送信 (Tキー → 入力 → Enter)
# =============================================================================

# ゲーム内コマンドを送信（結果を検証）
send_command() {
    local cmd="$1"
    log "📤 コマンド送信: $cmd"
    key "t"  # コマンド入力モードを開く
    sleep 0.3  # コマンドUIが開くまで待つ
    type_text "$cmd"
    sleep 0.2
    key "Return"
    sleep 0.5

    # panic検出
    if ! check_panic; then
        err "コマンド実行中にpanicが発生"
        return 1
    fi
    return 0
}

# テレポート
cmd_tp() {
    local x="$1"
    local y="$2"
    local z="$3"
    send_command "/tp $x $y $z"
}

# カメラ向きを設定（度）
cmd_look() {
    local pitch="$1"
    local yaw="$2"
    send_command "/look $pitch $yaw"
}

# ブロック配置
cmd_setblock() {
    local x="$1"
    local y="$2"
    local z="$3"
    local block="$4"
    send_command "/setblock $x $y $z $block"
}

# クリエイティブモード（結果を検証）
cmd_creative() {
    send_command "/creative"
    # 状態変化を待機して確認
    if wait_for '.creative_mode == true' 3; then
        log "✅ クリエイティブモード有効化"
        return 0
    else
        warn "⚠ クリエイティブモードの確認に失敗"
        return 1
    fi
}

# テレポート（結果を検証）
cmd_tp_verify() {
    local x="$1"
    local y="$2"
    local z="$3"
    send_command "/tp $x $y $z"
    sleep 0.3
    # 位置が近いか検証
    assert_near_pos "$x" "$y" "$z" 3
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
    log_state "起動後"

    key "e"
    sleep 0.3
    shot "inventory"
    key "Escape"
    sleep 0.3
    shot "closed"

    key "F3"
    sleep 0.3
    shot "debug"
    log_state "デバッグHUD"

    key "F3"
    key "2"
    sleep 0.3
    shot "conveyor_mode"
    log_state "コンベアモード"

    # 状態ファイルの内容を表示
    log "--- E2E状態ファイル ---"
    cat "$E2E_STATE_FILE" 2>/dev/null || warn "状態ファイルがありません"
    log "----------------------"

    cleanup
    log "=== 完了: 6枚 ==="
}

test_conveyor() {
    log "=== コンベアテスト（/look + クリック配置） ==="

    start_game || return 1

    activate_window
    click 640 360  # ポーズ解除
    sleep 1
    shot "cv_initial"
    log_state "初期状態"

    # ポーズ解除後にもう一度クリックしてフォーカス確保
    click 640 360
    sleep 0.5

    # クリエイティブモード（アイテム付与）
    send_command "/creative"
    sleep 0.3

    # 地面の近くにテレポート (Y=10 = 地面より少し上)
    send_command "/tp 8 10 20"
    sleep 0.3

    # コンベアを選択（ホットバー2）
    key "2"
    sleep 0.2

    # カメラを下に向ける (70度下向き = ほぼ真下)
    send_command "/look 70 0"
    sleep 0.3
    log_state "look後"

    # デバッグHUD表示
    key "F3"
    sleep 0.2
    shot "cv_debug_target"
    log_state "ターゲット確認"

    # 右クリックでコンベア配置（画面中央 = 十字線の位置）
    log "右クリックでコンベア配置..."
    rclick 640 360
    sleep 0.3
    shot "cv_placed1"
    log_state "配置1"

    # 少し向きを変えて2つ目を配置
    send_command "/look 60 10"
    sleep 0.2
    rclick 640 360
    sleep 0.3
    shot "cv_placed2"
    log_state "配置2"

    # 3つ目（L字になるか確認）
    send_command "/look 60 -30"
    sleep 0.2
    rclick 640 360
    sleep 0.3
    shot "cv_placed3"
    log_state "配置3"

    # 俯瞰で確認
    send_command "/look 80 0"
    sleep 0.3
    shot "cv_overview"

    # 状態ファイルの内容を表示
    log "--- E2E状態ファイル ---"
    cat "$E2E_STATE_FILE" 2>/dev/null || warn "状態ファイルがありません"
    log "----------------------"

    cleanup
    log "=== 完了 ==="
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
# コリジョン・プレイアビリティテスト
# =============================================================================

# 不変量違反をチェック
assert_no_violations() {
    if ! command -v jq >/dev/null 2>&1; then
        warn "jqがないため違反検証をスキップ"
        return 0
    fi

    local state=$(get_state)
    local violations=$(echo "$state" | jq '.violations | length' 2>/dev/null || echo 0)

    if [ "$violations" -eq 0 ]; then
        log "✅ 違反なし"
        return 0
    else
        err "❌ 違反検出: $violations 件"
        echo "$state" | jq -r '.violations[]' 2>/dev/null | while read v; do
            err "   - $v"
        done
        return 1
    fi
}

# プレイヤーが地面にいるか検証
assert_on_ground() {
    if ! command -v jq >/dev/null 2>&1; then
        warn "jqがないため検証をスキップ"
        return 0
    fi

    local state=$(get_state)
    local on_ground=$(echo "$state" | jq '.on_ground // false' 2>/dev/null)

    if [ "$on_ground" = "true" ]; then
        log "✅ プレイヤーは地面にいる"
        return 0
    else
        err "❌ プレイヤーが地面にいない"
        return 1
    fi
}

# スタックしていないか検証
assert_not_stuck() {
    local threshold="${1:-60}"  # デフォルト60フレーム（1秒）

    if ! command -v jq >/dev/null 2>&1; then
        warn "jqがないため検証をスキップ"
        return 0
    fi

    local state=$(get_state)
    local stuck_frames=$(echo "$state" | jq '.stuck_frames // 0' 2>/dev/null)

    if [ "$stuck_frames" -lt "$threshold" ]; then
        log "✅ スタックなし (frames=$stuck_frames < $threshold)"
        return 0
    else
        err "❌ スタック検出 (frames=$stuck_frames >= $threshold)"
        return 1
    fi
}

# プレイヤーがブロックに埋まっていないか検証
assert_not_embedded() {
    if ! command -v jq >/dev/null 2>&1; then
        warn "jqがないため検証をスキップ"
        return 0
    fi

    local state=$(get_state)
    local violations=$(echo "$state" | jq '.violations[]' 2>/dev/null | grep -c "EMBEDDED" || echo 0)

    if [ "$violations" -eq 0 ]; then
        log "✅ 埋まりなし"
        return 0
    else
        err "❌ 埋まり検出"
        return 1
    fi
}

# キーを押し続ける
hold_key() {
    local key="$1"
    local duration="$2"

    xdotool keydown "$key"
    sleep "$duration"
    xdotool keyup "$key"
    sleep 0.2
}

test_collision() {
    log "=== コリジョンテスト ==="

    start_game || return 1

    activate_window
    click 640 360  # ポーズ解除
    sleep 1

    # サバイバルモードにする
    send_command "/survival"
    sleep 0.5

    # ----- テスト1: 壁衝突テスト -----
    log "--- テスト1: 壁衝突 ---"

    # 既知の位置にテレポート
    send_command "/tp 10 10 10"
    sleep 0.5
    shot "col_start"

    # 壁を設置
    send_command "/setblock 10 10 12 stone"
    sleep 0.3

    # 壁に向かって歩く（3秒間）
    log "壁に向かって3秒歩く..."
    local before_pos=$(get_player_pos)
    send_command "/look 0 180"  # 南向き
    sleep 0.3
    hold_key "w" 3

    shot "col_wall"
    log_state "壁衝突後"

    # 埋まりがないか検証
    assert_not_embedded || warn "壁衝突テストで埋まり検出"

    # ----- テスト2: 落下テスト -----
    log "--- テスト2: 落下テスト ---"

    # 高い位置にテレポート
    send_command "/tp 20 30 20"
    sleep 0.3
    shot "col_fall_start"

    # 落下を待つ（5秒）
    log "落下中...（5秒待機）"
    sleep 5
    shot "col_fall_end"
    log_state "落下後"

    # 地面に着地しているか検証
    assert_on_ground || warn "落下テストで着地確認失敗"

    # 違反がないか検証
    assert_no_violations || warn "落下テストで違反検出"

    # ----- テスト3: 移動テスト -----
    log "--- テスト3: 移動確認テスト ---"

    send_command "/tp 30 10 30"
    sleep 0.5

    local start_pos=$(get_value '.player_pos[0]')
    log "開始位置: X=$start_pos"

    # 前に2秒歩く
    send_command "/look 0 0"  # 北向き
    sleep 0.3
    hold_key "w" 2

    local end_pos=$(get_value '.player_pos[2]')
    log "終了位置: Z=$end_pos"
    shot "col_move"

    # 移動できたか（スタックしていないか）
    assert_not_stuck 30 || warn "移動テストでスタック検出"

    # ----- テスト4: ジャンプテスト -----
    log "--- テスト4: ジャンプテスト ---"

    send_command "/tp 40 10 40"
    sleep 0.5

    local before_y=$(get_value '.player_pos[1]')
    key "space"
    sleep 0.3
    local during_y=$(get_value '.player_pos[1]')
    sleep 1
    local after_y=$(get_value '.player_pos[1]')

    log "ジャンプ: Y=$before_y → $during_y → $after_y"
    shot "col_jump"

    if [ $(echo "$during_y > $before_y" | bc -l 2>/dev/null || echo 0) -eq 1 ]; then
        log "✅ ジャンプ成功"
    else
        warn "⚠ ジャンプ確認失敗"
    fi

    # 最終確認
    log "--- 最終確認 ---"
    assert_no_violations
    assert_not_stuck
    assert_not_embedded

    cleanup
    log "=== コリジョンテスト完了 ==="
}

# =============================================================================
# ビジュアル比較 (smart_compare)
# =============================================================================

BASELINE_DIR="/home/bacon/idle_factory/screenshots/baseline"
SMART_COMPARE="/home/bacon/idle_factory/scripts/vlm_check/smart_compare.py"

# ベースライン保存
save_baseline() {
    mkdir -p "$BASELINE_DIR"
    local count=0
    for f in "$SCREENSHOTS_DIR"/*.png; do
        [ -f "$f" ] || continue
        cp "$f" "$BASELINE_DIR/$(basename "$f")"
        count=$((count + 1))
    done
    log "✅ ベースライン保存: $count 枚 → $BASELINE_DIR"
}

# スクリーンショット比較
compare_screenshots() {
    if [ ! -d "$BASELINE_DIR" ]; then
        warn "ベースラインがありません: $BASELINE_DIR"
        warn "先に実行: $0 basic && $0 save-baseline"
        return 1
    fi

    if ! command -v python3 >/dev/null 2>&1; then
        err "python3が見つかりません"
        return 1
    fi

    log "=== スクリーンショット比較 ==="
    local passed=0
    local failed=0
    local results=()

    for baseline in "$BASELINE_DIR"/*.png; do
        [ -f "$baseline" ] || continue
        local name=$(basename "$baseline")
        local current="$SCREENSHOTS_DIR/$name"

        if [ ! -f "$current" ]; then
            warn "⚠ $name: 現在の画像なし"
            continue
        fi

        # smart_compare実行
        local result=$(python3 "$SMART_COMPARE" "$baseline" "$current" --json 2>/dev/null)
        local severity=$(echo "$result" | jq -r '.severity // "error"')
        local ssim=$(echo "$result" | jq -r '.metrics.ssim // 0')

        case "$severity" in
            none)
                log "✅ $name: identical (SSIM=$ssim)"
                passed=$((passed + 1))
                ;;
            minor)
                log "⚠ $name: minor diff (SSIM=$ssim)"
                passed=$((passed + 1))
                ;;
            major|critical)
                err "❌ $name: $severity (SSIM=$ssim)"
                failed=$((failed + 1))
                # 詳細表示
                echo "$result" | jq -r '.issues[]' 2>/dev/null | while read issue; do
                    echo "     - $issue"
                done
                ;;
            *)
                warn "⚠ $name: 比較エラー"
                ;;
        esac
    done

    echo ""
    log "比較結果: ✅ $passed 枚 OK, ❌ $failed 枚 NG"

    if [ $failed -gt 0 ]; then
        return 1
    fi
    return 0
}

# =============================================================================
# メイン
# =============================================================================

case "${1:-basic}" in
    save-baseline|sb)
        save_baseline
        exit 0
        ;;
    compare|cmp)
        compare_screenshots
        exit $?
        ;;
esac

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
    collision|col)
        test_collision
        ;;
    *)
        echo "使い方: $0 [basic|conveyor|machines|collision|full|compare|save-baseline]"
        echo ""
        echo "テスト実行:"
        echo "  basic (b)         - 基本テスト（6枚）"
        echo "  conveyor (c)      - コンベアテスト（8枚）"
        echo "  machines (m)      - 機械テスト（6枚）"
        echo "  collision (col)   - コリジョンテスト（壁衝突・落下・移動・ジャンプ）"
        echo "  full (f)          - 全テスト（20枚）"
        echo ""
        echo "ビジュアル比較:"
        echo "  save-baseline (sb) - 現在のスクショをベースラインとして保存"
        echo "  compare (cmp)      - ベースラインと比較（smart_compare使用）"
        echo ""
        echo "ワークフロー:"
        echo "  1. $0 basic           # テスト実行"
        echo "  2. $0 save-baseline   # OK なら保存"
        echo "  3. (コード変更後)"
        echo "  4. $0 basic           # 再テスト"
        echo "  5. $0 compare         # 差分確認"
        exit 1
        ;;
esac

# 最終panic検出
check_panic

# 結果表示
echo ""
log "📂 $SCREENSHOTS_DIR"
echo "---"
ls -1 "$SCREENSHOTS_DIR"/*.png 2>/dev/null | while read f; do
    echo "  $(basename "$f")"
done
echo "---"
log "合計: $(ls -1 "$SCREENSHOTS_DIR"/*.png 2>/dev/null | wc -l) 枚"

# panic検出時はエラー終了
if [ $PANIC_DETECTED -eq 1 ]; then
    err "🚨 PANICが検出されました。テスト失敗。"
    exit 1
fi

log "✓ テスト成功（panicなし）"
exit 0
