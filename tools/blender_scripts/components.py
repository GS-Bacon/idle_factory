"""
Industrial Lowpoly Style - Intermediate Components
style-guide.json v1.0.0 準拠

アイテムカテゴリ: dropped_item (0.4x0.4x0.4)

モデル:
- circuit_board: 緑色の基板、チップ付き
- iron_mechanical_component: ギアとロッドの組み合わせ
- bearing: ベアリング（リング状）
- piston: ピストン
- advanced_circuit: より複雑な基板
- electric_motor: モーター形状
- battery: 円筒形バッテリー
- heat_exchanger: フィン付きブロック
- filter: メッシュ状のフィルター
- processor: 小さいチップ
- solar_cell: 青い板
"""

import bpy
from mathutils import Vector, Matrix
from math import pi, cos, sin
import os
import sys

# _base.pyのパスを追加
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.append(script_dir)

# 基本モジュールをインポート
from _base import (
    create_chamfered_cube,
    create_octagonal_prism,
    create_piston,
    create_gear,
    create_material,
    apply_preset_material,
    apply_material,
    finalize_model,
    export_gltf,
    clear_scene,
    GRID_UNIT,
    snap_vec
)

# =============================================================================
# 電子部品
# =============================================================================

def create_circuit_board(name="circuit_board"):
    """回路基板（緑色、金色チップ付き）"""
    clear_scene()

    # 基板本体（緑色）
    board_size = (0.25, 0.25, 0.02)
    board = create_chamfered_cube(
        size=board_size,
        chamfer=0.005,
        location=(0, 0, 0),
        name=f"{name}_board"
    )

    # 緑色のマテリアル
    mat_board = create_material(
        "circuit_green",
        color=(0.1, 0.5, 0.2, 1),
        metallic=0.0,
        roughness=0.6
    )
    apply_material(board, mat_board)

    # チップを追加（金色）
    chip_positions = [
        (-0.07, -0.07, board_size[2] / 2 + 0.01),
        (0.07, -0.07, board_size[2] / 2 + 0.01),
        (0.0, 0.05, board_size[2] / 2 + 0.01),
    ]

    mat_chip = create_material(
        "chip_gold",
        color=(0.79, 0.64, 0.15, 1),
        metallic=1.0,
        roughness=0.3
    )

    for i, pos in enumerate(chip_positions):
        chip = create_chamfered_cube(
            size=(0.04, 0.04, 0.02),
            chamfer=0.002,
            location=pos,
            name=f"{name}_chip_{i}"
        )
        apply_material(chip, mat_chip)

    # すべてを結合
    bpy.context.view_layer.objects.active = board
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.join()

    board.name = name
    finalize_model(board, category="item")
    return board


def create_advanced_circuit(name="advanced_circuit"):
    """高度な回路基板（より多くのチップ）"""
    clear_scene()

    # 基板本体（青緑色）
    board_size = (0.28, 0.28, 0.025)
    board = create_chamfered_cube(
        size=board_size,
        chamfer=0.005,
        location=(0, 0, 0),
        name=f"{name}_board"
    )

    mat_board = create_material(
        "circuit_advanced",
        color=(0.05, 0.4, 0.4, 1),
        metallic=0.0,
        roughness=0.5
    )
    apply_material(board, mat_board)

    # より多くのチップ
    chip_positions = [
        (-0.08, -0.08, board_size[2] / 2 + 0.015),
        (0.08, -0.08, board_size[2] / 2 + 0.015),
        (-0.08, 0.08, board_size[2] / 2 + 0.015),
        (0.08, 0.08, board_size[2] / 2 + 0.015),
        (0.0, 0.0, board_size[2] / 2 + 0.02),
    ]

    mat_chip_gold = create_material(
        "chip_gold_adv",
        color=(0.79, 0.64, 0.15, 1),
        metallic=1.0,
        roughness=0.3
    )

    mat_chip_silver = create_material(
        "chip_silver",
        color=(0.7, 0.7, 0.7, 1),
        metallic=1.0,
        roughness=0.2
    )

    for i, pos in enumerate(chip_positions):
        chip_size = (0.06, 0.06, 0.03) if i == 4 else (0.04, 0.04, 0.02)
        chip = create_chamfered_cube(
            size=chip_size,
            chamfer=0.002,
            location=pos,
            name=f"{name}_chip_{i}"
        )
        apply_material(chip, mat_chip_gold if i == 4 else mat_chip_silver)

    bpy.context.view_layer.objects.active = board
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.join()

    board.name = name
    finalize_model(board, category="item")
    return board


