#!/bin/bash
# Gemini出力監視スクリプト - 自動許可対応版
# 使い方: ./scripts/watch_gemini.sh [タイムアウト秒] [--auto-deny]
# 回答完了時に自動で出力して終了
# --auto-deny: 編集・コマンド実行の許可ダイアログを自動で拒否

SESSION="ai_gemini"
MAX_WAIT=${1:-120}
POLL_INTERVAL=2
AUTO_DENY=false

# オプション解析
for arg in "$@"; do
    case $arg in
        --auto-deny)
            AUTO_DENY=true
            ;;
    esac
done

echo "Watching Gemini session (timeout: ${MAX_WAIT}s)..."
[ "$AUTO_DENY" = true ] && echo "Auto-deny mode: ON (will reject edit/command dialogs)"
echo "Press Ctrl+C to stop"
echo ""

waited=0
last_state=""
while [ $waited -lt $MAX_WAIT ]; do
    sleep $POLL_INTERVAL
    waited=$((waited + POLL_INTERVAL))

    # 最新の出力を取得
    current=$(tmux capture-pane -t $SESSION -p 2>/dev/null | tail -15)

    # 許可ダイアログの自動処理
    if [ "$AUTO_DENY" = true ]; then
        # 編集確認ダイアログ
        if echo "$current" | grep -q "Apply this change"; then
            echo -e "\r[Auto] Denying edit request...           "
            tmux send-keys -t $SESSION "4"  # No
            sleep 1
            continue
        fi
        # コマンド実行確認ダイアログ
        if echo "$current" | grep -q "Allow execution of"; then
            echo -e "\r[Auto] Denying command execution...      "
            tmux send-keys -t $SESSION "3"  # No
            sleep 1
            continue
        fi
        # ファイル読み取り確認
        if echo "$current" | grep -q "Allow reading"; then
            echo -e "\r[Auto] Allowing file read...             "
            tmux send-keys -t $SESSION "1"  # Allow once
            sleep 1
            continue
        fi
    fi

    # 状態変化を検出
    if [ "$current" != "$last_state" ]; then
        # 処理中マーカーをチェック
        if echo "$current" | grep -qE "(esc to cancel|⠴|⠙|⠸|⠦|⠧|⠇|⠏)"; then
            status=$(echo "$current" | grep -oE "\(esc to cancel.*\)" | head -1)
            echo -ne "\r[Processing] $status    "
        elif echo "$current" | grep -q "Apply this change"; then
            echo -ne "\r[Waiting] Edit confirmation dialog...    "
        elif echo "$current" | grep -q "Allow execution of"; then
            echo -ne "\r[Waiting] Command execution dialog...    "
        elif echo "$current" | grep -q "Type your message"; then
            echo -e "\r[Complete] Response finished! (${waited}s)   "
            echo ""
            echo "=== Gemini Response ==="
            tmux capture-pane -t $SESSION -p -S -100 | grep -A 100 "^✦" | tail -60
            exit 0
        fi
        last_state="$current"
    fi
done

echo ""
echo "[Timeout] No response after ${MAX_WAIT}s"
exit 1
