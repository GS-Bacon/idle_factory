[日本語版はこちら](ホーム)

# Idle Factory Modding Wiki

Welcome to the official Modding documentation.

---

## What is Modding?

Idle Factory supports three types of mods that extend the game in different ways.

| Type | Difficulty | What You Can Do |
|------|------------|-----------------|
| **Data Mod** | Easy | Add items, machines, recipes |
| **Script Mod** | Medium | Build external tools, overlays |
| **Core Mod** | Advanced | Change game behavior |

---

## Quick Start

### Step 1: Choose Your Mod Type

**Want to add new content?**
→ Start with [Data Mod Guide](Data-Mod-Guide)

**Want to build external tools?**
→ See [Script Mod Guide](Script-Mod-Guide)

**Want to change game logic?**
→ Read [Core Mod Guide](Core-Mod-Guide)

---

## Learning Path

### Beginner

1. **[Getting Started](Getting-Started)** - Create your first mod in 5 minutes
2. **[Mod Structure](Mod-Structure)** - Understand mod.toml and folder layout

### Intermediate

3. **[Data Mod Guide](Data-Mod-Guide)** - Complete guide to items, machines, recipes
4. **[TOML Schema](TOML-Schema)** - All configuration fields reference

### Advanced

5. **[Script Mod Guide](Script-Mod-Guide)** - WebSocket API for external tools
6. **[Core Mod Guide](Core-Mod-Guide)** - WASM mods for game logic changes

---

## Example: Your First Data Mod

Create a new item in just 2 files:

**mods/my_mod/mod.toml**
```toml
[mod]
id = "my_mod"
name = "My Mod"
version = "1.0.0"

[mod.dependencies]
base = ">=0.3.0"
```

**mods/my_mod/items.toml**
```toml
[[item]]
id = "diamond"
name = "Diamond"
description = "A rare gem"
stack_size = 999
category = "ore"
```

Launch the game, and your diamond item appears automatically!

---

## File Structure Overview

```
mods/
└── my_mod/
    ├── mod.toml        # Required: Mod metadata
    ├── items.toml      # Optional: Item definitions
    ├── machines.toml   # Optional: Machine definitions
    ├── recipes.toml    # Optional: Recipe definitions
    ├── textures/       # Optional: PNG texture files
    └── models/         # Optional: GLB/GLTF 3D models
```

---

## API Reference

| Document | Description |
|----------|-------------|
| [TOML Schema](TOML-Schema) | All item, machine, recipe fields |
| [WebSocket API](WebSocket-API) | JSON-RPC 2.0 methods for Script Mods |
| [WASM Host Functions](WASM-Host-Functions) | Host functions for Core Mods |

---

## Community

- **[GitHub Issues](https://github.com/GS-Bacon/idle_factory/issues)** - Bug reports, feature requests
- **[Discussions](https://github.com/GS-Bacon/idle_factory/discussions)** - Questions, mod showcase
