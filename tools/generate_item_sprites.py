#!/usr/bin/env python3
"""
Item Sprite Generator - アイテムアイコン画像を生成

Blenderを使って32x32ピクセルのアイテムアイコンを生成する。
シンプルなアイソメトリック風アイコン。

使い方:
    blender --background --python generate_item_sprites.py
"""

import bpy
import os
from pathlib import Path


# アイテム定義: (名前, 色RGB, 形状タイプ)
ITEMS = {
    # 資源
    "iron_ore": ((0.6, 0.5, 0.4), "rock"),
    "copper_ore": ((0.7, 0.4, 0.3), "rock"),
    "coal": ((0.15, 0.15, 0.15), "rock"),
    "stone": ((0.5, 0.5, 0.5), "rock"),

    # 加工品
    "iron_ingot": ((0.8, 0.8, 0.85), "ingot"),
    "copper_ingot": ((0.9, 0.5, 0.3), "ingot"),

    # 機械
    "miner": ((0.8, 0.6, 0.2), "machine"),
    "conveyor": ((0.3, 0.3, 0.35), "belt"),
    "crusher": ((0.4, 0.3, 0.5), "machine"),
    "furnace": ((0.55, 0.35, 0.27), "furnace"),

    # 新機械
    "assembler": ((0.2, 0.8, 0.4), "machine"),
    "storage": ((0.55, 0.41, 0.08), "crate"),
    "generator": ((0.45, 0.45, 0.5), "generator"),
    "inserter": ((0.45, 0.45, 0.47), "arm"),
}


def clear_scene():
    """シーンをクリア"""
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()

    # カメラとライトを再作成
    bpy.ops.object.camera_add(location=(2, -2, 2))
    camera = bpy.context.object
    camera.rotation_euler = (1.1, 0, 0.78)
    bpy.context.scene.camera = camera

    bpy.ops.object.light_add(type='SUN', location=(3, -3, 5))
    light = bpy.context.object
    light.data.energy = 3


