"""
ピッケル (Pickaxe) - Minecraft/Unturned風ローポリツール
参照: .specify/memory/modeling-rules.md のツールデザインテンプレート

特徴:
- 両端が尖った横長ヘッド
- ハンドルが全長の68%
- 明確なシルエット
"""
import bpy
import os
from mathutils import Vector
from math import pi, cos, sin

script_dir = os.path.dirname(os.path.abspath(__file__))

# ============================================
# 推奨寸法（modeling-rules.mdより）
# ============================================
PICKAXE = {
    "total_height": 0.22,
    "handle_length": 0.14,      # 64% of total
    "handle_radius": 0.012,
    "head_width": 0.10,         # 4.2x handle diameter
    "head_depth": 0.032,        # 厚め（より存在感）
    "head_height": 0.04,        # 存在感ある高さ（増）
    "pick_tip_length": 0.048,   # 両端の尖った部分（増）
    "pick_tip_taper": 0.6,      # 先端は60%細くなる
    "collar_radius": 0.018,     # カラーを太く
    "collar_height": 0.028,     # カラーを高く
}

# ============================================
# プリミティブ生成関数
# ============================================
def create_oct_prism(radius, height, location, name):
    """八角柱"""
    verts = []
    for i in range(8):
        angle = i * pi / 4 + pi / 8
        verts.append((cos(angle) * radius, sin(angle) * radius, -height / 2))
        verts.append((cos(angle) * radius, sin(angle) * radius, height / 2))
    faces = []
    for i in range(8):
        j = (i + 1) % 8
        faces.append((i * 2, j * 2, j * 2 + 1, i * 2 + 1))
    faces.append(tuple(i * 2 for i in range(8)))
    faces.append(tuple(i * 2 + 1 for i in reversed(range(8))))
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector(location)
    bpy.context.collection.objects.link(obj)
    return obj

def create_chamfered_box(size, location, name, chamfer=0.1):
    """面取りキューブ"""
    sx, sy, sz = [s / 2 for s in size]
    c = min(size) * chamfer
    verts = [
        (-sx + c, -sy, -sz), (sx - c, -sy, -sz), (sx, -sy + c, -sz), (sx, sy - c, -sz),
        (sx - c, sy, -sz), (-sx + c, sy, -sz), (-sx, sy - c, -sz), (-sx, -sy + c, -sz),
        (-sx + c, -sy, sz), (sx - c, -sy, sz), (sx, -sy + c, sz), (sx, sy - c, sz),
        (sx - c, sy, sz), (-sx + c, sy, sz), (-sx, sy - c, sz), (-sx, -sy + c, sz),
    ]
    faces = [
        (0, 1, 2, 3, 4, 5, 6, 7), (15, 14, 13, 12, 11, 10, 9, 8),
        (0, 8, 9, 1), (1, 9, 10, 2), (2, 10, 11, 3), (3, 11, 12, 4),
        (4, 12, 13, 5), (5, 13, 14, 6), (6, 14, 15, 7), (7, 15, 8, 0),
    ]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector(location)
    bpy.context.collection.objects.link(obj)
    return obj

def create_tapered_box(base_size, tip_size, length, location, direction, name):
    """テーパー付きボックス（先端が細くなる）"""
    bw, bd, bh = [s / 2 for s in base_size]
    tw, td, th = [s / 2 for s in tip_size]

    if direction == 'X+':
        verts = [
            # Base (X=0)
            (0, -bd, -bh), (0, bd, -bh), (0, bd, bh), (0, -bd, bh),
            # Tip (X=length)
            (length, -td, -th), (length, td, -th), (length, td, th), (length, -td, th),
        ]
    elif direction == 'X-':
        verts = [
            # Base (X=0)
            (0, -bd, -bh), (0, bd, -bh), (0, bd, bh), (0, -bd, bh),
            # Tip (X=-length)
            (-length, -td, -th), (-length, td, -th), (-length, td, th), (-length, -td, th),
        ]
    else:
        raise ValueError(f"Unknown direction: {direction}")

    faces = [
        (0, 1, 2, 3),  # Base
        (4, 7, 6, 5),  # Tip
        (0, 4, 5, 1),  # Bottom
        (2, 6, 7, 3),  # Top
        (1, 5, 6, 2),  # Front
        (0, 3, 7, 4),  # Back
    ]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    obj.location = Vector(location)
    bpy.context.collection.objects.link(obj)
    return obj

