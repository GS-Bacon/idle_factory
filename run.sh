#!/bin/bash
# Idle Factory launcher - auto-detects available display and captures logs

set -e

# Auto-clean if disk usage exceeds threshold
DISK_THRESHOLD=85
CURRENT_USAGE=$(df / | tail -1 | awk '{print $5}' | tr -d '%')

if [ "$CURRENT_USAGE" -ge "$DISK_THRESHOLD" ]; then
    echo "Disk usage ${CURRENT_USAGE}% >= ${DISK_THRESHOLD}%. Running cargo clean..."
    cargo clean
    echo "Cleaned. New usage: $(df / | tail -1 | awk '{print $5}')"
fi

# Create logs directory if it doesn't exist
mkdir -p /home/bacon/idle_factory/logs

# Try to find a working display in this order:
# 1. RDP Xorg session
# 2. Xvfb
# 3. Default :0

DISPLAY_FOUND=""

# Check for xrdp Xorg session
RDP_DISPLAY=$(ps aux | grep "Xorg.*xrdp" | grep -v grep | awk '{for(i=1;i<=NF;i++) if($i ~ /^:[0-9]+$/) print $i}' | sort -t: -k2 -n | tail -1)
if [ -n "$RDP_DISPLAY" ]; then
    DISPLAY_FOUND="$RDP_DISPLAY"
    echo "Found RDP session on $DISPLAY_FOUND"
fi

# Check for Xvfb if no RDP found
if [ -z "$DISPLAY_FOUND" ]; then
    XVFB_DISPLAY=$(ps aux | grep "Xvfb" | grep -v grep | awk '{for(i=1;i<=NF;i++) if($i ~ /^:[0-9]+$/) print $i}' | head -1)
    if [ -n "$XVFB_DISPLAY" ]; then
        DISPLAY_FOUND="$XVFB_DISPLAY"
        echo "Found Xvfb on $DISPLAY_FOUND"
    fi
fi

# Fallback to :0
if [ -z "$DISPLAY_FOUND" ]; then
    DISPLAY_FOUND=":0"
    echo "No virtual display found. Using :0"
fi

echo "Starting Idle Factory on display $DISPLAY_FOUND"

cd /home/bacon/idle_factory
source ~/.cargo/env

# Generate timestamped log filename
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="logs/game_${TIMESTAMP}.log"

# Also maintain a symlink to the latest log
LATEST_LOG="logs/game_latest.log"

echo "Logging to $LOG_FILE"

# Run the game and capture all output to log file
# Use unbuffer or stdbuf to prevent buffering issues
if command -v stdbuf &> /dev/null; then
    DISPLAY=$DISPLAY_FOUND stdbuf -oL -eL cargo run --release "$@" 2>&1 | tee "$LOG_FILE"
else
    DISPLAY=$DISPLAY_FOUND cargo run --release "$@" 2>&1 | tee "$LOG_FILE"
fi

# Update latest log symlink
ln -sf "game_${TIMESTAMP}.log" "$LATEST_LOG"

echo ""
echo "Log saved to $LOG_FILE"
