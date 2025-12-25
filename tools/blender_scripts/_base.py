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

def snap(value, enabled=False):
    """グリッドスナップ（デフォルト無効）"""
    if not enabled:
        return value
    return round(value / GRID_UNIT) * GRID_UNIT

def snap_vec(v, enabled=False):
    """ベクトルをグリッドスナップ（デフォルト無効）"""
    if not enabled:
        return Vector((v.x, v.y, v.z)) if not isinstance(v, Vector) else v
    return Vector((snap(v.x, True), snap(v.y, True), snap(v.z, True)))

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
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = base
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    return bpy.context.active_object

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

    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = head
    shaft.select_set(True)
    head.select_set(True)
    bpy.ops.object.join()

    return bpy.context.active_object

def create_piston(rod_radius=0.05, rod_length=0.5, head_size=(0.2, 0.2, 0.1), location=(0, 0, 0), name="Piston"):
    """ピストン"""
    rod = create_octagonal_prism(rod_radius, rod_length, location, f"{name}_rod")
    head = create_chamfered_cube(head_size, None, (location[0], location[1], location[2] + rod_length / 2), f"{name}_head")

    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = rod
    head.select_set(True)
    rod.select_set(True)
    bpy.ops.object.join()

    return bpy.context.active_object

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
# ディテール追加関数（Minecraft/Unturned風）
# =============================================================================

