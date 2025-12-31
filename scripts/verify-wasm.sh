#!/bin/bash
# WASM game verification script
# Usage: ./verify-wasm.sh

set -e

SCREENSHOT_DIR="/home/bacon/idle_factory/screenshots/verify"
mkdir -p "$SCREENSHOT_DIR"

# Check if WASM server is running
echo "=== WASM Server Status ==="
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8080 | grep -q "200"; then
    echo "Server is running on http://localhost:8080"
else
    echo "ERROR: WASM server not running. Run ./deploy-wasm.sh first"
    exit 1
fi

# Check WASM file exists and is recent
WASM_FILE="/home/bacon/idle_factory/web/idle_factory_bg.wasm"
if [ -f "$WASM_FILE" ]; then
    echo "WASM file: $(ls -lh $WASM_FILE | awk '{print $5, $6, $7, $8}')"
else
    echo "ERROR: WASM file not found"
    exit 1
fi

# Check JavaScript bindings
JS_FILE="/home/bacon/idle_factory/web/idle_factory.js"
if [ -f "$JS_FILE" ]; then
    echo "JS bindings: $(ls -lh $JS_FILE | awk '{print $5, $6, $7, $8}')"
else
    echo "ERROR: JS bindings not found"
    exit 1
fi

# Find display for browser screenshot
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

if [ -n "$DISPLAY_FOUND" ]; then
    export DISPLAY=$DISPLAY_FOUND
    echo "Using display: $DISPLAY"

    # Take browser screenshot using Playwright if available
    if command -v npx &> /dev/null && [ -d "node_modules/playwright" ]; then
        echo "Taking browser screenshot with Playwright..."
        npx playwright screenshot --browser=chromium http://localhost:8080 "$SCREENSHOT_DIR/wasm_playwright.png" 2>/dev/null || echo "Playwright screenshot failed"
    else
        # Fallback: use Firefox in headless mode
        echo "Playwright not installed. Trying Firefox..."
        timeout 15 firefox --headless --screenshot="$SCREENSHOT_DIR/wasm_firefox.png" http://localhost:8080 2>/dev/null || echo "Firefox screenshot failed (may need manual browser check)"
    fi
fi

# Fetch page and check for errors
echo ""
echo "=== Page Content Check ==="
RESPONSE=$(curl -s http://localhost:8080)
if echo "$RESPONSE" | grep -q "idle_factory"; then
    echo "Page loads correctly (contains 'idle_factory')"
else
    echo "WARNING: Page may not be loading correctly"
fi

# Check browser console log endpoint if available
echo ""
echo "=== Access URLs ==="
echo "Local:     http://localhost:8080"
echo "Local IP:  http://10.13.1.1:8080"
echo "Tailscale: http://100.84.170.32:8080"

echo ""
echo "=== Manual Verification Required ==="
echo "1. Open the URL in a browser"
echo "2. Check browser console (F12) for errors"
echo "3. Run exportGameLogs() in console to get detailed logs"
echo "4. Take a screenshot if issues are found"

ls -la "$SCREENSHOT_DIR"/wasm_*.png 2>/dev/null && echo "Screenshots saved above" || echo "No automated screenshots (use browser manually)"
