"""
Tool Items - Blender生成スクリプト
pickaxes, axes, shovels, industrial tools

カテゴリ: item (handheld_item: 0.3x0.3x0.3)
"""

import bpy
from mathutils import Vector
from math import pi
import sys
import os

# _base.pyをインポート
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.append(script_dir)

from _base import (
    clear_scene,
    create_octagonal_prism,
    create_chamfered_cube,
    create_hexagon,
    apply_preset_material,
    create_material,
    apply_material,
    finalize_model,
    export_gltf,
    snap,
    snap_vec,
    GRID_UNIT,
)

# =============================================================================
# 共通パーツ
# =============================================================================

def create_handle(length=0.15, radius=0.01, material="wood"):
    """木製八角柱ハンドル"""
    handle = create_octagonal_prism(
        radius=radius,
        height=length,
        location=(0, 0, 0),
        name="Handle"
    )
    apply_preset_material(handle, material)
    return handle

def create_pickaxe_head(material_preset="stone"):
    """ピッケル先端（T字型、尖った形）"""
    objects = []

    # 中央ブロック（柄との接続部）
    center = create_chamfered_cube(
        size=(0.08, 0.03, 0.03),
        location=(0, 0, 0),
        name="PickHead_center"
    )
    objects.append(center)

    # 左右の尖った先端（ピラミッド型）
    for side in [-1, 1]:
        # 先端の尖り部分
        tip = create_chamfered_cube(
            size=(0.04, 0.025, 0.025),
            location=(side * 0.06, 0, 0),
            name=f"PickHead_tip_{side}"
        )
        # スケールで尖らせる
        tip.scale.x = 0.6

        # トランスフォーム適用
        bpy.context.view_layer.objects.active = tip
        bpy.ops.object.transform_apply(scale=True)
        objects.append(tip)

    # 結合
    bpy.context.view_layer.objects.active = center
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    apply_preset_material(center, material_preset)
    return center

def create_axe_head(material_preset="stone"):
    """斧頭（台形型ブレード）"""
    objects = []

    # 接続部
    mount = create_chamfered_cube(
        size=(0.03, 0.04, 0.03),
        location=(0, 0, 0),
        name="AxeHead_mount"
    )
    objects.append(mount)

    # ブレード（台形）
    blade = create_chamfered_cube(
        size=(0.025, 0.06, 0.04),
        location=(0, 0.04, 0),
        name="AxeHead_blade"
    )
    # 刃先を薄く
    blade.scale.y = 1.2
    blade.location.y = 0.035
    bpy.context.view_layer.objects.active = blade
    bpy.ops.object.transform_apply(scale=True)
    objects.append(blade)

    # 結合
    bpy.context.view_layer.objects.active = mount
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    apply_preset_material(mount, material_preset)
    return mount

def create_shovel_head(material_preset="iron"):
    """シャベル先端（平らな）"""
    blade = create_chamfered_cube(
        size=(0.05, 0.06, 0.01),
        location=(0, 0.03, 0),
        name="ShovelHead"
    )
    apply_preset_material(blade, material_preset)
    return blade

# =============================================================================
# ピッケル
# =============================================================================

def create_pickaxe(name, material_preset):
    """ピッケル生成"""
    clear_scene()

    # ハンドル
    handle = create_handle(length=0.18, radius=0.01)
    handle.location.y = -0.09

    # ヘッド
    head = create_pickaxe_head(material_preset)
    head.location.y = 0.02
    head.rotation_euler.x = pi / 2

    # 結合
    bpy.context.view_layer.objects.active = handle
    head.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    handle.name = name

    finalize_model(handle, category="item")
    return handle

# =============================================================================
# 斧
# =============================================================================

def create_axe(name, material_preset):
    """斧生成"""
    clear_scene()

    # ハンドル
    handle = create_handle(length=0.18, radius=0.01)
    handle.location.y = -0.09

    # ヘッド
    head = create_axe_head(material_preset)
    head.location.y = 0.02
    head.rotation_euler.x = pi / 2

    # 結合
    bpy.context.view_layer.objects.active = handle
    head.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    handle.name = name

    finalize_model(handle, category="item")
    return handle

# =============================================================================
# シャベル
# =============================================================================

def create_shovel(name, material_preset):
    """シャベル生成"""
    clear_scene()

    # ハンドル
    handle = create_handle(length=0.18, radius=0.01)
    handle.location.y = -0.09

    # ブレード
    blade = create_shovel_head(material_preset)
    blade.rotation_euler.x = pi / 2

    # 結合
    bpy.context.view_layer.objects.active = handle
    blade.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    handle.name = name

    finalize_model(handle, category="item")
    return handle

# =============================================================================
# 工業ツール
# =============================================================================

