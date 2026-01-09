# TOMLスキーマ

[English version](TOML-Schema)

全TOMLファイルの完全フィールドリファレンスです。

---

## mod.toml

```toml
[mod]
id = "string"              # 必須: 一意の小文字識別子
name = "string"            # 必須: 表示名
version = "string"         # 必須: セマンティックバージョン (X.Y.Z)
author = "string"          # 任意: 作成者名
description = "string"     # 任意: 短い説明
license = "string"         # 任意: ライセンス (MIT, GPL, etc.)
homepage = "string"        # 任意: URL
type = "data|core"         # 任意: デフォルト "data"

[mod.dependencies]
mod_id = "version_constraint"  # e.g., base = ">=0.3.0"

[mod.optional_dependencies]
mod_id = "version_constraint"

[mod.incompatible]
mod_id = "version_constraint"
```

### バージョン制約

| パターン | 意味 |
|---------|---------|
| `">=1.0.0"` | 1.0.0以上 |
| `"^1.2.0"` | >=1.2.0, <2.0.0 (互換) |
| `"~1.2.0"` | >=1.2.0, <1.3.0 (パッチのみ) |
| `"=1.2.3"` | 完全一致 |
| `"*"` | 任意のバージョン |

---

## items.toml

```toml
[[item]]
id = "string"              # 必須: mod内で一意
name = "string"            # 必須: 表示名
short_name = "string"      # 任意: 最大4文字、デフォルト=nameの最初4文字
description = "string"     # 任意: ツールチップテキスト
stack_size = 999           # 任意: 最大スタック、デフォルト=999
category = "string"        # 必須: terrain|ore|processed|machine|tool
is_placeable = false       # 任意: ワールドに設置可能
hardness = 1.0             # 任意: 採掘時間倍率
color = [1.0, 1.0, 1.0]    # 任意: RGB [0-1]
tags = ["string"]          # 任意: 検索可能なタグ
```

### カテゴリ値

| 値 | 用途 |
|-------|----------|
| `terrain` | 自然ブロック（石、草） |
| `ore` | 採掘資源（鉄鉱石、石炭） |
| `processed` | 加工アイテム（インゴット、粉） |
| `machine` | 設置機械 |
| `tool` | プレイヤー装備 |

### よく使うタグ

| タグ | 意味 |
|-----|---------|
| `ore`, `ore/iron` | 鉱石タイプ階層 |
| `ingot`, `ingot/iron` | インゴットタイプ階層 |
| `dust`, `dust/iron` | 粉タイプ階層 |
| `smeltable` | 精錬炉で精錬可能 |
| `crushable` | 粉砕機で粉砕可能 |
| `fuel` | 燃料として使用可能 |
| `rare` | レアアイテム |
| `metal` | 金属素材 |

---

## machines.toml

```toml
[[machine]]
id = "string"              # 必須: mod内で一意
name = "string"            # 必須: 表示名（日本語）
name_en = "string"         # 任意: 英語名
block_type = "string"      # 必須: 関連ブロックタイプ
process_time = 1.0         # 必須: 処理時間（秒）
buffer_size = 64           # 任意: 内部バッファ容量
requires_fuel = false      # 任意: 燃料が必要
auto_generate = false      # 任意: 入力なしで生成
process_type = "string"    # 必須: recipe|auto_generate
machine_type = "string"    # 任意: レシピフィルター

[machine.ports]
inputs = [
    { side = "back", slot_id = 0 },
    { side = "left", slot_id = 1 }
]
outputs = [
    { side = "front", slot_id = 0 }
]

[machine.ui_slots]
input = { slot_id = 0, label = "入力" }
fuel = { slot_id = 1, label = "燃料" }
output = { slot_id = 0, label = "出力" }
```

### ポート方向値

| 値 | 方向 |
|-------|-----------|
| `front` | 機械の向き |
| `back` | 正面の反対 |
| `left` | 正面を向いて左 |
| `right` | 正面を向いて右 |
| `top` | 機械の上 |
| `bottom` | 機械の下 |

### 処理タイプ値

| 値 | 動作 |
|-------|----------|
| `recipe` | レシピに従って処理 |
| `auto_generate` | 入力なしで出力を生成 |

---

## recipes.toml

```toml
[[recipe]]
id = "string"              # 必須: mod内で一意
machine = "string"         # 必須: 処理するmachine_type
craft_time = 1.0           # 必須: 完了までの秒数

[recipe.inputs]
item_id = 1                # item_id = count
another_item = 2           # 複数入力可

[recipe.outputs]
result_item = 1            # item_id = count
byproduct = 1              # 複数出力可

[recipe.fuel]              # 任意: requires_fuelを持つ機械のみ
coal = 1                   # fuel_item_id = count
```

### アイテムID解決

| 形式 | 解決 |
|--------|------------|
| `iron_ingot` | 同じmod: `my_mod:iron_ingot` |
| `base:iron_ingot` | 明示的: ベースゲームのアイテム |
| `other_mod:item` | 明示的: 他のmodのアイテム |

---

## 型リファレンス

| 型 | 形式 | 例 |
|------|--------|---------|
| `string` | クォートされたテキスト | `"hello"` |
| `int` | 整数 | `42` |
| `float` | 小数 | `1.5` |
| `bool` | ブール値 | `true`, `false` |
| `[string]` | 文字列配列 | `["a", "b"]` |
| `[f32;3]` | 浮動小数配列 (3) | `[1.0, 0.5, 0.2]` |
| `table` | キーバリュー | `{ key = value }` |

---

## バリデーションルール

### ID

- 小文字のみ
- アンダースコア可
- ハイフン不可
- 数字始まり不可
- スコープ内で一意

### 値

- `stack_size`: 1-9999
- `hardness`: 0.1-10.0
- `process_time`: 0.1-3600.0
- `buffer_size`: 1-1000
- `color`: 各要素 0.0-1.0

### 参照

- レシピの入出力は存在するアイテムを参照すること
- レシピのmachineは機械の`machine_type`と一致すること
- 依存関係はロード可能であること

---

## 関連

- [Data Mod Guide](Data-Mod-Guide) - 使用例
- [Mod Structure](Mod-Structure) - ファイル構成
