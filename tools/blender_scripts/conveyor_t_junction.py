"""
Conveyor Belt - T-Junction (T字型コンベア)
左右両側から入力を受け取る
"""
import bpy
from mathutils import Vector
import os

# === 定数 ===
BLOCK_SIZE = 1.0
BELT_WIDTH = 0.6
BELT_HEIGHT = 0.2
BELT_LENGTH = 1.0

# === ユーティリティ関数 ===
def clear_scene():
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()
    for mat in bpy.data.materials:
        bpy.data.materials.remove(mat)

def create_mat(name, color, metallic=0.0, roughness=0.8):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    for node in mat.node_tree.nodes:
        if node.type == 'BSDF_PRINCIPLED':
            node.inputs["Base Color"].default_value = (*color, 1)
            node.inputs["Metallic"].default_value = metallic
            node.inputs["Roughness"].default_value = roughness
            break
    return mat

def create_box(size, location, name):
    bpy.ops.mesh.primitive_cube_add(size=1, location=location)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = Vector(size)
    bpy.ops.object.transform_apply(scale=True)
    return obj

# === シーンクリア ===
clear_scene()

# === マテリアル作成 ===
mat_belt = create_mat("belt", (0.25, 0.25, 0.25), metallic=0.2, roughness=0.7)
mat_frame = create_mat("frame", (0.35, 0.35, 0.38), metallic=0.8, roughness=0.5)
mat_arrow = create_mat("arrow", (0.9, 0.85, 0.2), metallic=0.0, roughness=0.6)

# === パーツ作成 ===
parts = []

# メインベルト（前方向）
main_belt = create_box(
    (BELT_WIDTH * 0.9, BELT_LENGTH * 0.55, BELT_HEIGHT * 0.6),
    (0, -BELT_LENGTH * 0.225, BELT_HEIGHT * 0.3),
    "MainBelt"
)
main_belt.data.materials.append(mat_belt)
parts.append(main_belt)

# 左サイドベルト（+X側）
side_belt_l = create_box(
    (BELT_LENGTH * 0.35, BELT_WIDTH * 0.9, BELT_HEIGHT * 0.6),
    (BELT_LENGTH * 0.325, BELT_LENGTH * 0.15, BELT_HEIGHT * 0.3),
    "SideBeltL"
)
side_belt_l.data.materials.append(mat_belt)
parts.append(side_belt_l)

# 右サイドベルト（-X側）
side_belt_r = create_box(
    (BELT_LENGTH * 0.35, BELT_WIDTH * 0.9, BELT_HEIGHT * 0.6),
    (-BELT_LENGTH * 0.325, BELT_LENGTH * 0.15, BELT_HEIGHT * 0.3),
    "SideBeltR"
)
side_belt_r.data.materials.append(mat_belt)
parts.append(side_belt_r)

# 中央接続部（ベルト）
center_belt = create_box(
    (BELT_WIDTH * 0.9, BELT_WIDTH * 0.9, BELT_HEIGHT * 0.6),
    (0, BELT_LENGTH * 0.15, BELT_HEIGHT * 0.3),
    "CenterBelt"
)
center_belt.data.materials.append(mat_belt)
parts.append(center_belt)

# フレーム（外周）
frame_width = (BLOCK_SIZE - BELT_WIDTH) / 2 * 0.8

# 後ろフレーム（両サイド用）
frame_back = create_box(
    (BLOCK_SIZE, frame_width, BELT_HEIGHT),
    (0, BELT_LENGTH * 0.15 + BELT_WIDTH/2 + frame_width/2, BELT_HEIGHT/2),
    "FrameBack"
)
frame_back.data.materials.append(mat_frame)
parts.append(frame_back)

# ローラー（前）
roller_radius = BELT_HEIGHT * 0.35
bpy.ops.mesh.primitive_cylinder_add(
    vertices=8,
    radius=roller_radius,
    depth=BELT_WIDTH * 0.85,
    location=(0, -BELT_LENGTH/2 + roller_radius, roller_radius),
    rotation=(1.5708, 0, 0)
)
roller_front = bpy.context.active_object
roller_front.name = "RollerFront"
roller_front.data.materials.append(mat_frame)
parts.append(roller_front)

# 左サイドローラー
bpy.ops.mesh.primitive_cylinder_add(
    vertices=8,
    radius=roller_radius,
    depth=BELT_WIDTH * 0.85,
    location=(BELT_LENGTH/2 - roller_radius, BELT_LENGTH * 0.15, roller_radius),
    rotation=(0, 1.5708, 0)
)
roller_side_l = bpy.context.active_object
roller_side_l.name = "RollerSideL"
roller_side_l.data.materials.append(mat_frame)
parts.append(roller_side_l)

# 右サイドローラー
bpy.ops.mesh.primitive_cylinder_add(
    vertices=8,
    radius=roller_radius,
    depth=BELT_WIDTH * 0.85,
    location=(-BELT_LENGTH/2 + roller_radius, BELT_LENGTH * 0.15, roller_radius),
    rotation=(0, 1.5708, 0)
)
roller_side_r = bpy.context.active_object
roller_side_r.name = "RollerSideR"
roller_side_r.data.materials.append(mat_frame)
parts.append(roller_side_r)

# 方向矢印（中央前方）
bpy.ops.mesh.primitive_cube_add(size=1, location=(0, -0.15, BELT_HEIGHT * 0.62))
arrow = bpy.context.active_object
arrow.name = "Arrow"
arrow.scale = Vector((BELT_WIDTH * 0.15, BELT_LENGTH * 0.25, 0.02))
bpy.ops.object.transform_apply(scale=True)
arrow.data.materials.append(mat_arrow)
parts.append(arrow)

# === 結合 ===
bpy.ops.object.select_all(action='DESELECT')
for obj in parts:
    obj.select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()
result = bpy.context.active_object
result.name = "ConveyorTJunction"

bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
result.location.z = 0
bpy.ops.object.shade_flat()

# === エクスポート ===
output_dir = "/home/bacon/idle_factory/assets/models/machines/conveyor"
os.makedirs(output_dir, exist_ok=True)
bpy.ops.export_scene.gltf(
    filepath=os.path.join(output_dir, "t_junction.glb"),
    export_format='GLB',
    use_selection=True,
    export_materials='EXPORT',
    export_yup=True,
)
print(f"Exported: {output_dir}/t_junction.glb")
