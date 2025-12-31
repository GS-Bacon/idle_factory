"""
Conveyor Belt - Straight (ストレートコンベア)
ゲーム内サイズ: 1x1ブロック、幅0.6、高さ0.2
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
    """シンプルなボックスを作成"""
    bpy.ops.mesh.primitive_cube_add(size=1, location=location)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = Vector(size)
    bpy.ops.object.transform_apply(scale=True)
    return obj

def create_arrow(size, location, name):
    """矢印形状を作成（進行方向を示す）"""
    verts = [
        # 矢印本体（長方形部分）
        (-size[0]*0.3, -size[1]*0.5, 0),
        (size[0]*0.3, -size[1]*0.5, 0),
        (size[0]*0.3, size[1]*0.2, 0),
        (-size[0]*0.3, size[1]*0.2, 0),
        # 矢印の先端（三角形）
        (-size[0]*0.5, size[1]*0.2, 0),
        (size[0]*0.5, size[1]*0.2, 0),
        (0, size[1]*0.5, 0),
    ]
    # 厚みを持たせる
    verts_top = [(v[0], v[1], size[2]*0.5) for v in verts]
    verts_bottom = [(v[0], v[1], -size[2]*0.5) for v in verts]
    all_verts = verts_bottom + verts_top

    faces = [
        # 下面
        (0, 1, 2, 3),
        (4, 5, 6),
        (3, 4, 6),
        (3, 6, 5, 2),
        # 上面
        (7+3, 7+2, 7+1, 7+0),
        (7+6, 7+5, 7+4),
        (7+6, 7+4, 7+3),
        (7+2, 7+5, 7+6, 7+3),
        # 側面
        (0, 7, 8, 1),
        (1, 8, 9, 2),
        (2, 9, 12, 5),
        (5, 12, 13, 6),
        (6, 13, 11, 4),
        (4, 11, 10, 3),
        (3, 10, 7, 0),
    ]

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(all_verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector(location)
    bpy.context.collection.objects.link(obj)
    return obj

# === シーンクリア ===
clear_scene()

# === マテリアル作成 ===
mat_belt = create_mat("belt", (0.25, 0.25, 0.25), metallic=0.2, roughness=0.7)  # ダークグレー
mat_frame = create_mat("frame", (0.35, 0.35, 0.38), metallic=0.8, roughness=0.5)  # メタリックグレー
mat_arrow = create_mat("arrow", (0.9, 0.85, 0.2), metallic=0.0, roughness=0.6)  # 黄色

# === パーツ作成 ===
parts = []

# ベルト本体（中央）
belt = create_box(
    (BELT_WIDTH * 0.9, BELT_LENGTH, BELT_HEIGHT * 0.6),
    (0, 0, BELT_HEIGHT * 0.3),
    "Belt"
)
belt.data.materials.append(mat_belt)
parts.append(belt)

# フレーム（左右のレール）
frame_width = (BLOCK_SIZE - BELT_WIDTH) / 2 * 0.8
for side in [-1, 1]:
    x_pos = side * (BELT_WIDTH / 2 + frame_width / 2)
    frame = create_box(
        (frame_width, BELT_LENGTH, BELT_HEIGHT),
        (x_pos, 0, BELT_HEIGHT / 2),
        f"Frame_{['L', 'R'][(side+1)//2]}"
    )
    frame.data.materials.append(mat_frame)
    parts.append(frame)

# ローラー（前後）
roller_radius = BELT_HEIGHT * 0.35
for y_pos in [-BELT_LENGTH/2 + roller_radius, BELT_LENGTH/2 - roller_radius]:
    bpy.ops.mesh.primitive_cylinder_add(
        vertices=8,  # ローポリ
        radius=roller_radius,
        depth=BELT_WIDTH * 0.85,
        location=(0, y_pos, roller_radius),
        rotation=(1.5708, 0, 0)  # X軸周りに90度回転
    )
    roller = bpy.context.active_object
    roller.name = f"Roller_{['Front', 'Back'][int(y_pos > 0)]}"
    roller.data.materials.append(mat_frame)
    parts.append(roller)

# 方向矢印（ベルト上）
arrow = create_arrow(
    (BELT_WIDTH * 0.5, BELT_LENGTH * 0.4, 0.02),
    (0, 0, BELT_HEIGHT * 0.62),
    "Arrow"
)
arrow.data.materials.append(mat_arrow)
parts.append(arrow)

# === 結合 ===
bpy.ops.object.select_all(action='DESELECT')
for obj in parts:
    obj.select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()
result = bpy.context.active_object
result.name = "ConveyorStraight"

# 原点を底面中央に
bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
result.location.z = 0

# フラットシェーディング
bpy.ops.object.shade_flat()

# === エクスポート ===
output_dir = "/home/bacon/idle_factory/assets/models/machines/conveyor"
os.makedirs(output_dir, exist_ok=True)
bpy.ops.export_scene.gltf(
    filepath=os.path.join(output_dir, "straight.glb"),
    export_format='GLB',
    use_selection=True,
    export_materials='EXPORT',
    export_yup=True,
)
print(f"Exported: {output_dir}/straight.glb")
