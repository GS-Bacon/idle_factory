#!/bin/bash
# Gemini出力監視スクリプト
# 使い方: ./scripts/watch_gemini.sh [タイムアウト秒]
# 回答完了時に自動で出力して終了

SESSION="ai_gemini"
MAX_WAIT=${1:-120}
POLL_INTERVAL=2

echo "Watching Gemini session (timeout: ${MAX_WAIT}s)..."
echo "Press Ctrl+C to stop"
echo ""

waited=0
last_state=""
while [ $waited -lt $MAX_WAIT ]; do
    sleep $POLL_INTERVAL
    waited=$((waited + POLL_INTERVAL))

    # 最新の出力を取得
    current=$(tmux capture-pane -t $SESSION -p 2>/dev/null | tail -10)

    # 状態変化を検出
    if [ "$current" != "$last_state" ]; then
        # 処理中マーカーをチェック
        if echo "$current" | grep -qE "(esc to cancel|⠴|⠙|⠸|⠦|⠧|⠇|⠏)"; then
            status=$(echo "$current" | grep -oE "\(esc to cancel.*\)" | head -1)
            echo -ne "\r[Processing] $status    "
        elif echo "$current" | grep -q "Apply this change"; then
            echo -ne "\r[Waiting] Edit confirmation dialog...    "
        elif echo "$current" | grep -q "Type your message"; then
            echo -e "\r[Complete] Response finished!           "
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
