"""
Food Item Models - Industrial Lowpoly Style
アイテムカテゴリ: item (dropped_item: 0.4x0.4x0.4)
"""

import bpy
from mathutils import Vector
from math import pi
import os
import sys

# _base.pyをインポート
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.append(script_dir)

from _base import (
    create_chamfered_cube, create_octagon, create_octagonal_prism,
    create_material, apply_material, finalize_model, export_gltf,
    clear_scene, snap_vec, GRID_UNIT
)

# =============================================================================
# Food Models
# =============================================================================

def create_wheat():
    """麦の穂"""
    clear_scene()

    # 茎（細い八角柱）
    stem = create_octagonal_prism(
        radius=0.02,
        height=0.3,
        location=(0, 0, 0.15),
        name="Wheat_stem"
    )
    stem.rotation_euler.x = pi / 2

    # 穂（小さな面取りキューブを配列）
    grain_objects = [stem]

    for i in range(5):
        z_offset = 0.25 + i * 0.025
        scale = 1.0 - i * 0.15

        # 左右に粒を配置
        for side in [-1, 1]:
            grain = create_chamfered_cube(
                size=(0.02 * scale, 0.02 * scale, 0.04 * scale),
                location=(side * 0.015, 0, z_offset),
                name=f"Wheat_grain_{i}_{side}"
            )
            grain_objects.append(grain)

    # 結合
    bpy.context.view_layer.objects.active = stem
    for obj in grain_objects:
        obj.select_set(True)
    bpy.ops.object.join()

    # マテリアル（金色）
    mat = create_material("wheat_mat", color=(0.85, 0.7, 0.3, 1), metallic=0.0, roughness=0.7)
    apply_material(stem, mat)

    finalize_model(stem, category="item")
    return stem


def create_flour():
    """白い粉袋"""
    clear_scene()

    # 袋本体（面取りキューブ）
    bag = create_chamfered_cube(
        size=(0.25, 0.18, 0.3),
        location=(0, 0, 0.15),
        name="Flour_bag"
    )

    # 袋上部の閉じ込み部分（薄い直方体）
    top = create_chamfered_cube(
        size=(0.25, 0.18, 0.05),
        location=(0, 0, 0.325),
        name="Flour_top"
    )

    # 結合
    bpy.context.view_layer.objects.active = bag
    top.select_set(True)
    bag.select_set(True)
    bpy.ops.object.join()

    # マテリアル（白）
    mat = create_material("flour_mat", color=(0.95, 0.95, 0.95, 1), metallic=0.0, roughness=0.9)
    apply_material(bag, mat)

    finalize_model(bag, category="item")
    return bag


def create_bread():
    """パン（楕円形のローフ）"""
    clear_scene()

    # 楕円状の面取りキューブ
    loaf = create_chamfered_cube(
        size=(0.3, 0.15, 0.12),
        chamfer=0.04,
        location=(0, 0, 0.06),
        name="Bread_loaf"
    )

    # 上部の膨らみ（小さい面取りキューブ）
    top = create_chamfered_cube(
        size=(0.25, 0.12, 0.06),
        chamfer=0.03,
        location=(0, 0, 0.15),
        name="Bread_top"
    )

    # 結合
    bpy.context.view_layer.objects.active = loaf
    top.select_set(True)
    loaf.select_set(True)
    bpy.ops.object.join()

    # マテリアル（茶色）
    mat = create_material("bread_mat", color=(0.7, 0.5, 0.3, 1), metallic=0.0, roughness=0.8)
    apply_material(loaf, mat)

    finalize_model(loaf, category="item")
    return loaf


def create_vegetables():
    """野菜の盛り合わせ"""
    clear_scene()

    objects = []

    # トマト（赤い八角形）
    tomato = create_octagon(
        radius=0.08,
        depth=0.08,
        location=(-0.05, 0.05, 0.04),
        name="Veg_tomato"
    )
    mat_red = create_material("veg_tomato", color=(0.9, 0.2, 0.2, 1), metallic=0.0, roughness=0.6)
    apply_material(tomato, mat_red)
    objects.append(tomato)

    # ニンジン（橙色の細長い八角柱）
    carrot = create_octagonal_prism(
        radius=0.03,
        height=0.15,
        location=(0.05, -0.05, 0.075),
        name="Veg_carrot"
    )
    carrot.rotation_euler.x = pi / 4
    mat_orange = create_material("veg_carrot", color=(0.9, 0.5, 0.1, 1), metallic=0.0, roughness=0.7)
    apply_material(carrot, mat_orange)
    objects.append(carrot)

    # レタス（緑の面取りキューブ）
    lettuce = create_chamfered_cube(
        size=(0.1, 0.1, 0.06),
        location=(0, 0, 0.03),
        name="Veg_lettuce"
    )
    mat_green = create_material("veg_lettuce", color=(0.3, 0.7, 0.3, 1), metallic=0.0, roughness=0.8)
    apply_material(lettuce, mat_green)
    objects.append(lettuce)

    # 結合
    bpy.context.view_layer.objects.active = lettuce
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    finalize_model(lettuce, category="item")
    return lettuce


def create_raw_meat():
    """生肉（ピンク）"""
    clear_scene()

    # 肉の塊（不規則な形を面取りキューブで表現）
    meat = create_chamfered_cube(
        size=(0.25, 0.18, 0.1),
        chamfer=0.03,
        location=(0, 0, 0.05),
        name="RawMeat"
    )

    # マテリアル（ピンク）
    mat = create_material("raw_meat_mat", color=(0.9, 0.5, 0.5, 1), metallic=0.0, roughness=0.7)
    apply_material(meat, mat)

    finalize_model(meat, category="item")
    return meat


