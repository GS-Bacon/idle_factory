#!/bin/bash
# Camera behavior test script
# Uses xdotool to simulate mouse/keyboard input and verify camera movement

set -e

DISPLAY=${DISPLAY:-:12}
export DISPLAY

echo "=== Camera Test Script ==="
echo "Display: $DISPLAY"

# Kill any existing instance (but not this script)
pkill -f "target.*idle_factory" 2>/dev/null || true
sleep 1

# Start game in background
echo "Starting game..."
cd /home/bacon/github/idle_factory
./run.sh &
GAME_PID=$!
echo "Waiting for game to start (PID: $GAME_PID)..."
sleep 8

# Find game window
echo "Finding game window..."
WINDOW_ID=$(xdotool search --name "Idle Factory" | head -1)
if [ -z "$WINDOW_ID" ]; then
    echo "ERROR: Could not find game window"
    kill $GAME_PID 2>/dev/null || true
    exit 1
fi
echo "Window ID: $WINDOW_ID"

# Activate window and move mouse to center
xdotool windowactivate --sync $WINDOW_ID
sleep 0.5

# Get window geometry
eval $(xdotool getwindowgeometry --shell $WINDOW_ID)
CENTER_X=$((X + WIDTH/2))
CENTER_Y=$((Y + HEIGHT/2))
echo "Window center: $CENTER_X, $CENTER_Y"

# Take initial screenshot
echo "Taking initial screenshot..."
scrot /tmp/camera_test_1_initial.png
sleep 0.5

# Click to lock cursor
echo "Clicking to lock cursor..."
xdotool mousemove $CENTER_X $CENTER_Y
xdotool click 1
sleep 0.5

# Take screenshot after lock
scrot /tmp/camera_test_2_locked.png

# Move mouse to simulate camera rotation (relative movement)
echo "Testing mouse look (moving right)..."
xdotool mousemove_relative 100 0
sleep 0.3
scrot /tmp/camera_test_3_look_right.png

echo "Testing mouse look (moving down)..."
xdotool mousemove_relative 0 50
sleep 0.3
scrot /tmp/camera_test_4_look_down.png

# Test WASD movement
echo "Testing WASD movement..."
xdotool key --delay 100 w w w w w
sleep 0.5
scrot /tmp/camera_test_5_moved_forward.png

# Test arrow keys (should work without cursor lock)
echo "Testing arrow keys..."
xdotool key Escape  # Unlock cursor first
sleep 0.3
xdotool key --delay 50 Left Left Left Left Left
sleep 0.3
scrot /tmp/camera_test_6_arrow_left.png

# Clean up
echo "Stopping game..."
kill $GAME_PID 2>/dev/null || true
wait $GAME_PID 2>/dev/null || true

echo ""
echo "=== Test Complete ==="
echo "Screenshots saved to /tmp/camera_test_*.png"
echo "Review them to verify camera behavior"
ls -la /tmp/camera_test_*.png