def create_processor(name="processor"):
    """プロセッサチップ（小型）"""
    clear_scene()

    # チップ本体（黒）
    chip = create_chamfered_cube(
        size=(0.15, 0.15, 0.03),
        chamfer=0.005,
        location=(0, 0, 0),
        name=f"{name}_body"
    )

    mat_chip = create_material(
        "processor_black",
        color=(0.1, 0.1, 0.1, 1),
        metallic=0.2,
        roughness=0.3
    )
    apply_material(chip, mat_chip)

    # 金色のピン（下部）
    pins_count = 6
    pin_spacing = 0.02
    start_offset = -(pins_count - 1) * pin_spacing / 2

    mat_pin = create_material(
        "processor_pin",
        color=(0.79, 0.64, 0.15, 1),
        metallic=1.0,
        roughness=0.3
    )

    for i in range(pins_count):
        for j in range(2):  # 両側
            x = start_offset + i * pin_spacing
            y = -0.08 if j == 0 else 0.08
            pin = create_chamfered_cube(
                size=(0.01, 0.01, 0.02),
                chamfer=0.001,
                location=(x, y, -0.025),
                name=f"{name}_pin_{i}_{j}"
            )
            apply_material(pin, mat_pin)

    bpy.context.view_layer.objects.active = chip
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.join()

    chip.name = name
    finalize_model(chip, category="item")
    return chip


def create_solar_cell(name="solar_cell"):
    """太陽電池パネル（青い板）"""
    clear_scene()

    # パネル本体（濃い青）
    panel = create_chamfered_cube(
        size=(0.3, 0.3, 0.02),
        chamfer=0.003,
        location=(0, 0, 0),
        name=f"{name}_panel"
    )

    mat_panel = create_material(
        "solar_blue",
        color=(0.1, 0.2, 0.5, 1),
        metallic=0.3,
        roughness=0.2
    )
    apply_material(panel, mat_panel)

    # グリッドライン（銀色）
    mat_grid = create_material(
        "solar_grid",
        color=(0.7, 0.7, 0.7, 1),
        metallic=1.0,
        roughness=0.3
    )

    # 縦線
    for i in range(4):
        x = -0.12 + i * 0.08
        line = create_chamfered_cube(
            size=(0.005, 0.28, 0.005),
            chamfer=0.001,
            location=(x, 0, 0.0125),
            name=f"{name}_vline_{i}"
        )
        apply_material(line, mat_grid)

    # 横線
    for i in range(4):
        y = -0.12 + i * 0.08
        line = create_chamfered_cube(
            size=(0.28, 0.005, 0.005),
            chamfer=0.001,
            location=(0, y, 0.0125),
            name=f"{name}_hline_{i}"
        )
        apply_material(line, mat_grid)

    bpy.context.view_layer.objects.active = panel
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.join()

    panel.name = name
    finalize_model(panel, category="item")
    return panel

# =============================================================================
# 機械部品
# =============================================================================

def create_iron_mechanical_component(name="iron_mechanical_component"):
    """鉄製機械部品（ギアとロッドの組み合わせ）"""
    clear_scene()

    # 中央のギア
    gear = create_gear(
        radius=0.12,
        thickness=0.04,
        teeth=8,
        hole_radius=0.02,
        location=(0, 0, 0),
        name=f"{name}_gear"
    )

    # ロッド（縦）
    rod = create_octagonal_prism(
        radius=0.02,
        height=0.25,
        location=(0, 0, 0),
        name=f"{name}_rod"
    )

    # ロッド（横）
    rod2 = create_octagonal_prism(
        radius=0.015,
        height=0.2,
        location=(0, 0, 0),
        name=f"{name}_rod2"
    )
    rod2.rotation_euler.y = pi / 2

    # すべてを結合
    bpy.context.view_layer.objects.active = gear
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.join()

    gear.name = name
    apply_preset_material(gear, "iron")
    finalize_model(gear, category="item")
    return gear


