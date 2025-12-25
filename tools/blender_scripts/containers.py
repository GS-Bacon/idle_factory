"""
液体・気体容器アイテムモデル生成
カテゴリ: item (dropped_item: 0.4x0.4x0.4)

バケツ: 台形断面の円筒容器、液体色で区別
キャニスター: 八角柱の圧力タンク、ドーム付き
"""

import bpy
from mathutils import Vector
from math import pi, cos, sin
import sys
import os

# _base.pyをインポート
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.append(script_dir)

# _base.pyが既に実行されていることを前提とするが、念のため確認
try:
    from _base import (
        clear_scene, create_material, apply_material,
        finalize_model, export_gltf, snap_vec, create_octagonal_prism
    )
except ImportError:
    print("ERROR: _base.py must be executed first!")
    print("Please run _base.py before running this script.")
    sys.exit(1)

# =============================================================================
# 容器定義
# =============================================================================

# バケツサイズ（dropped_item 0.4x0.4x0.4 に収まる）
BUCKET_BOTTOM_RADIUS = 0.12
BUCKET_TOP_RADIUS = 0.15
BUCKET_HEIGHT = 0.28
BUCKET_WALL_THICKNESS = 0.015
HANDLE_HEIGHT = 0.08

# キャニスターサイズ
CANISTER_RADIUS = 0.13
CANISTER_HEIGHT = 0.32
CANISTER_DOME_HEIGHT = 0.04

# バケツの液体色定義
BUCKET_LIQUIDS = {
    "water": {"color": (0.2, 0.5, 0.8, 1), "name": "水"},
    "crude_oil": {"color": (0.1, 0.1, 0.08, 1), "name": "原油"},
    "fuel": {"color": (0.9, 0.7, 0.2, 1), "name": "燃料"},
    "lubricant": {"color": (0.8, 0.6, 0.1, 1), "name": "潤滑油"},
    "sulfuric_acid": {"color": (0.9, 0.9, 0.3, 1), "name": "硫酸"},
    "lava": {"color": (1.0, 0.3, 0.1, 1), "name": "溶岩"},
}

# キャニスターの気体ラベル色定義
CANISTER_GASES = {
    "oxygen": {"color": (0.3, 0.6, 0.9, 1), "name": "酸素"},
    "hydrogen": {"color": (0.9, 0.3, 0.3, 1), "name": "水素"},
    "nitrogen": {"color": (0.5, 0.5, 0.5, 1), "name": "窒素"},
    "natural_gas": {"color": (0.7, 0.6, 0.4, 1), "name": "天然ガス"},
    "steam": {"color": (0.85, 0.85, 0.9, 1), "name": "蒸気"},
}

# バケツ本体のマテリアル（金属）
BUCKET_METAL = {"color": (0.35, 0.35, 0.35, 1), "metallic": 1.0, "roughness": 0.4}

# キャニスター本体のマテリアル（鋼鉄）
CANISTER_METAL = {"color": (0.25, 0.25, 0.28, 1), "metallic": 1.0, "roughness": 0.5}

# =============================================================================
# バケツ形状生成
# =============================================================================

def create_tapered_cylinder(bottom_radius, top_radius, height, segments=8, name="TaperedCyl"):
    """
    台形断面の円筒（八角形ベース）
    - bottom_radius: 底面半径
    - top_radius: 上面半径
    - height: 高さ
    - segments: 分割数（デフォルト8 = 八角形）
    """
    verts = []

    # 底面の頂点
    for i in range(segments):
        angle = i * 2 * pi / segments + pi / segments  # 22.5度オフセット
        x = cos(angle) * bottom_radius
        y = sin(angle) * bottom_radius
        verts.append((x, y, 0))

    # 上面の頂点
    for i in range(segments):
        angle = i * 2 * pi / segments + pi / segments
        x = cos(angle) * top_radius
        y = sin(angle) * top_radius
        verts.append((x, y, height))

    faces = []

    # 側面
    for i in range(segments):
        j = (i + 1) % segments
        faces.append((i, j, j + segments, i + segments))

    # 底面
    faces.append(tuple(range(segments)))

    # 上面
    faces.append(tuple(range(segments, segments * 2)))

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()

    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector((0, 0, 0))
    bpy.context.collection.objects.link(obj)

    return obj

def create_bucket_handle(radius, height, thickness=0.02, name="Handle"):
    """
    バケツの取っ手（アーチ状）
    - radius: 取っ手の半径
    - height: 取っ手の高さ
    - thickness: 取っ手の太さ
    """
    verts = []
    segments = 8

    # アーチ部分（半円）
    for i in range(segments + 1):
        angle = pi * i / segments  # 0 to π
        x = cos(angle) * radius
        z = sin(angle) * height + BUCKET_HEIGHT - height * 0.3

        # 外側
        verts.append((x, -thickness / 2, z))
        verts.append((x, thickness / 2, z))

    faces = []

    # 側面
    for i in range(segments):
        base = i * 2
        faces.append((base, base + 2, base + 3, base + 1))

    # 端面
    for i in range(segments):
        base = i * 2
        if i > 0:
            faces.append((base, base + 1, 1, 0))  # 左端
        if i < segments - 1:
            next_base = (segments) * 2
            faces.append((next_base, next_base + 1, base + 3, base + 2))  # 右端

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()

    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector((0, 0, 0))
    bpy.context.collection.objects.link(obj)

    return obj

