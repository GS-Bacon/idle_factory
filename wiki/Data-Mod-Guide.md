# Data Mod Guide

[日本語版はこちら](データMod作成)

Complete guide to adding items, machines, and recipes via TOML.

---

## Overview

Data Mods use TOML files to extend the game without writing any code.

| File | Purpose |
|------|---------|
| `items.toml` | Define new items |
| `machines.toml` | Define new machines |
| `recipes.toml` | Define crafting recipes |

---

## Items

### Basic Item

A simple non-placeable item.

```toml
[[item]]
id = "copper_wire"
name = "Copper Wire"
description = "Thin copper wire for electronics"
stack_size = 999
category = "processed"
```

### Placeable Block

An item that can be placed in the world.

```toml
[[item]]
id = "reinforced_stone"
name = "Reinforced Stone"
description = "Stronger than regular stone"
stack_size = 999
category = "terrain"
is_placeable = true
hardness = 2.0
color = [0.4, 0.4, 0.5]
```

### Item with Tags

Tags enable filtering and processing hints.

```toml
[[item]]
id = "gold_ore"
name = "Gold Ore"
description = "Precious metal ore"
stack_size = 999
category = "ore"
is_placeable = true
hardness = 1.5
color = [0.9, 0.8, 0.2]
tags = ["ore", "ore/gold", "smeltable", "crushable", "rare"]
```

### All Item Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | string | Yes | - | Unique ID within mod |
| `name` | string | Yes | - | Display name |
| `short_name` | string | No | First 4 chars | Short display (max 4) |
| `description` | string | No | "" | Tooltip text |
| `stack_size` | int | No | 999 | Max stack amount |
| `category` | string | Yes | - | See categories below |
| `is_placeable` | bool | No | false | Can be placed in world |
| `hardness` | float | No | 1.0 | Mining time multiplier |
| `color` | [f32;3] | No | [1,1,1] | RGB color [0-1] |
| `tags` | [string] | No | [] | Searchable tags |

### Categories

| Category | Description | Example |
|----------|-------------|---------|
| `terrain` | Natural blocks | stone, grass, dirt |
| `ore` | Mineable resources | iron_ore, coal |
| `processed` | Crafted materials | iron_ingot, copper_wire |
| `machine` | Placeable machines | furnace_block, miner_block |
| `tool` | Player tools | pickaxe |

### Tag Conventions

```toml
# Hierarchy with /
tags = ["ore", "ore/iron"]           # Iron ore
tags = ["ingot", "ingot/iron"]       # Iron ingot

# Processing hints
tags = ["smeltable"]                 # Can be smelted
tags = ["crushable"]                 # Can be crushed
tags = ["fuel"]                      # Can be used as fuel

# Custom mod tags
tags = ["mymod/special", "mymod/tier2"]
```

---

## Machines

### Basic Machine

A processing machine that uses recipes.

```toml
[[machine]]
id = "electric_furnace"
name = "Electric Furnace"
block_type = "custom"
process_time = 1.5
buffer_size = 64
requires_fuel = false
process_type = "recipe"
machine_type = "furnace"

[machine.ports]
inputs = [{ side = "back", slot_id = 0 }]
outputs = [{ side = "front", slot_id = 0 }]

[machine.ui_slots]
input = { slot_id = 0, label = "Input" }
output = { slot_id = 0, label = "Output" }
```

### Machine with Fuel

A machine that requires fuel to operate.

```toml
[[machine]]
id = "blast_furnace"
name = "Blast Furnace"
block_type = "custom"
process_time = 3.0
buffer_size = 64
requires_fuel = true
process_type = "recipe"
machine_type = "blast_furnace"

[machine.ports]
inputs = [
    { side = "back", slot_id = 0 },   # Material
    { side = "left", slot_id = 1 },   # Fuel
    { side = "right", slot_id = 1 }   # Fuel (alternative)
]
outputs = [{ side = "front", slot_id = 0 }]

[machine.ui_slots]
input = { slot_id = 0, label = "Material" }
fuel = { slot_id = 1, label = "Fuel" }
output = { slot_id = 0, label = "Output" }
```

### Auto-Generate Machine

A machine that produces output without input.

```toml
[[machine]]
id = "water_pump"
name = "Water Pump"
block_type = "custom"
process_time = 2.0
buffer_size = 100
requires_fuel = false
auto_generate = true
process_type = "auto_generate"

[machine.ports]
outputs = [{ side = "front", slot_id = 0 }]

[machine.ui_slots]
output = { slot_id = 0, label = "Output" }
```