def add_surface_detail(obj, detail_type="bump", count=3, seed=0):
    """表面にローポリディテールを追加"""
    import random
    random.seed(seed)

    # バウンディングボックス取得
    bbox = [obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
    min_x = min(v.x for v in bbox)
    max_x = max(v.x for v in bbox)
    min_y = min(v.y for v in bbox)
    max_y = max(v.y for v in bbox)
    min_z = min(v.z for v in bbox)
    max_z = max(v.z for v in bbox)

    size_x = max_x - min_x
    size_y = max_y - min_y
    size_z = max_z - min_z

    details = []

    for i in range(count):
        # ランダム位置（表面付近）
        face = random.choice(['top', 'front', 'side'])

        if face == 'top':
            x = random.uniform(min_x + size_x * 0.2, max_x - size_x * 0.2)
            y = random.uniform(min_y + size_y * 0.2, max_y - size_y * 0.2)
            z = max_z
        elif face == 'front':
            x = random.uniform(min_x + size_x * 0.2, max_x - size_x * 0.2)
            y = max_y
            z = random.uniform(min_z + size_z * 0.2, max_z - size_z * 0.2)
        else:
            x = max_x
            y = random.uniform(min_y + size_y * 0.2, max_y - size_y * 0.2)
            z = random.uniform(min_z + size_z * 0.2, max_z - size_z * 0.2)

        detail_size = min(size_x, size_y, size_z) * random.uniform(0.08, 0.15)

        if detail_type == "bump":
            detail = create_chamfered_cube(
                size=(detail_size, detail_size, detail_size * 0.5),
                chamfer=detail_size * 0.1,
                location=(x, y, z),
                name=f"detail_{i}"
            )
        elif detail_type == "rivet":
            detail = create_octagonal_prism(
                radius=detail_size * 0.4,
                height=detail_size * 0.3,
                location=(x, y, z),
                name=f"rivet_{i}"
            )

        details.append(detail)

    return details

def create_rivet(radius=0.008, height=0.004, location=(0, 0, 0), name="Rivet"):
    """リベット（工業的ディテール）"""
    rivet = create_octagonal_prism(radius, height, location, name)
    return rivet

def create_weld_line(length=0.1, width=0.006, location=(0, 0, 0), rotation=(0, 0, 0), name="Weld"):
    """溶接線（工業的ディテール）"""
    weld = create_chamfered_cube(
        size=(length, width, width * 0.5),
        chamfer=width * 0.2,
        location=location,
        name=name
    )
    weld.rotation_euler = rotation
    return weld

def create_groove(length=0.1, width=0.01, depth=0.005, location=(0, 0, 0), name="Groove"):
    """溝（ディテール用）"""
    groove = create_chamfered_cube(
        size=(length, width, depth),
        chamfer=depth * 0.2,
        location=location,
        name=name
    )
    return groove

def create_edge_bevel(obj, segments=1):
    """エッジにベベルを追加（ローポリ風の面取り）"""
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    mod = obj.modifiers.new("Bevel", 'BEVEL')
    mod.width = 0.005
    mod.segments = segments
    mod.limit_method = 'ANGLE'
    mod.angle_limit = radians(30)
    bpy.ops.object.modifier_apply(modifier="Bevel")
    return obj

# =============================================================================
# 階層構造ヘルパー（キットバッシング用）
# =============================================================================

def create_root_empty(name="MachineRoot"):
    """ルートEmptyを作成（全パーツの親）"""
    root = bpy.data.objects.new(name, None)
    root.empty_display_type = 'ARROWS'
    root.empty_display_size = 0.5
    bpy.context.collection.objects.link(root)
    return root

def parent_to_root(objects, root):
    """複数オブジェクトをルートの子に設定"""
    for obj in objects:
        obj.parent = root
        # 相対位置を維持
        obj.matrix_parent_inverse = root.matrix_world.inverted()

def join_all_meshes(objects, name="CombinedMesh"):
    """複数メッシュを1つに結合"""
    if not objects:
        return None

    # メッシュオブジェクトのみフィルタ
    mesh_objects = [obj for obj in objects if obj.type == 'MESH']
    if not mesh_objects:
        return None

    bpy.ops.object.select_all(action='DESELECT')
    for obj in mesh_objects:
        obj.select_set(True)

    bpy.context.view_layer.objects.active = mesh_objects[0]
    bpy.ops.object.join()

    result = bpy.context.active_object
    result.name = name
    return result

# =============================================================================
# コンベア関連パーツ（Kenney Conveyor Kit風）
# =============================================================================

def create_roller(radius=0.1, length=0.8, location=(0, 0, 0), name="Roller"):
    """コンベアローラー（八角柱）"""
    roller = create_octagonal_prism(radius, length, location, name)
    roller.rotation_euler.y = pi / 2  # X軸方向に
    return roller

def create_conveyor_belt_segment(width=0.8, length=0.2, thickness=0.02, location=(0, 0, 0), name="BeltSegment"):
    """コンベアベルトセグメント"""
    segment = create_chamfered_cube(
        size=(width, length, thickness),
        chamfer=thickness * 0.3,
        location=location,
        name=name
    )
    return segment

def create_conveyor_frame(width=1.0, length=1.0, height=0.3, location=(0, 0, 0), name="ConveyorFrame"):
    """コンベアフレーム（サイドレール付き）"""
    parts = []
    rail_width = 0.08
    rail_height = height

    # 左レール
    left_rail = create_chamfered_cube(
        size=(rail_width, length, rail_height),
        location=(location[0] - width/2 + rail_width/2, location[1], location[2] + rail_height/2),
        name=f"{name}_LeftRail"
    )
    parts.append(left_rail)

    # 右レール
    right_rail = create_chamfered_cube(
        size=(rail_width, length, rail_height),
        location=(location[0] + width/2 - rail_width/2, location[1], location[2] + rail_height/2),
        name=f"{name}_RightRail"
    )
    parts.append(right_rail)

    return parts

def create_support_leg(height=0.5, width=0.1, location=(0, 0, 0), name="SupportLeg"):
    """サポート脚"""
    leg = create_chamfered_cube(
        size=(width, width, height),
        location=(location[0], location[1], location[2] - height/2),
        name=name
    )
    return leg

# =============================================================================
# Blender MCP連携ヘルパー
# =============================================================================

def get_scene_info():
    """シーン情報を取得（MCP経由でのデバッグ用）"""
    info = {
        "objects": [],
        "total_triangles": 0,
        "materials": []
    }

    for obj in bpy.context.scene.objects:
        if obj.type == 'MESH':
            # 三角形数を計算
            tri_count = sum(len(poly.vertices) - 2 for poly in obj.data.polygons)
            info["objects"].append({
                "name": obj.name,
                "location": list(obj.location),
                "triangles": tri_count,
                "materials": [mat.name for mat in obj.data.materials if mat]
            })
            info["total_triangles"] += tri_count

    info["materials"] = list(set(mat.name for mat in bpy.data.materials))
    return info

def validate_model(obj, category="machine"):
    """モデルのバリデーション"""
    issues = []

    if obj.type != 'MESH':
        issues.append(f"オブジェクト '{obj.name}' はメッシュではありません")
        return issues

    # 三角形数チェック
    tri_count = sum(len(poly.vertices) - 2 for poly in obj.data.polygons)

    budgets = {
        "item": (200, 500),
        "machine": (800, 1500),
        "structure": (2000, 4000)
    }

    if category in budgets:
        recommended, max_count = budgets[category]
        if tri_count > max_count:
            issues.append(f"三角形数 {tri_count} が上限 {max_count} を超えています")
        elif tri_count > recommended:
            issues.append(f"三角形数 {tri_count} が推奨値 {recommended} を超えています（上限: {max_count}）")

    # 原点チェック
    if category == "machine":
        bbox = [obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
        min_z = min(v.z for v in bbox)
        if abs(min_z) > 0.01:
            issues.append(f"原点が底面中心にありません（最小Z: {min_z:.3f}）")

    # マテリアルチェック
    if not obj.data.materials:
        issues.append("マテリアルが設定されていません")

    return issues

def print_validation_report(obj, category="machine"):
    """バリデーションレポートを出力"""
    issues = validate_model(obj, category)

    print(f"\n=== Validation Report: {obj.name} ===")
    if issues:
        print("Issues found:")
        for issue in issues:
            print(f"  ⚠️  {issue}")
    else:
        print("✅ All checks passed!")

    # 基本情報
    if obj.type == 'MESH':
        tri_count = sum(len(poly.vertices) - 2 for poly in obj.data.polygons)
        print(f"\nStats:")
        print(f"  Triangles: {tri_count}")
        print(f"  Location: {tuple(round(v, 3) for v in obj.location)}")
        print(f"  Materials: {len(obj.data.materials)}")

    return len(issues) == 0

# =============================================================================
# 高レベルパーツ（アイテム用）
# =============================================================================

def create_tool_handle(length=0.15, radius=0.012, material="wood", grip_grooves=3):
    """ツール用木製ハンドル（グリップ溝付き）

    Args:
        length: ハンドル長さ (default: 0.15)
        radius: ハンドル半径 (default: 0.012)
        material: マテリアルプリセット (default: "wood")
        grip_grooves: グリップ溝の数 (default: 3)

    Returns:
        結合されたハンドルオブジェクト
    """
    objects = []

    # メインハンドル
    handle = create_octagonal_prism(radius, length, (0, 0, 0), "Handle")
    apply_preset_material(handle, material)
    objects.append(handle)

    # グリップ溝
    groove_color = tuple(c * 0.8 for c in MATERIALS[material]["color"][:3]) + (1,)
    groove_mat = create_material("grip_groove", color=groove_color, metallic=0.0, roughness=0.9)

    for i in range(grip_grooves):
        z_pos = -length * 0.3 + i * 0.02
        groove = create_octagonal_prism(radius * 1.1, 0.004, (0, 0, z_pos), f"Grip_{i}")
        apply_material(groove, groove_mat)
        objects.append(groove)

    # 端のキャップ
    cap = create_octagonal_prism(radius * 1.15, 0.008, (0, 0, -length/2 + 0.004), "HandleCap")
    apply_preset_material(cap, material)
    objects.append(cap)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = handle
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    return bpy.context.active_object


def create_ingot(width=0.08, length=0.12, height=0.03, material="iron"):
    """インゴット（台形断面の金属塊）

    Args:
        width: 幅 (default: 0.08)
        length: 長さ (default: 0.12)
        height: 高さ (default: 0.03)
        material: マテリアルプリセット

    Returns:
        インゴットオブジェクト
    """
    # 上面がやや小さい台形断面
    ingot = create_chamfered_cube(
        size=(width, length, height),
        chamfer=height * 0.15,
        location=(0, 0, 0),
        name="Ingot"
    )
    apply_preset_material(ingot, material)
    return ingot


def create_ore_chunk(size=0.06, material="stone", irregularity=0.3):
    """鉱石塊（不規則な多面体）

    Args:
        size: 基本サイズ (default: 0.06)
        material: マテリアルプリセット
        irregularity: 不規則さ (0-1)

    Returns:
        鉱石オブジェクト
    """
    import random

    objects = []

    # メインの塊
    main = create_chamfered_cube(
        size=(size, size * 0.9, size * 0.8),
        chamfer=size * 0.1,
        location=(0, 0, 0),
        name="OreMain"
    )
    objects.append(main)

    # 突起（2-3個）
    for i in range(2):
        offset = size * 0.3
        bump = create_chamfered_cube(
            size=(size * 0.4, size * 0.35, size * 0.3),
            chamfer=size * 0.05,
            location=(
                (i - 0.5) * offset,
                (i % 2 - 0.5) * offset * 0.5,
                size * 0.3
            ),
            name=f"OreBump_{i}"
        )
        objects.append(bump)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = main
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    result = bpy.context.active_object
    apply_preset_material(result, material)
    return result


def create_plate(width=0.1, length=0.1, thickness=0.008, material="iron"):
    """金属プレート

    Args:
        width: 幅 (default: 0.1)
        length: 長さ (default: 0.1)
        thickness: 厚さ (default: 0.008)
        material: マテリアルプリセット

    Returns:
        プレートオブジェクト
    """
    plate = create_chamfered_cube(
        size=(width, length, thickness),
        chamfer=thickness * 0.2,
        location=(0, 0, 0),
        name="Plate"
    )
    apply_preset_material(plate, material)
    return plate


def create_dust_pile(radius=0.04, height=0.025, material="stone"):
    """粉末の山

    Args:
        radius: 半径 (default: 0.04)
        height: 高さ (default: 0.025)
        material: マテリアルプリセット

    Returns:
        粉末オブジェクト
    """
    # 八角形の円錐に近い形
    objects = []

    # ベース層
    base = create_octagonal_prism(radius, height * 0.4, (0, 0, height * 0.2), "DustBase")
    objects.append(base)

    # 中間層
    mid = create_octagonal_prism(radius * 0.7, height * 0.35, (0, 0, height * 0.5), "DustMid")
    objects.append(mid)

    # 頂点層
    top = create_octagonal_prism(radius * 0.3, height * 0.25, (0, 0, height * 0.75), "DustTop")
    objects.append(top)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = base
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    result = bpy.context.active_object
    apply_preset_material(result, material)
    return result


# =============================================================================
# 高レベルパーツ（機械用）
# =============================================================================

def create_machine_frame(width=0.9, depth=0.9, height=0.3, material="dark_steel"):
    """機械用ベースフレーム

    Args:
        width: 幅 (default: 0.9)
        depth: 奥行き (default: 0.9)
        height: 高さ (default: 0.3)
        material: マテリアルプリセット

    Returns:
        フレームオブジェクト
    """
    frame = create_chamfered_cube(
        size=(width, depth, height),
        chamfer=min(width, depth, height) * 0.05,
        location=(0, 0, height / 2),
        name="MachineFrame"
    )
    apply_preset_material(frame, material)
    return frame


def create_machine_body(width=0.9, depth=0.9, height=0.6, material="iron"):
    """機械用メインボディ

    Args:
        width: 幅 (default: 0.9)
        depth: 奥行き (default: 0.9)
        height: 高さ (default: 0.6)
        material: マテリアルプリセット

    Returns:
        ボディオブジェクト
    """
    body = create_chamfered_cube(
        size=(width, depth, height),
        chamfer=min(width, depth) * 0.05,
        location=(0, 0, height / 2),
        name="MachineBody"
    )
    apply_preset_material(body, material)
    return body


def create_corner_bolts(width=0.9, depth=0.9, z_pos=0.3, bolt_size=0.04, material="iron"):
    """四隅のボルト装飾

    Args:
        width: 配置幅
        depth: 配置奥行き
        z_pos: Z座標
        bolt_size: ボルトサイズ
        material: マテリアルプリセット

    Returns:
        ボルトオブジェクトのリスト
    """
    bolts = []
    offset = 0.4  # 中心からのオフセット比率

    positions = [
        (-width * offset, -depth * offset, z_pos),
        (width * offset, -depth * offset, z_pos),
        (-width * offset, depth * offset, z_pos),
        (width * offset, depth * offset, z_pos),
    ]

    for i, pos in enumerate(positions):
        bolt = create_bolt(bolt_size, bolt_size * 1.5, pos, f"CornerBolt_{i}")
        apply_preset_material(bolt, material)
        bolts.append(bolt)

    return bolts


def create_tank_body(radius=0.4, height=0.6, material="iron"):
    """タンク型ボディ（八角柱 + キャップ）

    Args:
        radius: 半径 (default: 0.4)
        height: 高さ (default: 0.6)
        material: マテリアルプリセット

    Returns:
        タンクパーツのリスト [body, top_cap, bottom_cap]
    """
    parts = []

    # メインボディ
    body = create_octagonal_prism(radius, height, (0, 0, height/2 + 0.05), "TankBody")
    apply_preset_material(body, material)
    parts.append(body)

    # 上部キャップ
    top_cap = create_octagonal_prism(radius * 1.05, 0.08, (0, 0, height + 0.05), "TankTopCap")
    apply_preset_material(top_cap, "brass")
    parts.append(top_cap)

    # 下部キャップ
    bottom_cap = create_octagonal_prism(radius * 1.05, 0.08, (0, 0, 0.04), "TankBottomCap")
    apply_preset_material(bottom_cap, "brass")
    parts.append(bottom_cap)

    return parts


def create_reinforcement_ribs(width=0.9, depth=0.9, z_pos=0.4, material="dark_steel"):
    """補強リブ（4辺）

    Args:
        width: 幅
        depth: 奥行き
        z_pos: Z座標
        material: マテリアルプリセット

    Returns:
        リブオブジェクトのリスト
    """
    ribs = []
    rib_size = 0.06

    # X軸方向のリブ
    for x_offset in [-width * 0.45, width * 0.45]:
        rib = create_chamfered_cube(
            size=(rib_size, depth, rib_size),
            chamfer=0.01,
            location=(x_offset, 0, z_pos),
            name=f"RibX_{x_offset}"
        )
        apply_preset_material(rib, material)
        ribs.append(rib)

    # Y軸方向のリブ
    for y_offset in [-depth * 0.45, depth * 0.45]:
        rib = create_chamfered_cube(
            size=(width, rib_size, rib_size),
            chamfer=0.01,
            location=(0, y_offset, z_pos),
            name=f"RibY_{y_offset}"
        )
        apply_preset_material(rib, material)
        ribs.append(rib)

    return ribs


def create_motor_housing(radius=0.2, height=0.15, location=(0, 0, 0), material="copper"):
    """モーターハウジング

    Args:
        radius: 半径 (default: 0.2)
        height: 高さ (default: 0.15)
        location: 位置
        material: マテリアルプリセット

    Returns:
        モーターオブジェクト
    """
    motor = create_octagonal_prism(radius, height, location, "MotorHousing")
    apply_preset_material(motor, material)
    return motor


# =============================================================================
# ヘルパー関数
# =============================================================================

def add_decorative_bolts_circle(radius, z_pos, count=8, bolt_size=0.03, material="brass"):
    """円形配置のボルト装飾

    Args:
        radius: 配置半径
        z_pos: Z座標
        count: ボルト数 (default: 8)
        bolt_size: ボルトサイズ
        material: マテリアルプリセット

    Returns:
        ボルトオブジェクトのリスト
    """
    bolts = []
    for i in range(count):
        angle = i * 2 * pi / count + pi / count  # オフセットして配置
        x = cos(angle) * radius
        y = sin(angle) * radius
        bolt = create_bolt(bolt_size, bolt_size * 1.3, (x, y, z_pos), f"CircleBolt_{i}")
        apply_preset_material(bolt, material)
        bolts.append(bolt)
    return bolts


def create_accent_light(size=0.05, location=(0, 0, 0), color_preset="power"):
    """アクセントライト（状態表示など）

    Args:
        size: サイズ
        location: 位置
        color_preset: danger/warning/power/active

    Returns:
        ライトオブジェクト
    """
    light = create_octagonal_prism(size, size * 0.5, location, "AccentLight")
    mat = create_material(
        f"accent_{color_preset}",
        color=ACCENT_COLORS.get(color_preset, ACCENT_COLORS["power"]),
        metallic=0.1,
        roughness=0.2
    )
    apply_material(light, mat)
    return light


# =============================================================================
# 登録完了メッセージ
# =============================================================================

print("=== Industrial Lowpoly Base Module Loaded ===")
print("Available functions:")
print("  Primitives: create_octagon, create_octagonal_prism, create_chamfered_cube, create_hexagon, create_trapezoid")
print("  Parts: create_gear, create_shaft, create_pipe, create_bolt, create_piston")
print("  Conveyor: create_roller, create_conveyor_belt_segment, create_conveyor_frame, create_support_leg")
print("  Hierarchy: create_root_empty, parent_to_root, join_all_meshes")
print("  Materials: create_material, apply_preset_material")
print("  Animation: create_rotation_animation, create_translation_animation")
print("  Validation: get_scene_info, validate_model, print_validation_report")
print("  Export: export_gltf, finalize_model")
print("  === NEW High-Level Parts ===")
print("  Items: create_tool_handle, create_ingot, create_ore_chunk, create_plate, create_dust_pile")
print("  Machines: create_machine_frame, create_machine_body, create_tank_body, create_motor_housing")
print("  Decorative: create_corner_bolts, create_reinforcement_ribs, add_decorative_bolts_circle, create_accent_light")
