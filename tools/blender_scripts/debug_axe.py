"""
Debug script for axe model
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
    apply_preset_material,
    finalize_model,
    export_gltf,
)

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
    blade.scale.y = 1.2
    blade.location.y = 0.035
    bpy.context.view_layer.objects.active = blade
    bpy.ops.object.transform_apply(scale=True)
    objects.append(blade)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = mount
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    apply_preset_material(result, material_preset)
    return result

def create_axe(name, material_preset):
    """斧生成 - デバッグ版"""
    clear_scene()

    # ハンドル（縦向き、Z軸に沿って立つ）
    handle = create_handle(length=0.18, radius=0.01)
    # ハンドルは原点を中心に0.18の高さ、つまり z=-0.09 から z=0.09 の範囲

    print(f"Handle location: {handle.location}")
    print(f"Handle bounds: {[v[:] for v in handle.bound_box]}")

    # ヘッド（ハンドル上部に配置）
    head = create_axe_head(material_preset)
    # 斧頭をハンドル上端(z=0.09)に配置
    head.location.z = 0.09

    print(f"Head location before rotation: {head.location}")

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

# 実行
if __name__ == "__main__":
    output_dir = os.path.join(script_dir, "..", "..", "assets", "models", "items")

    create_axe("iron_axe_debug", "iron")
    filepath = os.path.join(output_dir, "iron_axe_debug.gltf")
    export_gltf(filepath, export_animations=False)
    print(f"Exported: {filepath}")
