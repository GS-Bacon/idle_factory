"""Debug crusher model generation"""

import bpy
from mathutils import Vector, Matrix
from math import pi
import os

# _base.py をロード
exec(open("tools/blender_scripts/_base.py").read())

def create_crusher_debug():
    """粉砕機 - デバッグ版"""
    clear_scene()

    print(f"After clear_scene: {len(bpy.data.objects)} objects")

    # 本体
    body = create_chamfered_cube((0.9, 0.9, 0.8), chamfer=0.05,
                                 location=(0, 0, 0), name="Crusher_Body")
    apply_preset_material(body, "iron")
    print(f"After body: {len(bpy.data.objects)} objects")

    # 投入口（ホッパー型）
    hopper = create_trapezoid(top_width=0.4, bottom_width=0.5, height=0.15, depth=0.4,
                             location=(0, 0, 0.55), name="Crusher_Hopper")
    apply_preset_material(hopper, "brass")
    print(f"After hopper: {len(bpy.data.objects)} objects")

    # 内部ギア（見える部分）
    gear = create_gear(radius=0.3, thickness=0.1, teeth=8, hole_radius=0.05,
                      location=(0, 0.35, 0.3), name="Crusher_Gear")
    gear.rotation_euler.x = pi / 2
    bpy.ops.object.select_all(action='DESELECT')
    gear.select_set(True)
    bpy.context.view_layer.objects.active = gear
    bpy.ops.object.transform_apply(rotation=True)
    apply_preset_material(gear, "copper")
    print(f"After gear: {len(bpy.data.objects)} objects")

    # 補強リブ（4辺）
    for x_offset in [-0.4, 0.4]:
        rib = create_chamfered_cube((0.06, 0.9, 0.06), chamfer=0.01,
                                   location=(x_offset, 0, 0.4), name=f"Rib_X_{x_offset}")
        apply_preset_material(rib, "dark_steel")
    for y_offset in [-0.4, 0.4]:
        rib = create_chamfered_cube((0.9, 0.06, 0.06), chamfer=0.01,
                                   location=(0, y_offset, 0.4), name=f"Rib_Y_{y_offset}")
        apply_preset_material(rib, "dark_steel")
    print(f"After ribs: {len(bpy.data.objects)} objects")

    # ボルト
    bolt_positions = [(0.4, 0.4, 0.7), (-0.4, 0.4, 0.7),
                     (0.4, -0.4, 0.7), (-0.4, -0.4, 0.7)]
    for pos in bolt_positions:
        bolt = create_bolt(size=0.04, length=0.05, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "iron")
    print(f"After bolts: {len(bpy.data.objects)} objects")

    # 全オブジェクト一覧
    print("\nAll objects in scene:")
    for obj in bpy.data.objects:
        print(f"  - {obj.name} (type: {obj.type})")

    # 結合 - メッシュオブジェクトのみを選択
    bpy.ops.object.select_all(action='DESELECT')
    mesh_objects = [obj for obj in bpy.data.objects if obj.type == 'MESH']
    print(f"\nMesh objects to join: {len(mesh_objects)}")

    if mesh_objects:
        bpy.context.view_layer.objects.active = mesh_objects[0]
        for obj in mesh_objects:
            obj.select_set(True)
        bpy.ops.object.join()
        result = bpy.context.active_object
        result.name = "Crusher"
        print(f"After join: {len(bpy.data.objects)} objects")

        finalize_model(result, "machine")
        return result
    return None

# 実行
if __name__ == "__main__":
    script_dir = os.path.dirname(os.path.abspath(__file__))
    output_dir = os.path.join(script_dir, "..", "..", "assets", "models", "machines")
    os.makedirs(output_dir, exist_ok=True)

    print("=== Creating crusher (debug) ===")
    model = create_crusher_debug()
    if model:
        filepath = os.path.join(output_dir, "crusher.gltf")
        export_gltf(filepath, export_animations=False)
        print(f"Exported: {filepath}")
