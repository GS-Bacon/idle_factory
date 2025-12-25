"""
鉱石アイテムモデル生成スクリプト
dropped_item サイズ: 0.4x0.4x0.4
"""

exec(open("tools/blender_scripts/_base.py").read())

import random

# =============================================================================
# 鉱石マテリアル定義
# =============================================================================

ORE_MATERIALS = {
    "iron_ore": {
        "base": {"color": (0.35, 0.35, 0.35, 1), "metallic": 0.0, "roughness": 0.8},
        "vein": {"color": (0.5, 0.5, 0.5, 1), "metallic": 0.6, "roughness": 0.4},
    },
    "copper_ore": {
        "base": {"color": (0.45, 0.32, 0.25, 1), "metallic": 0.0, "roughness": 0.8},
        "vein": {"color": (0.72, 0.45, 0.20, 1), "metallic": 0.8, "roughness": 0.3},
    },
    "tin_ore": {
        "base": {"color": (0.40, 0.40, 0.40, 1), "metallic": 0.0, "roughness": 0.8},
        "vein": {"color": (0.65, 0.65, 0.70, 1), "metallic": 0.7, "roughness": 0.4},
    },
    "coal_ore": {
        "base": {"color": (0.25, 0.25, 0.25, 1), "metallic": 0.0, "roughness": 0.9},
        "vein": {"color": (0.10, 0.10, 0.10, 1), "metallic": 0.0, "roughness": 0.7},
    },
    "gold_ore": {
        "base": {"color": (0.45, 0.40, 0.30, 1), "metallic": 0.0, "roughness": 0.8},
        "vein": {"color": (0.85, 0.75, 0.20, 1), "metallic": 1.0, "roughness": 0.2},
    },
    "nickel_ore": {
        "base": {"color": (0.40, 0.38, 0.35, 1), "metallic": 0.0, "roughness": 0.8},
        "vein": {"color": (0.70, 0.70, 0.72, 1), "metallic": 0.9, "roughness": 0.3},
    },
    "sulfur_ore": {
        "base": {"color": (0.50, 0.48, 0.35, 1), "metallic": 0.0, "roughness": 0.8},
        "vein": {"color": (0.90, 0.85, 0.20, 1), "metallic": 0.0, "roughness": 0.5},
    },
    "uranium_ore": {
        "base": {"color": (0.35, 0.40, 0.35, 1), "metallic": 0.0, "roughness": 0.8},
        "vein": {"color": (0.40, 0.70, 0.35, 1), "metallic": 0.2, "roughness": 0.6},
    },
}

# =============================================================================
# 鉱石生成関数
# =============================================================================

def create_irregular_rock(size=0.4, seed=0, name="Rock"):
    """不規則な岩形状を生成"""
    random.seed(seed)

    # ベースとなる中心キューブ
    base_size = size * 0.6
    base = create_chamfered_cube(
        (base_size, base_size, base_size),
        chamfer=base_size * 0.15,
        location=(0, 0, 0),
        name=f"{name}_base"
    )

    objects = [base]

    # ランダムに小さなキューブを配置して岩っぽくする
    num_chunks = random.randint(4, 7)
    for i in range(num_chunks):
        # ランダムサイズ
        chunk_size = size * random.uniform(0.15, 0.35)

        # ランダム位置（ベースから少しはみ出す）
        angle = random.uniform(0, 2 * pi)
        distance = size * random.uniform(0.15, 0.30)
        x = cos(angle) * distance
        y = sin(angle) * distance
        z = random.uniform(-size * 0.15, size * 0.15)

        chunk = create_chamfered_cube(
            (chunk_size, chunk_size, chunk_size),
            chamfer=chunk_size * 0.2,
            location=(x, y, z),
            name=f"{name}_chunk_{i}"
        )

        # ランダム回転で自然な感じに
        chunk.rotation_euler.x = random.uniform(0, pi / 6)
        chunk.rotation_euler.y = random.uniform(0, pi / 6)
        chunk.rotation_euler.z = random.uniform(0, 2 * pi)

        objects.append(chunk)

    # すべて結合
    bpy.context.view_layer.objects.active = base
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    return base

