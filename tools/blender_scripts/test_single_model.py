"""
単一モデルテスト用スクリプト
使い方: blender --background --python tools/blender_scripts/test_single_model.py -- <model_name>
"""

import bpy
import sys
import os

# _base.py をロード
exec(open("tools/blender_scripts/_base.py").read())

# 引数からモデル名を取得
argv = sys.argv
if "--" in argv:
    argv = argv[argv.index("--") + 1:]
else:
    argv = []

model_name = argv[0] if argv else "bronze_pickaxe"

# 出力ディレクトリ
output_dir = "assets/models/items"
os.makedirs(output_dir, exist_ok=True)

# ピッケル関連関数をここに直接定義

def create_handle(length=0.15, radius=0.01, material="wood"):
    """木製八角柱ハンドル"""
    handle = create_octagonal_prism(
        radius=radius,
        height=length,
        location=(0, 0, 0),
        name="Handle"
    )
    apply_preset_material(handle, material)
    return handle

def create_pickaxe_head(material_preset="stone"):
    """ピッケル先端（T字型、尖った形）"""
    objects = []

    # 中央ブロック（柄との接続部）
    center = create_chamfered_cube(
        size=(0.08, 0.03, 0.03),
        location=(0, 0, 0),
        name="PickHead_center"
    )
    objects.append(center)

    # 左右の尖った先端
    for side in [-1, 1]:
        tip = create_chamfered_cube(
            size=(0.04, 0.025, 0.025),
            location=(side * 0.06, 0, 0),
            name=f"PickHead_tip_{side}"
        )
        tip.scale.x = 0.6
        bpy.context.view_layer.objects.active = tip
        bpy.ops.object.transform_apply(scale=True)
        objects.append(tip)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = center
    for obj in objects:
        obj.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object

    apply_preset_material(result, material_preset)
    return result

def create_pickaxe(name, material_preset):
    """ピッケル生成"""
    clear_scene()

    # ハンドル（縦向き）
    handle = create_handle(length=0.18, radius=0.012)

    # ヘッド（ハンドル上部に配置）
    head = create_pickaxe_head(material_preset)
    head.location.z = 0.09  # ハンドル上端
    head.rotation_euler.z = pi / 2  # Z軸回転でT字に

    # 回転を適用
    bpy.ops.object.select_all(action='DESELECT')
    head.select_set(True)
    bpy.context.view_layer.objects.active = head
    bpy.ops.object.transform_apply(rotation=True)

    # 結合
    bpy.ops.object.select_all(action='DESELECT')
    bpy.context.view_layer.objects.active = handle
    head.select_set(True)
    handle.select_set(True)
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name

    finalize_model(result, category="item")
    return result

# 実行
print(f"=== Creating {model_name} ===")

if model_name == "bronze_pickaxe":
    create_pickaxe("bronze_pickaxe", "brass")
    export_gltf(os.path.join(output_dir, "bronze_pickaxe.gltf"), export_animations=False)
    print("Exported: bronze_pickaxe.gltf")
else:
    print(f"Unknown model: {model_name}")
