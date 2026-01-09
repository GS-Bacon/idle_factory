[日本語版はこちら](TOMLスキーマ)

# TOML Schema

Complete field reference for all TOML files.

---

## mod.toml

```toml
[mod]
id = "string"              # Required: unique lowercase identifier
name = "string"            # Required: display name
version = "string"         # Required: semantic version (X.Y.Z)
author = "string"          # Optional: creator name
description = "string"     # Optional: short description
license = "string"         # Optional: license (MIT, GPL, etc.)
homepage = "string"        # Optional: URL
type = "data|core"         # Optional: default "data"

[mod.dependencies]
mod_id = "version_constraint"  # e.g., base = ">=0.3.0"

[mod.optional_dependencies]
mod_id = "version_constraint"

[mod.incompatible]
mod_id = "version_constraint"
```

### Version Constraints

| Pattern | Meaning |
|---------|---------|
| `">=1.0.0"` | 1.0.0 or higher |
| `"^1.2.0"` | >=1.2.0, <2.0.0 (compatible) |
| `"~1.2.0"` | >=1.2.0, <1.3.0 (patch only) |
| `"=1.2.3"` | Exact match |
| `"*"` | Any version |

---

## items.toml

```toml
[[item]]
id = "string"              # Required: unique within mod
name = "string"            # Required: display name
short_name = "string"      # Optional: max 4 chars, default=first 4 of name
description = "string"     # Optional: tooltip text
stack_size = 999           # Optional: max stack, default=999
category = "string"        # Required: terrain|ore|processed|machine|tool
is_placeable = false       # Optional: can place in world
hardness = 1.0             # Optional: mining time multiplier
color = [1.0, 1.0, 1.0]    # Optional: RGB [0-1]
tags = ["string"]          # Optional: searchable tags
```

### Category Values

| Value | Use Case |
|-------|----------|
| `terrain` | Natural blocks (stone, grass) |
| `ore` | Mineable resources (iron_ore, coal) |
| `processed` | Crafted items (ingots, dust) |
| `machine` | Placeable machines |
| `tool` | Player equipment |

### Common Tags

| Tag | Meaning |
|-----|---------|
| `ore`, `ore/iron` | Ore type hierarchy |
| `ingot`, `ingot/iron` | Ingot type hierarchy |
| `dust`, `dust/iron` | Dust type hierarchy |
| `smeltable` | Can be smelted in furnace |
| `crushable` | Can be crushed in crusher |
| `fuel` | Can be used as fuel |
| `rare` | Rare item |
| `metal` | Metal material |

---

## machines.toml

```toml
[[machine]]
id = "string"              # Required: unique within mod
name = "string"            # Required: display name
name_en = "string"         # Optional: English name
block_type = "string"      # Required: associated block type
process_time = 1.0         # Required: seconds per operation
buffer_size = 64           # Optional: internal buffer capacity
requires_fuel = false      # Optional: needs fuel
auto_generate = false      # Optional: produces without input
process_type = "string"    # Required: recipe|auto_generate
machine_type = "string"    # Optional: recipe filter

[machine.ports]
inputs = [
    { side = "back", slot_id = 0 },
    { side = "left", slot_id = 1 }
]
outputs = [
    { side = "front", slot_id = 0 }
]

[machine.ui_slots]
input = { slot_id = 0, label = "Input" }
fuel = { slot_id = 1, label = "Fuel" }
output = { slot_id = 0, label = "Output" }
```

### Port Side Values

| Value | Direction |
|-------|-----------|
| `front` | Machine facing direction |
| `back` | Opposite of front |
| `left` | Left when facing front |
| `right` | Right when facing front |
| `top` | Above machine |
| `bottom` | Below machine |

### Process Type Values

| Value | Behavior |
|-------|----------|
| `recipe` | Processes items according to recipes |
| `auto_generate` | Produces output without input |

---

## recipes.toml

```toml
[[recipe]]
id = "string"              # Required: unique within mod
machine = "string"         # Required: machine_type that processes this
craft_time = 1.0           # Required: seconds to complete

[recipe.inputs]
item_id = 1                # item_id = count
another_item = 2           # Can have multiple inputs

[recipe.outputs]
result_item = 1            # item_id = count
byproduct = 1              # Can have multiple outputs

[recipe.fuel]              # Optional: only for machines with requires_fuel
coal = 1                   # fuel_item_id = count
```

### Item ID Resolution

| Format | Resolution |
|--------|------------|
| `iron_ingot` | Same mod: `my_mod:iron_ingot` |
| `base:iron_ingot` | Explicit: base game item |
| `other_mod:item` | Explicit: another mod's item |

---

## Type Reference

| Type | Format | Example |
|------|--------|---------|
| `string` | Quoted text | `"hello"` |
| `int` | Integer | `42` |
| `float` | Decimal | `1.5` |
| `bool` | Boolean | `true`, `false` |
| `[string]` | String array | `["a", "b"]` |
| `[f32;3]` | Float array (3) | `[1.0, 0.5, 0.2]` |
| `table` | Key-value | `{ key = value }` |

---

## Validation Rules

### IDs

- Lowercase only
- Underscores allowed
- No hyphens
- Cannot start with number
- Must be unique within scope

### Values

- `stack_size`: 1-9999
- `hardness`: 0.1-10.0
- `process_time`: 0.1-3600.0
- `buffer_size`: 1-1000
- `color`: Each component 0.0-1.0

### References

- Recipe inputs/outputs must reference existing items
- Recipe machine must match a machine's `machine_type`
- Dependencies must be loadable

---

## See Also

- [Data Mod Guide](Data-Mod-Guide) - Usage examples
- [Mod Structure](Mod-Structure) - File organization
