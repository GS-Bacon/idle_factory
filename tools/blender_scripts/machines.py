"""
Machine Models - Industrial Lowpoly Style
Create Mod風の工業機械モデル群
カテゴリ: machine (single_block_machine: 1.0x1.0x1.0)

使い方:
1. Blenderで新規ファイル作成
2. _base.pyを実行
3. このスクリプトを実行
4. 各create_*関数を個別に実行するか、最後のexport_all()を実行
"""

import bpy
from mathutils import Vector, Matrix
from math import pi, cos, sin, radians
import os

# _base.py をロード
exec(open("tools/blender_scripts/_base.py").read())

# =============================================================================
# 1. Furnace (かまど) - 接続面: 前後(±Y)
# =============================================================================

def create_furnace():
    """かまど - 正面に開口部のあるキューブ
    接続面: front(+Y), back(-Y) - アイテム入出力用
    """
    clear_scene()

    parts = []

    # メイン本体（0.9x0.9x0.9）
    body = create_chamfered_cube((0.9, 0.9, 0.9), chamfer=0.05, name="Furnace_Body")
    apply_preset_material(body, "dark_steel")
    parts.append(body)

    # 正面開口部（窓）- 赤く光るアクセント
    opening_size = (0.05, 0.5, 0.3)
    opening = create_chamfered_cube(opening_size, chamfer=0.01,
                                    location=(0.45, 0, 0.1), name="Furnace_Opening")
    mat = create_material("furnace_fire", color=ACCENT_COLORS["danger"], metallic=0.0, roughness=0.3)
    apply_material(opening, mat)
    parts.append(opening)

    # フレーム（開口部の縁）
    frame_thickness = 0.08
    # 上下
    for y_offset in [-0.2, 0.2]:
        frame = create_chamfered_cube((0.08, 0.6, 0.05), chamfer=0.01,
                                     location=(0.42, y_offset, 0.1), name=f"Frame_{y_offset}")
        apply_preset_material(frame, "iron")
        parts.append(frame)
    # 左右
    for z_offset in [-0.1, 0.3]:
        frame = create_chamfered_cube((0.08, 0.05, 0.4), chamfer=0.01,
                                     location=(0.42, 0, z_offset), name=f"Frame_{z_offset}")
        apply_preset_material(frame, "iron")
        parts.append(frame)

    # ボルト装飾（4隅）
    bolt_positions = [(-0.35, -0.35, 0.35), (0.35, -0.35, 0.35),
                     (-0.35, 0.35, 0.35), (0.35, 0.35, 0.35)]
    for pos in bolt_positions:
        bolt = create_bolt(size=0.04, length=0.06, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "iron")
        parts.append(bolt)

    # 接続ポート（前後）- アイテム入出力
    # 入力ポート（後面）
    input_port = create_connection_port("pipe", radius=0.12, location=(0, -0.45, 0.45), facing="back", material="brass")
    parts.extend(input_port)

    # 出力ポート（前面下部）
    output_port = create_connection_port("pipe", radius=0.1, location=(0, 0.45, 0.2), facing="front", material="copper")
    parts.extend(output_port)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = body
    for obj in parts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 2. Conveyor Belt (コンベアベルト) - 接続面: 前後(±Y)
# =============================================================================

def create_conveyor():
    """コンベアベルト - 平たい台、ローラー付き
    接続面: front(+Y), back(-Y) - アイテム搬送方向
    """
    clear_scene()

    parts = []

    # ベース台
    base = create_chamfered_cube((0.9, 0.9, 0.15), chamfer=0.03,
                                 location=(0, 0, 0), name="Conveyor_Base")
    apply_preset_material(base, "dark_steel")
    parts.append(base)

    # ベルト面（茶色）
    belt = create_chamfered_cube((0.85, 0.7, 0.05), chamfer=0.01,
                                location=(0, 0, 0.1), name="Conveyor_Belt")
    apply_preset_material(belt, "wood")
    parts.append(belt)

    # ローラー（両端）
    roller_radius = 0.08
    roller_width = 0.7
    for x_offset in [-0.35, 0.35]:
        roller = create_octagonal_prism(roller_radius, roller_width,
                                       location=(x_offset, 0, 0.08), name=f"Roller_{x_offset}")
        roller.rotation_euler.x = pi / 2
        apply_preset_material(roller, "iron")
        parts.append(roller)

    # サイドフレーム（接続面のフランジ兼用）
    for y_offset in [-0.45, 0.45]:
        frame = create_chamfered_cube((0.9, 0.08, 0.12), chamfer=0.02,
                                     location=(0, y_offset, 0.06), name=f"Frame_{y_offset}")
        apply_preset_material(frame, "iron")
        parts.append(frame)

    # 接続用フランジ（前後のサイドレール端）
    # 前面フランジ
    front_flange = create_pipe_flange(0.35, (0, 0.5, 0.08), facing="front", bolt_count=4, material="brass")
    parts.extend(front_flange)
    # 後面フランジ
    back_flange = create_pipe_flange(0.35, (0, -0.5, 0.08), facing="back", bolt_count=4, material="brass")
    parts.extend(back_flange)

    # ボルト（装飾）
    bolt_positions = [(0.35, 0.35, 0.12), (-0.35, 0.35, 0.12),
                     (0.35, -0.35, 0.12), (-0.35, -0.35, 0.12)]
    for pos in bolt_positions:
        bolt = create_bolt(size=0.03, length=0.04, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "brass")
        parts.append(bolt)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = base
    for obj in parts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 3. Crusher (粉砕機)
