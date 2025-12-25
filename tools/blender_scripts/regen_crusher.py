"""Quick regenerate crusher only"""
import bpy
from mathutils import Vector, Matrix
from math import pi, cos, sin, radians
import os

script_dir = os.path.dirname(os.path.abspath(__file__))

# _base.pyをロード
exec(open(os.path.join(script_dir, "_base.py")).read())

def create_crusher():
    """粉砕機 - 箱型、上部に投入口"""
    clear_scene()

    # 本体
    body = create_chamfered_cube((0.9, 0.9, 0.8), chamfer=0.05,
                                 location=(0, 0, 0), name="Crusher_Body")
    apply_preset_material(body, "iron")

    # 投入口（ホッパー型）- 本体上面 z=0.4 に配置
    hopper = create_trapezoid(top_width=0.4, bottom_width=0.5, height=0.15, depth=0.4,
                             location=(0, 0, 0.4), name="Crusher_Hopper")
    hopper.rotation_euler.x = pi
    # 回転を適用
    bpy.ops.object.select_all(action='DESELECT')
    hopper.select_set(True)
    bpy.context.view_layer.objects.active = hopper
    bpy.ops.object.transform_apply(rotation=True)
    apply_preset_material(hopper, "brass")

    # 内部ギア（見える部分）
    gear = create_gear(radius=0.3, thickness=0.1, teeth=8, hole_radius=0.05,
                      location=(0, 0.35, 0.3), name="Crusher_Gear")
    gear.rotation_euler.x = pi / 2
    # 回転を適用
    bpy.ops.object.select_all(action='DESELECT')
    gear.select_set(True)
    bpy.context.view_layer.objects.active = gear
    bpy.ops.object.transform_apply(rotation=True)
    apply_preset_material(gear, "copper")

    # 補強リブ（4辺）
    ribs = []
    for x_offset in [-0.4, 0.4]:
        rib = create_chamfered_cube((0.06, 0.9, 0.06), chamfer=0.01,
                                   location=(x_offset, 0, 0.4), name=f"Rib_X_{x_offset}")
        apply_preset_material(rib, "dark_steel")
        ribs.append(rib)
    for y_offset in [-0.4, 0.4]:
        rib = create_chamfered_cube((0.9, 0.06, 0.06), chamfer=0.01,
                                   location=(0, y_offset, 0.4), name=f"Rib_Y_{y_offset}")
        apply_preset_material(rib, "dark_steel")
        ribs.append(rib)

    # ボルト（本体上面 z=0.4 に配置）
    bolt_positions = [(0.35, 0.35, 0.4), (-0.35, 0.35, 0.4),
                     (0.35, -0.35, 0.4), (-0.35, -0.35, 0.4)]
    bolts = []
    for pos in bolt_positions:
        bolt = create_bolt(size=0.04, length=0.05, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "iron")
        bolts.append(bolt)

    # 結合 - シーン内の全メッシュを選択
    mesh_objects = [obj for obj in bpy.context.scene.objects if obj.type == 'MESH']
    if len(mesh_objects) > 1:
        bpy.ops.object.select_all(action='DESELECT')
        for obj in mesh_objects:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = mesh_objects[0]
        bpy.ops.object.join()

    result = bpy.context.active_object
    result.name = "Crusher"

    finalize_model(result, "machine")
    return result

output_dir = os.path.join(script_dir, "..", "..", "assets", "models", "machines")
os.makedirs(output_dir, exist_ok=True)

print("=== Creating crusher ===")
create_crusher()
export_gltf(os.path.join(output_dir, "crusher.gltf"), export_animations=False)
print("Exported crusher")
