#!/bin/bash
set -e
cd "$(dirname "$0")/.."

echo "=== WebSocket Mod API E2E Test ==="

# 1. ゲーム起動
echo "Starting game..."
DISPLAY=:10 cargo run --bin idle_factory &
GAME_PID=$!
trap "kill $GAME_PID 2>/dev/null || true" EXIT

# 2. ポート待機
echo "Waiting for port 9877..."
for i in {1..30}; do
    if ss -tlnp 2>/dev/null | grep -q ":9877"; then
        echo "Port 9877 is ready"
        break
    fi
    sleep 1
done

# 確認: ポートが開いているか
if ! ss -tlnp 2>/dev/null | grep -q ":9877"; then
    echo "ERROR: Port 9877 not available after 30 seconds"
    exit 1
fi

# 3. テスト実行
echo "Running tests..."
python3 scripts/test_websocket_api.py
RESULT=$?

# 4. ゲーム終了
kill $GAME_PID 2>/dev/null || true

exit $RESULT
