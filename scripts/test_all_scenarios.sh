#!/bin/bash
# Run all scenario tests

PASSED=0
FAILED=0

for f in /home/bacon/idle_factory/tests/scenarios/*.toml; do
    name=$(basename "$f" .toml)
    if timeout 30 node /home/bacon/idle_factory/scripts/run-scenario.js "$f" > /tmp/out.txt 2>&1; then
        echo "PASS: $name"
        PASSED=$((PASSED+1))
    else
        echo "FAIL: $name"
        grep -E "(Error:|Assert:|Expected:|Actual:)" /tmp/out.txt | head -3
        FAILED=$((FAILED+1))
    fi
done

echo ""
echo "=== Results ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"

if [ $FAILED -gt 0 ]; then
    exit 1
fi
