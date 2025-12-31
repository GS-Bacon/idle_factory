#!/bin/bash
# Show game logs - helper script for debugging
#
# Usage:
#   ./show-logs.sh              # Show latest log (last 50 lines)
#   ./show-logs.sh 100          # Show last 100 lines
#   ./show-logs.sh all          # Show entire log
#   ./show-logs.sh errors       # Show only errors
#   ./show-logs.sh events       # Show only game events (BLOCK, MACHINE, QUEST)
#   ./show-logs.sh list         # List all log files

LOGS_DIR="/home/bacon/idle_factory/logs"
LATEST_NATIVE="$LOGS_DIR/game_latest.log"
LATEST_WASM="$LOGS_DIR/wasm_latest.log"

show_help() {
    echo "Usage: ./show-logs.sh [command]"
    echo ""
    echo "Commands:"
    echo "  (number)   Show last N lines (default: 50)"
    echo "  all        Show entire log"
    echo "  errors     Show only errors and warnings"
    echo "  events     Show only game events (BLOCK, MACHINE, QUEST)"
    echo "  list       List all log files"
    echo "  wasm       Show latest WASM log"
    echo "  help       Show this help"
    echo ""
    echo "Examples:"
    echo "  ./show-logs.sh 100     # Show last 100 lines"
    echo "  ./show-logs.sh events  # Show game events"
}

list_logs() {
    echo "=== Log Files ==="
    echo ""
    echo "Native logs:"
    ls -lah "$LOGS_DIR"/game_*.log 2>/dev/null | tail -10 || echo "  (none)"
    echo ""
    echo "WASM logs:"
    ls -lah "$LOGS_DIR"/wasm_*.log 2>/dev/null | tail -10 || echo "  (none)"
}

get_latest_log() {
    if [ -f "$LATEST_NATIVE" ]; then
        echo "$LATEST_NATIVE"
    elif [ -f "$LATEST_WASM" ]; then
        echo "$LATEST_WASM"
    else
        # Find most recent log file
        find "$LOGS_DIR" -name "*.log" -type f 2>/dev/null | head -1
    fi
}

case "$1" in
    help|--help|-h)
        show_help
        ;;
    list)
        list_logs
        ;;
    wasm)
        if [ -f "$LATEST_WASM" ]; then
            if [ -n "$2" ] && [ "$2" != "all" ]; then
                tail -n "$2" "$LATEST_WASM"
            elif [ "$2" = "all" ]; then
                cat "$LATEST_WASM"
            else
                tail -n 50 "$LATEST_WASM"
            fi
        else
            echo "No WASM log found. Run: node capture-wasm-logs.js"
        fi
        ;;
    errors)
        LOG=$(get_latest_log)
        if [ -n "$LOG" ] && [ -f "$LOG" ]; then
            echo "=== Errors from $LOG ==="
            grep -i -E "(error|warn|panic|fail)" "$LOG" | tail -100
        else
            echo "No log files found"
        fi
        ;;
    events)
        LOG=$(get_latest_log)
        if [ -n "$LOG" ] && [ -f "$LOG" ]; then
            echo "=== Game Events from $LOG ==="
            grep -E "(BLOCK|MACHINE|QUEST|category)" "$LOG" | tail -100
        else
            echo "No log files found"
        fi
        ;;
    all)
        LOG=$(get_latest_log)
        if [ -n "$LOG" ] && [ -f "$LOG" ]; then
            cat "$LOG"
        else
            echo "No log files found"
        fi
        ;;
    *)
        LINES="${1:-50}"
        LOG=$(get_latest_log)
        if [ -n "$LOG" ] && [ -f "$LOG" ]; then
            echo "=== Last $LINES lines from $LOG ==="
            tail -n "$LINES" "$LOG"
        else
            echo "No log files found. Run the game first: ./run.sh"
        fi
        ;;
esac
