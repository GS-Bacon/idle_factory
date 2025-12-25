"""
Industrial Lowpoly Style - Mechanical Parts (Wires, Rods, Gears)
style-guide.json v1.0.0 準拠

アイテムカテゴリ: dropped_item (0.4x0.4x0.4)

モデル:
- ワイヤー: copper_wire, iron_wire, gold_wire
- ロッド: iron_rod, copper_rod, steel_rod
- ギア: iron_gear, copper_gear, bronze_gear, steel_gear
"""

import bpy
from mathutils import Vector, Matrix
from math import pi, cos, sin
import os
import sys

# _base.pyのパスを追加
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.append(script_dir)

# 基本モジュールをインポート
from _base import (
    create_octagonal_prism,
    create_gear,
    apply_preset_material,
    finalize_model,
    export_gltf,
    clear_scene,
    GRID_UNIT,
    snap_vec
)

# =============================================================================
# ワイヤー生成
# =============================================================================

def create_wire_coil(name="wire", material_preset="copper"):
    """コイル状のワイヤー"""
    clear_scene()

    # ワイヤーのパラメータ
    wire_radius = 0.01  # ワイヤーの太さ
    coil_radius = 0.08  # コイルの半径
    coil_height = 0.25  # コイルの高さ
    turns = 5  # 巻き数
    segments_per_turn = 12  # 1周あたりのセグメント数

    objects = []

    # らせん状にワイヤーセグメントを配置
    total_segments = turns * segments_per_turn
    for i in range(total_segments):
        angle = (i / segments_per_turn) * 2 * pi
        z_pos = (i / total_segments) * coil_height - coil_height / 2

        x = cos(angle) * coil_radius
        y = sin(angle) * coil_radius

        # 短い八角柱を作成
        segment_height = coil_height / total_segments * 1.5  # 少し重ねる
        segment = create_octagonal_prism(
            radius=wire_radius,
            height=segment_height,
            location=(x, y, z_pos),
            name=f"{name}_seg_{i}"
        )

        # セグメントを接線方向に回転
        next_angle = ((i + 1) / segments_per_turn) * 2 * pi
        next_z = ((i + 1) / total_segments) * coil_height - coil_height / 2

        dx = cos(next_angle) * coil_radius - x
        dy = sin(next_angle) * coil_radius - y
        dz = next_z - z_pos

        # 回転を適用
        import math
        segment.rotation_euler.y = math.atan2(dx, dz)
        segment.rotation_euler.z = math.atan2(dy, dx)

        objects.append(segment)

    # すべてのセグメントを結合
    bpy.context.view_layer.objects.active = objects[0]
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    wire = objects[0]
    wire.name = name

    # マテリアル適用
    apply_preset_material(wire, material_preset)

    # 最終処理
    finalize_model(wire, category="item")

    return wire

# =============================================================================
# ロッド生成
# =============================================================================

def create_rod(name="rod", material_preset="iron"):
    """八角柱のロッド"""
    clear_scene()

    # ロッドのパラメータ
    radius = 0.03  # ロッドの半径
    length = 0.3  # ロッドの長さ

    rod = create_octagonal_prism(
        radius=radius,
        height=length,
        location=(0, 0, 0),
        name=name
    )

    # マテリアル適用
    apply_preset_material(rod, material_preset)

    # 最終処理
    finalize_model(rod, category="item")

    return rod

# =============================================================================
# ギア生成
# =============================================================================

def create_gear_item(name="gear", material_preset="iron"):
    """ギアアイテム"""
    clear_scene()

    # ギアのパラメータ
    radius = 0.15  # ギアの半径
    thickness = 0.04  # ギアの厚さ
    teeth = 8  # 歯の数

    gear = create_gear(
        radius=radius,
        thickness=thickness,
        teeth=teeth,
        hole_radius=0.03,
        location=(0, 0, 0),
        name=name
    )

    # マテリアル適用
    apply_preset_material(gear, material_preset)

    # 最終処理
    finalize_model(gear, category="item")

    return gear

# =============================================================================
# モデル生成・エクスポート
# =============================================================================

def generate_all_models():
    """すべてのモデルを生成してエクスポート"""
    output_dir = os.path.join(script_dir, "..", "..", "assets", "models", "items")
    os.makedirs(output_dir, exist_ok=True)

    models = []

    # ワイヤー
    wires = [
        ("copper_wire", "copper"),
        ("iron_wire", "iron"),
        ("gold_wire", "brass"),  # goldの代わりにbrass
    ]

    for name, material in wires:
        print(f"\n=== Generating {name} ===")
        wire = create_wire_coil(name, material)
        filepath = os.path.join(output_dir, f"{name}.gltf")
        export_gltf(filepath, export_animations=False)
        models.append(name)

    # ロッド
    rods = [
        ("iron_rod", "iron"),
        ("copper_rod", "copper"),
        ("steel_rod", "dark_steel"),
    ]

    for name, material in rods:
        print(f"\n=== Generating {name} ===")
        rod = create_rod(name, material)
        filepath = os.path.join(output_dir, f"{name}.gltf")
        export_gltf(filepath, export_animations=False)
        models.append(name)

    # ギア
    gears = [
        ("iron_gear", "iron"),
        ("copper_gear", "copper"),
        ("bronze_gear", "brass"),  # bronzeの代わりにbrass
        ("steel_gear", "dark_steel"),
    ]

    for name, material in gears:
        print(f"\n=== Generating {name} ===")
        gear = create_gear_item(name, material)
        filepath = os.path.join(output_dir, f"{name}.gltf")
        export_gltf(filepath, export_animations=False)
        models.append(name)

    print("\n=== All models generated ===")
    print(f"Total: {len(models)} models")
    print(f"Output directory: {output_dir}")
    for model in models:
        print(f"  - {model}.gltf")

# =============================================================================
# 実行
# =============================================================================

if __name__ == "__main__":
    generate_all_models()
