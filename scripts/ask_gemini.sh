#!/bin/bash
# Gemini連携スクリプト - 回答完了待機版
# 使い方: ./scripts/ask_gemini.sh "質問" [コンテキストファイル]

QUESTION="$1"
CONTEXT_FILE="$2"
SESSION="ai_gemini"
MAX_WAIT=120  # 最大待機秒数
POLL_INTERVAL=2

# セッション確認・作成
if ! tmux has-session -t $SESSION 2>/dev/null; then
    tmux new-session -d -s $SESSION
    tmux send-keys -t $SESSION "gemini" Enter
    sleep 5
fi

# 質問送信
if [ -n "$CONTEXT_FILE" ] && [ -f "$CONTEXT_FILE" ]; then
    tmux send-keys -t $SESSION "@$CONTEXT_FILE $QUESTION" Enter
else
    tmux send-keys -t $SESSION "$QUESTION" Enter
fi
tmux send-keys -t $SESSION Enter  # 念のためEnter追加

# 回答完了を待機（プロンプト ">" が最終行に出現するまで）
echo "Waiting for Gemini response..."
waited=0
while [ $waited -lt $MAX_WAIT ]; do
    sleep $POLL_INTERVAL
    waited=$((waited + POLL_INTERVAL))

    # 最新の出力を取得
    last_lines=$(tmux capture-pane -t $SESSION -p | tail -5)

    # プロンプト待ち状態かチェック（処理中マーカーがない）
    if echo "$last_lines" | grep -q "> .*Type your message" && \
       ! echo "$last_lines" | grep -qE "(esc to cancel|⠴|⠙|⠸|⠦|⠧|⠇|⠏)"; then
        echo "Response complete (${waited}s)"
        break
    fi

    # 編集確認ダイアログが出た場合はスキップ
    if echo "$last_lines" | grep -q "Apply this change"; then
        tmux send-keys -t $SESSION "4"  # No を選択
        sleep 1
    fi
done

if [ $waited -ge $MAX_WAIT ]; then
    echo "Timeout after ${MAX_WAIT}s"
fi

# 結果取得（回答部分のみ）
echo ""
echo "=== Gemini Response ==="
tmux capture-pane -t $SESSION -p -S -100 | grep -A 100 "^✦" | tail -60
