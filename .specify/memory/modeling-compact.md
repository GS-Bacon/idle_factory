# 3D Modeling Quick Reference

## Core Rules
- **Kit-bashing only**: No vertex editing. Combine primitives.
- **Hierarchy**: All parts under root Empty or joined mesh
- **Origin**: machines=bottom_center, items=center

### Part Connection (CRITICAL)
**全パーツは必ず接続する。浮いているパーツは禁止。**

```python
# ❌ NG: 独立した固定値で配置 → 隙間ができる
head_z = 0.17
handle_length = 0.15  # head_zと無関係！

# ✅ OK: 基準パーツから相対計算 → 必ず接続
head_z = 0.17
handle_top = head_z + 0.01  # ヘッド中心を貫通
handle_length = handle_top - handle_bottom
handle_center = handle_bottom + handle_length / 2
```

**接続パターン**:
- 貫通: `part_top = other_center + small_value`
- 重なり: `part_edge = other_edge - overlap`

## _base.py Functions

### Primitives
| Function | Use |
|----------|-----|
| `create_chamfered_cube(size, chamfer, location, name)` | Boxes, frames |
| `create_octagonal_prism(radius, height, location, name)` | Cylinders, shafts |
| `create_hexagon(radius, depth, location, name)` | Bolt heads |
| `create_trapezoid(top_w, bottom_w, height, depth, loc, name)` | Gear teeth, blades |

### Mechanical Parts
| Function | Use |
|----------|-----|
| `create_gear(radius, thickness, teeth, hole_radius, loc, name)` | Gears |
| `create_shaft(radius, length, location, name)` | Shafts |
| `create_pipe(radius, length, wall, location, name)` | Pipes |
| `create_bolt(size, length, location, name)` | Bolts |
| `create_roller(radius, length, location, name)` | Rollers |

### Workflow
| Function | Use |
|----------|-----|
| `clear_scene()` | Start fresh |
| `join_all_meshes(objects, name)` | Combine parts |
| `apply_preset_material(obj, preset)` | iron/copper/brass/dark_steel/wood/stone |
| `finalize_model(obj, category)` | Set origin |
| `export_gltf(filepath)` | Export |

## Materials
- `iron`: #4A4A4A, metallic, roughness 0.5
- `copper`: #B87333, metallic
- `brass`: #C9A227, metallic (gears, decorative)
- `dark_steel`: #2D2D2D, metallic (heavy machines)
- `wood`: #8B6914, non-metallic
- `stone`: #696969, non-metallic

## Scale Reference
| Category | Size | Tri Budget |
|----------|------|------------|
| handheld_item | 0.3x0.3x0.3 | 50-200 (max 500) |
| machine | 1.0x1.0x1.0 | 200-800 (max 1500) |
| structure | varies | 500-2000 |

## Script Template
```python
exec(open("tools/blender_scripts/_base.py").read())
clear_scene()

parts = []
# Create parts...
body = create_chamfered_cube((0.9, 0.9, 0.5), location=(0,0,0.25), name="Body")
parts.append(body)

result = join_all_meshes(parts, name="ModelName")
apply_preset_material(result, "iron")
finalize_model(result, "machine")  # or "item"
export_gltf("assets/models/machines/model_name.gltf")
```

## High-Level Parts (NEW)

### Item Parts
| Function | Use | Default Size |
|----------|-----|--------------|
| `create_tool_handle(length, radius, material, grip_grooves)` | Tool handles | 0.15 length |
| `create_ingot(width, length, height, material)` | Metal ingots | 0.08x0.12x0.03 |
| `create_ore_chunk(size, material)` | Ore chunks | 0.06 |
| `create_plate(width, length, thickness, material)` | Metal plates | 0.1x0.1x0.008 |
| `create_dust_pile(radius, height, material)` | Dust/powder | 0.04 radius |

### Machine Parts
| Function | Use | Default Size |
|----------|-----|--------------|
| `create_machine_frame(width, depth, height, material)` | Base frame | 0.9x0.9x0.3 |
| `create_machine_body(width, depth, height, material)` | Main body | 0.9x0.9x0.6 |
| `create_tank_body(radius, height, material)` | Tank (returns list) | 0.4 radius |
| `create_motor_housing(radius, height, location, material)` | Motor | 0.2 radius |
| `create_corner_bolts(width, depth, z_pos, bolt_size, material)` | 4 corner bolts | 0.04 size |
| `create_reinforcement_ribs(width, depth, z_pos, material)` | 4 edge ribs | 0.06 size |
| `add_decorative_bolts_circle(radius, z_pos, count, bolt_size, material)` | Circular bolts | 8 count |
| `create_accent_light(size, location, color_preset)` | Status light | 0.05 size |

## Dimension Guidelines

### Items (handheld)
- Handle: length 0.15-0.18, radius 0.01-0.012
- Tool head: 0.05-0.1 width
- Total height: ~0.2-0.25

### Machines (1 block)
- Base: 0.9x0.9, height 0.1-0.2
- Body: 0.8-0.9 width, height varies
- Details: bolts 0.03-0.04 size, pipes 0.06-0.1 radius

## Quick Examples

### Simple Machine
```python
exec(open("tools/blender_scripts/_base.py").read())
clear_scene()

parts = []
parts.append(create_machine_frame())  # Base
body = create_machine_body(height=0.5)
body.location.z = 0.3
parts.append(body)
parts.extend(create_corner_bolts(z_pos=0.6))

result = join_all_meshes(parts, "SimpleMachine")
finalize_model(result, "machine")
export_gltf("assets/models/machines/simple_machine.gltf")
```

### Simple Item (Ingot)
```python
exec(open("tools/blender_scripts/_base.py").read())
clear_scene()

ingot = create_ingot(material="copper")
finalize_model(ingot, "item")
export_gltf("assets/models/items/copper_ingot.gltf")
```
