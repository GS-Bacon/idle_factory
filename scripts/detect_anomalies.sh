#!/bin/bash
# E-2: Anomaly detection script
# Detects potential issues in game logs

set -e

LOGS_DIR="logs"

# Find latest log file
LOG_FILE=$(ls -t "$LOGS_DIR"/game_*.log 2>/dev/null | head -1)

if [ -z "$LOG_FILE" ]; then
    echo "No log files found in $LOGS_DIR/"
    exit 1
fi

echo "=== Anomaly Detection Report ==="
echo "File: $LOG_FILE"
echo "Time: $(date)"
echo ""

ANOMALIES=0

# 1. Check for panics/crashes
echo "=== Critical Errors ==="
PANICS=$(grep -c "panic" "$LOG_FILE" 2>/dev/null || echo 0)
if [ "$PANICS" -gt 0 ]; then
    echo "⚠️  PANIC detected: $PANICS occurrences"
    grep "panic" "$LOG_FILE" | tail -5
    ANOMALIES=$((ANOMALIES + 1))
else
    echo "✓ No panics"
fi

# 2. Check for errors
ERRORS=$(grep -c "ERROR" "$LOG_FILE" 2>/dev/null || echo 0)
if [ "$ERRORS" -gt 0 ]; then
    echo "⚠️  ERRORS: $ERRORS"
    grep "ERROR" "$LOG_FILE" | tail -10
    ANOMALIES=$((ANOMALIES + 1))
fi
echo ""

# 3. Check for repeated warnings (same warning > 5 times)
echo "=== Repeated Warnings ==="
grep "WARN" "$LOG_FILE" 2>/dev/null | sort | uniq -c | sort -rn | head -5 | while read count msg; do
    if [ "$count" -gt 5 ]; then
        echo "⚠️  Repeated $count times: ${msg:0:80}..."
        ANOMALIES=$((ANOMALIES + 1))
    fi
done || echo "✓ No repeated warnings"
echo ""

# 4. Check for conveyor jams (same position appearing many times)
echo "=== Conveyor Analysis ==="
if grep -q "item_transfer" "$LOG_FILE" 2>/dev/null; then
    echo "Item transfers detected. Checking for jams..."
    grep "item_transfer" "$LOG_FILE" | grep -oP 'to=\[[^\]]+\]' | sort | uniq -c | sort -rn | head -5 | while read count pos; do
        if [ "$count" -gt 20 ]; then
            echo "⚠️  Potential jam at $pos ($count items)"
            ANOMALIES=$((ANOMALIES + 1))
        fi
    done
else
    echo "✓ No item transfers logged"
fi
echo ""

# 5. Check for machine issues
echo "=== Machine Status ==="
MINER_COUNT=$(grep -c "Miner" "$LOG_FILE" 2>/dev/null || echo 0)
CONVEYOR_COUNT=$(grep -c "Conveyor" "$LOG_FILE" 2>/dev/null || echo 0)
FURNACE_COUNT=$(grep -c "Furnace" "$LOG_FILE" 2>/dev/null || echo 0)
echo "Miner mentions: $MINER_COUNT"
echo "Conveyor mentions: $CONVEYOR_COUNT"
echo "Furnace mentions: $FURNACE_COUNT"

# Check for idle machines (machines that haven't produced output)
echo ""
echo "=== Machine Activity ==="

# Check for machine spawn without corresponding output
MINERS_SPAWNED=$(grep -c "Spawned miner\|spawn.*miner" "$LOG_FILE" 2>/dev/null || echo 0)
ITEMS_MINED=$(grep -c "mined\|ore_output\|item_produce" "$LOG_FILE" 2>/dev/null || echo 0)
if [ "$MINERS_SPAWNED" -gt 0 ] && [ "$ITEMS_MINED" -eq 0 ]; then
    echo "⚠️  Miners spawned ($MINERS_SPAWNED) but no mining output detected"
    ANOMALIES=$((ANOMALIES + 1))
else
    echo "✓ Mining activity normal"
fi

# Check for furnace issues (fuel but no output)
FURNACE_FUEL=$(grep -c "fuel\|coal.*furnace\|furnace.*coal" "$LOG_FILE" 2>/dev/null || echo 0)
FURNACE_OUTPUT=$(grep -c "smelt\|ingot.*produce\|furnace.*output" "$LOG_FILE" 2>/dev/null || echo 0)
if [ "$FURNACE_FUEL" -gt 5 ] && [ "$FURNACE_OUTPUT" -eq 0 ]; then
    echo "⚠️  Furnace has fuel but no smelting output"
    ANOMALIES=$((ANOMALIES + 1))
else
    echo "✓ Furnace activity normal"
fi
echo ""

# 6. Check for chunk issues
echo "=== Chunk Generation ==="
CHUNK_ERRORS=$(grep -c "chunk.*error\|chunk.*fail" "$LOG_FILE" 2>/dev/null || echo 0)
if [ "$CHUNK_ERRORS" -gt 0 ]; then
    echo "⚠️  Chunk errors: $CHUNK_ERRORS"
    ANOMALIES=$((ANOMALIES + 1))
else
    echo "✓ No chunk errors"
fi
echo ""

# 7. Performance check (FPS drops)
echo "=== Performance ==="
if grep -q "FPS:" "$LOG_FILE" 2>/dev/null; then
    LOW_FPS=$(grep -oP 'FPS: \K[0-9]+' "$LOG_FILE" | awk '$1 < 30 {count++} END {print count+0}')
    if [ "$LOW_FPS" -gt 0 ]; then
        echo "⚠️  Low FPS (<30) detected $LOW_FPS times"
        ANOMALIES=$((ANOMALIES + 1))
    else
        echo "✓ FPS stable"
    fi
else
    echo "No FPS data in logs"
fi
echo ""

# Summary
echo "=== Summary ==="
if [ "$ANOMALIES" -gt 0 ]; then
    echo "⚠️  Found $ANOMALIES potential issues"
    exit 1
else
    echo "✓ No anomalies detected"
    exit 0
fi
