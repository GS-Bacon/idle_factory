"""
Stone Material Items - Industrial Lowpoly Style
石材アイテムモデル生成スクリプト

使い方:
1. Blenderで_base.pyを実行
2. このスクリプトを実行してモデルを生成
3. 各関数を個別に実行するか、create_all_stones()で一括生成

カテゴリ: item (dropped_item: 0.4x0.4x0.4)
"""

import bpy
from mathutils import Vector, Matrix
from math import pi, cos, sin
import os

# _base.pyから必要な関数をインポート（既に実行済みと仮定）
# create_chamfered_cube, create_octagon, apply_preset_material, create_material
# finalize_model, export_gltf, set_origin_center, snap_vec, clear_scene

# =============================================================================
# 定数
# =============================================================================

ITEM_SIZE = 0.4  # dropped_itemのサイズ

# カスタムカラー
COLORS = {
    "stone_gray": (0.41, 0.41, 0.41, 1),
    "cobblestone_gray": (0.35, 0.35, 0.35, 1),
    "gravel_gray": (0.38, 0.38, 0.38, 1),
    "sand_beige": (0.76, 0.70, 0.50, 1),
    "clay_brown": (0.55, 0.47, 0.42, 1),
}

# =============================================================================
# 石材アイテム生成
# =============================================================================

def create_stone(location=(0, 0, 0), name="stone"):
    """滑らかな石ブロック - 面取りキューブ"""
    size = ITEM_SIZE * 0.9
    chamfer = size * 0.15  # より大きな面取りで滑らかに

    stone = create_chamfered_cube(
        size=(size, size, size),
        chamfer=chamfer,
        location=location,
        name=name
    )

    # 石マテリアル適用
    mat = create_material(
        "stone_material",
        color=COLORS["stone_gray"],
        metallic=0.0,
        roughness=0.7
    )
    apply_material(stone, mat)

    finalize_model(stone, category="item")
    return stone

def create_cobblestone(location=(0, 0, 0), name="cobblestone"):
    """ゴツゴツした石 - 複数の小キューブを不規則に配置"""
    objects = []

    # 中心の大きめキューブ
    center_size = ITEM_SIZE * 0.4
    center = create_chamfered_cube(
        size=(center_size, center_size, center_size),
        chamfer=center_size * 0.08,
        location=location,
        name=f"{name}_center"
    )
    objects.append(center)

    # 周囲に小さいキューブを不規則に配置
    positions = [
        (0.12, 0.08, 0.05),
        (-0.10, 0.10, -0.03),
        (0.08, -0.12, 0.08),
        (-0.08, -0.08, -0.08),
        (0.05, 0.05, 0.15),
        (-0.12, 0.02, 0.10),
    ]

    for i, (dx, dy, dz) in enumerate(positions):
        small_size = ITEM_SIZE * 0.25
        small = create_chamfered_cube(
            size=(small_size * 0.8, small_size * 0.9, small_size * 1.1),
            chamfer=small_size * 0.05,
            location=(location[0] + dx, location[1] + dy, location[2] + dz),
            name=f"{name}_small_{i}"
        )
        # 少し回転させて不規則に
        small.rotation_euler = (i * 0.3, i * 0.4, i * 0.5)
        objects.append(small)

    # 結合
    bpy.context.view_layer.objects.active = center
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    # マテリアル適用
    mat = create_material(
        "cobblestone_material",
        color=COLORS["cobblestone_gray"],
        metallic=0.0,
        roughness=0.8
    )
    apply_material(center, mat)

    finalize_model(center, category="item")
    return center

def create_gravel(location=(0, 0, 0), name="gravel"):
    """砂利 - 小さい八角形を複数配置"""
    objects = []

    # 小さい八角形を複数配置
    positions = [
        (0, 0, 0),
        (0.10, 0.05, 0.03),
        (-0.08, 0.08, 0.02),
        (0.05, -0.10, 0.04),
        (-0.10, -0.05, 0.01),
        (0.08, 0.08, 0.08),
        (-0.05, -0.08, 0.06),
        (0.02, 0.02, -0.05),
    ]

    for i, (dx, dy, dz) in enumerate(positions):
        radius = ITEM_SIZE * 0.15
        depth = ITEM_SIZE * 0.12

        oct = create_octagon(
            radius=radius,
            depth=depth,
            location=(location[0] + dx, location[1] + dy, location[2] + dz),
            name=f"{name}_piece_{i}"
        )
        # ランダムな回転
        oct.rotation_euler = (i * 0.7, i * 0.5, i * 0.9)
        objects.append(oct)

    # 結合
    bpy.context.view_layer.objects.active = objects[0]
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    # マテリアル適用
    mat = create_material(
        "gravel_material",
        color=COLORS["gravel_gray"],
        metallic=0.0,
        roughness=0.75
    )
    apply_material(objects[0], mat)

    finalize_model(objects[0], category="item")
    return objects[0]

