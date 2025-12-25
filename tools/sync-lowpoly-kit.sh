#!/bin/bash
# モデリングスキルをblender-lowpoly-kitリポジトリに同期するスクリプト
#
# 使い方:
#   ./tools/sync-lowpoly-kit.sh           # 同期のみ
#   ./tools/sync-lowpoly-kit.sh --push    # 同期してプッシュ

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
IDLE_FACTORY_DIR="$(dirname "$SCRIPT_DIR")"
LOWPOLY_KIT_DIR="/home/bacon/github/blender-lowpoly-kit"

echo "=== Blender Lowpoly Kit 同期スクリプト ==="
echo "ソース: $IDLE_FACTORY_DIR"
echo "ターゲット: $LOWPOLY_KIT_DIR"
echo ""

# ターゲットディレクトリの確認
if [ ! -d "$LOWPOLY_KIT_DIR" ]; then
    echo "エラー: $LOWPOLY_KIT_DIR が存在しません"
    exit 1
fi

# 同期対象ファイル
echo "同期中..."

# コアライブラリ
cp "$IDLE_FACTORY_DIR/tools/blender_scripts/_base.py" "$LOWPOLY_KIT_DIR/src/lowpoly_base.py"
echo "  ✓ src/lowpoly_base.py"

# スタイルガイド
cp "$IDLE_FACTORY_DIR/docs/style-guide.json" "$LOWPOLY_KIT_DIR/docs/"
echo "  ✓ docs/style-guide.json"

# モデリングルール（modeling-rules.mdは別リポジトリ用に調整済みなのでスキップ）
# 必要に応じて手動で更新

# スキル定義
cp "$IDLE_FACTORY_DIR/.claude/commands/generate-model.md" "$LOWPOLY_KIT_DIR/claude-skill/"
echo "  ✓ claude-skill/generate-model.md"

# プレビュースクリプト
cp "$IDLE_FACTORY_DIR/tools/preview_model.sh" "$LOWPOLY_KIT_DIR/"
echo "  ✓ preview_model.sh"

echo ""
echo "同期完了！"

# プッシュオプション
if [ "$1" = "--push" ]; then
    echo ""
    echo "変更をコミット＆プッシュ中..."
    cd "$LOWPOLY_KIT_DIR"

    # 変更があるか確認
    if git diff --quiet && git diff --staged --quiet; then
        echo "変更なし"
    else
        git add -A
        git commit -m "sync: idle_factoryから同期

🤖 Generated with [Claude Code](https://claude.com/claude-code)"
        git push
        echo "プッシュ完了！"
    fi
fi

echo ""
echo "リポジトリURL: https://github.com/GS-Bacon/blender-lowpoly-kit-"
