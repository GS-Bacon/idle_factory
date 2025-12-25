"""
防具アイテムモデル生成スクリプト
category: item (dropped_item: 0.4x0.4x0.4)

使い方:
1. _base.pyを実行して共通関数をロード
2. このスクリプトを実行
"""

import bpy
from mathutils import Vector
from math import pi
import os

# _base.py をロード
exec(open("tools/blender_scripts/_base.py").read())

# 出力ディレクトリ
script_dir = os.path.dirname(os.path.abspath(__file__))
project_root = os.path.dirname(os.path.dirname(script_dir))
OUTPUT_DIR = os.path.join(project_root, "assets", "models", "items", "armor")
os.makedirs(OUTPUT_DIR, exist_ok=True)
OUTPUT_DIR = OUTPUT_DIR + os.sep

# =============================================================================
# ヘルメット
# =============================================================================

def create_helmet(material_type, name):
    """ヘルメット生成（ドーム状の頭部防具）"""
    clear_scene()

    # ベース: ドーム状の形
    base = create_chamfered_cube(
        size=(0.3, 0.25, 0.2),
        chamfer=0.04,
        location=(0, 0, 0.1),
        name=f"{name}_base"
    )

    # 前面部分（顔周り）
    front = create_chamfered_cube(
        size=(0.26, 0.05, 0.12),
        chamfer=0.02,
        location=(0, -0.13, 0.04),
        name=f"{name}_front"
    )

    # 側面保護
    side_l = create_chamfered_cube(
        size=(0.04, 0.18, 0.14),
        chamfer=0.015,
        location=(-0.13, -0.04, 0.03),
        name=f"{name}_side_l"
    )

    side_r = create_chamfered_cube(
        size=(0.04, 0.18, 0.14),
        chamfer=0.015,
        location=(0.13, -0.04, 0.03),
        name=f"{name}_side_r"
    )

    # 統合
    bpy.context.view_layer.objects.active = base
    for obj in [front, side_l, side_r]:
        obj.select_set(True)
    bpy.ops.object.join()

    # マテリアル適用
    if material_type == "leather":
        mat = create_material(
            f"{name}_mat",
            color=(0.4, 0.25, 0.1, 1),
            metallic=0.0,
            roughness=0.9
        )
    elif material_type == "iron":
        mat = create_material(
            f"{name}_mat",
            color=(0.5, 0.5, 0.5, 1),
            metallic=1.0,
            roughness=0.5
        )
    else:  # steel
        mat = create_material(
            f"{name}_mat",
            color=(0.25, 0.25, 0.28, 1),
            metallic=1.0,
            roughness=0.4
        )

    apply_material(base, mat)
    finalize_model(base, category="item")

    # エクスポート
    export_gltf(f"{OUTPUT_DIR}{name}.gltf", export_animations=False)
    print(f"Generated: {name}")

# =============================================================================
# チェストプレート
# =============================================================================

def create_chestplate(material_type, name):
    """チェストプレート生成（胴体を覆う形）"""
    clear_scene()

    # 胴体本体
    torso = create_chamfered_cube(
        size=(0.32, 0.15, 0.28),
        chamfer=0.04,
        location=(0, 0, 0),
        name=f"{name}_torso"
    )

    # 肩パッド左
    shoulder_l = create_chamfered_cube(
        size=(0.08, 0.12, 0.08),
        chamfer=0.02,
        location=(-0.18, -0.02, 0.1),
        name=f"{name}_shoulder_l"
    )

    # 肩パッド右
    shoulder_r = create_chamfered_cube(
        size=(0.08, 0.12, 0.08),
        chamfer=0.02,
        location=(0.18, -0.02, 0.1),
        name=f"{name}_shoulder_r"
    )

    # 腹部プレート
    belly = create_chamfered_cube(
        size=(0.28, 0.12, 0.1),
        chamfer=0.03,
        location=(0, 0, -0.16),
        name=f"{name}_belly"
    )

    # 統合
    bpy.context.view_layer.objects.active = torso
    for obj in [shoulder_l, shoulder_r, belly]:
        obj.select_set(True)
    bpy.ops.object.join()

    # マテリアル適用
    if material_type == "leather":
        mat = create_material(
            f"{name}_mat",
            color=(0.45, 0.28, 0.12, 1),
            metallic=0.0,
            roughness=0.85
        )
    elif material_type == "iron":
        mat = create_material(
            f"{name}_mat",
            color=(0.48, 0.48, 0.48, 1),
            metallic=1.0,
            roughness=0.5
        )
    else:  # steel
        mat = create_material(
            f"{name}_mat",
            color=(0.22, 0.22, 0.25, 1),
            metallic=1.0,
            roughness=0.4
        )

    apply_material(torso, mat)
    finalize_model(torso, category="item")

    # エクスポート
    export_gltf(f"{OUTPUT_DIR}{name}.gltf", export_animations=False)
    print(f"Generated: {name}")

# =============================================================================
# レギンス
# =============================================================================