# =============================================================================

def create_crusher():
    """粉砕機 - 箱型、上部に投入口"""
    clear_scene()

    # 本体
    body = create_chamfered_cube((0.9, 0.9, 0.8), chamfer=0.05,
                                 location=(0, 0, 0), name="Crusher_Body")
    apply_preset_material(body, "iron")

    # 投入口（ホッパー型）
    hopper = create_trapezoid(top_width=0.4, bottom_width=0.5, height=0.15, depth=0.4,
                             location=(0, 0, 0.8), name="Crusher_Hopper")
    hopper.rotation_euler.x = pi
    # 回転を適用
    bpy.ops.object.select_all(action='DESELECT')
    hopper.select_set(True)
    bpy.context.view_layer.objects.active = hopper
    bpy.ops.object.transform_apply(rotation=True)
    apply_preset_material(hopper, "brass")

    # 内部ギア（見える部分）
    gear = create_gear(radius=0.3, thickness=0.1, teeth=8, hole_radius=0.05,
                      location=(0, 0.35, 0.3), name="Crusher_Gear")
    gear.rotation_euler.x = pi / 2
    # 回転を適用
    bpy.ops.object.select_all(action='DESELECT')
    gear.select_set(True)
    bpy.context.view_layer.objects.active = gear
    bpy.ops.object.transform_apply(rotation=True)
    apply_preset_material(gear, "copper")

    # 補強リブ（4辺）
    ribs = []
    for x_offset in [-0.4, 0.4]:
        rib = create_chamfered_cube((0.06, 0.9, 0.06), chamfer=0.01,
                                   location=(x_offset, 0, 0.4), name=f"Rib_X_{x_offset}")
        apply_preset_material(rib, "dark_steel")
        ribs.append(rib)
    for y_offset in [-0.4, 0.4]:
        rib = create_chamfered_cube((0.9, 0.06, 0.06), chamfer=0.01,
                                   location=(0, y_offset, 0.4), name=f"Rib_Y_{y_offset}")
        apply_preset_material(rib, "dark_steel")
        ribs.append(rib)

    # ボルト
    bolt_positions = [(0.4, 0.4, 0.7), (-0.4, 0.4, 0.7),
                     (0.4, -0.4, 0.7), (-0.4, -0.4, 0.7)]
    bolts = []
    for pos in bolt_positions:
        bolt = create_bolt(size=0.04, length=0.05, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "iron")
        bolts.append(bolt)

    # 結合 - シーン内の全オブジェクトを選択
    bpy.ops.object.select_all(action='SELECT')
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = "Crusher"

    finalize_model(result, "machine")
    return result


# =============================================================================
# 4. Press (プレス機)
# =============================================================================

def create_press():
    """プレス機 - 上下に動く部分"""
    clear_scene()

    # ベース台
    base = create_chamfered_cube((0.9, 0.9, 0.2), chamfer=0.04,
                                location=(0, 0, 0), name="Press_Base")
    apply_preset_material(base, "dark_steel")

    # 支柱（4本）
    pillar_radius = 0.05
    pillar_height = 0.6
    pillars = []
    for x_offset in [-0.35, 0.35]:
        for y_offset in [-0.35, 0.35]:
            pillar = create_octagonal_prism(pillar_radius, pillar_height,
                                           location=(x_offset, y_offset, 0.2), name=f"Pillar_{x_offset}_{y_offset}")
            apply_preset_material(pillar, "brass")
            pillars.append(pillar)

    # 上部フレーム
    top_frame = create_chamfered_cube((0.9, 0.9, 0.1), chamfer=0.04,
                                     location=(0, 0, 0.85), name="Press_TopFrame")
    apply_preset_material(top_frame, "dark_steel")

    # プレスヘッド（可動部）
    press_head = create_chamfered_cube((0.6, 0.6, 0.15), chamfer=0.03,
                                      location=(0, 0, 0.35), name="Press_Head")
    apply_preset_material(press_head, "iron")

    # プレスロッド
    rod = create_octagonal_prism(0.08, 0.3, location=(0, 0, 0.55), name="Press_Rod")
    apply_preset_material(rod, "copper")

    # ギア（上部）
    gear = create_gear(radius=0.25, thickness=0.08, teeth=8, hole_radius=0.05,
                      location=(0, 0, 0.92), name="Press_Gear")
    apply_preset_material(gear, "brass")

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = base
    base.select_set(True)
    for obj in pillars + [top_frame, press_head, rod, gear]:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 5. Pump (ポンプ) - 接続面: 左(-X), 上(+Z)
