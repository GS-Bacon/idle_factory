"""Quick regenerate wrench and wire_cutter"""
import sys
import os

script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.append(script_dir)

from tools_items import create_wrench, create_wire_cutter
from _base import export_gltf

output_dir = os.path.join(script_dir, "..", "..", "assets", "models", "items")

print("=== Creating wrench ===")
create_wrench()
export_gltf(os.path.join(output_dir, "wrench.gltf"), export_animations=False)
print("Exported wrench")

print("=== Creating wire_cutter ===")
create_wire_cutter()
export_gltf(os.path.join(output_dir, "wire_cutter.gltf"), export_animations=False)
print("Exported wire_cutter")
