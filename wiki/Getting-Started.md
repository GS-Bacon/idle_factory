[日本語版はこちら](はじめに)

# Getting Started

Create your first mod in 5 minutes.

---

## Prerequisites

- Idle Factory game installed
- Text editor (VS Code, Notepad++, etc.)

---

## Tutorial: Add a Diamond Item

### Step 1: Create Mod Folder

Create folder `mods/my_first_mod/` in the game directory.

```
mods/
└── my_first_mod/
    ├── mod.toml      ← Mod metadata
    └── items.toml    ← Item definitions
```

### Step 2: Create mod.toml

Create the mod metadata file.

```toml
[mod]
id = "my_first_mod"
name = "My First Mod"
version = "1.0.0"
author = "Your Name"
description = "Adds diamond ore and related items"

[mod.dependencies]
base = ">=0.3.0"
```

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier (lowercase, underscores OK) |
| `name` | Yes | Display name |
| `version` | Yes | Semantic versioning (e.g., 1.0.0) |
| `dependencies` | No | Required mods |

### Step 3: Create items.toml

Define your new items.

```toml
# Diamond Ore
[[item]]
id = "diamond_ore"
name = "Diamond Ore"
short_name = "DiaO"
description = "Rare ore containing diamonds"
stack_size = 999
category = "ore"
is_placeable = true
hardness = 2.0
color = [0.2, 0.8, 0.9]
tags = ["ore", "rare", "crushable"]

# Diamond
[[item]]
id = "diamond"
name = "Diamond"
short_name = "Dia"
description = "A brilliant gem for advanced crafting"
stack_size = 999
category = "processed"
is_placeable = false
color = [0.6, 0.95, 1.0]
tags = ["gem", "rare", "valuable"]
```

### Step 4: Launch the Game

1. Start Idle Factory
2. Mod loads automatically
3. Press E to open inventory

### Step 5: Verify Your Mod

Open console (T key) and type:

```
/item list my_first_mod
```

Expected output:

```
my_first_mod:diamond_ore - Diamond Ore
my_first_mod:diamond - Diamond
```

---

## Next Steps: Add a Recipe

Want your diamond ore to be useful? Add a recipe!

Create `recipes.toml`:

```toml
[[recipe]]
id = "crush_diamond_ore"
machine = "crusher"
craft_time = 3.0

[recipe.inputs]
diamond_ore = 1

[recipe.outputs]
diamond = 2
```

Now diamond ore can be crushed into diamonds!

---

## Next Steps: Add a Machine

Create a custom machine for your mod.

Create `machines.toml`:

```toml
[[machine]]
id = "gem_polisher"
name = "Gem Polisher"
block_type = "custom"
process_time = 5.0
buffer_size = 32
requires_fuel = false
process_type = "recipe"
machine_type = "gem_polisher"

[machine.ports]
inputs = [{ side = "back", slot_id = 0 }]
outputs = [{ side = "front", slot_id = 0 }]
```

---

## Troubleshooting

### Mod not loading

- Check `mod.toml` syntax
- Ensure `id` is unique and lowercase

### Items not appearing

- Verify `category` is valid: `terrain`, `ore`, `processed`, `machine`, `tool`

### Recipe not working

- Ensure input/output items exist
- Check machine type matches

### Check Logs

Look at `logs/game.log` for error details.

---

## Complete File Structure

```
mods/my_first_mod/
├── mod.toml        # Required
├── items.toml      # Optional
├── machines.toml   # Optional
├── recipes.toml    # Optional
├── textures/       # Optional: PNG files
│   └── diamond.png
└── models/         # Optional: GLB files
    └── gem_polisher.glb
```

---

## See Also

- **[Mod Structure](Mod-Structure)** - Detailed mod.toml reference
- **[Data Mod Guide](Data-Mod-Guide)** - Complete TOML documentation
- **[TOML Schema](TOML-Schema)** - All fields reference