# =============================================================================

def create_pump():
    """ポンプ - 八角柱ベース
    接続面: left(-X) 入力, top(+Z) 出力 - 液体/気体輸送
    """
    clear_scene()

    parts = []

    # メイン本体（八角柱）
    body = create_octagonal_prism(radius=0.4, height=0.5, location=(0, 0, 0.25), name="Pump_Body")
    apply_preset_material(body, "copper")
    parts.append(body)

    # ベース台
    base = create_chamfered_cube((0.7, 0.7, 0.1), chamfer=0.03, location=(0, 0, 0), name="Pump_Base")
    apply_preset_material(base, "dark_steel")
    parts.append(base)

    # 入力接続ポート（左側面）- フランジ付き
    inlet_port = create_connection_port("pipe", radius=0.1, location=(-0.45, 0, 0.4), facing="left", material="brass")
    parts.extend(inlet_port)

    # 出力接続ポート（上部）- フランジ付き
    outlet_port = create_connection_port("pipe", radius=0.12, location=(0, 0, 0.55), facing="top", material="brass")
    parts.extend(outlet_port)

    # 内部ローター（見える）
    rotor = create_gear(radius=0.25, thickness=0.08, teeth=6, hole_radius=0.04,
                       location=(0, 0.35, 0.3), name="Pump_Rotor")
    rotor.rotation_euler.x = pi / 2
    apply_preset_material(rotor, "iron")
    parts.append(rotor)

    # ボルト（補強）
    bolt_positions = [(0.3, 0.3, 0.15), (-0.3, 0.3, 0.15),
                     (0.3, -0.3, 0.15), (-0.3, -0.3, 0.15)]
    for pos in bolt_positions:
        bolt = create_bolt(size=0.035, length=0.05, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "iron")
        parts.append(bolt)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = body
    for obj in parts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 6. Tank (タンク) - 接続面: 右(+X)
# =============================================================================

def create_tank():
    """タンク - 大きな円筒
    接続面: right(+X) - 液体入出力ポート
    """
    clear_scene()

    parts = []

    # メイン本体（八角柱）
    body = create_octagonal_prism(radius=0.4, height=0.8, location=(0, 0, 0.1), name="Tank_Body")
    apply_preset_material(body, "iron")
    parts.append(body)

    # 上部キャップ
    top_cap = create_octagonal_prism(radius=0.42, height=0.08, location=(0, 0, 0.54), name="Tank_TopCap")
    apply_preset_material(top_cap, "brass")
    parts.append(top_cap)

    # 下部キャップ
    bottom_cap = create_octagonal_prism(radius=0.42, height=0.08, location=(0, 0, 0.06), name="Tank_BottomCap")
    apply_preset_material(bottom_cap, "brass")
    parts.append(bottom_cap)

    # 補強バンド（3本）
    for z_offset in [0.2, 0.35, 0.5]:
        band = create_octagonal_prism(radius=0.43, height=0.04, location=(0, 0, z_offset), name=f"Band_{z_offset}")
        apply_preset_material(band, "dark_steel")
        parts.append(band)

    # 接続ポート（側面）- フランジ付き
    side_port = create_connection_port("pipe", radius=0.08, location=(0.45, 0, 0.3), facing="right", material="copper")
    parts.extend(side_port)

    # 液面ゲージ（窓）
    gauge_height = 0.5
    gauge = create_chamfered_cube((0.03, 0.15, gauge_height), chamfer=0.005,
                                 location=(0.41, 0.25, 0.3), name="Tank_Gauge")
    mat = create_material("gauge_glass", color=(0.3, 0.6, 0.8, 0.7), metallic=0.1, roughness=0.1)
    apply_material(gauge, mat)
    parts.append(gauge)

    # ボルト（上下）
    bolt_positions = []
    for z in [0.58, 0.02]:
        for i in range(8):
            angle = i * pi / 4 + pi / 8
            x = cos(angle) * 0.4
            y = sin(angle) * 0.4
            bolt_positions.append((x, y, z))

    for pos in bolt_positions:
        bolt = create_bolt(size=0.03, length=0.04, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "brass")
        parts.append(bolt)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = body
    for obj in parts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 7. Miner (採掘機)
