"""
インゴットアイテムモデル生成
カテゴリ: item (dropped_item: 0.4x0.4x0.4)
"""

import bpy
from mathutils import Vector
import sys
import os

# _base.pyをインポート
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.append(script_dir)

# _base.pyが既に実行されていることを前提とするが、念のため確認
try:
    from _base import (
        clear_scene, create_material, apply_material,
        finalize_model, export_gltf, snap_vec
    )
except ImportError:
    print("ERROR: _base.py must be executed first!")
    print("Please run _base.py before running this script.")
    sys.exit(1)

# =============================================================================
# インゴット定義
# =============================================================================

INGOT_SIZE = (0.35, 0.20, 0.08)  # dropped_item 0.4x0.4x0.4 に収まるサイズ
TAPER_RATIO = 0.85  # 上面が底面の85%幅

INGOT_MATERIALS = {
    "iron": {"color": (0.29, 0.29, 0.29, 1), "metallic": 1.0, "roughness": 0.5},
    "copper": {"color": (0.72, 0.45, 0.20, 1), "metallic": 1.0, "roughness": 0.4},
    "tin": {"color": (0.75, 0.75, 0.75, 1), "metallic": 1.0, "roughness": 0.35},
    "gold": {"color": (1.0, 0.84, 0.0, 1), "metallic": 1.0, "roughness": 0.3},
    "nickel": {"color": (0.58, 0.58, 0.58, 1), "metallic": 1.0, "roughness": 0.4},
    "bronze": {"color": (0.80, 0.50, 0.20, 1), "metallic": 1.0, "roughness": 0.45},
    "steel": {"color": (0.20, 0.20, 0.22, 1), "metallic": 1.0, "roughness": 0.55},
    "invar": {"color": (0.50, 0.55, 0.60, 1), "metallic": 1.0, "roughness": 0.45},
}

INGOT_TYPES = [
    "iron", "copper", "tin", "gold", "nickel", "bronze", "steel", "invar"
]

# =============================================================================
# インゴット形状生成
# =============================================================================

def create_ingot_shape(length, width, height, taper_ratio=TAPER_RATIO, name="Ingot"):
    """
    台形断面のインゴット形状を生成
    - length: X軸方向の長さ
    - width: Y軸方向の幅（底面）
    - height: Z軸方向の高さ
    - taper_ratio: 上面幅の比率（0.85 = 底面の85%）
    """
    l, w, h = length / 2, width / 2, height / 2
    tw = w * taper_ratio  # 上面の幅

    # 頂点定義（台形断面の直方体）
    verts = [
        # 底面（Z-）広い
        (-l, -w, -h), (l, -w, -h), (l, w, -h), (-l, w, -h),
        # 上面（Z+）狭い
        (-l, -tw, h), (l, -tw, h), (l, tw, h), (-l, tw, h),
    ]

    faces = [
        (0, 1, 2, 3),  # 底面
        (7, 6, 5, 4),  # 上面
        (0, 4, 5, 1),  # 前面（Y-側、台形）
        (2, 6, 7, 3),  # 後面（Y+側、台形）
        (0, 3, 7, 4),  # 左側面
        (1, 5, 6, 2),  # 右側面
    ]

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()

    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector((0, 0, 0))
    bpy.context.collection.objects.link(obj)

    return obj

# =============================================================================
# インゴット生成関数
# =============================================================================

def create_ingot(material_type, name=None):
    """指定された素材のインゴットを生成"""
    if material_type not in INGOT_MATERIALS:
        print(f"ERROR: Unknown material type '{material_type}'")
        return None

    if name is None:
        name = f"{material_type}_ingot"

    # インゴット形状作成
    l, w, h = INGOT_SIZE
    obj = create_ingot_shape(l, w, h, TAPER_RATIO, name)

    # マテリアル適用
    mat_data = INGOT_MATERIALS[material_type]
    mat = create_material(
        f"{material_type}_ingot_mat",
        color=mat_data["color"],
        metallic=mat_data["metallic"],
        roughness=mat_data["roughness"]
    )
    apply_material(obj, mat)

    # 最終処理（原点を中心に設定、dropped_itemなので）
    finalize_model(obj, category="item")

    print(f"Created: {name}")
    return obj

# =============================================================================
# 全インゴット生成・エクスポート
# =============================================================================

def generate_all_ingots(output_dir="./models/items/ingots"):
    """全インゴットを生成してエクスポート"""
    os.makedirs(output_dir, exist_ok=True)

    for ingot_type in INGOT_TYPES:
        print(f"\n--- Generating {ingot_type}_ingot ---")
        clear_scene()

        obj = create_ingot(ingot_type)
        if obj:
            output_path = os.path.join(output_dir, f"{ingot_type}_ingot.gltf")
            export_gltf(output_path, export_animations=False)

    print("\n=== All ingots generated successfully ===")

# =============================================================================
# 実行
# =============================================================================

if __name__ == "__main__":
    # 既存シーンをクリア
    clear_scene()

    # 全インゴットを生成してエクスポート
    # 出力先ディレクトリを指定（必要に応じて変更）
    output_directory = "./models/items/ingots"
    generate_all_ingots(output_directory)