def create_leggings(material_type, name):
    """レギンス生成（腰から太ももを覆う形）"""
    clear_scene()

    # ウエストバンド
    waist = create_chamfered_cube(
        size=(0.3, 0.14, 0.06),
        chamfer=0.02,
        location=(0, 0, 0.12),
        name=f"{name}_waist"
    )

    # 左脚
    leg_l = create_chamfered_cube(
        size=(0.12, 0.12, 0.2),
        chamfer=0.025,
        location=(-0.08, 0, 0),
        name=f"{name}_leg_l"
    )

    # 右脚
    leg_r = create_chamfered_cube(
        size=(0.12, 0.12, 0.2),
        chamfer=0.025,
        location=(0.08, 0, 0),
        name=f"{name}_leg_r"
    )

    # 左膝パッド
    knee_l = create_chamfered_cube(
        size=(0.1, 0.08, 0.06),
        chamfer=0.015,
        location=(-0.08, -0.05, -0.08),
        name=f"{name}_knee_l"
    )

    # 右膝パッド
    knee_r = create_chamfered_cube(
        size=(0.1, 0.08, 0.06),
        chamfer=0.015,
        location=(0.08, -0.05, -0.08),
        name=f"{name}_knee_r"
    )

    # 統合
    bpy.context.view_layer.objects.active = waist
    for obj in [leg_l, leg_r, knee_l, knee_r]:
        obj.select_set(True)
    bpy.ops.object.join()

    # マテリアル適用
    if material_type == "leather":
        mat = create_material(
            f"{name}_mat",
            color=(0.42, 0.26, 0.11, 1),
            metallic=0.0,
            roughness=0.88
        )
    elif material_type == "iron":
        mat = create_material(
            f"{name}_mat",
            color=(0.46, 0.46, 0.46, 1),
            metallic=1.0,
            roughness=0.52
        )
    else:  # steel
        mat = create_material(
            f"{name}_mat",
            color=(0.23, 0.23, 0.26, 1),
            metallic=1.0,
            roughness=0.42
        )

    apply_material(waist, mat)
    finalize_model(waist, category="item")

    # エクスポート
    export_gltf(f"{OUTPUT_DIR}{name}.gltf", export_animations=False)
    print(f"Generated: {name}")

# =============================================================================
# ブーツ
# =============================================================================

def create_boots(material_type, name):
    """ブーツ生成（靴の形）"""
    clear_scene()

    # 左足ベース
    boot_l = create_chamfered_cube(
        size=(0.1, 0.16, 0.12),
        chamfer=0.02,
        location=(-0.08, 0, 0),
        name=f"{name}_boot_l"
    )

    # 左つま先
    toe_l = create_chamfered_cube(
        size=(0.08, 0.06, 0.08),
        chamfer=0.015,
        location=(-0.08, -0.1, -0.01),
        name=f"{name}_toe_l"
    )

    # 右足ベース
    boot_r = create_chamfered_cube(
        size=(0.1, 0.16, 0.12),
        chamfer=0.02,
        location=(0.08, 0, 0),
        name=f"{name}_boot_r"
    )

    # 右つま先
    toe_r = create_chamfered_cube(
        size=(0.08, 0.06, 0.08),
        chamfer=0.015,
        location=(0.08, -0.1, -0.01),
        name=f"{name}_toe_r"
    )

    # 左脛当て
    shin_l = create_chamfered_cube(
        size=(0.08, 0.06, 0.08),
        chamfer=0.015,
        location=(-0.08, 0.04, 0.08),
        name=f"{name}_shin_l"
    )

    # 右脛当て
    shin_r = create_chamfered_cube(
        size=(0.08, 0.06, 0.08),
        chamfer=0.015,
        location=(0.08, 0.04, 0.08),
        name=f"{name}_shin_r"
    )

    # 統合
    bpy.context.view_layer.objects.active = boot_l
    for obj in [toe_l, boot_r, toe_r, shin_l, shin_r]:
        obj.select_set(True)
    bpy.ops.object.join()

    # マテリアル適用
    if material_type == "leather":
        mat = create_material(
            f"{name}_mat",
            color=(0.38, 0.24, 0.1, 1),
            metallic=0.0,
            roughness=0.92
        )
    elif material_type == "iron":
        mat = create_material(
            f"{name}_mat",
            color=(0.44, 0.44, 0.44, 1),
            metallic=1.0,
            roughness=0.54
        )
    else:  # steel
        mat = create_material(
            f"{name}_mat",
            color=(0.2, 0.2, 0.23, 1),
            metallic=1.0,
            roughness=0.44
        )

    apply_material(boot_l, mat)
    finalize_model(boot_l, category="item")

    # エクスポート
    export_gltf(f"{OUTPUT_DIR}{name}.gltf", export_animations=False)
    print(f"Generated: {name}")

# =============================================================================
# 全モデル生成
# =============================================================================

def generate_all():
    """全防具モデルを生成"""
    print("=== Generating Armor Models ===")

    # ヘルメット
    create_helmet("leather", "leather_helmet")
    create_helmet("iron", "iron_helmet")
    create_helmet("steel", "steel_helmet")

    # チェストプレート
    create_chestplate("leather", "leather_chestplate")
    create_chestplate("iron", "iron_chestplate")
    create_chestplate("steel", "steel_chestplate")

    # レギンス
    create_leggings("leather", "leather_leggings")
    create_leggings("iron", "iron_leggings")
    create_leggings("steel", "steel_leggings")

    # ブーツ
    create_boots("leather", "leather_boots")
    create_boots("iron", "iron_boots")
    create_boots("steel", "steel_boots")

    print("=== All Armor Models Generated ===")

# 実行
if __name__ == "__main__":
    generate_all()
