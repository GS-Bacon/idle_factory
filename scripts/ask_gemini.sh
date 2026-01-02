#!/bin/bash
# Gemini連携スクリプト - 非インタラクティブモード版
# 使い方: ./scripts/ask_gemini.sh "質問" [コンテキストファイル...]
#
# 例:
#   ./scripts/ask_gemini.sh "このコードをレビューして" src/main.rs
#   ./scripts/ask_gemini.sh "アーキテクチャを評価して" src/*.rs
#   echo "質問" | ./scripts/ask_gemini.sh

set -e

TIMEOUT="${GEMINI_TIMEOUT:-120}"  # デフォルト2分

# 引数処理
QUESTION=""
FILES=()

for arg in "$@"; do
    if [[ -f "$arg" ]]; then
        FILES+=("$arg")
    else
        if [[ -z "$QUESTION" ]]; then
            QUESTION="$arg"
        else
            QUESTION="$QUESTION $arg"
        fi
    fi
done

# stdinからの入力があれば追加
if [ ! -t 0 ]; then
    STDIN_INPUT=$(cat)
    if [[ -n "$STDIN_INPUT" ]]; then
        QUESTION="${QUESTION:+$QUESTION\n\n}$STDIN_INPUT"
    fi
fi

# 質問がなければエラー
if [[ -z "$QUESTION" && ${#FILES[@]} -eq 0 ]]; then
    echo "Usage: $0 \"質問\" [ファイル...]" >&2
    echo "  例: $0 \"このコードをレビューして\" src/main.rs" >&2
    exit 1
fi

# ファイル指定がある場合は@プレフィックスを付ける
FILE_ARGS=""
for file in "${FILES[@]}"; do
    FILE_ARGS="$FILE_ARGS @$file"
done

# 実行
echo "🤖 Gemini に質問中... (タイムアウト: ${TIMEOUT}秒)" >&2

if timeout "$TIMEOUT" gemini $FILE_ARGS "$QUESTION" --approval-mode yolo 2>&1; then
    exit 0
else
    EXIT_CODE=$?
    if [[ $EXIT_CODE -eq 124 ]]; then
        echo "❌ タイムアウト (${TIMEOUT}秒)" >&2
    else
        echo "❌ エラー (exit code: $EXIT_CODE)" >&2
    fi
    exit $EXIT_CODE
fi
