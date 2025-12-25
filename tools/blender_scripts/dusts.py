"""
粉末アイテムモデル生成スクリプト
カテゴリ: item (dropped_item: 0.4x0.4x0.4)

使い方:
1. Blenderで新規ファイル作成
2. _base.pyを実行
3. このスクリプトを実行
"""

import bpy
from mathutils import Vector
from math import pi
import random

# _base.pyの関数を使用
# create_chamfered_cube, create_octagon, create_material, apply_material, finalize_model, export_gltf

# =============================================================================
# 粉末の色と材質定義
# =============================================================================

DUST_MATERIALS = {
    "iron_dust": {
        "color": (0.22, 0.22, 0.22, 1),
        "metallic": 0.0,
        "roughness": 0.9
    },
    "copper_dust": {
        "color": (0.60, 0.38, 0.18, 1),
        "metallic": 0.0,
        "roughness": 0.85
    },
    "tin_dust": {
        "color": (0.65, 0.65, 0.67, 1),
        "metallic": 0.0,
        "roughness": 0.88
    },
    "gold_dust": {
        "color": (0.85, 0.70, 0.0, 1),
        "metallic": 0.0,
        "roughness": 0.8
    },
    "coal_dust": {
        "color": (0.05, 0.05, 0.05, 1),
        "metallic": 0.0,
        "roughness": 0.95
    },
    "sulfur": {
        "color": (0.85, 0.85, 0.15, 1),
        "metallic": 0.0,
        "roughness": 0.9
    }
}

# =============================================================================
# 粉末モデル生成
# =============================================================================

def create_dust_particle(size, particle_type="cube"):
    """粉末の個別パーティクルを生成"""
    if particle_type == "cube":
        return create_chamfered_cube(
            size=(size, size, size),
            chamfer=size * 0.15,
            location=(0, 0, 0),
            name="dust_particle"
        )
    else:  # octagon
        return create_octagon(
            radius=size * 0.6,
            depth=size,
            location=(0, 0, 0),
            name="dust_particle"
        )

def create_dust(name, material_props):
    """粉末アイテムを生成（小さな山のような形）"""
    # アイテムボックス: 0.4x0.4x0.4
    # 粉末: 小さいキューブと八角形の集まり

    particles = []
    random.seed(hash(name))  # 名前で決定的なシード

    # 中央の大きめパーティクル（ベース）
    base_size = 0.12
    base = create_dust_particle(base_size, "cube")
    base.location = Vector((0, 0, 0))
    particles.append(base)

    # 周辺の小さなパーティクル（山状に配置）
    num_particles = 8
    for i in range(num_particles):
        particle_type = "cube" if i % 2 == 0 else "octagon"
        size = random.uniform(0.06, 0.09)

        # 円状に配置（少しランダムに）
        angle = (i / num_particles) * 2 * pi
        radius = random.uniform(0.08, 0.12)
        x = radius * random.uniform(0.7, 1.3) * (1 if i % 2 == 0 else -1) * 0.5
        y = radius * random.uniform(0.7, 1.3) * (1 if i % 3 == 0 else -1) * 0.5
        z = random.uniform(-0.04, 0.06)

        particle = create_dust_particle(size, particle_type)
        particle.location = Vector((x, y, z))

        # 少し回転させる
        particle.rotation_euler.z = random.uniform(0, pi / 4)

        particles.append(particle)

    # 全パーティクルを結合
    bpy.context.view_layer.objects.active = base
    for obj in particles:
        obj.select_set(True)
    bpy.ops.object.join()

    dust = base
    dust.name = name

    # マテリアル適用
    mat = create_material(
        name=f"{name}_material",
        color=material_props["color"],
        metallic=material_props["metallic"],
        roughness=material_props["roughness"]
    )
    apply_material(dust, mat)

    # アイテムカテゴリとして中心原点に設定
    finalize_model(dust, category="item")

    return dust

# =============================================================================
# 全粉末を生成
# =============================================================================

def create_all_dusts():
    """全ての粉末モデルを生成してエクスポート"""
    import os

    output_dir = bpy.path.abspath("//models/items/")
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    for dust_name, mat_props in DUST_MATERIALS.items():
        print(f"Creating {dust_name}...")

        # シーンクリア
        clear_scene()

        # 粉末生成
        dust = create_dust(dust_name, mat_props)

        # エクスポート
        export_path = os.path.join(output_dir, f"{dust_name}.gltf")
        export_gltf(export_path, export_animations=False)

        print(f"  Exported: {export_path}")

    print("All dusts created successfully!")

# =============================================================================
# 実行
# =============================================================================

if __name__ == "__main__":
    create_all_dusts()
