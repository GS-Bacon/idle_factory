#!/bin/bash
# sync-check.sh - ドキュメント整合性チェック
#
# Usage: ./scripts/sync-check.sh [--fix]
#   --fix: バージョン番号を自動修正

cd "$(dirname "$0")/.."

FIX_MODE=false
if [[ "$1" == "--fix" ]]; then
    FIX_MODE=true
fi

ERRORS=0
WARNINGS=0

echo "=== ドキュメント整合性チェック ==="
echo ""

# 1. バージョン整合性チェック
echo "[1/4] バージョン整合性..."
CARGO_VER=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
echo "  Cargo.toml: $CARGO_VER"

VERSION_FILES=(
    "CLAUDE.md"
    ".claude/implementation-plan.md"
    ".specify/roadmap.md"
)

for file in "${VERSION_FILES[@]}"; do
    if [[ -f "$file" ]]; then
        DOC_VER=$(grep -oP 'バージョン.*\*\*\K[0-9]+\.[0-9]+\.[0-9]+' "$file" 2>/dev/null | head -1 || echo "")
        if [[ -n "$DOC_VER" && "$DOC_VER" != "$CARGO_VER" ]]; then
            echo "  ❌ $file: $DOC_VER (不一致)"
            if $FIX_MODE; then
                sed -i "s/バージョン | \*\*$DOC_VER\*\*/バージョン | **$CARGO_VER**/g" "$file"
                echo "     → 修正しました"
            fi
            ((ERRORS++))
        else
            echo "  ✅ $file"
        fi
    fi
done
echo ""

# 2. ドキュメント参照チェック
echo "[2/4] ドキュメント参照..."
# 存在しないファイルへの参照をチェック
BROKEN_REFS=$(grep -rn "\.claude/[a-z-]*\.md" CLAUDE.md .claude/*.md 2>/dev/null | while read line; do
    ref=$(echo "$line" | grep -oP '\.claude/[a-z-]+\.md' | head -1)
    if [[ -n "$ref" && ! -f "$ref" ]]; then
        echo "$line"
    fi
done || true)

if [[ -n "$BROKEN_REFS" ]]; then
    echo "  ❌ 存在しないファイルへの参照:"
    echo "$BROKEN_REFS" | sed 's/^/     /'
    ((ERRORS++))
else
    echo "  ✅ 参照先は全て存在"
fi
echo ""

# 3. 大きいファイルチェック
echo "[3/4] ファイルサイズ..."
LARGE_FILES=$(find src -name "*.rs" -exec wc -l {} \; | awk '$1 > 1000 {print}' | sort -rn)
if [[ -n "$LARGE_FILES" ]]; then
    echo "  ⚠️ 1000行超えファイル（分割推奨）:"
    echo "$LARGE_FILES" | while read line; do
        echo "     $line"
    done
    ((WARNINGS++))
else
    echo "  ✅ 1000行超えファイルなし"
fi
echo ""

# 4. bugs.md ↔ implementation-plan.md 整合性チェック
echo "[4/5] バグ修正 ↔ タスク整合性..."
BUGS_FILE=".claude/bugs.md"
IMPL_FILE=".claude/implementation-plan.md"

if [[ -f "$BUGS_FILE" && -f "$IMPL_FILE" ]]; then
    # BUG-UI-1〜4をチェック
    for i in 1 2 3 4; do
        bug_fixed=$(grep -c "BUG-UI-$i:.*✅ 修正済み" "$BUGS_FILE" 2>/dev/null | head -1 || echo "0")
        task_done=$(grep -c "B\.$i.*✅.*BUG-UI-$i" "$IMPL_FILE" 2>/dev/null | head -1 || echo "0")

        if [[ "$bug_fixed" -gt 0 ]] && [[ "$task_done" -eq 0 ]]; then
            echo "  ⚠️ BUG-UI-$i は修正済みだが implementation-plan.md B.$i が未更新"
            ((WARNINGS++))
        fi
    done

    # 注: BUG-10〜17は作業中バグで個別対応タスクがないためスキップ

    if [[ $WARNINGS -eq 0 ]]; then
        echo "  ✅ バグ-タスク整合性OK"
    fi
else
    echo "  ⚠️ bugs.md または implementation-plan.md が見つかりません"
fi
echo ""

# 5. roadmap完了状態チェック
echo "[5/5] タスク完了状態..."
ROADMAP=".specify/roadmap.md"
if [[ -f "$ROADMAP" ]]; then
    INCOMPLETE=$(grep -c "| ❌" "$ROADMAP" 2>/dev/null || echo "0")
    COMPLETE=$(grep -c "| ✅" "$ROADMAP" 2>/dev/null || echo "0")
    echo "  完了: $COMPLETE件"
    echo "  未完了: $INCOMPLETE件"

    # WORK_LOGと矛盾がないかチェック（簡易）
    WORKLOG_COMPLETE=$(grep -c "^## 2026" WORK_LOG.md 2>/dev/null || echo "0")
    echo "  WORK_LOG作業数: $WORKLOG_COMPLETE件"
fi
echo ""

# 6. ドキュメント齟齬修正カウンター
echo "[6/6] 齟齬修正カウンター..."
DOC_SYNC_LOG=".claude/doc-sync-fixes.log"
if [[ -f "$DOC_SYNC_LOG" ]]; then
    # コメント行を除いてカウント
    FIX_COUNT=$(grep -v "^#" "$DOC_SYNC_LOG" | grep -c "^[0-9]" 2>/dev/null || echo "0")
    echo "  齟齬修正回数: $FIX_COUNT 回"
    if [[ "$FIX_COUNT" -ge 10 ]]; then
        echo ""
        echo "  ⚠️⚠️⚠️ 齟齬修正が10回を超えました ⚠️⚠️⚠️"
        echo "  → tasks.toml ベースの構造化データ管理への移行を検討してください"
        echo "  → 参照: CLAUDE.md「タスク完了の定義」セクション"
        echo ""
        ((WARNINGS++))
    fi
else
    echo "  ログファイルなし"
fi
echo ""

# サマリー
echo "=== サマリー ==="
if [[ $ERRORS -gt 0 ]]; then
    echo "❌ エラー: $ERRORS件"
    if ! $FIX_MODE; then
        echo "   → ./scripts/sync-check.sh --fix で自動修正可能"
    fi
fi
if [[ $WARNINGS -gt 0 ]]; then
    echo "⚠️ 警告: $WARNINGS件"
fi
if [[ $ERRORS -eq 0 && $WARNINGS -eq 0 ]]; then
    echo "✅ 全てOK"
fi

# エラーがある場合のみ非ゼロで終了（警告は無視）
exit $ERRORS
