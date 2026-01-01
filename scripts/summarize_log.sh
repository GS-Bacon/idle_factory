#!/bin/bash
# E-1: Log summarization script
# Summarizes recent game logs using AI (gemini CLI)

set -e

LOGS_DIR="logs"
LINES=${1:-100}

# Find latest log file
LOG_FILE=$(ls -t "$LOGS_DIR"/game_*.log 2>/dev/null | head -1)

if [ -z "$LOG_FILE" ]; then
    echo "No log files found in $LOGS_DIR/"
    exit 1
fi

echo "=== Log Summary ==="
echo "File: $LOG_FILE"
echo "Analyzing last $LINES lines..."
echo ""

# Check if gemini CLI is available
if command -v gemini >/dev/null 2>&1; then
    tail -"$LINES" "$LOG_FILE" | gemini -p "このゲームログを要約してください。
以下の点を強調して:
1. エラーや警告があれば最初に報告
2. 機械の動作状況（Miner, Conveyor, Furnace等）
3. プレイヤーの操作（ブロック設置/破壊、移動）
4. パフォーマンス問題（FPS低下、チャンク生成遅延等）
5. 異常なパターン（連続エラー、同じ警告の繰り返し等）

簡潔に箇条書きで報告してください。"
else
    echo "gemini CLI not found. Manual summary:"
    echo ""
    echo "=== Errors/Warnings ==="
    grep -E "(ERROR|WARN|panic)" "$LOG_FILE" | tail -20 || echo "None found"
    echo ""
    echo "=== Recent Events ==="
    tail -"$LINES" "$LOG_FILE" | grep -E "(BLOCK|MACHINE|spawn|place|break)" | tail -20 || echo "No events"
    echo ""
    echo "=== Statistics ==="
    echo "Total lines: $(wc -l < "$LOG_FILE")"
    echo "Errors: $(grep -c "ERROR" "$LOG_FILE" || echo 0)"
    echo "Warnings: $(grep -c "WARN" "$LOG_FILE" || echo 0)"
fi
