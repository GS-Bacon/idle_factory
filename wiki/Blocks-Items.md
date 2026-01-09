# Blocks & Items

Overview of the block and item system in Idle Factory.

Idle Factoryのブロックとアイテムシステムの概要。

---

## Basics

### What are Blocks?

Blocks are the fundamental building units of the world. Each block occupies a 1x1x1 space in the voxel grid.

ブロックはワールドの基本的な構成単位です。各ブロックはボクセルグリッド内の1x1x1の空間を占めます。

### What are Items?

Items are objects that can be stored in inventories, placed as blocks, or used as materials in recipes.

アイテムはインベントリに保存、ブロックとして設置、またはレシピの材料として使用できるオブジェクトです。

---

## Categories

| Category | Placeable | Description |
|----------|-----------|-------------|
| `terrain` | Yes | Natural blocks like stone, grass, dirt |
| `ore` | Yes | Mineable resources like iron_ore, coal |
| `processed` | No | Crafted materials like iron_ingot |
| `machine` | Yes | Functional blocks like furnace, miner |
| `tool` | No | Player equipment like pickaxe |

| カテゴリ | 設置可能 | 説明 |
|---------|---------|------|
| `terrain` | はい | 石、草、土などの自然ブロック |
| `ore` | はい | 鉄鉱石、石炭などの採掘資源 |
| `processed` | いいえ | 鉄インゴットなどの加工素材 |
| `machine` | はい | 精錬炉、採掘機などの機能ブロック |
| `tool` | いいえ | ピッケルなどのプレイヤー装備 |

---

## Block Properties

### Hardness

Determines how long it takes to mine a block.

ブロックの採掘にかかる時間を決定します。

| Hardness | Mining Time | Examples |
|----------|-------------|----------|
| 0.5 | Fast | Grass, leaves |
| 1.0 | Normal | Dirt, sand |
| 1.5 | Slow | Stone, ores |
| 2.0+ | Very slow | Reinforced blocks |

| 硬度 | 採掘時間 | 例 |
|------|---------|-----|
| 0.5 | 速い | 草、葉 |
| 1.0 | 普通 | 土、砂 |
| 1.5 | 遅い | 石、鉱石 |
| 2.0+ | とても遅い | 強化ブロック |

### Color

RGB color values from 0.0 to 1.0.

0.0から1.0のRGB色値。

```toml
color = [0.8, 0.4, 0.2]  # Orange / オレンジ
color = [0.2, 0.8, 0.9]  # Cyan / シアン
```

---

## Item Stacking

Items stack up to their `stack_size` value (default: 999).

アイテムは `stack_size` 値までスタックします（デフォルト: 999）。

---

## Tags

Tags are used for filtering and processing hints.

タグはフィルタリングと加工ヒントに使用されます。

### Common Tags

| Tag | Meaning |
|-----|---------|
| `smeltable` | Can be processed in furnace |
| `crushable` | Can be processed in crusher |
| `fuel` | Can be used as fuel |
| `ore/*` | Ore type (e.g., ore/iron) |
| `ingot/*` | Ingot type (e.g., ingot/iron) |

| タグ | 意味 |
|-----|------|
| `smeltable` | 精錬炉で加工可能 |
| `crushable` | 粉砕機で加工可能 |
| `fuel` | 燃料として使用可能 |
| `ore/*` | 鉱石タイプ（例: ore/iron） |
| `ingot/*` | インゴットタイプ（例: ingot/iron） |

---

## See Also

- **[Machines](Machines)** - Machine processing system

  機械加工システム

- **[Recipes](Recipes)** - Crafting recipes

  クラフトレシピ

- **[Data Mod Guide](Data-Mod-Guide)** - Adding custom items

  カスタムアイテムの追加