# =============================================================================

def create_miner():
    """採掘機 - ドリル付き"""
    clear_scene()

    # 本体
    body = create_chamfered_cube((0.85, 0.85, 0.6), chamfer=0.05,
                                location=(0, 0, 0.3), name="Miner_Body")
    apply_preset_material(body, "dark_steel")

    # ドリル本体（八角錐型）
    drill_base = create_octagonal_prism(radius=0.15, height=0.3, location=(0, 0, 0.1), name="Miner_DrillBase")
    apply_preset_material(drill_base, "copper")

    # ドリル先端（小さい八角錐）
    drill_tip = create_octagonal_prism(radius=0.08, height=0.15, location=(0, 0, -0.05), name="Miner_DrillTip")
    apply_preset_material(drill_tip, "iron")

    # ドリルシャフト
    shaft = create_octagonal_prism(radius=0.05, height=0.25, location=(0, 0, 0.25), name="Miner_Shaft")
    apply_preset_material(shaft, "brass")

    # モーターハウジング（上部）
    motor = create_octagonal_prism(radius=0.25, height=0.2, location=(0, 0, 0.7), name="Miner_Motor")
    apply_preset_material(motor, "copper")

    # ギア（可視部分）
    gear = create_gear(radius=0.2, thickness=0.08, teeth=8, hole_radius=0.04,
                      location=(0.3, 0, 0.45), name="Miner_Gear")
    gear.rotation_euler.y = pi / 2
    apply_preset_material(gear, "brass")

    # 補強フレーム（4隅）
    frames = []
    for x_offset in [-0.35, 0.35]:
        for y_offset in [-0.35, 0.35]:
            frame = create_chamfered_cube((0.08, 0.08, 0.6), chamfer=0.02,
                                         location=(x_offset, y_offset, 0.3), name=f"Frame_{x_offset}_{y_offset}")
            apply_preset_material(frame, "iron")
            frames.append(frame)

    # ボルト
    bolt_positions = [(0.35, 0.35, 0.55), (-0.35, 0.35, 0.55),
                     (0.35, -0.35, 0.55), (-0.35, -0.35, 0.55)]
    bolts = []
    for pos in bolt_positions:
        bolt = create_bolt(size=0.04, length=0.05, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "dark_steel")
        bolts.append(bolt)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = body
    body.select_set(True)
    for obj in [drill_base, drill_tip, shaft, motor, gear] + frames + bolts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 8. Assembler (組立機)
# =============================================================================

def create_assembler():
    """組立機 - 箱型、アーム付き"""
    clear_scene()

    # 本体
    body = create_chamfered_cube((0.9, 0.9, 0.6), chamfer=0.05,
                                location=(0, 0, 0.3), name="Assembler_Body")
    apply_preset_material(body, "iron")

    # 作業台（上部）
    workbench = create_chamfered_cube((0.7, 0.7, 0.05), chamfer=0.01,
                                     location=(0, 0, 0.65), name="Assembler_Workbench")
    apply_preset_material(workbench, "brass")

    # ロボットアーム1（ベース）
    arm1_base = create_octagonal_prism(radius=0.08, height=0.15, location=(0.25, 0, 0.75), name="Arm1_Base")
    apply_preset_material(arm1_base, "copper")

    # ロボットアーム2（リンク）
    arm1_link = create_chamfered_cube((0.08, 0.08, 0.25), chamfer=0.02,
                                     location=(0.25, 0.15, 0.85), name="Arm1_Link")
    apply_preset_material(arm1_link, "dark_steel")

    # アーム関節（ギア）
    joint = create_gear(radius=0.06, thickness=0.05, teeth=6, hole_radius=0.02,
                       location=(0.25, 0, 0.83), name="Arm_Joint")
    joint.rotation_euler.x = pi / 2
    apply_preset_material(joint, "brass")

    # グリッパー（先端）
    gripper = create_chamfered_cube((0.1, 0.06, 0.06), chamfer=0.01,
                                   location=(0.25, 0.3, 0.9), name="Gripper")
    apply_preset_material(gripper, "iron")

    # ギアシステム（側面可視）
    gears = []
    for y_offset in [-0.2, 0.2]:
        gear = create_gear(radius=0.15, thickness=0.08, teeth=8, hole_radius=0.03,
                          location=(0.4, y_offset, 0.4), name=f"Gear_{y_offset}")
        gear.rotation_euler.y = pi / 2
        apply_preset_material(gear, "copper")
        gears.append(gear)

    # パイプ配線
    pipes = []
    for i, z_offset in enumerate([0.2, 0.35, 0.5]):
        pipe = create_pipe(radius=0.03, length=0.15, wall=0.01,
                          location=(0.45, 0.15 * (i - 1), z_offset), name=f"Pipe_{i}")
        pipe.rotation_euler.y = pi / 2
        apply_preset_material(pipe, "brass")
        pipes.append(pipe)

    # ボルト
    bolt_positions = [(0.4, 0.4, 0.6), (-0.4, 0.4, 0.6),
                     (0.4, -0.4, 0.6), (-0.4, -0.4, 0.6)]
    bolts = []
    for pos in bolt_positions:
        bolt = create_bolt(size=0.04, length=0.05, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "iron")
        bolts.append(bolt)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = body
    body.select_set(True)
    for obj in [workbench, arm1_base, arm1_link, joint, gripper] + gears + pipes + bolts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 9. Mixer (ミキサー) - 接続面: 右(+X)入力, 下(-Z)出力