def create_bearing_item(name="bearing"):
    """ベアリング（リング状）"""
    clear_scene()

    # 外側のリング
    outer_ring = create_octagonal_prism(
        radius=0.1,
        height=0.05,
        location=(0, 0, 0),
        name=f"{name}_outer"
    )

    # 内側のリング（くり抜き用）
    inner_ring = create_octagonal_prism(
        radius=0.06,
        height=0.06,
        location=(0, 0, 0),
        name=f"{name}_inner"
    )

    # Boolean差分
    mod = outer_ring.modifiers.new("Bool", 'BOOLEAN')
    mod.operation = 'DIFFERENCE'
    mod.object = inner_ring
    bpy.context.view_layer.objects.active = outer_ring
    bpy.ops.object.modifier_apply(modifier="Bool")
    bpy.data.objects.remove(inner_ring)

    outer_ring.name = name
    apply_preset_material(outer_ring, "dark_steel")
    finalize_model(outer_ring, category="item")
    return outer_ring


def create_piston_item(name="piston"):
    """ピストンアイテム"""
    clear_scene()

    piston = create_piston(
        rod_radius=0.03,
        rod_length=0.2,
        head_size=(0.12, 0.12, 0.06),
        location=(0, 0, 0),
        name=name
    )

    apply_preset_material(piston, "iron")
    finalize_model(piston, category="item")
    return piston


def create_electric_motor(name="electric_motor"):
    """電気モーター"""
    clear_scene()

    # モーター本体（円筒形）
    body = create_octagonal_prism(
        radius=0.1,
        height=0.15,
        location=(0, 0, 0),
        name=f"{name}_body"
    )

    # シャフト
    shaft = create_octagonal_prism(
        radius=0.02,
        height=0.08,
        location=(0, 0, 0.115),
        name=f"{name}_shaft"
    )

    # 端子（銅色）
    mat_terminal = create_material(
        "motor_terminal",
        color=(0.72, 0.45, 0.20, 1),
        metallic=1.0,
        roughness=0.4
    )

    terminal1 = create_chamfered_cube(
        size=(0.02, 0.02, 0.04),
        chamfer=0.002,
        location=(0.08, 0, -0.04),
        name=f"{name}_terminal1"
    )
    apply_material(terminal1, mat_terminal)

    terminal2 = create_chamfered_cube(
        size=(0.02, 0.02, 0.04),
        chamfer=0.002,
        location=(-0.08, 0, -0.04),
        name=f"{name}_terminal2"
    )
    apply_material(terminal2, mat_terminal)

    # 結合
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.join()

    body.name = name
    apply_preset_material(body, "dark_steel")
    finalize_model(body, category="item")
    return body


def create_battery_item(name="battery"):
    """円筒形バッテリー"""
    clear_scene()

    # バッテリー本体（円筒、濃い青）
    body = create_octagonal_prism(
        radius=0.06,
        height=0.2,
        location=(0, 0, 0),
        name=f"{name}_body"
    )

    mat_body = create_material(
        "battery_blue",
        color=(0.1, 0.2, 0.4, 1),
        metallic=0.1,
        roughness=0.5
    )
    apply_material(body, mat_body)

    # プラス端子（金色）
    terminal_plus = create_octagonal_prism(
        radius=0.02,
        height=0.015,
        location=(0, 0, 0.1075),
        name=f"{name}_plus"
    )

    mat_terminal = create_material(
        "battery_terminal",
        color=(0.79, 0.64, 0.15, 1),
        metallic=1.0,
        roughness=0.3
    )
    apply_material(terminal_plus, mat_terminal)

    # 結合
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.join()

    body.name = name
    finalize_model(body, category="item")
    return body


