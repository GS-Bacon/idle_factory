#!/bin/bash
# カバレッジ分析スクリプト
# Usage: ./scripts/analyze_coverage.sh [--run]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

COVERAGE_DIR="$PROJECT_DIR/coverage"
RUN_COVERAGE=false

[ "$1" = "--run" ] && RUN_COVERAGE=true

echo "=== カバレッジ分析 ==="
echo ""

# カバレッジ計測（オプション）
if [ "$RUN_COVERAGE" = true ]; then
    echo "カバレッジ計測中..."
    if command -v cargo-tarpaulin >/dev/null 2>&1; then
        mkdir -p "$COVERAGE_DIR"
        cargo tarpaulin --out Json --out Html --output-dir "$COVERAGE_DIR" --skip-clean 2>/dev/null || {
            echo "警告: tarpaulinが失敗しました"
        }
    else
        echo "警告: cargo-tarpaulin がインストールされていません"
        echo "インストール: cargo install cargo-tarpaulin"
    fi
    echo ""
fi

# レポート分析
if [ -f "$COVERAGE_DIR/tarpaulin-report.json" ]; then
    echo "--- カバレッジサマリ ---"

    TOTAL_COV=$(jq -r '.coverage_percentage // 0' "$COVERAGE_DIR/tarpaulin-report.json")
    echo "全体カバレッジ: ${TOTAL_COV}%"
    echo ""

    echo "--- ファイル別カバレッジ ---"
    jq -r '.files[] | "\(.path): \(.covered)/\(.coverable) (\(if .coverable > 0 then (.covered/.coverable*100|floor) else 0 end)%)"' \
        "$COVERAGE_DIR/tarpaulin-report.json" 2>/dev/null | sort -t'(' -k2 -n | head -20

    echo ""
    echo "--- 低カバレッジファイル (要注意) ---"
    jq -r '.files[] | select(.coverable > 10) | select((.covered/.coverable) < 0.3) | "\(.path): \((.covered/.coverable*100)|floor)%"' \
        "$COVERAGE_DIR/tarpaulin-report.json" 2>/dev/null || echo "(なし)"

    echo ""
    echo "--- 未カバー行が多いファイル ---"
    jq -r '.files[] | select(.coverable > 0) | "\(.path): \(.coverable - .covered) 行未カバー"' \
        "$COVERAGE_DIR/tarpaulin-report.json" 2>/dev/null | sort -t':' -k2 -rn | head -10

else
    echo "カバレッジレポートがありません"
    echo "計測する場合: ./scripts/analyze_coverage.sh --run"
fi

echo ""
echo "--- 重要ファイルのカバレッジ目標 ---"
echo ""
echo "| ファイル | 目標 | 理由 |"
echo "|---------|------|------|"
echo "| systems/miner.rs | 70%+ | コアゲームロジック |"
echo "| systems/conveyor.rs | 70%+ | コアゲームロジック |"
echo "| systems/furnace.rs | 70%+ | コアゲームロジック |"
echo "| world/mod.rs | 50%+ | ワールド生成 |"
echo "| save.rs | 60%+ | データ永続化 |"
echo "| components/*.rs | 50%+ | データ構造 |"

echo ""
echo "HTMLレポート: $COVERAGE_DIR/tarpaulin-report.html"
