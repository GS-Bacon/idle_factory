#!/usr/bin/env python3
"""
HTMLプレビューのパラメータからBlender Pythonスクリプトを生成

使い方:
  # JSONファイルから
  python scripts/generate-blender-model.py config.json miner

  # 標準入力から
  echo '{"bodyWidth": 0.26, ...}' | python scripts/generate-blender-model.py - miner

  # クリップボードから（xclip必要）
  xclip -o | python scripts/generate-blender-model.py - conveyor
"""

import sys
import json

TEMPLATE = '''import bpy
import math
import os

# === 設定値（HTMLプレビューから自動生成） ===
config = {config_json}

def hex_to_rgb(hex_color):
    hex_color = hex_color.lstrip('#')
    return tuple(int(hex_color[i:i+2], 16) / 255.0 for i in (0, 2, 4))

def create_material(name, hex_color):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    rgb = hex_to_rgb(hex_color)
    bsdf.inputs["Base Color"].default_value = (*rgb, 1.0)
    return mat

# シーンクリア
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete()
for mat in bpy.data.materials:
    bpy.data.materials.remove(mat)

# マテリアル作成
mat_body = create_material("Body_Mat", config["bodyColor"])
mat_shaft = create_material("Shaft_Mat", config["shaftColor"])
mat_drill = create_material("Drill_Mat", config["drillColor"])
mat_leg = create_material("Leg_Mat", config["legColor"])
mat_indicator = create_material("Indicator_Mat", config["indicatorColor"])
mat_outlet = create_material("Outlet_Mat", config["outletColor"])
mat_motor = create_material("Motor_Mat", "#333333")

body_bottom_z = 0.0
body_center_z = body_bottom_z + config["bodyHeight"] / 2

# === Body ===
bpy.ops.mesh.primitive_cube_add(size=1, location=(0, 0, body_center_z))
body = bpy.context.active_object
body.name = "Body"
body.scale = (config["bodyWidth"], config["bodyDepth"], config["bodyHeight"])
bpy.ops.object.transform_apply(scale=True)
body.data.materials.append(mat_body)

body_top_z = body_bottom_z + config["bodyHeight"]

# === Motor ===
motor_radius = 0.05
motor_height = 0.06
motor_z = body_top_z + motor_height / 2
bpy.ops.mesh.primitive_cylinder_add(vertices=8, radius=motor_radius, depth=motor_height, location=(0, 0, motor_z))
motor = bpy.context.active_object
motor.name = "Motor"
motor.data.materials.append(mat_motor)

# === Indicator ===
indicator_size = 0.025
indicator_z = body_top_z - 0.02
indicator_y = config["bodyDepth"] / 2 + indicator_size / 2
bpy.ops.mesh.primitive_cube_add(size=indicator_size, location=(0, indicator_y, indicator_z))
indicator = bpy.context.active_object
indicator.name = "Indicator"
indicator.data.materials.append(mat_indicator)

# === Outlet ===
outlet_size = config["outletSize"]
outlet_depth = 0.04
outlet_y = config["bodyDepth"] / 2 + outlet_depth / 2
outlet_z = body_top_z / 2
bpy.ops.mesh.primitive_cube_add(size=1, location=(0, outlet_y, outlet_z))
outlet = bpy.context.active_object
outlet.name = "Outlet"
outlet.scale = (outlet_size, outlet_depth, outlet_size)
bpy.ops.object.transform_apply(scale=True)
outlet.data.materials.append(mat_outlet)

# Outlet Inner
mat_inner = create_material("OutletInner_Mat", "#1a1a1a")
inner_size = outlet_size * 0.6
bpy.ops.mesh.primitive_cube_add(size=1, location=(0, outlet_y + 0.01, outlet_z))
outlet_inner = bpy.context.active_object
outlet_inner.name = "OutletInner"
outlet_inner.scale = (inner_size, 0.02, inner_size)
bpy.ops.object.transform_apply(scale=True)
outlet_inner.data.materials.append(mat_inner)

# === Legs ===
leg_length = 0.15
leg_angles = [math.pi/4, 3*math.pi/4, 5*math.pi/4, 7*math.pi/4]
for i in range(int(config["legCount"])):
    angle = leg_angles[i] if i < 4 else (i / config["legCount"]) * math.pi * 2
    top_x = math.cos(angle) * (config["bodyWidth"] / 2 - 0.02)
    top_y = math.sin(angle) * (config["bodyDepth"] / 2 - 0.02)
    top_z = body_bottom_z
    bottom_x = math.cos(angle) * config["legSpread"]
    bottom_y = math.sin(angle) * config["legSpread"]
    bottom_z = -leg_length
    mid_x, mid_y, mid_z = (top_x + bottom_x) / 2, (top_y + bottom_y) / 2, (top_z + bottom_z) / 2
    dx, dy, dz = bottom_x - top_x, bottom_y - top_y, bottom_z - top_z
    length = math.sqrt(dx*dx + dy*dy + dz*dz)
    bpy.ops.mesh.primitive_cube_add(size=1, location=(mid_x, mid_y, mid_z))
    leg = bpy.context.active_object
    leg.name = f"Leg_{{i}}"
    leg.scale = (config["legThickness"], config["legThickness"], length)
    bpy.ops.object.transform_apply(scale=True)
    leg.rotation_euler.x = math.atan2(math.sqrt(dx*dx + dy*dy), -dz)
    leg.rotation_euler.z = math.atan2(dy, dx)
    leg.data.materials.append(mat_leg)

# === Shaft ===
shaft_top_z = body_bottom_z
shaft_bottom_z = shaft_top_z - config["shaftLength"]
shaft_center_z = (shaft_top_z + shaft_bottom_z) / 2
bpy.ops.mesh.primitive_cube_add(size=1, location=(0, 0, shaft_center_z))
shaft = bpy.context.active_object
shaft.name = "Shaft"
shaft.scale = (config["shaftWidth"], config["shaftWidth"], config["shaftLength"])
bpy.ops.object.transform_apply(scale=True)
shaft.data.materials.append(mat_shaft)

# === Drill ===
drill_radius = config["drillWidth"] / 2  # drillWidthは直径
drill_top_z = shaft_bottom_z
drill_tip_z = drill_top_z - config["drillLength"]
drill_center_z = (drill_top_z + drill_tip_z) / 2

drill_vertices = {{"cone": 16, "pyramid": 4, "octagon": 8, "spiral": 8}}.get(config["drillStyle"], 8)
bpy.ops.mesh.primitive_cone_add(vertices=drill_vertices, radius1=drill_radius, radius2=0, depth=config["drillLength"], location=(0, 0, drill_center_z))

drill = bpy.context.active_object
drill.name = "Drill"
drill.rotation_euler.x = math.pi  # 先端を下向きに
drill.data.materials.append(mat_drill)

# 全オブジェクトの回転・スケールを適用
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.transform_apply(rotation=True, scale=True)

print("Model created successfully!")
print(f"Body: {{config['bodyWidth']}} x {{config['bodyDepth']}} x {{config['bodyHeight']}}")
print(f"Drill: diameter={{config['drillWidth']}}, length={{config['drillLength']}}")

# === GLBエクスポート ===
export_path = "{export_path}"
os.makedirs(os.path.dirname(export_path), exist_ok=True)

bpy.ops.object.select_all(action='DESELECT')
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        obj.select_set(True)

bpy.ops.export_scene.gltf(
    filepath=export_path,
    export_format='GLB',
    use_selection=True,
    export_apply=True
)

file_size = os.path.getsize(export_path)
print(f"Exported to {{export_path}}")
print(f"File size: {{file_size / 1024:.1f}} KB")
'''

def generate_blender_code(config: dict, model_name: str) -> str:
    """パラメータからBlenderスクリプトを生成"""
    export_path = f"/home/bacon/idle_factory/assets/models/machines/{model_name}.glb"
    config_json = json.dumps(config, indent=4, ensure_ascii=False)
    return TEMPLATE.format(config_json=config_json, export_path=export_path)

def main():
    if len(sys.argv) < 3:
        print("Usage: python generate-blender-model.py <config.json | -> <model_name>")
        print("  config.json: JSONファイルパス、または '-' で標準入力")
        print("  model_name: 出力モデル名 (例: miner, conveyor)")
        sys.exit(1)

    config_source = sys.argv[1]
    model_name = sys.argv[2]

    if config_source == '-':
        config = json.load(sys.stdin)
    else:
        with open(config_source) as f:
            config = json.load(f)

    code = generate_blender_code(config, model_name)
    print(code)

if __name__ == "__main__":
    main()
