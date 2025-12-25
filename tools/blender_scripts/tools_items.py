"""
Tool Items - Blender生成スクリプト
pickaxes, axes, shovels, industrial tools

カテゴリ: item (handheld_item: 0.3x0.3x0.3)

改善版: Minecraft/Unturned風のディテール追加
- リベット、溶接線、面取りエッジ
- より立体的なヘッド形状
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
    create_trapezoid,
    apply_preset_material,
    create_material,
    apply_material,
    finalize_model,
    export_gltf,
    snap,
    snap_vec,
    create_rivet,
    create_weld_line,
    create_groove,
    GRID_UNIT,
)

# =============================================================================
# 共通パーツ（改善版）
# =============================================================================

def create_handle(length=0.15, radius=0.012, material="wood"):
    """木製八角柱ハンドル（グリップ溝付き）"""
    handle = create_octagonal_prism(
        radius=radius,
        height=length,
        location=(0, 0, 0),
        name="Handle"
    )
    apply_preset_material(handle, material)

    objects = [handle]

    # グリップ部分の溝（3本）
    groove_mat = create_material(
        "grip_groove",
        color=(0.45, 0.33, 0.06, 1),  # 少し暗い木目
        metallic=0.0,
        roughness=0.9
    )

    for i in range(3):
        z_pos = -length * 0.3 + i * 0.02
        groove = create_octagonal_prism(
            radius=radius * 1.1,
            height=0.004,
            location=(0, 0, z_pos),
            name=f"Grip_{i}"
        )
        apply_material(groove, groove_mat)
        objects.append(groove)

    # 柄の端のキャップ
    cap = create_octagonal_prism(
        radius=radius * 1.15,
        height=0.008,
        location=(0, 0, -length / 2 + 0.004),
        name="HandleCap"
    )
    apply_preset_material(cap, material)
    objects.append(cap)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = handle
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    return bpy.context.active_object

def create_pickaxe_head(material_preset="stone"):
    """ピッケル先端（改善版：より立体的、リベット付き）"""
    objects = []

    # 中央ブロック（柄との接続部、より厚みを持たせる）
    center = create_chamfered_cube(
        size=(0.10, 0.035, 0.035),
        chamfer=0.004,
        location=(0, 0, 0),
        name="PickHead_center"
    )
    objects.append(center)

    # 柄を通す穴の表現（リング）
    mount_ring = create_octagonal_prism(
        radius=0.018,
        height=0.04,
        location=(0, 0, -0.01),
        name="PickHead_mount"
    )
    objects.append(mount_ring)

    # 左右の尖った先端（より詳細な形状）
    for side in [-1, 1]:
        # ベース部分
        base = create_chamfered_cube(
            size=(0.035, 0.03, 0.028),
            chamfer=0.003,
            location=(side * 0.05, 0, 0),
            name=f"PickHead_base_{side}"
        )
        objects.append(base)

        # 先端部分（尖り）
        tip = create_chamfered_cube(
            size=(0.025, 0.022, 0.020),
            chamfer=0.002,
            location=(side * 0.075, 0, 0),
            name=f"PickHead_tip_{side}"
        )
        # スケールで尖らせる
        tip.scale.x = 0.5
        tip.scale.y = 0.7
        bpy.context.view_layer.objects.active = tip
        bpy.ops.object.transform_apply(scale=True)
        objects.append(tip)

    # リベット（両側）
    rivet_mat = create_material(
        "rivet_metal",
        color=(0.35, 0.35, 0.35, 1),
        metallic=1.0,
        roughness=0.5
    )
    for side in [-1, 1]:
        rivet = create_rivet(
            radius=0.006,
            height=0.004,
            location=(side * 0.025, 0.018, 0),
            name=f"Rivet_{side}"
        )
        apply_material(rivet, rivet_mat)
        objects.append(rivet)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = center
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    apply_preset_material(result, material_preset)
    return result

def create_axe_head(material_preset="stone"):
    """斧頭（改善版：より詳細な形状）"""
    objects = []

    # 接続部（柄に接する部分）
    mount = create_chamfered_cube(
        size=(0.035, 0.045, 0.035),
        chamfer=0.004,
        location=(0, 0, 0),
        name="AxeHead_mount"
    )
    objects.append(mount)

    # 柄を通すリング表現
    mount_ring = create_octagonal_prism(
        radius=0.015,
        height=0.04,
        location=(0, 0, -0.01),
        name="AxeHead_ring"
    )
    objects.append(mount_ring)

    # ブレードベース
    blade_base = create_chamfered_cube(
        size=(0.028, 0.055, 0.035),
        chamfer=0.003,
        location=(0, 0.035, 0),
        name="AxeHead_blade_base"
    )
    objects.append(blade_base)

    # ブレード刃先（薄くなる）
    blade_edge = create_chamfered_cube(
        size=(0.018, 0.065, 0.04),
        chamfer=0.002,
        location=(0, 0.065, 0),
        name="AxeHead_blade_edge"
    )
    # 刃先を薄く
    blade_edge.scale.x = 0.4
    bpy.context.view_layer.objects.active = blade_edge
    bpy.ops.object.transform_apply(scale=True)
    objects.append(blade_edge)

    # 背面の突起（ハンマー部分）
    back = create_chamfered_cube(
        size=(0.025, 0.02, 0.025),
        chamfer=0.003,
        location=(0, -0.025, 0),
        name="AxeHead_back"
    )
    objects.append(back)

    # リベット
    rivet_mat = create_material(
        "rivet_metal",
        color=(0.35, 0.35, 0.35, 1),
        metallic=1.0,
        roughness=0.5
    )
    for z_side in [-1, 1]:
        rivet = create_rivet(
            radius=0.005,
            height=0.003,
            location=(0.018, 0.02, z_side * 0.012),
            name=f"Rivet_{z_side}"
        )
        apply_material(rivet, rivet_mat)
        objects.append(rivet)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = mount
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    apply_preset_material(result, material_preset)
    return result

def create_shovel_head(material_preset="iron"):
    """シャベル先端（改善版：曲面風、補強リブ付き）"""
    objects = []

    # ブレード本体（やや湾曲を表現）
    blade = create_chamfered_cube(
        size=(0.055, 0.07, 0.012),
        chamfer=0.003,
        location=(0, 0.035, 0),
        name="ShovelHead_blade"
    )
    objects.append(blade)

    # 先端（尖らせる）
    tip = create_trapezoid(
        top_width=0.04,
        bottom_width=0.055,
        height=0.02,
        depth=0.01,
        location=(0, 0.08, 0),
        name="ShovelHead_tip"
    )
    tip.rotation_euler.x = pi / 2
    bpy.context.view_layer.objects.active = tip
    bpy.ops.object.transform_apply(rotation=True)
    objects.append(tip)

    # ソケット（柄との接続部）
    socket = create_octagonal_prism(
        radius=0.015,
        height=0.03,
        location=(0, -0.01, 0.008),
        name="ShovelHead_socket"
    )
    socket.rotation_euler.x = pi / 2
    bpy.context.view_layer.objects.active = socket
    bpy.ops.object.transform_apply(rotation=True)
    objects.append(socket)

    # 補強リブ（中央の縦線）
    rib = create_chamfered_cube(
        size=(0.008, 0.06, 0.018),
        chamfer=0.002,
        location=(0, 0.03, 0.003),
        name="ShovelHead_rib"
    )
    objects.append(rib)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = blade
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    apply_preset_material(result, material_preset)
    return result

# =============================================================================
# ピッケル
# =============================================================================

def create_pickaxe(name, material_preset):
    """ピッケル生成（改善版）"""
    clear_scene()

    # ハンドル（縦向き）
    handle = create_handle(length=0.18, radius=0.012)

    # ヘッド（ハンドル上部に配置）
    head = create_pickaxe_head(material_preset)
    head.location.z = 0.09  # ハンドル上端
    head.rotation_euler.z = pi / 2  # Z軸回転でT字に

    # 回転を適用
    bpy.ops.object.select_all(action='DESELECT')
    head.select_set(True)
    bpy.context.view_layer.objects.active = head
    bpy.ops.object.transform_apply(rotation=True)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = handle
    head.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name

    finalize_model(result, category="item")
    return result

# =============================================================================
# 斧
# =============================================================================

def create_axe(name, material_preset):
    """斧生成（改善版）"""
    clear_scene()

    # ハンドル（縦向き、Z軸に沿う）
    handle = create_handle(length=0.18, radius=0.011)

    # ヘッド（ハンドル上端 z=0.09 に配置）
    head = create_axe_head(material_preset)
    head.location.z = 0.09

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = handle
    head.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name

    finalize_model(result, category="item")
    return result

# =============================================================================
# シャベル
# =============================================================================

def create_shovel(name, material_preset):
    """シャベル生成（改善版）"""
    clear_scene()

    # ハンドル（縦向き、Z軸に沿う）
    handle = create_handle(length=0.18, radius=0.01)

    # ブレード（ハンドル上端 z=0.09 に配置）
    blade = create_shovel_head(material_preset)
    blade.location.z = 0.09
    blade.rotation_euler.x = -pi / 6  # 少し傾ける

    # 回転を適用
    bpy.ops.object.select_all(action='DESELECT')
    blade.select_set(True)
    bpy.context.view_layer.objects.active = blade
    bpy.ops.object.transform_apply(rotation=True)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = handle
    blade.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name

    finalize_model(result, category="item")
    return result

# =============================================================================
# 工業ツール（改善版）
# =============================================================================

def create_hammer(name="hammer"):
    """ハンマー（改善版：より詳細な形状）"""
    clear_scene()

    objects = []

    # ハンドル（縦向き、Z軸に沿う）
    handle = create_handle(length=0.15, radius=0.009)
    objects.append(handle)

    # ヘッド本体
    head = create_chamfered_cube(
        size=(0.028, 0.028, 0.055),
        chamfer=0.003,
        location=(0, 0, 0.075),
        name="HammerHead"
    )
    apply_preset_material(head, "iron")
    objects.append(head)

    # 打撃面（両端）
    for z_off in [-0.03, 0.03]:
        face = create_chamfered_cube(
            size=(0.030, 0.030, 0.008),
            chamfer=0.002,
            location=(0, 0, 0.075 + z_off),
            name=f"HammerFace_{z_off}"
        )
        apply_preset_material(face, "iron")
        objects.append(face)

    # 溝（グリップ表現は handle に含まれる）

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = handle
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name

    finalize_model(result, category="item")
    return result

def create_wrench(name="wrench"):
    """スパナ（改善版：より詳細）"""
    clear_scene()

    objects = []

    # ハンドル部（Z軸方向に長い）
    handle_part = create_chamfered_cube(
        size=(0.018, 0.012, 0.12),
        chamfer=0.002,
        location=(0, 0, -0.04),
        name="WrenchHandle"
    )
    apply_preset_material(handle_part, "iron")
    objects.append(handle_part)

    # ハンドル溝（グリップ）
    groove_mat = create_material(
        "wrench_groove",
        color=(0.22, 0.22, 0.22, 1),
        metallic=1.0,
        roughness=0.7
    )
    for i in range(4):
        z_pos = -0.08 + i * 0.015
        groove = create_chamfered_cube(
            size=(0.020, 0.014, 0.004),
            chamfer=0.001,
            location=(0, 0, z_pos),
            name=f"Groove_{i}"
        )
        apply_material(groove, groove_mat)
        objects.append(groove)

    # 開口部（六角形を使用、上端に配置）
    jaw = create_hexagon(
        radius=0.028,
        depth=0.012,
        location=(0, 0, 0.04),
        name="WrenchJaw"
    )
    jaw.rotation_euler.x = pi / 2
    bpy.ops.object.select_all(action='DESELECT')
    jaw.select_set(True)
    bpy.context.view_layer.objects.active = jaw
    bpy.ops.object.transform_apply(rotation=True)
    apply_preset_material(jaw, "iron")
    objects.append(jaw)

    # 中央の穴（六角形で小さく）
    hole = create_hexagon(
        radius=0.016,
        depth=0.014,
        location=(0, 0, 0.04),
        name="WrenchHole"
    )
    hole.rotation_euler.x = pi / 2
    bpy.ops.object.select_all(action='DESELECT')
    hole.select_set(True)
    bpy.context.view_layer.objects.active = hole
    bpy.ops.object.transform_apply(rotation=True)

    # Boolean差分
    mod = jaw.modifiers.new("Bool", 'BOOLEAN')
    mod.operation = 'DIFFERENCE'
    mod.object = hole
    bpy.context.view_layer.objects.active = jaw
    bpy.ops.object.modifier_apply(modifier="Bool")
    bpy.data.objects.remove(hole)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = handle_part
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name

    finalize_model(result, category="item")
    return result

def create_wire_cutter(name="wire_cutter"):
    """ペンチ（改善版：より詳細）"""
    clear_scene()

    objects = []

    # 左右のハンドル（Z軸方向に長い）
    handle_mat = create_material(
        "plier_handle",
        color=(0.8, 0.15, 0.1, 1),  # 赤いグリップ
        metallic=0.0,
        roughness=0.6
    )

    for side in [-1, 1]:
        # グリップ部分
        grip = create_chamfered_cube(
            size=(0.018, 0.012, 0.08),
            chamfer=0.002,
            location=(side * 0.016, 0, -0.04),
            name=f"PlierGrip_{side}"
        )
        apply_material(grip, handle_mat)
        objects.append(grip)

        # 金属部分（グリップから先端へ）
        metal = create_chamfered_cube(
            size=(0.014, 0.010, 0.04),
            chamfer=0.001,
            location=(side * 0.018, 0, 0.01),
            name=f"PlierMetal_{side}"
        )
        apply_preset_material(metal, "iron")
        objects.append(metal)

        # 先端（刃）
        jaw = create_chamfered_cube(
            size=(0.010, 0.008, 0.035),
            chamfer=0.001,
            location=(side * 0.020, 0, 0.045),
            name=f"PlierJaw_{side}"
        )
        apply_preset_material(jaw, "iron")
        objects.append(jaw)

    # ピボット（中央、X軸方向に伸びる）
    pivot = create_octagonal_prism(
        radius=0.010,
        height=0.045,
        location=(0, 0, 0.01),
        name="PlierPivot"
    )
    pivot.rotation_euler.y = pi / 2
    apply_preset_material(pivot, "dark_steel")

    # 回転を適用
    bpy.ops.object.select_all(action='DESELECT')
    pivot.select_set(True)
    bpy.context.view_layer.objects.active = pivot
    bpy.ops.object.transform_apply(rotation=True)
    objects.append(pivot)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    first_obj = objects[0]
    bpy.context.view_layer.objects.active = first_obj
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name

    finalize_model(result, category="item")
    return result

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