def create_bucket_base(name="Bucket"):
    """バケツ本体（外殻 + 取っ手）"""
    # 外殻
    outer = create_tapered_cylinder(
        BUCKET_BOTTOM_RADIUS, BUCKET_TOP_RADIUS, BUCKET_HEIGHT,
        segments=8, name=f"{name}_outer"
    )

    # 内側（くり抜き用）
    inner = create_tapered_cylinder(
        BUCKET_BOTTOM_RADIUS - BUCKET_WALL_THICKNESS,
        BUCKET_TOP_RADIUS - BUCKET_WALL_THICKNESS,
        BUCKET_HEIGHT - BUCKET_WALL_THICKNESS,
        segments=8, name=f"{name}_inner"
    )
    inner.location.z = BUCKET_WALL_THICKNESS

    # Boolean差分で中空に
    mod = outer.modifiers.new("Bool", 'BOOLEAN')
    mod.operation = 'DIFFERENCE'
    mod.object = inner
    bpy.context.view_layer.objects.active = outer
    bpy.ops.object.modifier_apply(modifier="Bool")
    bpy.data.objects.remove(inner)

    # 取っ手を追加
    handle = create_bucket_handle(BUCKET_TOP_RADIUS * 0.8, HANDLE_HEIGHT, name=f"{name}_handle")

    # 結合
    bpy.context.view_layer.objects.active = outer
    outer.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()

    outer.name = name
    return outer

def create_liquid_surface(radius, height, name="Liquid"):
    """液体表面（円盤）"""
    segments = 8
    verts = []

    # 中心点
    verts.append((0, 0, height))

    # 外周
    for i in range(segments):
        angle = i * 2 * pi / segments + pi / segments
        x = cos(angle) * radius
        y = sin(angle) * radius
        verts.append((x, y, height))

    faces = []
    for i in range(segments):
        j = (i + 1) % segments
        faces.append((0, i + 1, j + 1))

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()

    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector((0, 0, 0))
    bpy.context.collection.objects.link(obj)

    return obj

# =============================================================================
# キャニスター形状生成
# =============================================================================

def create_dome(radius, height, top=True, segments=8, name="Dome"):
    """
    ドーム形状（八角錐の頂点を丸めたもの）
    - top: True=上ドーム, False=下ドーム
    """
    verts = []

    # ベースの円周
    for i in range(segments):
        angle = i * 2 * pi / segments + pi / segments
        x = cos(angle) * radius
        y = sin(angle) * radius
        z = 0 if top else height
        verts.append((x, y, z))

    # 頂点
    apex_z = height if top else 0
    verts.append((0, 0, apex_z))

    faces = []

    # 側面
    for i in range(segments):
        j = (i + 1) % segments
        if top:
            faces.append((i, j, segments))
        else:
            faces.append((segments, j, i))

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()

    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector((0, 0, 0))
    bpy.context.collection.objects.link(obj)

    return obj

def create_canister_base(name="Canister"):
    """キャニスター本体（八角柱 + 上下ドーム）"""
    # 中央の円柱部分
    cylinder = create_octagonal_prism(
        CANISTER_RADIUS, CANISTER_HEIGHT, (0, 0, CANISTER_DOME_HEIGHT),
        name=f"{name}_cylinder"
    )

    # 上部ドーム
    top_dome = create_dome(
        CANISTER_RADIUS, CANISTER_DOME_HEIGHT, top=True,
        name=f"{name}_top_dome"
    )
    top_dome.location.z = CANISTER_DOME_HEIGHT + CANISTER_HEIGHT

    # 下部ドーム
    bottom_dome = create_dome(
        CANISTER_RADIUS, CANISTER_DOME_HEIGHT, top=False,
        name=f"{name}_bottom_dome"
    )
    bottom_dome.location.z = CANISTER_DOME_HEIGHT

    # 結合
    bpy.context.view_layer.objects.active = cylinder
    cylinder.select_set(True)
    top_dome.select_set(True)
    bottom_dome.select_set(True)
    bpy.ops.object.join()

    cylinder.name = name
    return cylinder

