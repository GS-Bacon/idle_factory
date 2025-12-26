#!/bin/bash
# Idle Factory launcher - auto-detects available display

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
DISPLAY=$DISPLAY_FOUND cargo run "$@"
