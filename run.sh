#!/bin/bash
# Idle Factory launcher - auto-detects RDP display

# Find the most recent xrdp Xorg session (highest display number)
RDP_DISPLAY=$(ps aux | grep "Xorg.*xrdp" | grep -v grep | awk '{for(i=1;i<=NF;i++) if($i ~ /^:[0-9]+$/) print $i}' | sort -t: -k2 -n | tail -1)

if [ -z "$RDP_DISPLAY" ]; then
    echo "No RDP session found. Using :0"
    RDP_DISPLAY=":0"
fi

echo "Starting Idle Factory on display $RDP_DISPLAY"

cd /home/bacon/github/idle_factory
DISPLAY=$RDP_DISPLAY cargo run "$@"
