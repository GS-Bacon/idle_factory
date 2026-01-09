# Data Mod Guide

Complete guide to adding items, machines, and recipes via TOML.

TOMLでアイテム・機械・レシピを追加する完全ガイド。

---

## Overview

Data Mods use TOML files to extend the game without writing any code.

Data ModはTOMLファイルでコードを書かずにゲームを拡張できます。

| File | Purpose |
|------|---------|
| `items.toml` | Define new items |
| `machines.toml` | Define new machines |
| `recipes.toml` | Define crafting recipes |

| ファイル | 用途 |
|---------|------|
| `items.toml` | 新アイテムの定義 |
| `machines.toml` | 新機械の定義 |
| `recipes.toml` | クラフトレシピの定義 |

---

## Items

### Basic Item

A simple non-placeable item.

シンプルな設置不可アイテム。

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

ワールドに設置できるアイテム。

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

タグでフィルタリングや加工ヒントを設定。

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

| フィールド | 型 | 必須 | デフォルト | 説明 |
|-----------|-----|------|-----------|------|
| `id` | string | はい | - | Mod内で一意のID |
| `name` | string | はい | - | 表示名 |
| `short_name` | string | いいえ | 最初の4文字 | 短縮表示（最大4文字） |
| `description` | string | いいえ | "" | ツールチップ |
| `stack_size` | int | いいえ | 999 | 最大スタック数 |
| `category` | string | はい | - | 下記カテゴリ参照 |
| `is_placeable` | bool | いいえ | false | 設置可能か |
| `hardness` | float | いいえ | 1.0 | 採掘時間係数 |
| `color` | [f32;3] | いいえ | [1,1,1] | RGB色 [0-1] |
| `tags` | [string] | いいえ | [] | 検索用タグ |

### Categories

| Category | Description | Example |
|----------|-------------|---------|
| `terrain` | Natural blocks | stone, grass, dirt |
| `ore` | Mineable resources | iron_ore, coal |
| `processed` | Crafted materials | iron_ingot, copper_wire |
| `machine` | Placeable machines | furnace_block, miner_block |
| `tool` | Player tools | pickaxe |

| カテゴリ | 説明 | 例 |
|---------|------|-----|
| `terrain` | 自然ブロック | stone, grass, dirt |
| `ore` | 採掘資源 | iron_ore, coal |
| `processed` | 加工素材 | iron_ingot, copper_wire |
| `machine` | 設置機械 | furnace_block, miner_block |
| `tool` | プレイヤーツール | pickaxe |

### Tag Conventions

```toml
# Hierarchy with /
# 階層は/で表現
tags = ["ore", "ore/iron"]           # Iron ore
tags = ["ingot", "ingot/iron"]       # Iron ingot

# Processing hints
# 加工ヒント
tags = ["smeltable"]                 # Can be smelted / 精錬可能
tags = ["crushable"]                 # Can be crushed / 粉砕可能
tags = ["fuel"]                      # Can be used as fuel / 燃料使用可能

# Custom mod tags
# カスタムModタグ
tags = ["mymod/special", "mymod/tier2"]
```

---

## Machines

### Basic Machine

A processing machine that uses recipes.

レシピを使用する加工機械。

```toml
[[machine]]
id = "electric_furnace"
name = "電気炉"
name_en = "Electric Furnace"
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
input = { slot_id = 0, label = "入力" }
output = { slot_id = 0, label = "出力" }
```

### Machine with Fuel

A machine that requires fuel to operate.

燃料が必要な機械。

```toml
[[machine]]
id = "blast_furnace"
name = "高炉"
name_en = "Blast Furnace"
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
input = { slot_id = 0, label = "素材" }
fuel = { slot_id = 1, label = "燃料" }
output = { slot_id = 0, label = "出力" }
```

### Auto-Generate Machine

A machine that produces output without input.

入力なしで出力を生産する機械。

```toml
[[machine]]
id = "water_pump"
name = "ポンプ"
name_en = "Water Pump"
block_type = "custom"
process_time = 2.0
buffer_size = 100
requires_fuel = false
auto_generate = true
process_type = "auto_generate"

[machine.ports]
outputs = [{ side = "front", slot_id = 0 }]

[machine.ui_slots]
output = { slot_id = 0, label = "出力" }
```

