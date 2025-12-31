"""
Conveyor Belt - Splitter (スプリッターコンベア)
前・左・右の3方向に出力するY字型
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
mat_splitter = create_mat("splitter", (0.2, 0.5, 0.8), metallic=0.3, roughness=0.5)  # 青いアクセント

# === パーツ作成 ===
parts = []

# 中央ハブ（入力側）
center_belt = create_box(
    (BELT_WIDTH * 0.9, BELT_WIDTH * 0.9, BELT_HEIGHT * 0.6),
    (0, BELT_LENGTH * 0.2, BELT_HEIGHT * 0.3),
    "CenterBelt"
)
center_belt.data.materials.append(mat_belt)
parts.append(center_belt)

# 前方出力ベルト（-Y方向）
front_belt = create_box(
    (BELT_WIDTH * 0.9, BELT_LENGTH * 0.4, BELT_HEIGHT * 0.6),
    (0, -BELT_LENGTH * 0.3, BELT_HEIGHT * 0.3),
    "FrontBelt"
)
front_belt.data.materials.append(mat_belt)
parts.append(front_belt)

# 左出力ベルト（+X方向）
left_belt = create_box(
    (BELT_LENGTH * 0.35, BELT_WIDTH * 0.9, BELT_HEIGHT * 0.6),
    (BELT_LENGTH * 0.325, BELT_LENGTH * 0.2, BELT_HEIGHT * 0.3),
    "LeftBelt"
)
left_belt.data.materials.append(mat_belt)
parts.append(left_belt)

# 右出力ベルト（-X方向）
right_belt = create_box(
    (BELT_LENGTH * 0.35, BELT_WIDTH * 0.9, BELT_HEIGHT * 0.6),
    (-BELT_LENGTH * 0.325, BELT_LENGTH * 0.2, BELT_HEIGHT * 0.3),
    "RightBelt"
)
right_belt.data.materials.append(mat_belt)
parts.append(right_belt)

# 入力側フレーム（後ろ）
frame_width = (BLOCK_SIZE - BELT_WIDTH) / 2 * 0.8
frame_back = create_box(
    (BELT_WIDTH + frame_width * 2, frame_width, BELT_HEIGHT),
    (0, BELT_LENGTH/2 - frame_width/2, BELT_HEIGHT/2),
    "FrameBack"
)
frame_back.data.materials.append(mat_frame)
parts.append(frame_back)

# スプリッター中央マーカー（青い円柱）
bpy.ops.mesh.primitive_cylinder_add(
    vertices=8,
    radius=BELT_WIDTH * 0.2,
    depth=BELT_HEIGHT * 0.8,
    location=(0, BELT_LENGTH * 0.2, BELT_HEIGHT * 0.4)
)
splitter_marker = bpy.context.active_object
splitter_marker.name = "SplitterMarker"
splitter_marker.data.materials.append(mat_splitter)
parts.append(splitter_marker)

# ローラー（各出力）
roller_radius = BELT_HEIGHT * 0.35

# 前方ローラー
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

# 左ローラー
bpy.ops.mesh.primitive_cylinder_add(
    vertices=8,
    radius=roller_radius,
    depth=BELT_WIDTH * 0.85,
    location=(BELT_LENGTH/2 - roller_radius, BELT_LENGTH * 0.2, roller_radius),
    rotation=(0, 1.5708, 0)
)
roller_left = bpy.context.active_object
roller_left.name = "RollerLeft"
roller_left.data.materials.append(mat_frame)
parts.append(roller_left)

# 右ローラー
bpy.ops.mesh.primitive_cylinder_add(
    vertices=8,
    radius=roller_radius,
    depth=BELT_WIDTH * 0.85,
    location=(-BELT_LENGTH/2 + roller_radius, BELT_LENGTH * 0.2, roller_radius),
    rotation=(0, 1.5708, 0)
)
roller_right = bpy.context.active_object
roller_right.name = "RollerRight"
roller_right.data.materials.append(mat_frame)
parts.append(roller_right)

# 入力ローラー
bpy.ops.mesh.primitive_cylinder_add(
    vertices=8,
    radius=roller_radius,
    depth=BELT_WIDTH * 0.85,
    location=(0, BELT_LENGTH/2 - roller_radius, roller_radius),
    rotation=(1.5708, 0, 0)
)
roller_input = bpy.context.active_object
roller_input.name = "RollerInput"
roller_input.data.materials.append(mat_frame)
parts.append(roller_input)

# 方向矢印（3方向）
for angle, pos, name in [
    (0, (0, -0.3, BELT_HEIGHT * 0.62), "ArrowFront"),
    (1.5708, (0.3, 0.2, BELT_HEIGHT * 0.62), "ArrowLeft"),
    (-1.5708, (-0.3, 0.2, BELT_HEIGHT * 0.62), "ArrowRight"),
]:
    bpy.ops.mesh.primitive_cube_add(size=1, location=pos)
    arrow = bpy.context.active_object
    arrow.name = name
    arrow.scale = Vector((BELT_WIDTH * 0.12, BELT_LENGTH * 0.15, 0.02))
    arrow.rotation_euler.z = angle
    bpy.ops.object.transform_apply(scale=True, rotation=True)
    arrow.data.materials.append(mat_arrow)
    parts.append(arrow)

# === 結合 ===
bpy.ops.object.select_all(action='DESELECT')
for obj in parts:
    obj.select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()
result = bpy.context.active_object
result.name = "ConveyorSplitter"

bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
result.location.z = 0
bpy.ops.object.shade_flat()

# === エクスポート ===
output_dir = "/home/bacon/idle_factory/assets/models/machines/conveyor"
os.makedirs(output_dir, exist_ok=True)
bpy.ops.export_scene.gltf(
    filepath=os.path.join(output_dir, "splitter.glb"),
    export_format='GLB',
    use_selection=True,
    export_materials='EXPORT',
    export_yup=True,
)
print(f"Exported: {output_dir}/splitter.glb")