def create_sand(location=(0, 0, 0), name="sand"):
    """砂の山 - 低い台形っぽい山"""
    # 台形を作成（底面が広く、上面が狭い）
    bottom_size = ITEM_SIZE * 0.9
    top_size = ITEM_SIZE * 0.5
    height = ITEM_SIZE * 0.5

    # 面取りキューブを使って台形風に
    # 底面
    base_verts = [
        (-bottom_size/2, -bottom_size/2, 0),
        (bottom_size/2, -bottom_size/2, 0),
        (bottom_size/2, bottom_size/2, 0),
        (-bottom_size/2, bottom_size/2, 0),
        # 上面
        (-top_size/2, -top_size/2, height),
        (top_size/2, -top_size/2, height),
        (top_size/2, top_size/2, height),
        (-top_size/2, top_size/2, height),
    ]

    faces = [
        (0, 1, 2, 3),  # 底面
        (7, 6, 5, 4),  # 上面
        (0, 4, 5, 1),  # 側面
        (1, 5, 6, 2),
        (2, 6, 7, 3),
        (3, 7, 4, 0),
    ]

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(base_verts, [], faces)
    mesh.update()

    sand = bpy.data.objects.new(name, mesh)
    sand.location = snap_vec(Vector(location))
    bpy.context.collection.objects.link(sand)

    # マテリアル適用
    mat = create_material(
        "sand_material",
        color=COLORS["sand_beige"],
        metallic=0.0,
        roughness=0.85
    )
    apply_material(sand, mat)

    finalize_model(sand, category="item")
    return sand

def create_clay(location=(0, 0, 0), name="clay"):
    """粘土のかたまり - 丸みを帯びた形"""
    # 面取りを大きくして丸みを出す
    size = ITEM_SIZE * 0.85
    chamfer = size * 0.25  # 大きな面取りで丸みを表現

    # 少し扁平な形に
    clay = create_chamfered_cube(
        size=(size, size, size * 0.8),
        chamfer=chamfer,
        location=location,
        name=name
    )

    # マテリアル適用
    mat = create_material(
        "clay_material",
        color=COLORS["clay_brown"],
        metallic=0.0,
        roughness=0.9
    )
    apply_material(clay, mat)

    finalize_model(clay, category="item")
    return clay

# =============================================================================
# 一括生成・エクスポート
# =============================================================================

def create_all_stones(clear=True):
    """全ての石材アイテムを生成"""
    if clear:
        clear_scene()

    # 横一列に配置
    spacing = 1.0
    models = []

    print("Creating stone items...")

    models.append(create_stone(location=(0 * spacing, 0, 0)))
    models.append(create_cobblestone(location=(1 * spacing, 0, 0)))
    models.append(create_gravel(location=(2 * spacing, 0, 0)))
    models.append(create_sand(location=(3 * spacing, 0, 0)))
    models.append(create_clay(location=(4 * spacing, 0, 0)))

    print(f"Created {len(models)} stone item models")
    return models

def export_all_stones(output_dir=""):
    """全ての石材アイテムを個別にエクスポート"""
    items = ["stone", "cobblestone", "gravel", "sand", "clay"]

    for item_name in items:
        # シーンをクリア
        clear_scene()

        # モデルを生成
        if item_name == "stone":
            obj = create_stone(name=item_name)
        elif item_name == "cobblestone":
            obj = create_cobblestone(name=item_name)
        elif item_name == "gravel":
            obj = create_gravel(name=item_name)
        elif item_name == "sand":
            obj = create_sand(name=item_name)
        elif item_name == "clay":
            obj = create_clay(name=item_name)

        # エクスポート
        filepath = os.path.join(output_dir, f"{item_name}.gltf")
        export_gltf(filepath, export_animations=False)
        print(f"Exported: {item_name}")

# =============================================================================
# 実行例
# =============================================================================

if __name__ == "__main__":
    # 全モデルを生成して表示
    create_all_stones()

    # 個別エクスポートする場合は下記を実行
    # export_all_stones(output_dir="/path/to/output")

    print("=== Stone Items Generation Complete ===")
    print("To export individually, run: export_all_stones(output_dir='your_path')")
