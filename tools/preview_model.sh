#!/bin/bash
# モデルプレビュー＆スクリーンショット撮影スクリプト
# 使い方: ./tools/preview_model.sh <model_path>

MODEL_PATH="$1"
SCREENSHOT_DIR="tools/model_screenshots"

if [ -z "$MODEL_PATH" ]; then
    echo "使い方: $0 <model_path>"
    echo "例: $0 assets/models/items/bronze_pickaxe.gltf"
    exit 1
fi

if [ ! -f "$MODEL_PATH" ]; then
    echo "エラー: ファイルが見つかりません: $MODEL_PATH"
    exit 1
fi

# スクリーンショットディレクトリ作成
mkdir -p "$SCREENSHOT_DIR"

# モデル名を取得
MODEL_NAME=$(basename "$MODEL_PATH" .gltf)
MODEL_NAME=$(basename "$MODEL_NAME" .glb)
SCREENSHOT_PATH="$SCREENSHOT_DIR/${MODEL_NAME}.png"

echo "=== モデルプレビュー ==="
echo "モデル: $MODEL_PATH"
echo "スクリーンショット保存先: $SCREENSHOT_PATH"
echo ""

# F3Dでスクリーンショット撮影（DISPLAY設定必須、斜めアングル）
DISPLAY=:10 f3d "$MODEL_PATH" \
    --output="$SCREENSHOT_PATH" \
    --resolution=800,600 \
    --bg-color=0.2,0.2,0.2 \
    --up=+Y \
    --camera-position=0.8,0.8,0.5 \
    2>/dev/null

if [ -f "$SCREENSHOT_PATH" ]; then
    echo "スクリーンショット保存完了: $SCREENSHOT_PATH"
    echo ""
    echo "確認コマンド:"
    echo "  xdg-open $SCREENSHOT_PATH"
else
    echo "スクリーンショット撮影失敗"
fi
