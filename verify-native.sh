#!/bin/bash
# Native game verification script - takes screenshots for AI review
# Usage: ./verify-native.sh [--skip-build]

set -e

SKIP_BUILD=false
if [ "$1" = "--skip-build" ]; then
    SKIP_BUILD=true
fi

SCREENSHOT_DIR="/home/bacon/idle_factory/screenshots/verify"
mkdir -p "$SCREENSHOT_DIR"

# Clean up old verification screenshots
rm -f "$SCREENSHOT_DIR"/native_*.png 2>/dev/null || true

# Find display
DISPLAY_FOUND=""
RDP_DISPLAY=$(ps aux | grep "Xorg.*xrdp" | grep -v grep | awk '{for(i=1;i<=NF;i++) if($i ~ /^:[0-9]+$/) print $i}' | sort -t: -k2 -n | tail -1)
if [ -n "$RDP_DISPLAY" ]; then
    DISPLAY_FOUND="$RDP_DISPLAY"
fi
if [ -z "$DISPLAY_FOUND" ]; then
    XVFB_DISPLAY=$(ps aux | grep "Xvfb" | grep -v grep | awk '{for(i=1;i<=NF;i++) if($i ~ /^:[0-9]+$/) print $i}' | head -1)
    if [ -n "$XVFB_DISPLAY" ]; then
        DISPLAY_FOUND="$XVFB_DISPLAY"
    fi
fi
if [ -z "$DISPLAY_FOUND" ]; then
    echo "ERROR: No display found (RDP or Xvfb)"
    exit 1
fi

export DISPLAY=$DISPLAY_FOUND
echo "Using display: $DISPLAY"

# Build if not skipped
cd /home/bacon/idle_factory
source ~/.cargo/env
if [ "$SKIP_BUILD" = false ]; then
    echo "Building..."
    cargo build --release 2>&1 | tail -5
else
    echo "Skipping build (--skip-build)"
fi

# Start game in background
echo "Starting game..."
if [ "$SKIP_BUILD" = false ]; then
    RUST_LOG=warn cargo run --release &
else
    # Run binary directly
    RUST_LOG=warn ./target/release/idle_factory &
fi
GAME_PID=$!

# Wait for window to appear
echo "Waiting for game window..."
sleep 3

# Take screenshots at intervals
for i in 1 2 3; do
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    FILENAME="$SCREENSHOT_DIR/native_${i}_${TIMESTAMP}.png"
    scrot -d 0 "$FILENAME" 2>/dev/null || true
    echo "Screenshot $i: $FILENAME"
    sleep 2
done

# Capture game logs
echo "=== Game Output (last 50 lines) ===" > "$SCREENSHOT_DIR/native_log.txt"
timeout 3 cat /proc/$GAME_PID/fd/1 2>/dev/null >> "$SCREENSHOT_DIR/native_log.txt" || true

# Kill game
kill $GAME_PID 2>/dev/null || true
wait $GAME_PID 2>/dev/null || true

echo ""
echo "=== Verification Complete ==="
echo "Screenshots saved to: $SCREENSHOT_DIR"
ls -la "$SCREENSHOT_DIR"/*.png 2>/dev/null || echo "No screenshots captured"
