#!/bin/bash
# 全VOXファイルをGLBに変換するスクリプト

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

# 変換関数
convert_vox() {
    local vox_file="$1"
    local glb_file="${vox_file%.vox}.glb"

    if [[ -f "$vox_file" ]]; then
        echo "Converting: $vox_file -> $glb_file"
        DISPLAY=:10 blender --background --python tools/vox_to_gltf.py -- "$vox_file" "$glb_file" 2>&1 | grep -E "(Exported|Error)" || true
    fi
}

# 引数チェック
if [[ "$1" == "--items" ]]; then
    echo "=== Converting item models ==="
    for vox in assets/models/items/*.vox; do
        [[ -f "$vox" ]] && convert_vox "$vox"
    done
elif [[ "$1" == "--machines" ]]; then
    echo "=== Converting machine models ==="
    for vox in assets/models/machines/*.vox; do
        [[ -f "$vox" ]] && convert_vox "$vox"
    done
elif [[ "$1" == "--conveyors" ]]; then
    echo "=== Converting conveyor models ==="
    for vox in assets/models/machines/conveyor/*.vox; do
        [[ -f "$vox" ]] && convert_vox "$vox"
    done
elif [[ -n "$1" && -f "$1" ]]; then
    # 単一ファイル
    convert_vox "$1"
else
    # 全てのVOXファイル
    echo "=== Converting all VOX files ==="

    echo ""
    echo "--- Items ---"
    for vox in assets/models/items/*.vox; do
        [[ -f "$vox" ]] && convert_vox "$vox"
    done

    echo ""
    echo "--- Machines ---"
    for vox in assets/models/machines/*.vox; do
        [[ -f "$vox" ]] && convert_vox "$vox"
    done

    echo ""
    echo "--- Conveyors ---"
    for vox in assets/models/machines/conveyor/*.vox; do
        [[ -f "$vox" ]] && convert_vox "$vox"
    done
fi

echo ""
echo "Done!"