# =============================================================================

def create_mixer():
    """ミキサー - タンク型、攪拌羽根
    接続面: right(+X) 入力, bottom(-Z) 出力
    """
    clear_scene()

    parts = []

    # タンク本体（八角柱）
    tank = create_octagonal_prism(radius=0.4, height=0.6, location=(0, 0, 0.2), name="Mixer_Tank")
    apply_preset_material(tank, "iron")
    parts.append(tank)

    # 上部蓋
    lid = create_octagonal_prism(radius=0.42, height=0.1, location=(0, 0, 0.55), name="Mixer_Lid")
    apply_preset_material(lid, "brass")
    parts.append(lid)

    # モーターハウジング（上部）
    motor = create_octagonal_prism(radius=0.2, height=0.15, location=(0, 0, 0.7), name="Mixer_Motor")
    apply_preset_material(motor, "copper")
    parts.append(motor)

    # シャフト（中央）
    shaft = create_octagonal_prism(radius=0.04, height=0.7, location=(0, 0, 0.3), name="Mixer_Shaft")
    apply_preset_material(shaft, "dark_steel")
    parts.append(shaft)

    # 攪拌羽根（3枚）
    for i in range(3):
        angle = i * 2 * pi / 3
        blade = create_chamfered_cube((0.3, 0.05, 0.08), chamfer=0.01,
                                     location=(0, 0, 0.25 + i * 0.15), name=f"Blade_{i}")
        blade.rotation_euler.z = angle
        apply_preset_material(blade, "brass")
        parts.append(blade)

    # 補強バンド（2本）
    for z_offset in [0.3, 0.45]:
        band = create_octagonal_prism(radius=0.42, height=0.04, location=(0, 0, z_offset), name=f"Band_{z_offset}")
        apply_preset_material(band, "dark_steel")
        parts.append(band)

    # 入力接続ポート（側面上部）- フランジ付き
    inlet_port = create_connection_port("pipe", radius=0.08, location=(0.45, 0, 0.5), facing="right", material="copper")
    parts.extend(inlet_port)

    # 出力接続ポート（底部）- フランジ付き
    outlet_port = create_connection_port("pipe", radius=0.1, location=(0, 0, 0.02), facing="bottom", material="brass")
    parts.extend(outlet_port)

    # ボルト（蓋）
    bolt_positions = []
    for i in range(8):
        angle = i * pi / 4 + pi / 8
        x = cos(angle) * 0.38
        y = sin(angle) * 0.38
        bolt_positions.append((x, y, 0.6))

    for pos in bolt_positions:
        bolt = create_bolt(size=0.03, length=0.04, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "iron")
        parts.append(bolt)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = tank
    for obj in parts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 10. Centrifuge (遠心分離機)
# =============================================================================