def create_heat_exchanger(name="heat_exchanger"):
    """ヒートエクスチェンジャー（フィン付きブロック）"""
    clear_scene()

    # ベース
    base = create_chamfered_cube(
        size=(0.25, 0.25, 0.06),
        chamfer=0.005,
        location=(0, 0, 0),
        name=f"{name}_base"
    )

    # フィン
    fin_count = 6
    fin_spacing = 0.04
    start_offset = -(fin_count - 1) * fin_spacing / 2

    for i in range(fin_count):
        y = start_offset + i * fin_spacing
        fin = create_chamfered_cube(
            size=(0.22, 0.01, 0.1),
            chamfer=0.002,
            location=(0, y, 0.08),
            name=f"{name}_fin_{i}"
        )

    # 結合
    bpy.context.view_layer.objects.active = base
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.join()

    base.name = name
    apply_preset_material(base, "copper")
    finalize_model(base, category="item")
    return base


def create_filter_item(name="filter"):
    """フィルター（メッシュ状）"""
    clear_scene()

    # フレーム
    frame = create_chamfered_cube(
        size=(0.25, 0.25, 0.05),
        chamfer=0.005,
        location=(0, 0, 0),
        name=f"{name}_frame"
    )

    # 内側をくり抜き
    inner = create_chamfered_cube(
        size=(0.18, 0.18, 0.06),
        chamfer=0.003,
        location=(0, 0, 0),
        name=f"{name}_inner"
    )

    mod = frame.modifiers.new("Bool", 'BOOLEAN')
    mod.operation = 'DIFFERENCE'
    mod.object = inner
    bpy.context.view_layer.objects.active = frame
    bpy.ops.object.modifier_apply(modifier="Bool")
    bpy.data.objects.remove(inner)

    # メッシュライン（薄い金属色）
    mat_mesh = create_material(
        "filter_mesh",
        color=(0.5, 0.5, 0.5, 1),
        metallic=0.5,
        roughness=0.6
    )

    # 縦線
    for i in range(5):
        x = -0.08 + i * 0.04
        line = create_chamfered_cube(
            size=(0.005, 0.18, 0.005),
            chamfer=0.001,
            location=(x, 0, 0),
            name=f"{name}_vline_{i}"
        )
        apply_material(line, mat_mesh)

    # 横線
    for i in range(5):
        y = -0.08 + i * 0.04
        line = create_chamfered_cube(
            size=(0.18, 0.005, 0.005),
            chamfer=0.001,
            location=(0, y, 0),
            name=f"{name}_hline_{i}"
        )
        apply_material(line, mat_mesh)

    # 結合
    bpy.context.view_layer.objects.active = frame
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.join()

    frame.name = name
    apply_preset_material(frame, "iron")
    finalize_model(frame, category="item")
    return frame

# =============================================================================
# モデル生成・エクスポート
# =============================================================================

def generate_all_models():
    """すべてのモデルを生成してエクスポート"""
    output_dir = os.path.join(script_dir, "..", "..", "assets", "models", "items")
    os.makedirs(output_dir, exist_ok=True)

    models = [
        # 電子部品
        ("circuit_board", create_circuit_board),
        ("advanced_circuit", create_advanced_circuit),
        ("processor", create_processor),
        ("solar_cell", create_solar_cell),

        # 機械部品
        ("iron_mechanical_component", create_iron_mechanical_component),
        ("bearing", create_bearing_item),
        ("piston", create_piston_item),
        ("electric_motor", create_electric_motor),
        ("battery", create_battery_item),
        ("heat_exchanger", create_heat_exchanger),
        ("filter", create_filter_item),
    ]

    generated = []
    for name, create_func in models:
        print(f"\n=== Generating {name} ===")
        create_func(name)
        filepath = os.path.join(output_dir, f"{name}.gltf")
        export_gltf(filepath, export_animations=False)
        generated.append(name)

    print("\n=== All models generated ===")
    print(f"Total: {len(generated)} models")
    print(f"Output directory: {output_dir}")
    for model in generated:
        print(f"  - {model}.gltf")

# =============================================================================
# 実行
# =============================================================================

if __name__ == "__main__":
    generate_all_models()
