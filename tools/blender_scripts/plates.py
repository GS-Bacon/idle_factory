"""
板材アイテムモデル生成スクリプト
カテゴリ: item (dropped_item: 0.4x0.4x0.4)

使い方:
1. Blenderで新規ファイル作成
2. _base.pyを実行
3. このスクリプトを実行
"""

import bpy
from mathutils import Vector

# _base.pyの関数を使用
# create_chamfered_cube, create_material, apply_material, finalize_model, export_gltf

# =============================================================================
# 板材の色と材質定義
# =============================================================================

PLATE_MATERIALS = {
    "iron_plate": {
        "color": (0.29, 0.29, 0.29, 1),
        "metallic": 1.0,
        "roughness": 0.5
    },
    "copper_plate": {
        "color": (0.72, 0.45, 0.20, 1),
        "metallic": 1.0,
        "roughness": 0.4
    },
    "bronze_plate": {
        "color": (0.65, 0.50, 0.25, 1),
        "metallic": 1.0,
        "roughness": 0.45
    },
    "gold_plate": {
        "color": (1.0, 0.84, 0.0, 1),
        "metallic": 1.0,
        "roughness": 0.3
    },
    "steel_plate": {
        "color": (0.45, 0.45, 0.47, 1),
        "metallic": 1.0,
        "roughness": 0.4
    }
}

# =============================================================================
# 板材モデル生成
# =============================================================================

def create_plate(name, material_props):
    """板材アイテムを生成"""
    # アイテムボックス: 0.4x0.4x0.4
    # 板材: 薄い正方形の板（厚さ0.05程度）
    plate_size = 0.35
    plate_thickness = 0.05

    # 面取りキューブで板材を作成
    plate = create_chamfered_cube(
        size=(plate_size, plate_size, plate_thickness),
        chamfer=0.01,
        location=(0, 0, 0),
        name=name
    )

    # マテリアル適用
    mat = create_material(
        name=f"{name}_material",
        color=material_props["color"],
        metallic=material_props["metallic"],
        roughness=material_props["roughness"]
    )
    apply_material(plate, mat)

    # アイテムカテゴリとして中心原点に設定
    finalize_model(plate, category="item")

    return plate

# =============================================================================
# 全板材を生成
# =============================================================================

def create_all_plates():
    """全ての板材モデルを生成してエクスポート"""
    import os

    output_dir = bpy.path.abspath("//models/items/")
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    for plate_name, mat_props in PLATE_MATERIALS.items():
        print(f"Creating {plate_name}...")

        # シーンクリア
        clear_scene()

        # 板材生成
        plate = create_plate(plate_name, mat_props)

        # エクスポート
        export_path = os.path.join(output_dir, f"{plate_name}.gltf")
        export_gltf(export_path, export_animations=False)

        print(f"  Exported: {export_path}")

    print("All plates created successfully!")

# =============================================================================
# 実行
# =============================================================================

if __name__ == "__main__":
    create_all_plates()
