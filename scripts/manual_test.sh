#!/bin/bash
export DISPLAY=:10

WINDOW=$(xdotool search --name "Idle Factory" | head -1)
echo "Window ID: $WINDOW"

if [ -z "$WINDOW" ]; then
    echo "ERROR: No game window found"
    exit 1
fi

# Activate window
xdotool windowactivate --sync "$WINDOW"
sleep 0.5

# Space to unpause
xdotool key space
sleep 0.5

# Click for pointer lock
xdotool click 1
sleep 0.5

# Screenshot 1
scrot /home/bacon/idle_factory/screenshots/verify/manual_1.png
echo "Screenshot 1 taken"

# T key for command mode
xdotool key t
sleep 0.3

# Screenshot 2 - should show command input
scrot /home/bacon/idle_factory/screenshots/verify/manual_2.png
echo "Screenshot 2 taken (command mode)"

# Type a simple command /creative
xdotool key slash
sleep 0.05
xdotool key c
sleep 0.05
xdotool key r
sleep 0.05
xdotool key e
sleep 0.05
xdotool key a
sleep 0.05
xdotool key t
sleep 0.05
xdotool key i
sleep 0.05
xdotool key v
sleep 0.05
xdotool key e
sleep 0.1

# Screenshot 3 - should show typed command
scrot /home/bacon/idle_factory/screenshots/verify/manual_3.png
echo "Screenshot 3 taken (command typed)"

# Press Enter
xdotool key Return
sleep 0.5

# Click to dismiss
xdotool click 1
sleep 0.3

# Screenshot 4 - result
scrot /home/bacon/idle_factory/screenshots/verify/manual_4.png
echo "Screenshot 4 taken (after command)"

echo "=== Manual test complete ==="
