"""
Machine Template - カテゴリ: machine (single_block_machine: 1.0x1.0x1.0)

使い方:
1. このテンプレートをコピー
2. {MODEL_NAME}, {EXPORT_NAME} を置換
3. # TODO セクションを実装

ポリゴン目安: 200-800 (max 1500)
原点: bottom_center (0, 0, 0)
"""

import bpy
from mathutils import Vector
from math import pi
import os
import sys

# _base.py をロード
script_dir = os.path.dirname(os.path.abspath(__file__))
base_path = os.path.join(os.path.dirname(script_dir), "_base.py")
exec(open(base_path).read())

# =============================================================================
# モデル定義
# =============================================================================

MODEL_NAME = "{MODEL_NAME}"  # 例: "electric_furnace"
EXPORT_NAME = "{EXPORT_NAME}"  # 例: "electric_furnace"


def create_model():
    """モデル生成"""
    clear_scene()

    parts = []

    # ==========================================================================
    # ベースフレーム（ほぼ全ての機械に必要）
    # ==========================================================================

    # オプション1: シンプルなフレーム
    # frame = create_machine_frame(width=0.9, depth=0.9, height=0.2, material="dark_steel")
    # parts.append(frame)

    # オプション2: ボディのみ
    # body = create_machine_body(width=0.9, depth=0.9, height=0.8, material="iron")
    # parts.append(body)

    # オプション3: タンク型
    # tank_parts = create_tank_body(radius=0.4, height=0.7, material="iron")
    # parts.extend(tank_parts)

    # ==========================================================================
    # TODO: メインの機能パーツ
    # ==========================================================================

    # 例: ギア
    # gear = create_gear(radius=0.25, thickness=0.08, teeth=8, location=(0, 0, 0.5), name="MainGear")
    # apply_preset_material(gear, "brass")
    # parts.append(gear)

    # 例: パイプ
    # pipe = create_pipe(radius=0.08, length=0.3, wall=0.015, location=(0.4, 0, 0.4), name="Outlet")
    # pipe.rotation_euler.y = pi / 2
    # apply_preset_material(pipe, "copper")
    # parts.append(pipe)

    # 例: モーター
    # motor = create_motor_housing(radius=0.15, height=0.12, location=(0, 0, 0.8), material="copper")
    # parts.append(motor)

    # ==========================================================================
    # 装飾パーツ（オプション）
    # ==========================================================================

    # 四隅のボルト
    # bolts = create_corner_bolts(width=0.9, depth=0.9, z_pos=0.8, bolt_size=0.04, material="iron")
    # parts.extend(bolts)

    # 補強リブ
    # ribs = create_reinforcement_ribs(width=0.9, depth=0.9, z_pos=0.4, material="dark_steel")
    # parts.extend(ribs)

    # 円形配置のボルト（タンク用）
    # circle_bolts = add_decorative_bolts_circle(radius=0.38, z_pos=0.7, count=8, material="brass")
    # parts.extend(circle_bolts)

    # ステータスライト
    # light = create_accent_light(size=0.04, location=(0.4, 0, 0.6), color_preset="power")
    # parts.append(light)

    # ==========================================================================
    # 結合 & 仕上げ
    # ==========================================================================

    if len(parts) == 1:
        result = parts[0]
    else:
        result = join_all_meshes(parts, MODEL_NAME)

    result.name = MODEL_NAME

    # バリデーション
    print_validation_report(result, category="machine")

    # 仕上げ（原点をbottom_centerに設定）
    finalize_model(result, category="machine")

    return result


# =============================================================================
# エクスポート
# =============================================================================

def export():
    """エクスポート"""
    model = create_model()

    output_dir = os.path.join(script_dir, "..", "..", "assets", "models", "machines")
    os.makedirs(output_dir, exist_ok=True)

    filepath = os.path.join(output_dir, f"{EXPORT_NAME}.gltf")
    export_gltf(filepath, export_animations=False)
    print(f"Exported: {filepath}")


if __name__ == "__main__":
    export()
