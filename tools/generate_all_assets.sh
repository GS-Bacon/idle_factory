#!/bin/bash
# 全アセット一括生成スクリプト
#
# 使い方:
#   ./tools/generate_all_assets.sh        # 全て生成
#   ./tools/generate_all_assets.sh --vox  # VOXのみ
#   ./tools/generate_all_assets.sh --glb  # GLB変換のみ
#   ./tools/generate_all_assets.sh --tex  # テクスチャのみ
#   ./tools/generate_all_assets.sh --spr  # スプライトのみ

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

# 色付き出力
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

generate_vox() {
    echo -e "${YELLOW}=== Generating VOX files ===${NC}"
    python3 tools/voxel_generator.py all
    echo -e "${GREEN}VOX generation complete${NC}"
}

convert_glb() {
    echo -e "${YELLOW}=== Converting VOX to GLB ===${NC}"
    ./tools/convert_all_vox.sh
    echo -e "${GREEN}GLB conversion complete${NC}"
}

generate_textures() {
    echo -e "${YELLOW}=== Generating textures ===${NC}"
    DISPLAY=:10 blender --background --python tools/generate_textures.py 2>&1 | grep -E "(Saved|Generated|Error)" || true
    echo -e "${GREEN}Texture generation complete${NC}"
}

generate_sprites() {
    echo -e "${YELLOW}=== Generating sprites ===${NC}"
    DISPLAY=:10 blender --background --python tools/generate_item_sprites.py 2>&1 | grep -E "(Saved|Generated|Error)" || true
    echo -e "${GREEN}Sprite generation complete${NC}"
}

# 引数処理
case "$1" in
    --vox)
        generate_vox
        ;;
    --glb)
        convert_glb
        ;;
    --tex|--textures)
        generate_textures
        ;;
    --spr|--sprites)
        generate_sprites
        ;;
    --help|-h)
        echo "Usage: $0 [option]"
        echo ""
        echo "Options:"
        echo "  (none)      Generate all assets"
        echo "  --vox       Generate VOX files only"
        echo "  --glb       Convert VOX to GLB only"
        echo "  --tex       Generate textures only"
        echo "  --spr       Generate sprites only"
        echo "  --help      Show this help"
        ;;
    *)
        # 全て生成
        echo "=== Full Asset Generation ==="
        echo ""
        generate_vox
        echo ""
        convert_glb
        echo ""
        generate_textures
        echo ""
        generate_sprites
        echo ""
        echo -e "${GREEN}=== All assets generated! ===${NC}"
        ;;
esac