def create_ore_vein(size=0.4, seed=0, name="Vein"):
    """鉱脈部分（光沢のある小さな結晶風）"""
    random.seed(seed + 100)

    objects = []

    # 鉱脈は2-4個の小さな結晶
    num_veins = random.randint(2, 4)
    for i in range(num_veins):
        vein_size = size * random.uniform(0.08, 0.15)

        # ランダム位置
        angle = random.uniform(0, 2 * pi)
        distance = size * random.uniform(0.20, 0.35)
        x = cos(angle) * distance
        y = sin(angle) * distance
        z = random.uniform(-size * 0.20, size * 0.20)

        vein = create_chamfered_cube(
            (vein_size, vein_size, vein_size * 0.8),
            chamfer=vein_size * 0.1,
            location=(x, y, z),
            name=f"{name}_crystal_{i}"
        )

        # 結晶らしく尖った感じに回転
        vein.rotation_euler.x = random.uniform(0, pi / 4)
        vein.rotation_euler.y = random.uniform(0, pi / 4)
        vein.rotation_euler.z = random.uniform(0, 2 * pi)

        objects.append(vein)

    if not objects:
        return None

    # 鉱脈を結合
    bpy.context.view_layer.objects.active = objects[0]
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    return objects[0]

def create_ore(ore_name, seed=0):
    """鉱石モデルを生成"""
    clear_scene()

    size = 0.4

    # 岩ベース生成
    rock = create_irregular_rock(size=size, seed=seed, name=f"{ore_name}_rock")

    # 鉱脈生成
    vein = create_ore_vein(size=size, seed=seed, name=f"{ore_name}_vein")

    # マテリアル適用
    if ore_name in ORE_MATERIALS:
        mat_data = ORE_MATERIALS[ore_name]

        # 岩ベースのマテリアル
        rock_mat = create_material(
            f"{ore_name}_base",
            color=mat_data["base"]["color"],
            metallic=mat_data["base"]["metallic"],
            roughness=mat_data["base"]["roughness"]
        )
        apply_material(rock, rock_mat)

        # 鉱脈のマテリアル
        if vein:
            vein_mat = create_material(
                f"{ore_name}_vein",
                color=mat_data["vein"]["color"],
                metallic=mat_data["vein"]["metallic"],
                roughness=mat_data["vein"]["roughness"]
            )
            apply_material(vein, vein_mat)

    # 岩と鉱脈を結合
    objects = [rock]
    if vein:
        objects.append(vein)

    bpy.context.view_layer.objects.active = rock
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()

    # 最終処理（dropped_itemなのでcenter）
    finalize_model(rock, category="item")

    return rock

# =============================================================================
# 全鉱石生成
# =============================================================================

def main():
    """全鉱石を生成してエクスポート"""
    ores = [
        "iron_ore",
        "copper_ore",
        "tin_ore",
        "coal_ore",
        "gold_ore",
        "nickel_ore",
        "sulfur_ore",
        "uranium_ore",
    ]

    # スクリプトのディレクトリからプロジェクトルートを推定
    import os
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(os.path.dirname(script_dir))
    output_dir = os.path.join(project_root, "assets", "models", "items")

    # 出力ディレクトリを作成
    os.makedirs(output_dir, exist_ok=True)

    for ore_name in ores:
        print(f"\n=== Generating {ore_name} ===")

        # シード値は鉱石名から生成（再現性のため）
        seed = hash(ore_name) % 10000

        # 鉱石生成
        ore_obj = create_ore(ore_name, seed=seed)

        # エクスポート
        output_path = os.path.join(output_dir, f"{ore_name}.gltf")
        export_gltf(output_path, export_animations=False)

        print(f"✓ {ore_name} exported to {output_path}")

    print("\n=== All ores generated successfully ===")

# =============================================================================
# 実行
# =============================================================================

if __name__ == "__main__":
    main()