def create_centrifuge():
    """遠心分離機 - 回転ドラム"""
    clear_scene()

    # ベース台
    base = create_chamfered_cube((0.9, 0.9, 0.15), chamfer=0.04,
                                location=(0, 0, 0.075), name="Centrifuge_Base")
    apply_preset_material(base, "dark_steel")

    # 回転ドラム（八角柱、横向き）
    drum = create_octagonal_prism(radius=0.35, height=0.7, location=(0, 0, 0.4), name="Centrifuge_Drum")
    drum.rotation_euler.y = pi / 2
    apply_preset_material(drum, "iron")

    # ドラムキャップ（両端）
    caps = []
    for x_offset in [-0.38, 0.38]:
        cap = create_octagonal_prism(radius=0.37, height=0.06, location=(x_offset, 0, 0.4), name=f"Cap_{x_offset}")
        cap.rotation_euler.y = pi / 2
        apply_preset_material(cap, "brass")
        caps.append(cap)

    # 内部ローター（可視窓部分）
    rotor = create_gear(radius=0.25, thickness=0.08, teeth=12, hole_radius=0.05,
                       location=(0, 0.32, 0.4), name="Centrifuge_Rotor")
    rotor.rotation_euler.x = pi / 2
    apply_preset_material(rotor, "copper")

    # 支柱（2本）
    pillars = []
    for x_offset in [-0.3, 0.3]:
        pillar = create_chamfered_cube((0.08, 0.08, 0.25), chamfer=0.02,
                                      location=(x_offset, 0, 0.27), name=f"Pillar_{x_offset}")
        apply_preset_material(pillar, "dark_steel")
        pillars.append(pillar)

    # モーター（側面）
    motor = create_octagonal_prism(radius=0.15, height=0.2, location=(0.5, 0, 0.4), name="Centrifuge_Motor")
    motor.rotation_euler.y = pi / 2
    apply_preset_material(motor, "copper")

    # ドライブシャフト
    shaft = create_octagonal_prism(radius=0.05, height=0.15, location=(0.42, 0, 0.4), name="Centrifuge_Shaft")
    shaft.rotation_euler.y = pi / 2
    apply_preset_material(shaft, "brass")

    # 補強リング（ドラム中央）
    ring = create_octagonal_prism(radius=0.38, height=0.05, location=(0, 0, 0.4), name="Centrifuge_Ring")
    ring.rotation_euler.y = pi / 2
    apply_preset_material(ring, "dark_steel")

    # ボルト（ベース）
    bolt_positions = [(0.4, 0.4, 0.15), (-0.4, 0.4, 0.15),
                     (0.4, -0.4, 0.15), (-0.4, -0.4, 0.15)]
    bolts = []
    for pos in bolt_positions:
        bolt = create_bolt(size=0.035, length=0.05, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "iron")
        bolts.append(bolt)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = base
    base.select_set(True)
    for obj in [drum] + caps + [rotor] + pillars + [motor, shaft, ring] + bolts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 11. Generator (発電機)
# =============================================================================

def create_generator():
    """発電機 - ローター付き"""
    clear_scene()

    # 本体（箱型）
    body = create_chamfered_cube((0.8, 0.8, 0.7), chamfer=0.05,
                                location=(0, 0, 0.35), name="Generator_Body")
    apply_preset_material(body, "iron")

    # ローターハウジング（前面）
    housing = create_octagonal_prism(radius=0.35, height=0.15, location=(0.45, 0, 0.45), name="Generator_Housing")
    housing.rotation_euler.y = pi / 2
    apply_preset_material(housing, "copper")

    # ローター（ファン）
    rotor = create_gear(radius=0.28, thickness=0.08, teeth=8, hole_radius=0.05,
                       location=(0.5, 0, 0.45), name="Generator_Rotor")
    rotor.rotation_euler.y = pi / 2
    apply_preset_material(rotor, "brass")

    # ローター羽根（8枚）
    blades = []
    for i in range(8):
        angle = i * pi / 4
        blade = create_trapezoid(top_width=0.08, bottom_width=0.12, height=0.2, depth=0.04,
                                location=(0.5, 0, 0.45), name=f"Blade_{i}")
        blade.rotation_euler = (0, pi / 2, angle)
        apply_preset_material(blade, "dark_steel")
        blades.append(blade)

    # 中央シャフト
    shaft = create_octagonal_prism(radius=0.06, height=0.6, location=(0.15, 0, 0.45), name="Generator_Shaft")
    shaft.rotation_euler.y = pi / 2
    apply_preset_material(shaft, "iron")

    # コイルハウジング（内部可視）
    coil = create_octagonal_prism(radius=0.25, height=0.3, location=(0, 0, 0.45), name="Generator_Coil")
    coil.rotation_euler.y = pi / 2
    mat = create_material("coil_copper", color=ACCENT_COLORS["warning"], metallic=0.8, roughness=0.4)
    apply_material(coil, mat)

    # 冷却フィン（側面）
    fins = []
    for z_offset in [0.25, 0.35, 0.45, 0.55, 0.65]:
        fin = create_chamfered_cube((0.75, 0.85, 0.03), chamfer=0.01,
                                   location=(0, 0, z_offset), name=f"Fin_{z_offset}")
        apply_preset_material(fin, "dark_steel")
        fins.append(fin)

    # 電力出力端子（上部）
    terminals = []
    for y_offset in [-0.15, 0.15]:
        terminal = create_octagonal_prism(radius=0.06, height=0.1, location=(0, y_offset, 0.75), name=f"Terminal_{y_offset}")
        mat_power = create_material("terminal", color=ACCENT_COLORS["power"], metallic=1.0, roughness=0.3)
        apply_material(terminal, mat_power)
        terminals.append(terminal)

    # ボルト
    bolt_positions = [(0.35, 0.35, 0.7), (-0.35, 0.35, 0.7),
                     (0.35, -0.35, 0.7), (-0.35, -0.35, 0.7)]
    bolts = []
    for pos in bolt_positions:
        bolt = create_bolt(size=0.04, length=0.05, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "brass")
        bolts.append(bolt)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = body
    body.select_set(True)
    for obj in [housing, rotor] + blades + [shaft, coil] + fins + terminals + bolts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 12. Chemical Reactor (化学反応器) - 接続面: 上(+Z)入力×3, 右(+X)出力
