"""
Industrial Lowpoly Style - Blender共通モジュール
style-guide.json v1.0.0 準拠

使い方:
1. Blenderで新規ファイル作成
2. このスクリプトを実行（基本関数をロード）
3. モデル固有のスクリプトを実行
"""

import bpy
import bmesh
from mathutils import Vector, Matrix
from math import pi, cos, sin, radians
import os

# =============================================================================
# 定数
# =============================================================================

GRID_UNIT = 0.0625  # 1/16ブロック
CHAMFER_RATIO = 0.1
EDGE_DARKEN = 0.85

# マテリアルプリセット
MATERIALS = {
    "iron": {"color": (0.29, 0.29, 0.29, 1), "metallic": 1.0, "roughness": 0.5},
    "copper": {"color": (0.72, 0.45, 0.20, 1), "metallic": 1.0, "roughness": 0.4},
    "brass": {"color": (0.79, 0.64, 0.15, 1), "metallic": 1.0, "roughness": 0.4},
    "dark_steel": {"color": (0.18, 0.18, 0.18, 1), "metallic": 1.0, "roughness": 0.6},
    "wood": {"color": (0.55, 0.41, 0.08, 1), "metallic": 0.0, "roughness": 0.8},
    "stone": {"color": (0.41, 0.41, 0.41, 1), "metallic": 0.0, "roughness": 0.7},
}

ACCENT_COLORS = {
    "danger": (0.8, 0.2, 0.2, 1),
    "warning": (0.8, 0.67, 0.2, 1),
    "power": (0.2, 0.4, 0.8, 1),
    "active": (0.2, 0.8, 0.4, 1),
}

# =============================================================================
# ユーティリティ
# =============================================================================

def snap(value):
    """グリッドスナップ"""
    return round(value / GRID_UNIT) * GRID_UNIT

def snap_vec(v):
    """ベクトルをグリッドスナップ"""
    return Vector((snap(v.x), snap(v.y), snap(v.z)))

def clear_scene():
    """シーンをクリア"""
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()

