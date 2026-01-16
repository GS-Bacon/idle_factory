#!/bin/bash
# Blender MCP サーバー起動スクリプト
#
# 使い方:
#   ./scripts/blender-mcp.sh        # GUI付きで起動
#   ./scripts/blender-mcp.sh --bg   # バックグラウンドで起動
#   ./scripts/blender-mcp.sh --stop # 停止

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
STARTUP_SCRIPT="$SCRIPT_DIR/start-blender-mcp.py"
PID_FILE="/tmp/blender-mcp.pid"

case "${1:-}" in
    --bg|--background)
        echo "Starting Blender MCP in background..."
        DISPLAY=:10 blender --python "$STARTUP_SCRIPT" &
        echo $! > "$PID_FILE"
        sleep 3
        echo "Blender MCP started (PID: $(cat $PID_FILE))"
        echo "Server: localhost:9876"
        ;;
    --stop)
        if [ -f "$PID_FILE" ]; then
            PID=$(cat "$PID_FILE")
            echo "Stopping Blender MCP (PID: $PID)..."
            kill "$PID" 2>/dev/null || true
            rm -f "$PID_FILE"
            echo "Stopped"
        else
            echo "No PID file found, trying to kill blender..."
            pkill -f "blender.*start-blender-mcp" || echo "No process found"
        fi
        ;;
    --status)
        if [ -f "$PID_FILE" ] && kill -0 "$(cat $PID_FILE)" 2>/dev/null; then
            echo "Running (PID: $(cat $PID_FILE))"
        else
            echo "Not running"
        fi
        ;;
    *)
        echo "Starting Blender MCP with GUI..."
        DISPLAY=:10 blender --python "$STARTUP_SCRIPT"
        ;;
esac