def create_hammer(name="hammer"):
    """ハンマー"""
    clear_scene()

    # ハンドル
    handle = create_handle(length=0.15, radius=0.008)
    handle.location.y = -0.08

    # ヘッド（直方体）
    head = create_chamfered_cube(
        size=(0.025, 0.05, 0.025),
        location=(0, 0.01, 0),
        name="HammerHead"
    )
    head.rotation_euler.x = pi / 2
    apply_preset_material(head, "iron")

    # 結合
    bpy.context.view_layer.objects.active = handle
    head.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    handle.name = name

    finalize_model(handle, category="item")
    return handle

def create_wrench(name="wrench"):
    """スパナ"""
    clear_scene()

    # ハンドル部
    handle_part = create_chamfered_cube(
        size=(0.015, 0.12, 0.01),
        location=(0, -0.04, 0),
        name="WrenchHandle"
    )
    apply_preset_material(handle_part, "iron")

    # 開口部（六角形を使用）
    jaw = create_hexagon(
        radius=0.025,
        depth=0.01,
        location=(0, 0.04, 0),
        name="WrenchJaw"
    )
    apply_preset_material(jaw, "iron")

    # 中央の穴（六角形で小さく）
    hole = create_hexagon(
        radius=0.015,
        depth=0.012,
        location=(0, 0.04, 0),
        name="WrenchHole"
    )

    # Boolean差分
    mod = jaw.modifiers.new("Bool", 'BOOLEAN')
    mod.operation = 'DIFFERENCE'
    mod.object = hole
    bpy.context.view_layer.objects.active = jaw
    bpy.ops.object.modifier_apply(modifier="Bool")
    bpy.data.objects.remove(hole)

    # 結合
    bpy.context.view_layer.objects.active = handle_part
    jaw.select_set(True)
    handle_part.select_set(True)
    bpy.ops.object.join()
    handle_part.name = name

    finalize_model(handle_part, category="item")
    return handle_part

def create_wire_cutter(name="wire_cutter"):
    """ペンチ（簡素化）"""
    clear_scene()

    objects = []

    # 左右のハンドル
    for side in [-1, 1]:
        handle = create_chamfered_cube(
            size=(0.015, 0.1, 0.008),
            location=(side * 0.015, -0.035, 0),
            name=f"PlierHandle_{side}"
        )
        apply_preset_material(handle, "iron")
        objects.append(handle)

        # 先端（刃）
        jaw = create_chamfered_cube(
            size=(0.012, 0.03, 0.008),
            location=(side * 0.018, 0.03, 0),
            name=f"PlierJaw_{side}"
        )
        apply_preset_material(jaw, "iron")
        objects.append(jaw)

    # ピボット（中央）
    pivot = create_octagonal_prism(
        radius=0.008,
        height=0.04,
        location=(0, 0, 0),
        name="PlierPivot"
    )
    pivot.rotation_euler.y = pi / 2
    apply_preset_material(pivot, "dark_steel")
    objects.append(pivot)

    # 結合
    bpy.context.view_layer.objects.active = objects[0]
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()
    objects[0].name = name

    finalize_model(objects[0], category="item")
    return objects[0]

# =============================================================================
# エクスポート
# =============================================================================

def export_all():
    """全ツールをエクスポート"""
    output_dir = os.path.join(script_dir, "..", "..", "assets", "models", "items")
    os.makedirs(output_dir, exist_ok=True)

    tools = [
        # Pickaxes
        ("wooden_pickaxe", lambda: create_pickaxe("wooden_pickaxe", "wood")),
        ("stone_pickaxe", lambda: create_pickaxe("stone_pickaxe", "stone")),
        ("iron_pickaxe", lambda: create_pickaxe("iron_pickaxe", "iron")),
        ("bronze_pickaxe", lambda: create_pickaxe("bronze_pickaxe", "brass")),
        ("steel_pickaxe", lambda: create_pickaxe("steel_pickaxe", "dark_steel")),

        # Axes
        ("wooden_axe", lambda: create_axe("wooden_axe", "wood")),
        ("stone_axe", lambda: create_axe("stone_axe", "stone")),
        ("iron_axe", lambda: create_axe("iron_axe", "iron")),

        # Shovels
        ("wooden_shovel", lambda: create_shovel("wooden_shovel", "wood")),
        ("iron_shovel", lambda: create_shovel("iron_shovel", "iron")),

        # Industrial tools
        ("hammer", create_hammer),
        ("wrench", create_wrench),
        ("wire_cutter", create_wire_cutter),
    ]

    for tool_name, create_func in tools:
        print(f"\n=== Creating {tool_name} ===")
        create_func()

        filepath = os.path.join(output_dir, f"{tool_name}.gltf")
        export_gltf(filepath, export_animations=False)
        print(f"Exported: {filepath}")

# =============================================================================
# 実行
# =============================================================================

if __name__ == "__main__":
    export_all()
    print("\n=== All tools exported successfully ===")