# =============================================================================

def create_chemical_reactor():
    """化学反応器 - 配管付きタンク
    接続面: top(+Z) 入力×3, right(+X) 出力
    """
    clear_scene()

    parts = []

    # メインタンク（八角柱）
    tank = create_octagonal_prism(radius=0.38, height=0.65, location=(0, 0, 0.325), name="Reactor_Tank")
    apply_preset_material(tank, "iron")
    parts.append(tank)

    # 反応器上部ドーム
    dome = create_octagonal_prism(radius=0.4, height=0.12, location=(0, 0, 0.71), name="Reactor_Dome")
    apply_preset_material(dome, "brass")
    parts.append(dome)

    # 底部コーン（逆さま）
    cone = create_octagonal_prism(radius=0.3, height=0.1, location=(0, 0, 0.05), name="Reactor_Cone")
    apply_preset_material(cone, "brass")
    parts.append(cone)

    # 入力接続ポート（上部、3本）- フランジ付き
    for i in range(3):
        angle = i * 2 * pi / 3
        x = cos(angle) * 0.25
        y = sin(angle) * 0.25
        inlet_port = create_connection_port("pipe", radius=0.06, location=(x, y, 0.78), facing="top", material="copper")
        parts.extend(inlet_port)

    # 出力接続ポート（側面中段）- フランジ付き
    outlet_port = create_connection_port("pipe", radius=0.1, location=(0.45, 0, 0.4), facing="right", material="brass")
    parts.extend(outlet_port)

    # 冷却ジャケット（外部コイル）
    for i, z_offset in enumerate([0.2, 0.35, 0.5, 0.65]):
        coil = create_pipe(radius=0.04, length=0.5, wall=0.008, location=(0, 0, z_offset), name=f"Coil_{i}")
        coil.rotation_euler.y = pi / 2
        coil.location.x = 0.42
        apply_preset_material(coil, "copper")
        parts.append(coil)

    # 圧力ゲージ（前面）
    gauge = create_octagonal_prism(radius=0.08, height=0.05, location=(0.42, 0.2, 0.55), name="Reactor_Gauge")
    gauge.rotation_euler.y = pi / 2
    mat = create_material("gauge", color=ACCENT_COLORS["warning"], metallic=0.5, roughness=0.3)
    apply_material(gauge, mat)
    parts.append(gauge)

    # 補強バンド（3本）
    for z_offset in [0.25, 0.45, 0.6]:
        band = create_octagonal_prism(radius=0.41, height=0.04, location=(0, 0, z_offset), name=f"Band_{z_offset}")
        apply_preset_material(band, "dark_steel")
        parts.append(band)

    # 安全リリーフバルブ（上部）
    relief = create_octagonal_prism(radius=0.05, height=0.15, location=(0, 0, 0.92), name="Reactor_Relief")
    mat_danger = create_material("relief", color=ACCENT_COLORS["danger"], metallic=0.7, roughness=0.4)
    apply_material(relief, mat_danger)
    parts.append(relief)

    # ボルト（上部フランジ）
    bolt_positions = []
    for i in range(8):
        angle = i * pi / 4 + pi / 8
        x = cos(angle) * 0.36
        y = sin(angle) * 0.36
        bolt_positions.append((x, y, 0.67))

    for pos in bolt_positions:
        bolt = create_bolt(size=0.03, length=0.04, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "iron")
        parts.append(bolt)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = tank
    for obj in parts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# 13. Solar Panel (ソーラーパネル)
# =============================================================================

