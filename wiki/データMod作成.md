[English version](Data-Mod-Guide)

# Data Modガイド

TOMLでアイテム・機械・レシピを追加する完全ガイド。

---

## 概要

Data ModはTOMLファイルでコードを書かずにゲームを拡張できます。

| ファイル | 用途 |
|---------|------|
| `items.toml` | 新アイテムの定義 |
| `machines.toml` | 新機械の定義 |
| `recipes.toml` | クラフトレシピの定義 |

---

## アイテム

### 基本アイテム

シンプルな設置不可アイテム。

```toml
[[item]]
id = "copper_wire"
name = "Copper Wire"
description = "Thin copper wire for electronics"
stack_size = 999
category = "processed"
```

### 設置可能ブロック

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

### タグ付きアイテム

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

### アイテムの全フィールド

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

### カテゴリ

| カテゴリ | 説明 | 例 |
|---------|------|-----|
| `terrain` | 自然ブロック | stone, grass, dirt |
| `ore` | 採掘資源 | iron_ore, coal |
| `processed` | 加工素材 | iron_ingot, copper_wire |
| `machine` | 設置機械 | furnace_block, miner_block |
| `tool` | プレイヤーツール | pickaxe |

### タグの規則

```toml
# 階層は/で表現
tags = ["ore", "ore/iron"]           # 鉄鉱石
tags = ["ingot", "ingot/iron"]       # 鉄インゴット

# 加工ヒント
tags = ["smeltable"]                 # 精錬可能
tags = ["crushable"]                 # 粉砕可能
tags = ["fuel"]                      # 燃料使用可能

# カスタムModタグ
tags = ["mymod/special", "mymod/tier2"]
```

---

## 機械

### 基本的な機械

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

### 燃料が必要な機械

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
    { side = "back", slot_id = 0 },   # 素材
    { side = "left", slot_id = 1 },   # 燃料
    { side = "right", slot_id = 1 }   # 燃料（代替）
]
outputs = [{ side = "front", slot_id = 0 }]

[machine.ui_slots]
input = { slot_id = 0, label = "素材" }
fuel = { slot_id = 1, label = "燃料" }
output = { slot_id = 0, label = "出力" }
```

### 自動生成機械

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

### 機械の全フィールド

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

### ポート方向

| 方向 | 説明 |
|------|------|
| `front` | 出力方向（機械の向き） |
| `back` | 機械の後ろ |
| `left` | 左側 |
| `right` | 右側 |
| `top` | 上 |
| `bottom` | 下 |

---

## レシピ

### 基本レシピ

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

### 複数入力レシピ

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

### 複数出力レシピ

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
stone = 1        # 副産物
```

### クロスModレシピ

他のModのアイテムを使用するレシピ。

```toml
[[recipe]]
id = "craft_advanced_circuit"
machine = "assembler"
craft_time = 10.0

[recipe.inputs]
base:iron_ingot = 2           # ベースゲームから
other_mod:special_dust = 1    # 他Modから
my_item = 1                   # このModから

[recipe.outputs]
advanced_circuit = 1
```

### レシピの全フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | string | はい | 一意のID |
| `machine` | string | はい | 処理する機械タイプ |
| `craft_time` | float | はい | 完了までの秒数 |
| `inputs` | table | はい | 入力アイテム {id = count} |
| `outputs` | table | はい | 出力アイテム {id = count} |
| `fuel` | table | いいえ | 燃料アイテム {id = count} |

---

## 完全な例: 新しい鉱石チェーン

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

## 検証

ゲームは読み込み時にTOMLを検証します。エラーは `logs/game.log` を確認。

### よくあるエラー

| エラー | 原因 | 修正方法 |
|--------|------|---------|
| `duplicate id` | 同じIDを2回使用 | 一意のIDを使用 |
| `unknown category` | 無効なカテゴリ | 有効なカテゴリを使用 |
| `missing field` | 必須フィールドがない | フィールドを追加 |
| `item not found` | レシピが存在しないアイテムを参照 | アイテムIDを確認 |

---

## 関連ドキュメント

- **[はじめに](はじめに)** - クイックチュートリアル

- **[Mod構造](Mod構造)** - mod.toml詳細

- **[TOML Schema](TOML-Schema)** - 完全フィールドリファレンス