# ============================================
# マテリアル
# ============================================
def create_material(name, color, metallic=0.0, roughness=0.5):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = (*color, 1.0)
        bsdf.inputs["Metallic"].default_value = metallic
        bsdf.inputs["Roughness"].default_value = roughness
    return mat

def apply_material(obj, mat):
    if obj.data.materials:
        obj.data.materials[0] = mat
    else:
        obj.data.materials.append(mat)

# ============================================
# ヘルパー
# ============================================
def clear_scene():
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()

def join_meshes(objects, name):
    if not objects:
        return None
    bpy.ops.object.select_all(action='DESELECT')
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name
    return result

# ============================================
# 検証関数（スクショ削減のため）
# ============================================
def validate_tool_proportions(handle_len, head_width, handle_radius, total_height):
    """ツールの比率を検証"""
    errors = []

    # ハンドル長は全長の60-70%
    handle_ratio = handle_len / total_height
    if not 0.60 <= handle_ratio <= 0.75:
        errors.append(f"Handle ratio {handle_ratio:.2f} not in 0.60-0.75")

    # ヘッド幅はハンドル直径の4-6倍
    head_handle_ratio = head_width / (handle_radius * 2)
    if not 3.5 <= head_handle_ratio <= 6.0:
        errors.append(f"Head/handle ratio {head_handle_ratio:.1f} not in 3.5-6.0x")

    # 全長チェック
    if not 0.18 <= total_height <= 0.28:
        errors.append(f"Total height {total_height:.3f} not in 0.18-0.28")

    if errors:
        print("❌ VALIDATION FAILED:")
        for e in errors:
            print(f"   - {e}")
        return False
    else:
        print("✅ Proportions OK")
        return True