def create_cooked_meat():
    """焼いた肉（茶色）"""
    clear_scene()

    # 焼けた肉（raw_meatと同形状、色違い）
    meat = create_chamfered_cube(
        size=(0.25, 0.18, 0.1),
        chamfer=0.03,
        location=(0, 0, 0.05),
        name="CookedMeat"
    )

    # マテリアル（茶色）
    mat = create_material("cooked_meat_mat", color=(0.55, 0.35, 0.2, 1), metallic=0.0, roughness=0.6)
    apply_material(meat, mat)

    finalize_model(meat, category="item")
    return meat


def create_apple():
    """りんご（赤い球状）"""
    clear_scene()

    # 本体（八角形を重ねて球状に）
    body_objects = []

    # 中心部（最大直径）
    center = create_octagon(
        radius=0.12,
        depth=0.08,
        location=(0, 0, 0),
        name="Apple_center"
    )
    body_objects.append(center)

    # 上下の八角形（小さくしていく）
    for i, z_offset in enumerate([0.06, 0.1, -0.06, -0.1]):
        scale = 0.8 - abs(z_offset) * 2
        ring = create_octagon(
            radius=0.12 * scale,
            depth=0.04,
            location=(0, 0, z_offset),
            name=f"Apple_ring_{i}"
        )
        body_objects.append(ring)

    # 茎（細い八角柱）
    stem = create_octagonal_prism(
        radius=0.015,
        height=0.05,
        location=(0, 0, 0.14),
        name="Apple_stem"
    )
    stem.rotation_euler.x = pi / 2

    # 本体を結合
    bpy.context.view_layer.objects.active = center
    for obj in body_objects:
        obj.select_set(True)
    bpy.ops.object.join()

    # マテリアル（赤）
    mat_red = create_material("apple_mat", color=(0.8, 0.15, 0.15, 1), metallic=0.0, roughness=0.5)
    apply_material(center, mat_red)

    # 茎のマテリアル（茶色）
    mat_stem = create_material("apple_stem_mat", color=(0.4, 0.3, 0.2, 1), metallic=0.0, roughness=0.8)
    apply_material(stem, mat_stem)

    # 全体を結合
    bpy.context.view_layer.objects.active = center
    stem.select_set(True)
    center.select_set(True)
    bpy.ops.object.join()

    finalize_model(center, category="item")
    return center


def create_golden_apple():
    """金色のりんご"""
    clear_scene()

    # 本体（八角形を重ねて球状に）
    body_objects = []

    # 中心部（最大直径）
    center = create_octagon(
        radius=0.12,
        depth=0.08,
        location=(0, 0, 0),
        name="GoldenApple_center"
    )
    body_objects.append(center)

    # 上下の八角形（小さくしていく）
    for i, z_offset in enumerate([0.06, 0.1, -0.06, -0.1]):
        scale = 0.8 - abs(z_offset) * 2
        ring = create_octagon(
            radius=0.12 * scale,
            depth=0.04,
            location=(0, 0, z_offset),
            name=f"GoldenApple_ring_{i}"
        )
        body_objects.append(ring)

    # 茎（細い八角柱）
    stem = create_octagonal_prism(
        radius=0.015,
        height=0.05,
        location=(0, 0, 0.14),
        name="GoldenApple_stem"
    )
    stem.rotation_euler.x = pi / 2

    # 本体を結合
    bpy.context.view_layer.objects.active = center
    for obj in body_objects:
        obj.select_set(True)
    bpy.ops.object.join()

    # マテリアル（金色）
    mat_gold = create_material("golden_apple_mat", color=(0.95, 0.8, 0.2, 1), metallic=0.8, roughness=0.3)
    apply_material(center, mat_gold)

    # 茎のマテリアル（茶色）
    mat_stem = create_material("golden_apple_stem_mat", color=(0.4, 0.3, 0.2, 1), metallic=0.0, roughness=0.8)
    apply_material(stem, mat_stem)

    # 全体を結合
    bpy.context.view_layer.objects.active = center
    stem.select_set(True)
    center.select_set(True)
    bpy.ops.object.join()

    finalize_model(center, category="item")
    return center


# =============================================================================
# Batch Export
# =============================================================================

def export_all_food():
    """全食料アイテムをエクスポート"""
    output_dir = os.path.join(script_dir, "../../assets/models/items/food")
    os.makedirs(output_dir, exist_ok=True)

    models = {
        "wheat": create_wheat,
        "flour": create_flour,
        "bread": create_bread,
        "vegetables": create_vegetables,
        "raw_meat": create_raw_meat,
        "cooked_meat": create_cooked_meat,
        "apple": create_apple,
        "golden_apple": create_golden_apple,
    }

    for name, create_func in models.items():
        print(f"\n=== Creating {name} ===")
        create_func()
        filepath = os.path.join(output_dir, f"{name}.gltf")
        export_gltf(filepath, export_animations=False)
        print(f"Exported: {name}")

    print("\n=== All food items exported ===")


# =============================================================================
# Main
# =============================================================================

if __name__ == "__main__":
    # 個別実行例
    # create_wheat()

    # 全エクスポート
    export_all_food()
