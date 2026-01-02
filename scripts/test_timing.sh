#!/bin/bash
# テスト実行時間の監視
# Usage: ./scripts/test_timing.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

echo "=== テスト実行時間 ==="
echo ""

# 目標時間
declare -A TARGETS
TARGETS["lib"]=5
TARGETS["e2e"]=10
TARGETS["proptest"]=5
TARGETS["clippy"]=10

measure_time() {
    local name="$1"
    local cmd="$2"
    local target="${TARGETS[$name]:-10}"

    local start=$(date +%s.%N)
    eval "$cmd" >/dev/null 2>&1 || true
    local end=$(date +%s.%N)
    local elapsed=$(echo "$end - $start" | bc)
    local elapsed_int=${elapsed%.*}

    local status="✓"
    if [ "${elapsed_int:-0}" -gt "$target" ]; then
        status="⚠ (目標: ${target}s)"
    fi

    printf "%-15s %6.2f sec %s\n" "$name:" "$elapsed" "$status"
}

echo "カテゴリ         時間       状態"
echo "----------------------------------------"

measure_time "lib" "cargo test --lib --quiet"
measure_time "e2e" "cargo test --test e2e_test --quiet"
measure_time "proptest" "cargo test --test proptest_invariants --quiet"
measure_time "clippy" "cargo clippy --quiet"

echo ""
echo "=== 遅いテストの特定 ==="

# 個別テストの時間計測（上位5件）
echo ""
echo "最も遅いテスト（概算）:"

# cargo testの出力から時間を取得（nightlyが必要だが、ここでは簡易版）
cargo test --lib 2>&1 | grep -E "test .* \.\.\. ok" | while read line; do
    echo "  $line"
done | head -5

echo ""
echo "詳細な時間計測には cargo +nightly test -- -Z unstable-options --report-time を使用"
