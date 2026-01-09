[日本語版はこちら](ブロックとアイテム)

# Blocks & Items

Overview of the block and item system in Idle Factory.

---

## Basics

### What are Blocks?

Blocks are the fundamental building units of the world. Each block occupies a 1x1x1 space in the voxel grid.

### What are Items?

Items are objects that can be stored in inventories, placed as blocks, or used as materials in recipes.

---

## Categories

| Category | Placeable | Description |
|----------|-----------|-------------|
| `terrain` | Yes | Natural blocks like stone, grass, dirt |
| `ore` | Yes | Mineable resources like iron_ore, coal |
| `processed` | No | Crafted materials like iron_ingot |
| `machine` | Yes | Functional blocks like furnace, miner |
| `tool` | No | Player equipment like pickaxe |

---

## Block Properties

### Hardness

Determines how long it takes to mine a block.

| Hardness | Mining Time | Examples |
|----------|-------------|----------|
| 0.5 | Fast | Grass, leaves |
| 1.0 | Normal | Dirt, sand |
| 1.5 | Slow | Stone, ores |
| 2.0+ | Very slow | Reinforced blocks |

### Color

RGB color values from 0.0 to 1.0.

```toml
color = [0.8, 0.4, 0.2]  # Orange
color = [0.2, 0.8, 0.9]  # Cyan
```

---

## Item Stacking

Items stack up to their `stack_size` value (default: 999).

---

## Tags

Tags are used for filtering and processing hints.

### Common Tags

| Tag | Meaning |
|-----|---------|
| `smeltable` | Can be processed in furnace |
| `crushable` | Can be processed in crusher |
| `fuel` | Can be used as fuel |
| `ore/*` | Ore type (e.g., ore/iron) |
| `ingot/*` | Ingot type (e.g., ingot/iron) |

---

## See Also

- **[Machines](Machines)** - Machine processing system
- **[Recipes](Recipes)** - Crafting recipes
- **[Data Mod Guide](Data-Mod-Guide)** - Adding custom items