def create_rock_shape(color):
    """岩のような不規則な形状"""
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1, radius=0.4, location=(0, 0, 0))
    obj = bpy.context.object

    # 少しつぶす
    obj.scale = (1.2, 1.0, 0.8)
    bpy.ops.object.transform_apply(scale=True)

    # マテリアル
    mat = bpy.data.materials.new("RockMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.9
    obj.data.materials.append(mat)

    return obj


def create_ingot_shape(color):
    """インゴット（金属の延べ棒）形状"""
    bpy.ops.mesh.primitive_cube_add(size=0.6, location=(0, 0, 0))
    obj = bpy.context.object

    # 台形っぽく
    obj.scale = (1.5, 0.8, 0.4)
    bpy.ops.object.transform_apply(scale=True)

    # マテリアル（金属的）
    mat = bpy.data.materials.new("IngotMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Metallic"].default_value = 0.9
    bsdf.inputs["Roughness"].default_value = 0.3
    obj.data.materials.append(mat)

    return obj


def create_machine_shape(color):
    """機械の形状（箱型）"""
    bpy.ops.mesh.primitive_cube_add(size=0.7, location=(0, 0, 0))
    obj = bpy.context.object

    # マテリアル
    mat = bpy.data.materials.new("MachineMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Metallic"].default_value = 0.6
    bsdf.inputs["Roughness"].default_value = 0.5
    obj.data.materials.append(mat)

    return obj


def create_belt_shape(color):
    """コンベアベルト形状"""
    bpy.ops.mesh.primitive_cube_add(size=0.5, location=(0, 0, 0))
    obj = bpy.context.object
    obj.scale = (1.5, 1.5, 0.3)
    bpy.ops.object.transform_apply(scale=True)

    # マテリアル
    mat = bpy.data.materials.new("BeltMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.7
    obj.data.materials.append(mat)

    return obj


def create_furnace_shape(color):
    """炉の形状"""
    # メインボディ
    bpy.ops.mesh.primitive_cube_add(size=0.6, location=(0, 0, 0))
    body = bpy.context.object

    # 煙突
    bpy.ops.mesh.primitive_cylinder_add(radius=0.1, depth=0.4, location=(0.15, 0, 0.5))
    chimney = bpy.context.object

    # 結合
    body.select_set(True)
    chimney.select_set(True)
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.join()
    obj = bpy.context.object

    # マテリアル
    mat = bpy.data.materials.new("FurnaceMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.8
    obj.data.materials.append(mat)

    return obj


def create_crate_shape(color):
    """木箱形状"""
    bpy.ops.mesh.primitive_cube_add(size=0.7, location=(0, 0, 0))
    obj = bpy.context.object

    # マテリアル（木材風）
    mat = bpy.data.materials.new("CrateMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.9
    obj.data.materials.append(mat)

    return obj


def create_generator_shape(color):
    """発電機形状"""
    # メインボディ
    bpy.ops.mesh.primitive_cube_add(size=0.5, location=(0, 0, 0))
    body = bpy.context.object

    # タービン部
    bpy.ops.mesh.primitive_cylinder_add(radius=0.2, depth=0.3, location=(0.35, 0, 0))
    bpy.context.object.rotation_euler = (0, 1.57, 0)
    turbine = bpy.context.object

    # 結合
    body.select_set(True)
    turbine.select_set(True)
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.join()
    obj = bpy.context.object

    # マテリアル
    mat = bpy.data.materials.new("GeneratorMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Metallic"].default_value = 0.7
    bsdf.inputs["Roughness"].default_value = 0.4
    obj.data.materials.append(mat)

    return obj


def create_arm_shape(color):
    """ロボットアーム形状"""
    # ベース
    bpy.ops.mesh.primitive_cylinder_add(radius=0.2, depth=0.15, location=(0, 0, 0))
    base = bpy.context.object

    # アーム
    bpy.ops.mesh.primitive_cube_add(size=0.1, location=(0, 0.3, 0.2))
    bpy.context.object.scale = (1, 4, 1)
    arm = bpy.context.object

    # 結合
    base.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = base
    bpy.ops.object.join()
    obj = bpy.context.object

    # マテリアル
    mat = bpy.data.materials.new("ArmMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Metallic"].default_value = 0.8
    bsdf.inputs["Roughness"].default_value = 0.3
    obj.data.materials.append(mat)

    return obj


def setup_render():
    """レンダリング設定"""
    scene = bpy.context.scene
    scene.render.resolution_x = 32
    scene.render.resolution_y = 32
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = 'PNG'
    scene.render.image_settings.color_mode = 'RGBA'

    # Eeveeを使用（高速）
    scene.render.engine = 'BLENDER_EEVEE'


def render_item(name, output_dir):
    """アイテムをレンダリング"""
    output_path = str(output_dir / f"{name}.png")
    bpy.context.scene.render.filepath = output_path
    bpy.ops.render.render(write_still=True)
    print(f"Rendered: {output_path}")


def generate_all_sprites():
    """全アイテムスプライトを生成"""
    output_dir = Path("/home/bacon/idle_factory/assets/textures/items")
    output_dir.mkdir(parents=True, exist_ok=True)

    setup_render()

    shape_creators = {
        "rock": create_rock_shape,
        "ingot": create_ingot_shape,
        "machine": create_machine_shape,
        "belt": create_belt_shape,
        "furnace": create_furnace_shape,
        "crate": create_crate_shape,
        "generator": create_generator_shape,
        "arm": create_arm_shape,
    }

    for name, (color, shape_type) in ITEMS.items():
        clear_scene()

        creator = shape_creators.get(shape_type, create_machine_shape)
        creator(color)

        render_item(name, output_dir)

    print(f"\nGenerated {len(ITEMS)} item sprites")


if __name__ == "__main__":
    generate_all_sprites()