def set_origin_bottom_center(obj):
    """原点を底面中央に設定"""
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
    bbox = [obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
    min_z = min(v.z for v in bbox)
    center_x = sum(v.x for v in bbox) / 8
    center_y = sum(v.y for v in bbox) / 8
    offset = Vector((center_x, center_y, min_z)) - obj.location
    obj.data.transform(Matrix.Translation(-offset))
    obj.location = Vector((0, 0, 0))

def set_origin_center(obj):
    """原点を中心に設定"""
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
    obj.location = Vector((0, 0, 0))

# =============================================================================
# プリミティブ生成
# =============================================================================

def create_octagon(radius=0.5, depth=0.1, location=(0, 0, 0), name="Octagon"):
    """八角形（円の代替）"""
    verts = []
    for i in range(8):
        angle = i * pi / 4 + pi / 8  # 22.5度オフセット
        verts.append((cos(angle) * radius, sin(angle) * radius, -depth / 2))
        verts.append((cos(angle) * radius, sin(angle) * radius, depth / 2))

    faces = []
    # 側面
    for i in range(8):
        j = (i + 1) % 8
        faces.append((i * 2, j * 2, j * 2 + 1, i * 2 + 1))
    # 上下面
    faces.append(tuple(i * 2 for i in range(8)))
    faces.append(tuple(i * 2 + 1 for i in reversed(range(8))))

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()

    obj = bpy.data.objects.new(name, mesh)
    obj.location = snap_vec(Vector(location))
    bpy.context.collection.objects.link(obj)
    return obj

def create_octagonal_prism(radius=0.5, height=1.0, location=(0, 0, 0), name="OctPrism"):
    """八角柱（円柱の代替）"""
    return create_octagon(radius, height, location, name)

def create_chamfered_cube(size=(1, 1, 1), chamfer=None, location=(0, 0, 0), name="ChamfCube"):
    """面取りキューブ"""
    if chamfer is None:
        chamfer = min(size) * CHAMFER_RATIO

    sx, sy, sz = [s / 2 for s in size]
    c = chamfer

    # 面取りされた頂点
    verts = [
        # 下面（Z-）
        (-sx + c, -sy, -sz), (sx - c, -sy, -sz),
        (sx, -sy + c, -sz), (sx, sy - c, -sz),
        (sx - c, sy, -sz), (-sx + c, sy, -sz),
        (-sx, sy - c, -sz), (-sx, -sy + c, -sz),
        # 上面（Z+）
        (-sx + c, -sy, sz), (sx - c, -sy, sz),
        (sx, -sy + c, sz), (sx, sy - c, sz),
        (sx - c, sy, sz), (-sx + c, sy, sz),
        (-sx, sy - c, sz), (-sx, -sy + c, sz),
    ]

    faces = [
        # 下面
        (0, 1, 2, 3, 4, 5, 6, 7),
        # 上面
        (15, 14, 13, 12, 11, 10, 9, 8),
        # 側面
        (0, 8, 9, 1), (1, 9, 10, 2), (2, 10, 11, 3), (3, 11, 12, 4),
        (4, 12, 13, 5), (5, 13, 14, 6), (6, 14, 15, 7), (7, 15, 8, 0),
    ]

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()

    obj = bpy.data.objects.new(name, mesh)
    obj.location = snap_vec(Vector(location))
    bpy.context.collection.objects.link(obj)
    return obj

def create_hexagon(radius=0.5, depth=0.1, location=(0, 0, 0), name="Hexagon"):
    """六角形（ボルト頭など）"""
    verts = []
    for i in range(6):
        angle = i * pi / 3
        verts.append((cos(angle) * radius, sin(angle) * radius, -depth / 2))
        verts.append((cos(angle) * radius, sin(angle) * radius, depth / 2))

    faces = []
    for i in range(6):
        j = (i + 1) % 6
        faces.append((i * 2, j * 2, j * 2 + 1, i * 2 + 1))
    faces.append(tuple(i * 2 for i in range(6)))
    faces.append(tuple(i * 2 + 1 for i in reversed(range(6))))

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()

    obj = bpy.data.objects.new(name, mesh)
    obj.location = snap_vec(Vector(location))
    bpy.context.collection.objects.link(obj)
    return obj

def create_trapezoid(top_width, bottom_width, height, depth, location=(0, 0, 0), name="Trapezoid"):
    """台形（ギア歯、ファンブレードなど）"""
    tw, bw, h, d = top_width / 2, bottom_width / 2, height, depth / 2

    verts = [
        (-bw, 0, -d), (bw, 0, -d), (tw, h, -d), (-tw, h, -d),
        (-bw, 0, d), (bw, 0, d), (tw, h, d), (-tw, h, d),
    ]
    faces = [
        (0, 1, 2, 3), (7, 6, 5, 4),  # 前後
        (0, 4, 5, 1), (2, 6, 7, 3),  # 上下
        (0, 3, 7, 4), (1, 5, 6, 2),  # 左右
    ]

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()

    obj = bpy.data.objects.new(name, mesh)
    obj.location = snap_vec(Vector(location))
    bpy.context.collection.objects.link(obj)
    return obj

# =============================================================================
# 機械パーツ
# =============================================================================

def create_gear(radius=0.5, thickness=0.1, teeth=8, hole_radius=0.1, location=(0, 0, 0), name="Gear"):
    """ギア（八角形ベース + 台形歯）"""
    # ベースの八角形
    base = create_octagon(radius * 0.8, thickness, location, f"{name}_base")

    # 歯を追加
    tooth_height = radius * 0.2
    tooth_width = 2 * pi * radius * 0.8 / teeth * 0.6

    objects = [base]
    for i in range(teeth):
        angle = i * 2 * pi / teeth
        x = cos(angle) * radius * 0.8
        y = sin(angle) * radius * 0.8

        tooth = create_trapezoid(
            tooth_width * 0.6, tooth_width,
            tooth_height, thickness,
            location, f"{name}_tooth_{i}"
        )
        tooth.rotation_euler.z = angle + pi / 2
        tooth.location = Vector(location) + Vector((x, y, 0))
        objects.append(tooth)

    # 結合
    bpy.context.view_layer.objects.active = base
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    return base

def create_shaft(radius=0.1, length=1.0, location=(0, 0, 0), name="Shaft"):
    """シャフト（八角柱）"""
    shaft = create_octagonal_prism(radius, length, location, name)
    shaft.rotation_euler.x = pi / 2  # Y軸方向に
    return shaft

def create_pipe(radius=0.2, length=1.0, wall=0.03, location=(0, 0, 0), name="Pipe"):
    """パイプ（八角形断面）"""
    outer = create_octagonal_prism(radius, length, location, f"{name}_outer")
    inner = create_octagonal_prism(radius - wall, length + 0.01, location, f"{name}_inner")

    # Boolean差分
    mod = outer.modifiers.new("Bool", 'BOOLEAN')
    mod.operation = 'DIFFERENCE'
    mod.object = inner
    bpy.context.view_layer.objects.active = outer
    bpy.ops.object.modifier_apply(modifier="Bool")
    bpy.data.objects.remove(inner)

    return outer

def create_bolt(size=0.0625, length=0.125, location=(0, 0, 0), name="Bolt"):
    """ボルト（六角頭）"""
    head = create_hexagon(size, size * 0.5, location, f"{name}_head")
    head.location.z += size * 0.25

    shaft_loc = (location[0], location[1], location[2] - length / 2)
    shaft = create_octagonal_prism(size * 0.4, length, shaft_loc, f"{name}_shaft")

    bpy.context.view_layer.objects.active = head
    shaft.select_set(True)
    head.select_set(True)
    bpy.ops.object.join()

    return head

def create_piston(rod_radius=0.05, rod_length=0.5, head_size=(0.2, 0.2, 0.1), location=(0, 0, 0), name="Piston"):
    """ピストン"""
    rod = create_octagonal_prism(rod_radius, rod_length, location, f"{name}_rod")
    head = create_chamfered_cube(head_size, None, (location[0], location[1], location[2] + rod_length / 2), f"{name}_head")

    bpy.context.view_layer.objects.active = rod
    head.select_set(True)
    rod.select_set(True)
    bpy.ops.object.join()

    return rod

# =============================================================================
# マテリアル
# =============================================================================

def create_material(name, preset=None, color=None, metallic=None, roughness=None):
    """PBRマテリアル作成"""
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")

    if preset and preset in MATERIALS:
        p = MATERIALS[preset]
        bsdf.inputs["Base Color"].default_value = p["color"]
        bsdf.inputs["Metallic"].default_value = p["metallic"]
        bsdf.inputs["Roughness"].default_value = p["roughness"]
    else:
        if color:
            bsdf.inputs["Base Color"].default_value = color
        if metallic is not None:
            bsdf.inputs["Metallic"].default_value = metallic
        if roughness is not None:
            bsdf.inputs["Roughness"].default_value = roughness

    return mat

def apply_material(obj, material):
    """マテリアルを適用"""
    if obj.data.materials:
        obj.data.materials[0] = material
    else:
        obj.data.materials.append(material)

def apply_preset_material(obj, preset_name):
    """プリセットマテリアルを適用"""
    mat = create_material(preset_name, preset=preset_name)
    apply_material(obj, mat)
    return mat

# =============================================================================
# 頂点カラー
# =============================================================================

def apply_edge_darkening(obj, factor=EDGE_DARKEN):
    """エッジを暗くして立体感を出す"""
    mesh = obj.data
    if not mesh.vertex_colors:
        mesh.vertex_colors.new(name="Col")

    color_layer = mesh.vertex_colors.active

    bm = bmesh.new()
    bm.from_mesh(mesh)
    bm.verts.ensure_lookup_table()

    # 各頂点の接続エッジ数でエッジ度を判定
    edge_counts = {v.index: len(v.link_edges) for v in bm.verts}
    max_edges = max(edge_counts.values()) if edge_counts else 1

    bm.to_mesh(mesh)
    bm.free()

    for poly in mesh.polygons:
        for loop_idx in poly.loop_indices:
            vert_idx = mesh.loops[loop_idx].vertex_index
            edge_factor = edge_counts.get(vert_idx, 1) / max_edges
            darkness = 1.0 - (1.0 - factor) * edge_factor
            color_layer.data[loop_idx].color = (darkness, darkness, darkness, 1.0)

# =============================================================================
# ボーン/アーマチュア
# =============================================================================

def create_armature(name="Armature"):
    """アーマチュア作成"""
    arm = bpy.data.armatures.new(name)
    arm_obj = bpy.data.objects.new(name, arm)
    bpy.context.collection.objects.link(arm_obj)
    bpy.context.view_layer.objects.active = arm_obj
    return arm_obj

def add_bone(armature_obj, name, head=(0, 0, 0), tail=(0, 0, 1), parent=None):
    """ボーン追加"""
    bpy.context.view_layer.objects.active = armature_obj
    bpy.ops.object.mode_set(mode='EDIT')

    bone = armature_obj.data.edit_bones.new(name)
    bone.head = Vector(head)
    bone.tail = Vector(tail)

    if parent:
        parent_bone = armature_obj.data.edit_bones.get(parent)
        if parent_bone:
            bone.parent = parent_bone

    bpy.ops.object.mode_set(mode='OBJECT')
    return bone

def parent_to_bone(obj, armature_obj, bone_name):
    """オブジェクトをボーンにペアレント"""
    obj.parent = armature_obj
    obj.parent_type = 'BONE'
    obj.parent_bone = bone_name

# =============================================================================
# アニメーション
# =============================================================================

def create_rotation_animation(obj, axis='Z', frames=30, rotations=1, name="rotate_cycle"):
    """回転アニメーション"""
    obj.rotation_mode = 'XYZ'
    obj.keyframe_insert(data_path="rotation_euler", frame=1)

    axis_idx = {'X': 0, 'Y': 1, 'Z': 2}[axis.upper()]
    obj.rotation_euler[axis_idx] = rotations * 2 * pi
    obj.keyframe_insert(data_path="rotation_euler", frame=frames + 1)

    # リニア補間
    if obj.animation_data and obj.animation_data.action:
        for fc in obj.animation_data.action.fcurves:
            for kp in fc.keyframe_points:
                kp.interpolation = 'LINEAR'

def create_translation_animation(obj, axis='Z', distance=0.5, frames=30, name="move_cycle"):
    """往復移動アニメーション"""
    obj.keyframe_insert(data_path="location", frame=1)

    axis_idx = {'X': 0, 'Y': 1, 'Z': 2}[axis.upper()]
    original = obj.location[axis_idx]

    obj.location[axis_idx] = original + distance
    obj.keyframe_insert(data_path="location", frame=frames // 2 + 1)

    obj.location[axis_idx] = original
    obj.keyframe_insert(data_path="location", frame=frames + 1)

    if obj.animation_data and obj.animation_data.action:
        for fc in obj.animation_data.action.fcurves:
            for kp in fc.keyframe_points:
                kp.interpolation = 'LINEAR'

# =============================================================================
# エクスポート
# =============================================================================

def export_gltf(filepath, export_animations=True):
    """glTFエクスポート"""
    bpy.ops.export_scene.gltf(
        filepath=filepath,
        export_format='GLTF_SEPARATE',
        export_texcoords=True,
        export_normals=True,
        export_tangents=True,
        export_colors=True,
        export_materials='EXPORT',
        export_animations=export_animations,
        export_yup=True,
    )
    print(f"Exported: {filepath}")

def apply_transforms(obj):
    """トランスフォームを適用"""
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)

def finalize_model(obj, category="machine"):
    """モデルの最終処理"""
    apply_transforms(obj)

    if category == "machine":
        set_origin_bottom_center(obj)
    else:
        set_origin_center(obj)

# =============================================================================
# 登録完了メッセージ
# =============================================================================

print("=== Industrial Lowpoly Base Module Loaded ===")
print("Available functions:")
print("  Primitives: create_octagon, create_octagonal_prism, create_chamfered_cube, create_hexagon, create_trapezoid")
print("  Parts: create_gear, create_shaft, create_pipe, create_bolt, create_piston")
print("  Materials: create_material, apply_preset_material")
print("  Animation: create_rotation_animation, create_translation_animation")
print("  Export: export_gltf, finalize_model")
