# Data Mod Guide / Data Modガイド

Complete guide to adding items, machines, and recipes via TOML.
TOMLでアイテム・機械・レシピを追加する完全ガイド。

---

## Overview / 概要

Data Mods use TOML files to extend the game without code.
Data ModはTOMLファイルでコードなしにゲームを拡張。

| File | Purpose |
|------|---------|
| `items.toml` | Define new items / 新アイテム定義 |
| `machines.toml` | Define new machines / 新機械定義 |
| `recipes.toml` | Define crafting recipes / クラフトレシピ定義 |

---

## Items / アイテム

### Basic Item / 基本アイテム

```toml
[[item]]
id = "copper_wire"
name = "Copper Wire"
description = "Thin copper wire for electronics"
stack_size = 999
category = "processed"
```

### Placeable Block / 設置可能ブロック

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

### Item with Tags / タグ付きアイテム

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

### All Item Fields / アイテム全フィールド

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | string | Yes | - | Unique ID within mod / Mod内で一意のID |
| `name` | string | Yes | - | Display name / 表示名 |
| `short_name` | string | No | First 4 chars | Short display (max 4) / 短縮表示 |
| `description` | string | No | "" | Tooltip text / ツールチップ |
| `stack_size` | int | No | 999 | Max stack amount / 最大スタック数 |
| `category` | string | Yes | - | See categories below / 下記カテゴリ参照 |
| `is_placeable` | bool | No | false | Can be placed in world / ワールドに設置可能か |
| `hardness` | float | No | 1.0 | Mining time multiplier / 採掘時間係数 |
| `color` | [f32;3] | No | [1,1,1] | RGB color [0-1] / RGB色 |
| `tags` | [string] | No | [] | Searchable tags / 検索用タグ |

### Categories / カテゴリ

| Category | Description | Example |
|----------|-------------|---------|
| `terrain` | Natural blocks / 自然ブロック | stone, grass, dirt |
| `ore` | Mineable resources / 採掘資源 | iron_ore, coal |
| `processed` | Crafted materials / 加工素材 | iron_ingot, copper_wire |
| `machine` | Placeable machines / 設置機械 | furnace_block, miner_block |
| `tool` | Player tools / プレイヤーツール | pickaxe |

### Tag Conventions / タグ規約

```toml
# Hierarchy with / / 階層は/で
tags = ["ore", "ore/iron"]           # Iron ore
tags = ["ingot", "ingot/iron"]       # Iron ingot

# Processing hints / 加工ヒント
tags = ["smeltable"]                 # Can be smelted / 精錬可能
tags = ["crushable"]                 # Can be crushed / 粉砕可能
tags = ["fuel"]                      # Can be used as fuel / 燃料として使用可能

# Custom mod tags / カスタムModタグ
tags = ["mymod/special", "mymod/tier2"]
```

---

## Machines / 機械

### Basic Machine / 基本機械

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

### Machine with Fuel / 燃料が必要な機械

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

### Auto-Generate Machine / 自動生成機械

```toml
[[machine]]
id = "water_pump"
name = "ポンプ"
name_en = "Water Pump"
block_type = "custom"
process_time = 2.0
buffer_size = 100
requires_fuel = false
auto_generate = true          # Generates without input / 入力なしで生成
process_type = "auto_generate"

[machine.ports]
outputs = [{ side = "front", slot_id = 0 }]

[machine.ui_slots]
output = { slot_id = 0, label = "出力" }
```

### All Machine Fields / 機械全フィールド

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique ID / 一意のID |
| `name` | string | Yes | Display name (Japanese) / 表示名（日本語） |
| `name_en` | string | No | English name / 英語名 |
| `block_type` | string | Yes | Associated block type / 関連ブロックタイプ |
| `process_time` | float | Yes | Seconds per operation / 1操作の秒数 |
| `buffer_size` | int | No | Internal buffer capacity / 内部バッファ容量 |
| `requires_fuel` | bool | No | Needs fuel to operate / 燃料が必要か |
| `auto_generate` | bool | No | Produces without input / 入力なしで生産 |
| `process_type` | string | Yes | `"recipe"` or `"auto_generate"` |
| `machine_type` | string | No | Recipe filter / レシピフィルタ |

### Port Sides / ポート方向

| Side | Description |
|------|-------------|
| `front` | Output direction (machine facing) / 出力方向（機械の向き） |
| `back` | Behind machine / 機械の後ろ |
| `left` | Left side / 左側 |
| `right` | Right side / 右側 |
| `top` | Above / 上 |
| `bottom` | Below / 下 |

---

## Recipes / レシピ

### Basic Recipe / 基本レシピ

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

### Multi-Input Recipe / 複数入力レシピ

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

### Multi-Output Recipe / 複数出力レシピ

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

### Cross-Mod Recipe / Mod間レシピ

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

### All Recipe Fields / レシピ全フィールド

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique ID / 一意のID |
| `machine` | string | Yes | Machine type that processes this / 処理する機械タイプ |
| `craft_time` | float | Yes | Seconds to complete / 完了までの秒数 |
| `inputs` | table | Yes | Input items {id = count} / 入力アイテム |
| `outputs` | table | Yes | Output items {id = count} / 出力アイテム |
| `fuel` | table | No | Fuel items {id = count} / 燃料アイテム |

---

## Examples / 例

### Complete Mod: New Ore Chain / 完全なMod: 新しい鉱石チェーン

```toml
# items.toml

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

```toml
# recipes.toml

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

## Validation / バリデーション

The game validates TOML on load. Check logs for errors:
ゲームは読み込み時にTOMLを検証。エラーはログを確認:

```
logs/game.log
```

Common errors / よくあるエラー:

| Error | Cause | Fix |
|-------|-------|-----|
| `duplicate id` | Same ID used twice / 同じIDを2回使用 | Use unique IDs / 一意のIDを使用 |
| `unknown category` | Invalid category / 無効なカテゴリ | Use valid category / 有効なカテゴリを使用 |
| `missing field` | Required field absent / 必須フィールドがない | Add the field / フィールドを追加 |
| `item not found` | Recipe references missing item / レシピが存在しないアイテムを参照 | Check item ID / アイテムIDを確認 |

---

## See Also / 関連

- [Getting Started](Getting-Started) - Quick tutorial / クイックチュートリアル
- [Mod Structure](Mod-Structure) - mod.toml reference / mod.toml詳細
- [TOML Schema](TOML-Schema) - Complete field reference / 完全フィールドリファレンス
