"""
ハンマー (Hammer) - 手持ちツールアイテム
Industrial Lowpoly Style

パーツ接続ルール: すべてのパーツは物理的に接触または重なり合う
"""

import os
import sys

script_dir = os.path.dirname(os.path.abspath(__file__))
base_path = os.path.join(script_dir, "_base.py")
exec(open(base_path).read())

def create_hammer():
    """ハンマーを生成"""
    clear_scene()
    parts = []

    # ===========================================
    # 寸法（すべて接続を考慮して計算）
    # ===========================================
    # ヘッドを基準に、ハンドルがヘッドを貫通する構造

    head_width = 0.08      # X方向（横に長い）
    head_depth = 0.04      # Y方向
    head_height = 0.04     # Z方向
    head_z = 0.17          # ヘッド中心のZ座標

    handle_radius = 0.012
    handle_bottom = 0.0
    # ハンドルはヘッドの中心まで伸びる（貫通）
    handle_top = head_z + 0.005  # ヘッド中心を少し超える
    handle_length = handle_top - handle_bottom

    # ===========================================
    # 1. ハンドル（ヘッドを貫通）
    # ===========================================
    handle = create_octagonal_prism(
        radius=handle_radius,
        height=handle_length,
        location=(0, 0, handle_bottom + handle_length / 2),
        name="Handle"
    )
    parts.append(handle)

    # グリップ（ハンドル下部）
    for i in range(3):
        grip = create_octagonal_prism(
            radius=handle_radius * 1.15,
            height=0.006,
            location=(0, 0, 0.02 + i * 0.02),
            name=f"Grip_{i}"
        )
        parts.append(grip)

    # ハンドルキャップ（最下部）
    cap = create_octagonal_prism(
        radius=handle_radius * 1.2,
        height=0.01,
        location=(0, 0, 0.005),
        name="HandleCap"
    )
    parts.append(cap)

    # ===========================================
    # 2. ハンマーヘッド（ハンドルと交差）
    # ===========================================
    head = create_chamfered_cube(
        size=(head_width, head_depth, head_height),
        chamfer=0.005,
        location=(0, 0, head_z),
        name="HammerHead"
    )
    parts.append(head)

    # 打撃面（ヘッド右端に接続）
    strike_width = 0.02
    strike_x = head_width / 2 + strike_width / 2 - 0.005  # 重なり
    strike = create_chamfered_cube(
        size=(strike_width, head_depth + 0.005, head_height + 0.005),
        chamfer=0.003,
        location=(strike_x, 0, head_z),
        name="StrikeFace"
    )
    parts.append(strike)

    # くさび抜き（ヘッド左端に接続）
    peen_width = 0.025
    peen_x = -(head_width / 2 + peen_width / 2 - 0.005)
    peen = create_chamfered_cube(
        size=(peen_width, head_depth * 0.7, head_height * 0.7),
        chamfer=0.003,
        location=(peen_x, 0, head_z),
        name="Peen"
    )
    parts.append(peen)

    # カラー（ヘッドとハンドルの接続部、ヘッド下面）
    collar_z = head_z - head_height / 2 + 0.008
    collar = create_octagonal_prism(
        radius=0.018,
        height=0.016,
        location=(0, 0, collar_z),
        name="Collar"
    )
    parts.append(collar)

    # ===========================================
    # 3. 結合 & マテリアル
    # ===========================================
    hammer = join_all_meshes(parts, name="Hammer")

    mat = bpy.data.materials.new(name="HammerMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = (0.3, 0.3, 0.3, 1)
        bsdf.inputs["Metallic"].default_value = 0.8
        bsdf.inputs["Roughness"].default_value = 0.5
    hammer.data.materials.append(mat)

    # ===========================================
    # 4. 仕上げ
    # ===========================================
    bpy.context.view_layer.objects.active = hammer
    hammer.select_set(True)
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)
    bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
    hammer.location = (0, 0, 0)

    tri_count = sum(len(p.vertices) - 2 for p in hammer.data.polygons)
    print(f"Triangles: {tri_count}")

    return hammer

# 実行
hammer = create_hammer()
output_dir = os.path.join(script_dir, "../../assets/models/items")
os.makedirs(output_dir, exist_ok=True)
export_gltf(os.path.join(output_dir, "hammer.gltf"))
print("✅ Hammer exported")