### All Machine Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique ID |
| `name` | string | Yes | Display name |
| `block_type` | string | Yes | Associated block type |
| `process_time` | float | Yes | Seconds per operation |
| `buffer_size` | int | No | Internal buffer capacity |
| `requires_fuel` | bool | No | Needs fuel to operate |
| `auto_generate` | bool | No | Produces without input |
| `process_type` | string | Yes | `"recipe"` or `"auto_generate"` |
| `machine_type` | string | No | Recipe filter |

### Port Sides

| Side | Description |
|------|-------------|
| `front` | Output direction (machine facing) |
| `back` | Behind machine |
| `left` | Left side |
| `right` | Right side |
| `top` | Above |
| `bottom` | Below |

---

## Recipes

### Basic Recipe

A simple single-input, single-output recipe.

```toml
[[recipe]]
id = "smelt_gold"
machine = "furnace"
craft_time = 2.5

[recipe.inputs]
gold_ore = 1

[recipe.outputs]
gold_ingot = 1

[recipe.fuel]
coal = 1
```

### Multi-Input Recipe

A recipe requiring multiple different inputs.

```toml
[[recipe]]
id = "craft_circuit"
machine = "assembler"
craft_time = 5.0

[recipe.inputs]
copper_wire = 4
iron_ingot = 2

[recipe.outputs]
circuit_board = 1
```

### Multi-Output Recipe

A recipe producing multiple outputs (byproducts).

```toml
[[recipe]]
id = "crush_iron"
machine = "crusher"
craft_time = 1.5

[recipe.inputs]
iron_ore = 1

[recipe.outputs]
iron_dust = 2
stone = 1        # Byproduct
```

### Cross-Mod Recipe

A recipe using items from other mods.

```toml
[[recipe]]
id = "craft_advanced_circuit"
machine = "assembler"
craft_time = 10.0

[recipe.inputs]
base:iron_ingot = 2           # From base game
other_mod:special_dust = 1    # From another mod
my_item = 1                   # From this mod

[recipe.outputs]
advanced_circuit = 1
```

### All Recipe Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique ID |
| `machine` | string | Yes | Machine type that processes this |
| `craft_time` | float | Yes | Seconds to complete |
| `inputs` | table | Yes | Input items {id = count} |
| `outputs` | table | Yes | Output items {id = count} |
| `fuel` | table | No | Fuel items {id = count} |

---

## Complete Example: New Ore Chain

A full example adding titanium ore processing.

**items.toml**

```toml
[[item]]
id = "titanium_ore"
name = "Titanium Ore"
description = "Extremely hard ore"
stack_size = 999
category = "ore"
is_placeable = true
hardness = 2.5
color = [0.6, 0.7, 0.8]
tags = ["ore", "ore/titanium", "crushable"]

[[item]]
id = "titanium_dust"
name = "Titanium Dust"
description = "Crushed titanium ore"
stack_size = 999
category = "processed"
color = [0.7, 0.8, 0.9]
tags = ["dust", "dust/titanium", "smeltable"]

[[item]]
id = "titanium_ingot"
name = "Titanium Ingot"
description = "Strong and lightweight metal"
stack_size = 999
category = "processed"
color = [0.8, 0.85, 0.9]
tags = ["ingot", "ingot/titanium", "metal"]
```

**recipes.toml**

```toml
[[recipe]]
id = "crush_titanium"
machine = "crusher"
craft_time = 2.0

[recipe.inputs]
titanium_ore = 1

[recipe.outputs]
titanium_dust = 2

[[recipe]]
id = "smelt_titanium"
machine = "furnace"
craft_time = 3.0

[recipe.inputs]
titanium_dust = 1

[recipe.outputs]
titanium_ingot = 1

[recipe.fuel]
coal = 2
```

---

## Validation

The game validates TOML on load. Check `logs/game.log` for errors.

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `duplicate id` | Same ID used twice | Use unique IDs |
| `unknown category` | Invalid category | Use valid category |
| `missing field` | Required field absent | Add the field |
| `item not found` | Recipe references missing item | Check item ID |

---

## See Also

- **[Getting Started](Getting-Started)** - Quick tutorial
- **[Mod Structure](Mod-Structure)** - mod.toml reference
- **[TOML Schema](TOML-Schema)** - Complete field reference
