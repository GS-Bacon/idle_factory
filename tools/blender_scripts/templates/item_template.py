"""
Item Template - カテゴリ: item (handheld_item: 0.3x0.3x0.3)

使い方:
1. このテンプレートをコピー
2. {MODEL_NAME}, {EXPORT_NAME} を置換
3. # TODO セクションを実装

ポリゴン目安: 50-200 (max 500)
原点: center
"""

import bpy
from mathutils import Vector
from math import pi
import os
import sys

# _base.py をロード
script_dir = os.path.dirname(os.path.abspath(__file__))
base_path = os.path.join(os.path.dirname(script_dir), "_base.py")
exec(open(base_path).read())

# =============================================================================
# モデル定義
# =============================================================================

MODEL_NAME = "{MODEL_NAME}"  # 例: "copper_pickaxe"
EXPORT_NAME = "{EXPORT_NAME}"  # 例: "copper_pickaxe"


def create_model():
    """モデル生成"""
    clear_scene()

    parts = []

    # ==========================================================================
    # TODO: パーツを生成
    # ==========================================================================

    # 例: ツールの場合
    # handle = create_tool_handle(length=0.15, radius=0.012, material="wood")
    # parts.append(handle)

    # 例: インゴットの場合
    # ingot = create_ingot(material="copper")
    # parts.append(ingot)

    # 例: プレートの場合
    # plate = create_plate(material="iron")
    # parts.append(plate)

    # ==========================================================================
    # 結合 & 仕上げ
    # ==========================================================================

    if len(parts) == 1:
        result = parts[0]
    else:
        result = join_all_meshes(parts, MODEL_NAME)

    result.name = MODEL_NAME

    # バリデーション
    print_validation_report(result, category="item")

    # 仕上げ（原点をcenterに設定）
    finalize_model(result, category="item")

    return result


# =============================================================================
# エクスポート
# =============================================================================

def export():
    """エクスポート"""
    model = create_model()

    output_dir = os.path.join(script_dir, "..", "..", "assets", "models", "items")
    os.makedirs(output_dir, exist_ok=True)

    filepath = os.path.join(output_dir, f"{EXPORT_NAME}.gltf")
    export_gltf(filepath, export_animations=False)
    print(f"Exported: {filepath}")


if __name__ == "__main__":
    export()
