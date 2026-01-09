[日本語版はこちら](Mod構造)

# Mod Structure

Complete reference for mod.toml and folder organization.

---

## Folder Structure

```
mods/
└── your_mod_id/
    ├── mod.toml          # Required: Metadata
    ├── items.toml        # Item definitions
    ├── machines.toml     # Machine definitions
    ├── recipes.toml      # Recipe definitions
    ├── textures/         # PNG textures
    │   ├── my_item.png
    │   └── my_block_top.png
    ├── models/           # 3D models (GLB/GLTF)
    │   └── my_machine.glb
    └── scripts/          # Core Mod WASM
        └── main.wasm
```

---

## mod.toml Reference

### Minimal Example

```toml
[mod]
id = "my_mod"
name = "My Mod"
version = "1.0.0"
```

### Full Example

```toml
[mod]
id = "advanced_machines"
name = "Advanced Machines"
version = "2.1.0"
author = "ModderName"
description = "Adds tier 2 machines with improved efficiency"
license = "MIT"
homepage = "https://github.com/user/advanced-machines"
type = "data"  # "data" | "core"

[mod.dependencies]
base = ">=0.3.0"           # Base game version
another_mod = ">=1.0.0"    # Other mod dependency

[mod.optional_dependencies]
optional_mod = ">=1.0.0"   # Loads if available

[mod.incompatible]
broken_mod = "*"           # Cannot coexist
```

---

## Field Reference

### [mod] Section

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier. Lowercase, underscores allowed. Used as namespace prefix. |
| `name` | string | Yes | Human-readable name shown in UI |
| `version` | string | Yes | Semantic versioning (MAJOR.MINOR.PATCH) |
| `author` | string | No | Creator name |
| `description` | string | No | Short description |
| `license` | string | No | License (MIT, GPL, etc.) |
| `homepage` | string | No | URL to mod page |
| `type` | string | No | `"data"` (default) or `"core"` (WASM) |

### [mod.dependencies]

Version constraints:

| Syntax | Meaning |
|--------|---------|
| `">=1.0.0"` | 1.0.0 or higher |
| `"^1.2.0"` | Compatible with 1.2.0 (>=1.2.0, <2.0.0) |
| `"~1.2.0"` | Patch updates only (>=1.2.0, <1.3.0) |
| `"=1.2.3"` | Exact version |
| `"*"` | Any version |

---

## ID Naming Rules

### Mod ID

```
✓ my_mod
✓ advanced_machines
✓ bob_ores
✗ MyMod          (no uppercase)
✗ my-mod         (no hyphens)
✗ 123mod         (cannot start with number)
```

### Item/Machine/Recipe ID

Full ID format: `mod_id:item_id`

```toml
# In my_mod/items.toml
[[item]]
id = "super_ore"  # Full ID becomes: my_mod:super_ore

# Referencing in recipes
[recipe.inputs]
my_mod:super_ore = 1    # Explicit namespace
super_ore = 1           # Same mod, namespace optional
base:iron_ore = 1       # Base game item
```

---

## Load Order

1. `base` mod (always first)
2. Mods sorted by dependencies
3. Within same priority, alphabetical

### Debugging Load Order

Console command:
```
/mod list
```

Output:
```
[1] base v0.3.78 (enabled)
[2] my_first_mod v1.0.0 (enabled)
[3] advanced_machines v2.1.0 (enabled)
```

---

## Mod Types

### Data Mod (Default)

- TOML files only
- Add items, machines, recipes
- No code execution
- Safe, sandboxed

```toml
[mod]
type = "data"  # Optional, this is the default
```

### Core Mod (WASM)

- Rust compiled to WASM
- Custom game logic
- Access to host functions
- Requires explicit permission

```toml
[mod]
type = "core"
```

See [Core Mod Guide](Core-Mod-Guide) for WASM development.

---

## Asset Paths

### Textures

```
mods/my_mod/textures/
├── my_item.png           → my_mod:my_item
├── my_block.png          → my_mod:my_block (all sides)
├── my_block_top.png      → my_mod:my_block (top)
├── my_block_side.png     → my_mod:my_block (sides)
└── my_block_bottom.png   → my_mod:my_block (bottom)
```

**Requirements:**
- Format: PNG (RGBA)
- Size: 16x16 or 32x32 pixels
- Power of 2 recommended

### Models

```
mods/my_mod/models/
└── my_machine.glb        → Referenced in machines.toml
```

**Requirements:**
- Format: GLB or GLTF
- Origin: Center bottom
- Scale: 1 unit = 1 block

---

## Best Practices

### 1. Use Descriptive IDs

```toml
# Good
id = "titanium_drill"
id = "advanced_furnace"

# Bad
id = "item1"
id = "machine_new"
```

### 2. Namespace Your Tags

```toml
tags = ["mymod/special", "ore", "rare"]
```

### 3. Version Properly

- Bug fix: 1.0.0 → 1.0.1
- New feature: 1.0.0 → 1.1.0
- Breaking change: 1.0.0 → 2.0.0

### 4. Document Dependencies

```toml
[mod.dependencies]
base = ">=0.3.0"  # Required for new item tags
```

---

## See Also

- [Getting Started](Getting-Started) - Quick tutorial
- [Data Mod Guide](Data-Mod-Guide) - TOML deep dive
- [TOML Schema](TOML-Schema) - All fields