def create_label_ring(radius, height, thickness=0.02, name="Label"):
    """ラベル帯（八角リング）"""
    outer = create_octagonal_prism(radius + thickness, thickness, (0, 0, height), name=f"{name}_outer")
    inner = create_octagonal_prism(radius, thickness + 0.01, (0, 0, height), name=f"{name}_inner")

    # Boolean差分
    mod = outer.modifiers.new("Bool", 'BOOLEAN')
    mod.operation = 'DIFFERENCE'
    mod.object = inner
    bpy.context.view_layer.objects.active = outer
    bpy.ops.object.modifier_apply(modifier="Bool")
    bpy.data.objects.remove(inner)

    outer.name = name
    return outer

# =============================================================================
# バケツ生成関数
# =============================================================================

def create_bucket(liquid_type, name=None):
    """指定された液体のバケツを生成"""
    if liquid_type not in BUCKET_LIQUIDS:
        print(f"ERROR: Unknown liquid type '{liquid_type}'")
        return None

    if name is None:
        name = f"{liquid_type}_bucket"

    # バケツ本体
    bucket = create_bucket_base(name)

    # バケツ本体のマテリアル
    bucket_mat = create_material(
        f"{name}_metal",
        color=BUCKET_METAL["color"],
        metallic=BUCKET_METAL["metallic"],
        roughness=BUCKET_METAL["roughness"]
    )
    apply_material(bucket, bucket_mat)

    # 液体表面
    liquid_height = BUCKET_HEIGHT * 0.75  # 75%まで満たす
    liquid_radius = BUCKET_BOTTOM_RADIUS + (BUCKET_TOP_RADIUS - BUCKET_BOTTOM_RADIUS) * (liquid_height / BUCKET_HEIGHT)
    liquid = create_liquid_surface(liquid_radius * 0.95, liquid_height, f"{name}_liquid")

    # 液体のマテリアル
    liquid_data = BUCKET_LIQUIDS[liquid_type]
    liquid_mat = create_material(
        f"{name}_liquid_mat",
        color=liquid_data["color"],
        metallic=0.2,
        roughness=0.1
    )
    apply_material(liquid, liquid_mat)

    # 結合
    bpy.context.view_layer.objects.active = bucket
    bucket.select_set(True)
    liquid.select_set(True)
    bpy.ops.object.join()

    # 最終処理
    finalize_model(bucket, category="item")

    print(f"Created: {name} ({liquid_data['name']})")
    return bucket

# =============================================================================
# キャニスター生成関数
# =============================================================================

def create_canister(gas_type, name=None):
    """指定された気体のキャニスターを生成"""
    if gas_type not in CANISTER_GASES:
        print(f"ERROR: Unknown gas type '{gas_type}'")
        return None

    if name is None:
        name = f"{gas_type}_canister"

    # キャニスター本体
    canister = create_canister_base(name)

    # 本体のマテリアル
    canister_mat = create_material(
        f"{name}_metal",
        color=CANISTER_METAL["color"],
        metallic=CANISTER_METAL["metallic"],
        roughness=CANISTER_METAL["roughness"]
    )
    apply_material(canister, canister_mat)

    # ラベルリング（中央に配置）
    label_height = CANISTER_DOME_HEIGHT + CANISTER_HEIGHT / 2
    label = create_label_ring(CANISTER_RADIUS, label_height, name=f"{name}_label")

    # ラベルのマテリアル（気体の色）
    gas_data = CANISTER_GASES[gas_type]
    label_mat = create_material(
        f"{name}_label_mat",
        color=gas_data["color"],
        metallic=0.0,
        roughness=0.8
    )
    apply_material(label, label_mat)

    # 結合
    bpy.context.view_layer.objects.active = canister
    canister.select_set(True)
    label.select_set(True)
    bpy.ops.object.join()

    # 最終処理
    finalize_model(canister, category="item")

    print(f"Created: {name} ({gas_data['name']})")
    return canister

# =============================================================================
# 全容器生成・エクスポート
# =============================================================================

def generate_all_containers(output_dir="./models/items/containers"):
    """全容器を生成してエクスポート"""
    os.makedirs(output_dir, exist_ok=True)

    # バケツ
    print("\n=== Generating Buckets ===")
    for liquid_type in BUCKET_LIQUIDS.keys():
        print(f"\n--- Generating {liquid_type}_bucket ---")
        clear_scene()

        obj = create_bucket(liquid_type)
        if obj:
            output_path = os.path.join(output_dir, f"{liquid_type}_bucket.gltf")
            export_gltf(output_path, export_animations=False)

    # キャニスター
    print("\n=== Generating Canisters ===")
    for gas_type in CANISTER_GASES.keys():
        print(f"\n--- Generating {gas_type}_canister ---")
        clear_scene()

        obj = create_canister(gas_type)
        if obj:
            output_path = os.path.join(output_dir, f"{gas_type}_canister.gltf")
            export_gltf(output_path, export_animations=False)

    print("\n=== All containers generated successfully ===")

# =============================================================================
# 実行
# =============================================================================

if __name__ == "__main__":
    # 既存シーンをクリア
    clear_scene()

    # 全容器を生成してエクスポート
    output_directory = "./models/items/containers"
    generate_all_containers(output_directory)
