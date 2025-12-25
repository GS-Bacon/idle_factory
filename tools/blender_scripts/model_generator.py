"""
Model Generator - JSON定義からBlenderスクリプトを自動生成

使い方:
1. model_definitions/ にJSONファイルを配置
2. python model_generator.py [model_name] を実行
3. 生成されたスクリプトがBlenderで実行される

JSON定義例:
{
    "name": "copper_ingot",
    "category": "item",
    "parts": [
        {"type": "ingot", "material": "copper"}
    ]
}
"""

import json
import os
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DEFINITIONS_DIR = os.path.join(SCRIPT_DIR, "model_definitions")


def generate_script(definition: dict) -> str:
    """JSON定義からBlenderスクリプトを生成"""
    name = definition["name"]
    category = definition.get("category", "item")
    parts = definition.get("parts", [])

    # ヘッダー
    script = f'''"""
Auto-generated model: {name}
Category: {category}
"""

import bpy
from mathutils import Vector
from math import pi
import os

exec(open("tools/blender_scripts/_base.py").read())

def create_{name}():
    clear_scene()
    parts = []

'''

    # パーツ生成
    for i, part in enumerate(parts):
        part_type = part.get("type")
        part_name = part.get("name", f"Part_{i}")
        location = part.get("location", (0, 0, 0))
        material = part.get("material", "iron")
        rotation = part.get("rotation", None)

        loc_str = f"({location[0]}, {location[1]}, {location[2]})"

        if part_type == "ingot":
            width = part.get("width", 0.08)
            length = part.get("length", 0.12)
            height = part.get("height", 0.03)
            script += f'''    # {part_name}
    obj = create_ingot(width={width}, length={length}, height={height}, material="{material}")
    obj.location = Vector({loc_str})
    obj.name = "{part_name}"
    parts.append(obj)

'''

        elif part_type == "plate":
            width = part.get("width", 0.1)
            length = part.get("length", 0.1)
            thickness = part.get("thickness", 0.008)
            script += f'''    # {part_name}
    obj = create_plate(width={width}, length={length}, thickness={thickness}, material="{material}")
    obj.location = Vector({loc_str})
    obj.name = "{part_name}"
    parts.append(obj)

'''

        elif part_type == "ore_chunk":
            size = part.get("size", 0.06)
            script += f'''    # {part_name}
    obj = create_ore_chunk(size={size}, material="{material}")
    obj.location = Vector({loc_str})
    obj.name = "{part_name}"
    parts.append(obj)

'''

        elif part_type == "dust_pile":
            radius = part.get("radius", 0.04)
            height = part.get("height", 0.025)
            script += f'''    # {part_name}
    obj = create_dust_pile(radius={radius}, height={height}, material="{material}")
    obj.location = Vector({loc_str})
    obj.name = "{part_name}"
    parts.append(obj)

'''

        elif part_type == "tool_handle":
            length = part.get("length", 0.15)
            radius = part.get("radius", 0.012)
            script += f'''    # {part_name}
    obj = create_tool_handle(length={length}, radius={radius}, material="{material}")
    obj.location = Vector({loc_str})
    obj.name = "{part_name}"
    parts.append(obj)

'''

        elif part_type == "machine_frame":
            width = part.get("width", 0.9)
            depth = part.get("depth", 0.9)
            height = part.get("height", 0.3)
            script += f'''    # {part_name}
    obj = create_machine_frame(width={width}, depth={depth}, height={height}, material="{material}")
    obj.name = "{part_name}"
    parts.append(obj)

'''

        elif part_type == "machine_body":
            width = part.get("width", 0.9)
            depth = part.get("depth", 0.9)
            height = part.get("height", 0.6)
            script += f'''    # {part_name}
    obj = create_machine_body(width={width}, depth={depth}, height={height}, material="{material}")
    obj.name = "{part_name}"
    parts.append(obj)

'''

        elif part_type == "tank_body":
            radius = part.get("radius", 0.4)
            height = part.get("height", 0.6)
            script += f'''    # {part_name}
    tank_parts = create_tank_body(radius={radius}, height={height}, material="{material}")
    parts.extend(tank_parts)

'''

        elif part_type == "gear":
            radius = part.get("radius", 0.25)
            thickness = part.get("thickness", 0.08)
            teeth = part.get("teeth", 8)
            script += f'''    # {part_name}
    obj = create_gear(radius={radius}, thickness={thickness}, teeth={teeth}, location={loc_str}, name="{part_name}")
    apply_preset_material(obj, "{material}")
    parts.append(obj)

'''

        elif part_type == "pipe":
            radius = part.get("radius", 0.08)
            length = part.get("length", 0.3)
            wall = part.get("wall", 0.015)
            script += f'''    # {part_name}
    obj = create_pipe(radius={radius}, length={length}, wall={wall}, location={loc_str}, name="{part_name}")
    apply_preset_material(obj, "{material}")
    parts.append(obj)

'''

        elif part_type == "motor_housing":
            radius = part.get("radius", 0.2)
            height = part.get("height", 0.15)
            script += f'''    # {part_name}
    obj = create_motor_housing(radius={radius}, height={height}, location={loc_str}, material="{material}")
    obj.name = "{part_name}"
    parts.append(obj)

'''

        elif part_type == "corner_bolts":
            width = part.get("width", 0.9)
            depth = part.get("depth", 0.9)
            z_pos = location[2]
            bolt_size = part.get("bolt_size", 0.04)
            script += f'''    # {part_name}
    bolts = create_corner_bolts(width={width}, depth={depth}, z_pos={z_pos}, bolt_size={bolt_size}, material="{material}")
    parts.extend(bolts)

'''

        elif part_type == "reinforcement_ribs":
            width = part.get("width", 0.9)
            depth = part.get("depth", 0.9)
            z_pos = location[2]
            script += f'''    # {part_name}
    ribs = create_reinforcement_ribs(width={width}, depth={depth}, z_pos={z_pos}, material="{material}")
    parts.extend(ribs)

'''

        elif part_type == "chamfered_cube":
            size = part.get("size", (0.5, 0.5, 0.5))
            chamfer = part.get("chamfer", None)
            size_str = f"({size[0]}, {size[1]}, {size[2]})"
            chamfer_str = f", chamfer={chamfer}" if chamfer else ""
            script += f'''    # {part_name}
    obj = create_chamfered_cube(size={size_str}{chamfer_str}, location={loc_str}, name="{part_name}")
    apply_preset_material(obj, "{material}")
    parts.append(obj)

'''

        elif part_type == "octagonal_prism":
            radius = part.get("radius", 0.2)
            height = part.get("height", 0.5)
            script += f'''    # {part_name}
    obj = create_octagonal_prism(radius={radius}, height={height}, location={loc_str}, name="{part_name}")
    apply_preset_material(obj, "{material}")
    parts.append(obj)

'''

        # 回転がある場合
        if rotation:
            rot_str = f"({rotation[0]}, {rotation[1]}, {rotation[2]})"
            script += f'''    obj.rotation_euler = {rot_str}
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.transform_apply(rotation=True)

'''

    # 結合と仕上げ
    script += f'''    # 結合
    if len(parts) == 1:
        result = parts[0]
    else:
        result = join_all_meshes(parts, "{name}")

    result.name = "{name}"

    # バリデーション
    print_validation_report(result, category="{category}")

    # 仕上げ
    finalize_model(result, category="{category}")

    return result


def export():
    model = create_{name}()
    output_dir = "assets/models/{category}s"
    import os
    os.makedirs(output_dir, exist_ok=True)
    filepath = os.path.join(output_dir, "{name}.gltf")
    export_gltf(filepath, export_animations=False)
    print(f"Exported: {{filepath}}")


if __name__ == "__main__":
    export()
'''

    return script


def load_definition(name: str) -> dict:
    """JSON定義を読み込む"""
    filepath = os.path.join(DEFINITIONS_DIR, f"{name}.json")
    if not os.path.exists(filepath):
        raise FileNotFoundError(f"Definition not found: {filepath}")

    with open(filepath, 'r', encoding='utf-8') as f:
        return json.load(f)


def generate_from_definition(name: str) -> str:
    """定義名からスクリプトを生成"""
    definition = load_definition(name)
    return generate_script(definition)


def main():
    if len(sys.argv) < 2:
        print("Usage: python model_generator.py <model_name>")
        print("\nAvailable definitions:")
        if os.path.exists(DEFINITIONS_DIR):
            for f in os.listdir(DEFINITIONS_DIR):
                if f.endswith('.json'):
                    print(f"  - {f[:-5]}")
        return

    name = sys.argv[1]
    script = generate_from_definition(name)
    print(script)


if __name__ == "__main__":
    main()