# ============================================
# ピッケル生成
# ============================================
def create_pickaxe():
    clear_scene()

    P = PICKAXE
    parts_handle = []
    parts_head = []

    # === 座標計算（相対計算で接続保証）===
    # 目標: 全長0.22、ハンドル66%=0.145
    target_total = P["total_height"]  # 0.22

    handle_bottom = 0.0
    handle_top = P["handle_length"]  # 0.145
    handle_center = handle_bottom + P["handle_length"] / 2

    # カラーはハンドル上端に接続（少し重なる）
    collar_bottom = handle_top - 0.005  # 5mm重なり
    collar_top = collar_bottom + P["collar_height"]
    collar_z = collar_bottom + P["collar_height"] / 2

    # ヘッドはカラー上端に接続（少し重なる）
    head_bottom = collar_top - 0.003  # 3mm重なり
    head_z = head_bottom + P["head_height"] / 2
    head_top = head_z + P["head_height"] / 2

    total_height = head_top

    print("=== PICKAXE DIMENSIONS ===")
    print(f"Handle: bottom={handle_bottom:.4f}, top={handle_top:.4f}")
    print(f"Collar: bottom={collar_bottom:.4f}, top={collar_top:.4f}, center={collar_z:.4f}")
    print(f"Head: bottom={head_bottom:.4f}, top={head_top:.4f}, center={head_z:.4f}")
    print(f"Total height: {total_height:.4f}")
    print(f"Handle-Collar overlap: {handle_top - collar_bottom:.4f} (should be > 0)")
    print(f"Collar-Head overlap: {collar_top - head_bottom:.4f} (should be > 0)")

    # === ハンドル部分（木製）===

    # メインハンドル
    handle = create_oct_prism(
        P["handle_radius"],
        P["handle_length"],
        (0, 0, handle_center),
        "Handle"
    )
    parts_handle.append(handle)

    # グリップリング（4本）
    grip_spacing = 0.018
    grip_start = 0.012
    for i in range(4):
        grip = create_oct_prism(
            P["handle_radius"] * 1.18,
            0.006,
            (0, 0, grip_start + i * grip_spacing),
            f"Grip_{i}"
        )
        parts_handle.append(grip)

    # キャップ（底部）
    cap = create_oct_prism(
        P["handle_radius"] * 1.25,
        0.01,
        (0, 0, 0.005),
        "Cap"
    )
    parts_handle.append(cap)

    # === ヘッド部分（金属）===

    # カラー（接続部）
    collar = create_oct_prism(
        P["collar_radius"],
        P["collar_height"],
        (0, 0, collar_z),
        "Collar"
    )
    parts_head.append(collar)

    # ヘッド中央ブロック
    head_center_width = P["head_width"] * 0.35  # 中央は35%幅
    head = create_chamfered_box(
        (head_center_width, P["head_depth"], P["head_height"]),
        (0, 0, head_z),
        "HeadCenter"
    )
    parts_head.append(head)

    # ピック先端（右）- テーパー付き
    tip_base_w = head_center_width * 0.8
    tip_base_d = P["head_depth"] * 0.8
    tip_base_h = P["head_height"] * 0.7
    tip_end_w = tip_base_w * 0.3  # 先端は30%に
    tip_end_d = tip_base_d * 0.4
    tip_end_h = tip_base_h * 0.4

    tip_r = create_tapered_box(
        (tip_base_w, tip_base_d, tip_base_h),
        (tip_end_w, tip_end_d, tip_end_h),
        P["pick_tip_length"],
        (head_center_width / 2, 0, head_z),
        'X+',
        "PickTip_R"
    )
    parts_head.append(tip_r)

    # ピック先端（左）
    tip_l = create_tapered_box(
        (tip_base_w, tip_base_d, tip_base_h),
        (tip_end_w, tip_end_d, tip_end_h),
        P["pick_tip_length"],
        (-head_center_width / 2, 0, head_z),
        'X-',
        "PickTip_L"
    )
    parts_head.append(tip_l)

    # === マテリアル適用 ===
    mat_wood = create_material("Wood", (0.55, 0.35, 0.15), metallic=0.0, roughness=0.8)
    mat_iron = create_material("Iron", (0.4, 0.4, 0.42), metallic=0.85, roughness=0.45)

    for obj in parts_handle:
        apply_material(obj, mat_wood)
    for obj in parts_head:
        apply_material(obj, mat_iron)

    # === 結合 ===
    all_parts = parts_handle + parts_head
    result = join_meshes(all_parts, "Pickaxe")

    # === 原点設定（アイテムは中心）===
    bpy.context.view_layer.objects.active = result
    result.select_set(True)
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)
    bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
    result.location = (0, 0, 0)

    # === 検証 ===
    tri_count = sum(len(p.vertices) - 2 for p in result.data.polygons)
    print(f"\n=== MODEL VALIDATION ===")
    print(f"Total height: {total_height:.4f} (expected: 0.18-0.25)")
    print(f"Handle length: {P['handle_length']:.4f} ({P['handle_length']/total_height*100:.1f}% of total)")
    print(f"Head width: {P['head_width']:.4f}")
    print(f"Head/handle ratio: {P['head_width']/(P['handle_radius']*2):.1f}x (expected: 4-6x)")
    print(f"Triangle count: {tri_count} (budget: 50-200)")

    validate_tool_proportions(
        P["handle_length"],
        P["head_width"],
        P["handle_radius"],
        total_height
    )

    return result

# ============================================
# 実行
# ============================================
if __name__ == "__main__":
    pickaxe = create_pickaxe()

    # エクスポート
    output_path = os.path.join(script_dir, "../../assets/models/items/pickaxe.gltf")
    bpy.ops.export_scene.gltf(
        filepath=output_path,
        export_format='GLTF_SEPARATE'
    )
    print(f"\n✅ Pickaxe exported to {output_path}")
