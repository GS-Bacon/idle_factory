#!/bin/bash
# Auto-clean cargo build when disk usage is high
# Usage: ./auto-clean-build.sh [cargo args...]
# Example: ./auto-clean-build.sh build --release

set -e

DISK_THRESHOLD=${DISK_THRESHOLD:-85}
CURRENT_USAGE=$(df / | tail -1 | awk '{print $5}' | tr -d '%')

if [ "$CURRENT_USAGE" -ge "$DISK_THRESHOLD" ]; then
    echo "[auto-clean] Disk usage ${CURRENT_USAGE}% >= ${DISK_THRESHOLD}%"
    cargo clean
    echo "[auto-clean] Cleaned. New usage: $(df / | tail -1 | awk '{print $5}')"
fi

# Run cargo with provided arguments
cargo "$@"