### All Machine Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique ID |
| `name` | string | Yes | Display name (Japanese) |
| `name_en` | string | No | English name |
| `block_type` | string | Yes | Associated block type |
| `process_time` | float | Yes | Seconds per operation |
| `buffer_size` | int | No | Internal buffer capacity |
| `requires_fuel` | bool | No | Needs fuel to operate |
| `auto_generate` | bool | No | Produces without input |
| `process_type` | string | Yes | `"recipe"` or `"auto_generate"` |
| `machine_type` | string | No | Recipe filter |

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | string | はい | 一意のID |
| `name` | string | はい | 表示名（日本語） |
| `name_en` | string | いいえ | 英語名 |
| `block_type` | string | はい | 関連ブロックタイプ |
| `process_time` | float | はい | 1操作の秒数 |
| `buffer_size` | int | いいえ | 内部バッファ容量 |
| `requires_fuel` | bool | いいえ | 燃料が必要か |
| `auto_generate` | bool | いいえ | 入力なしで生産 |
| `process_type` | string | はい | `"recipe"` or `"auto_generate"` |
| `machine_type` | string | いいえ | レシピフィルタ |

### Port Sides

| Side | Description |
|------|-------------|
| `front` | Output direction (machine facing) |
| `back` | Behind machine |
| `left` | Left side |
| `right` | Right side |
| `top` | Above |
| `bottom` | Below |

| 方向 | 説明 |
|------|------|
| `front` | 出力方向（機械の向き） |
| `back` | 機械の後ろ |
| `left` | 左側 |
| `right` | 右側 |
| `top` | 上 |
| `bottom` | 下 |

---

## Recipes

### Basic Recipe

A simple single-input, single-output recipe.

シンプルな単一入力・単一出力レシピ。

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

複数の異なる入力が必要なレシピ。

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

複数出力（副産物）を生産するレシピ。

```toml
[[recipe]]
id = "crush_iron"
machine = "crusher"
craft_time = 1.5

[recipe.inputs]
iron_ore = 1

[recipe.outputs]
iron_dust = 2
stone = 1        # Byproduct / 副産物
```

### Cross-Mod Recipe

A recipe using items from other mods.

他のModのアイテムを使用するレシピ。

```toml
[[recipe]]
id = "craft_advanced_circuit"
machine = "assembler"
craft_time = 10.0

[recipe.inputs]
base:iron_ingot = 2           # From base game / ベースゲームから
other_mod:special_dust = 1    # From another mod / 他Modから
my_item = 1                   # From this mod / このModから

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

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | string | はい | 一意のID |
| `machine` | string | はい | 処理する機械タイプ |
| `craft_time` | float | はい | 完了までの秒数 |
| `inputs` | table | はい | 入力アイテム {id = count} |
| `outputs` | table | はい | 出力アイテム {id = count} |
| `fuel` | table | いいえ | 燃料アイテム {id = count} |

---

## Complete Example: New Ore Chain

A full example adding titanium ore processing.

チタン鉱石の加工を追加する完全な例。

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

ゲームは読み込み時にTOMLを検証します。エラーは `logs/game.log` を確認。

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `duplicate id` | Same ID used twice | Use unique IDs |
| `unknown category` | Invalid category | Use valid category |
| `missing field` | Required field absent | Add the field |
| `item not found` | Recipe references missing item | Check item ID |

| エラー | 原因 | 修正方法 |
|--------|------|---------|
| `duplicate id` | 同じIDを2回使用 | 一意のIDを使用 |
| `unknown category` | 無効なカテゴリ | 有効なカテゴリを使用 |
| `missing field` | 必須フィールドがない | フィールドを追加 |
| `item not found` | レシピが存在しないアイテムを参照 | アイテムIDを確認 |

---

## See Also

- **[Getting Started](Getting-Started)** - Quick tutorial

  クイックチュートリアル

- **[Mod Structure](Mod-Structure)** - mod.toml reference

  mod.toml詳細

- **[TOML Schema](TOML-Schema)** - Complete field reference

  完全フィールドリファレンス
