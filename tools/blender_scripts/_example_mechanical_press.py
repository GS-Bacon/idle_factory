"""
Mechanical Press (プレス機) - サンプルスクリプト
カテゴリ: single_block_machine
ポリゴン目安: 200-800 (max 1500)

使い方:
1. Blenderを開く
2. Scriptingタブに切り替え
3. このスクリプトを貼り付けて実行
4. assets/models/machines/ にエクスポート
"""

# ベースモジュール読み込み
import bpy
import os

# _base.pyのパスを取得して実行
script_dir = os.path.dirname(bpy.data.filepath) if bpy.data.filepath else os.getcwd()
base_path = os.path.join(script_dir, "_base.py")
if os.path.exists(base_path):
    exec(open(base_path).read())
else:
    # 直接実行する場合のフォールバック
    exec(open("tools/blender_scripts/_base.py").read())

# =============================================================================
# プレス機モデル生成
# =============================================================================

clear_scene()

# --- ベースフレーム ---
frame = create_chamfered_cube(
    size=(0.9, 0.9, 0.2),
    location=(0, 0, 0.1),
    name="frame_base"
)

# --- 支柱（4本） ---
pillar_positions = [
    (-0.35, -0.35), (0.35, -0.35),
    (-0.35, 0.35), (0.35, 0.35)
]
pillars = []
for i, (x, y) in enumerate(pillar_positions):
    pillar = create_chamfered_cube(
        size=(0.1, 0.1, 0.6),
        location=(x, y, 0.5),
        name=f"pillar_{i}"
    )
    pillars.append(pillar)

# --- 上部フレーム ---
top_frame = create_chamfered_cube(
    size=(0.9, 0.9, 0.15),
    location=(0, 0, 0.875),
    name="frame_top"
)

# --- プレスヘッド（可動部） ---
press_head = create_chamfered_cube(
    size=(0.6, 0.6, 0.15),
    location=(0, 0, 0.6),
    name="press_head"
)

# --- ピストンロッド ---
piston_rod = create_octagonal_prism(
    radius=0.06,
    height=0.2,
    location=(0, 0, 0.75),
    name="piston_rod"
)

# --- ボルト装飾 ---
bolt_positions = [
    (-0.3, -0.3, 0.2), (0.3, -0.3, 0.2),
    (-0.3, 0.3, 0.2), (0.3, 0.3, 0.2)
]
for i, pos in enumerate(bolt_positions):
    bolt = create_bolt(size=0.04, length=0.02, location=pos, name=f"bolt_{i}")

# =============================================================================
# 全オブジェクト結合
# =============================================================================

bpy.ops.object.select_all(action='SELECT')
bpy.context.view_layer.objects.active = frame
bpy.ops.object.join()
frame.name = "mechanical_press"

# =============================================================================
# マテリアル適用
# =============================================================================

apply_preset_material(frame, "dark_steel")

# プレスヘッドに別マテリアル（オプション）
# 結合済みなので頂点グループで分ける場合はここで処理

# =============================================================================
# アニメーション設定
# =============================================================================

# プレスヘッドの上下動（ボーンを使う場合）
# 簡易版: オブジェクト全体のアニメーションは省略
# 本番ではアーマチュアを使用

# =============================================================================
# 最終処理
# =============================================================================

finalize_model(frame, category="machine")

# =============================================================================
# エクスポート
# =============================================================================

# export_gltf("assets/models/machines/mechanical_press.gltf")

print("=== Mechanical Press Complete ===")
print("Blender上で確認後、以下を実行してエクスポート:")
print("export_gltf('assets/models/machines/mechanical_press.gltf')")