def create_solar_panel():
    """ソーラーパネル - 青い板"""
    clear_scene()

    # ベース台
    base = create_chamfered_cube((0.9, 0.9, 0.08), chamfer=0.02,
                                location=(0, 0, 0.04), name="Solar_Base")
    apply_preset_material(base, "dark_steel")

    # パネル本体（青い板）
    panel = create_chamfered_cube((0.85, 0.85, 0.03), chamfer=0.005,
                                 location=(0, 0, 0.115), name="Solar_Panel")
    mat = create_material("solar_cell", color=ACCENT_COLORS["power"], metallic=0.2, roughness=0.1)
    apply_material(panel, mat)

    # グリッドライン（セル分割）- 埋め込みディテール
    grid_lines = []
    grid_size = 0.2
    for i in range(-2, 3):
        # 縦線
        if i != 0:
            line_v = create_chamfered_cube((0.01, 0.85, 0.01), chamfer=0.002,
                                          location=(i * grid_size, 0, 0.125), name=f"Grid_V_{i}")
            apply_preset_material(line_v, "dark_steel")
            grid_lines.append(line_v)
        # 横線
        if i != 0:
            line_h = create_chamfered_cube((0.85, 0.01, 0.01), chamfer=0.002,
                                          location=(0, i * grid_size, 0.125), name=f"Grid_H_{i}")
            apply_preset_material(line_h, "dark_steel")
            grid_lines.append(line_h)

    # フレーム（4辺）
    frames = []
    frame_thickness = 0.04
    # 上下
    for y_offset in [-0.44, 0.44]:
        frame = create_chamfered_cube((0.9, frame_thickness, 0.05), chamfer=0.01,
                                     location=(0, y_offset, 0.115), name=f"Frame_Y_{y_offset}")
        apply_preset_material(frame, "brass")
        frames.append(frame)
    # 左右
    for x_offset in [-0.44, 0.44]:
        frame = create_chamfered_cube((frame_thickness, 0.9, 0.05), chamfer=0.01,
                                     location=(x_offset, 0, 0.115), name=f"Frame_X_{x_offset}")
        apply_preset_material(frame, "brass")
        frames.append(frame)

    # 接続ボックス（底面）
    junction_box = create_chamfered_cube((0.15, 0.1, 0.04), chamfer=0.01,
                                        location=(0, -0.35, 0.06), name="Solar_JunctionBox")
    apply_preset_material(junction_box, "dark_steel")

    # 配線（ジャンクションボックスから）
    wire = create_pipe(radius=0.015, length=0.12, wall=0.005, location=(0, -0.35, 0.08), name="Solar_Wire")
    wire.rotation_euler.x = pi / 2
    apply_preset_material(wire, "copper")

    # マウント脚（4隅）
    mounts = []
    for x_offset in [-0.38, 0.38]:
        for y_offset in [-0.38, 0.38]:
            mount = create_chamfered_cube((0.06, 0.06, 0.08), chamfer=0.015,
                                         location=(x_offset, y_offset, 0.04), name=f"Mount_{x_offset}_{y_offset}")
            apply_preset_material(mount, "iron")
            mounts.append(mount)

    # ボルト（マウント）
    bolt_positions = [(0.38, 0.38, 0.08), (-0.38, 0.38, 0.08),
                     (0.38, -0.38, 0.08), (-0.38, -0.38, 0.08)]
    bolts = []
    for pos in bolt_positions:
        bolt = create_bolt(size=0.025, length=0.03, location=pos, name=f"Bolt_{pos}")
        apply_preset_material(bolt, "brass")
        bolts.append(bolt)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = base
    base.select_set(True)
    for obj in [panel] + grid_lines + frames + [junction_box, wire] + mounts + bolts:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    finalize_model(result, "machine")
    return result


# =============================================================================
# エクスポート一括処理
# =============================================================================

def export_all():
    """全モデルをエクスポート"""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(os.path.dirname(script_dir))
    output_dir = os.path.join(project_root, "assets", "models", "machines")
    os.makedirs(output_dir, exist_ok=True)

    machines = [
        ("furnace", create_furnace),
        ("conveyor", create_conveyor),
        ("crusher", create_crusher),
        ("press", create_press),
        ("pump", create_pump),
        ("tank", create_tank),
        ("miner", create_miner),
        ("assembler", create_assembler),
        ("mixer", create_mixer),
        ("centrifuge", create_centrifuge),
        ("generator", create_generator),
        ("chemical_reactor", create_chemical_reactor),
        ("solar_panel", create_solar_panel),
    ]

    for name, create_func in machines:
        print(f"\n=== Creating {name} ===")
        model = create_func()
        filepath = os.path.join(output_dir, f"{name}.gltf")
        export_gltf(filepath, export_animations=False)
        print(f"Exported: {filepath}")


# =============================================================================
# 実行確認
# =============================================================================

print("=== Machine Models Script Loaded ===")
print("Available models:")
print("  create_furnace, create_conveyor, create_crusher, create_press")
print("  create_pump, create_tank, create_miner, create_assembler")
print("  create_mixer, create_centrifuge, create_generator")
print("  create_chemical_reactor, create_solar_panel")
print("\nRun export_all() to export all models")

# 自動実行
if __name__ == "__main__":
    export_all()
