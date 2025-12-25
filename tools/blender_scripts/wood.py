"""
木材アイテムモデル生成スクリプト
カテゴリ: item (dropped_item: 0.4x0.4x0.4)

モデル:
- log: 丸太（八角柱、横倒し）
- wood_plank: 板（平たい直方体）
- stick: 棒（細い八角柱）

使用方法:
1. _base.py を実行してベースモジュールをロード
2. このスクリプトを実行してモデルを生成
3. 各モデルは個別にエクスポート可能
"""

import bpy
from mathutils import Vector
from math import pi
import os

# _base.py をロード
exec(open("tools/blender_scripts/_base.py").read())

# =============================================================================
# 定数
# =============================================================================

ITEM_SIZE = 0.4  # dropped_item のサイズ上限

# =============================================================================
# モデル生成関数
# =============================================================================

def create_log(location=(0, 0, 0), name="log"):
    """丸太モデル（八角柱、横倒し）"""
    clear_scene()

    # 八角柱を作成（直径0.3、長さ0.4）
    radius = 0.15  # 直径0.3 -> 半径0.15
    length = 0.4

    log = create_octagonal_prism(
        radius=radius,
        height=length,
        location=location,
        name=name
    )

    # Y軸方向に横倒し（90度回転）
    log.rotation_euler.x = pi / 2

    # 樹皮色のマテリアル適用
    # woodプリセットを使用（茶色系）
    apply_preset_material(log, "wood")

    # トランスフォーム適用と原点設定
    finalize_model(log, category="item")

    return log


def create_wood_plank(location=(0, 0, 0), name="wood_plank"):
    """板モデル（平たい直方体）"""
    clear_scene()

    # 平たい面取りキューブ（0.35x0.35x0.08）
    size = (0.35, 0.35, 0.08)
    chamfer = 0.01  # 小さめの面取り

    plank = create_chamfered_cube(
        size=size,
        chamfer=chamfer,
        location=location,
        name=name
    )

    # 木目色のマテリアル適用
    # woodプリセットよりやや明るい色に調整
    mat = create_material(
        name="wood_plank_mat",
        color=(0.62, 0.48, 0.12, 1),  # やや明るい木目色
        metallic=0.0,
        roughness=0.8
    )
    apply_material(plank, mat)

    # トランスフォーム適用と原点設定
    finalize_model(plank, category="item")

    return plank


def create_stick(location=(0, 0, 0), name="stick"):
    """棒モデル（細い八角柱）"""
    clear_scene()

    # 細長い八角柱（直径0.04、長さ0.38）
    radius = 0.02  # 直径0.04 -> 半径0.02
    length = 0.38

    stick = create_octagonal_prism(
        radius=radius,
        height=length,
        location=location,
        name=name
    )

    # 立てた状態（デフォルトのZ軸方向）のまま
    # 回転なし

    # 木材色のマテリアル適用
    apply_preset_material(stick, "wood")

    # トランスフォーム適用と原点設定
    finalize_model(stick, category="item")

    return stick


# =============================================================================
# エクスポート関数
# =============================================================================

def export_log(output_dir="./"):
    """丸太モデルをエクスポート"""
    log = create_log()
    filepath = os.path.join(output_dir, "log.gltf")
    export_gltf(filepath, export_animations=False)
    print(f"Exported log to {filepath}")


def export_wood_plank(output_dir="./"):
    """板モデルをエクスポート"""
    plank = create_wood_plank()
    filepath = os.path.join(output_dir, "wood_plank.gltf")
    export_gltf(filepath, export_animations=False)
    print(f"Exported wood_plank to {filepath}")


def export_stick(output_dir="./"):
    """棒モデルをエクスポート"""
    stick = create_stick()
    filepath = os.path.join(output_dir, "stick.gltf")
    export_gltf(filepath, export_animations=False)
    print(f"Exported stick to {filepath}")


def export_all_wood_items(output_dir="./"):
    """全ての木材アイテムをエクスポート"""
    export_log(output_dir)
    export_wood_plank(output_dir)
    export_stick(output_dir)
    print("All wood items exported successfully!")


# =============================================================================
# 実行例
# =============================================================================

if __name__ == "__main__":
    # 全アイテムをエクスポート
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(os.path.dirname(script_dir))
    output_dir = os.path.join(project_root, "assets", "models", "items")
    os.makedirs(output_dir, exist_ok=True)

    export_all_wood_items(output_dir)

    print("=== Wood Items Script Loaded ===")
    print("Available functions:")
    print("  create_log() - 丸太モデル生成")
    print("  create_wood_plank() - 板モデル生成")
    print("  create_stick() - 棒モデル生成")
    print("  export_log(output_dir) - 丸太エクスポート")
    print("  export_wood_plank(output_dir) - 板エクスポート")
    print("  export_stick(output_dir) - 棒エクスポート")
    print("  export_all_wood_items(output_dir) - 全アイテムエクスポート")
