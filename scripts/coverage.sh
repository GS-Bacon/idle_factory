#!/bin/bash
# Coverage measurement script using cargo-tarpaulin

set -e

echo "=== Code Coverage Measurement ==="

# Check if cargo-tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Create coverage directory
mkdir -p coverage

# Run coverage with HTML report
echo "Running coverage analysis..."
cargo tarpaulin \
    --out Html \
    --output-dir coverage/ \
    --ignore-tests \
    --exclude-files "tests/*" \
    --skip-clean \
    --timeout 300 \
    2>&1 | tee coverage/tarpaulin.log

echo ""
echo "=== Coverage Report Generated ==="
echo "HTML report: coverage/tarpaulin-report.html"
echo ""

# Extract summary from log
if grep -q "^[0-9]*\.[0-9]*% coverage" coverage/tarpaulin.log; then
    COVERAGE=$(grep "^[0-9]*\.[0-9]*% coverage" coverage/tarpaulin.log | tail -1)
    echo "Summary: $COVERAGE"
fi

echo ""
echo "To view the report, open: file://$(pwd)/coverage/tarpaulin-report.html"
