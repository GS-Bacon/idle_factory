# Getting Started

Create your first mod in 5 minutes.

5分で最初のModを作成しましょう。

---

## Prerequisites / 必要なもの

- Idle Factory game installed
- Text editor (VS Code, Notepad++, etc.)

必要なもの：
- Idle Factoryゲーム本体
- テキストエディタ（VS Code、メモ帳など）

---

## Tutorial: Add a Diamond Item

### Step 1: Create Mod Folder

Create folder `mods/my_first_mod/` in the game directory.

ゲームディレクトリに `mods/my_first_mod/` フォルダを作成します。

```
mods/
└── my_first_mod/
    ├── mod.toml      ← Mod metadata
    └── items.toml    ← Item definitions
```

### Step 2: Create mod.toml

Create the mod metadata file.

Modメタデータファイルを作成します。

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

| フィールド | 必須 | 説明 |
|-----------|------|------|
| `id` | はい | 一意の識別子（小文字、アンダースコアOK） |
| `name` | はい | 表示名 |
| `version` | はい | セマンティックバージョン（例: 1.0.0） |
| `dependencies` | いいえ | 依存Mod |

### Step 3: Create items.toml

Define your new items.

新しいアイテムを定義します。

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

手順：
1. Idle Factoryを起動
2. Modが自動で読み込まれる
3. Eキーでインベントリを開く

### Step 5: Verify Your Mod

Open console (T key) and type:

コンソール（Tキー）を開いて入力：

```
/item list my_first_mod
```

Expected output:

期待される出力：

```
my_first_mod:diamond_ore - Diamond Ore
my_first_mod:diamond - Diamond
```

---

## Next Steps: Add a Recipe

Want your diamond ore to be useful? Add a recipe!

ダイヤモンド鉱石を活用したい？レシピを追加しましょう！

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

これでダイヤモンド鉱石を粉砕してダイヤモンドを入手できます！

---

## Next Steps: Add a Machine

Create a custom machine for your mod.

Mod専用の機械を作成できます。

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

Modが読み込まれない場合：
- `mod.toml` の構文を確認
- `id` が一意の小文字であることを確認

### Items not appearing

- Verify `category` is valid: `terrain`, `ore`, `processed`, `machine`, `tool`

アイテムが表示されない場合：
- `category` が有効値か確認

### Recipe not working

- Ensure input/output items exist
- Check machine type matches

レシピが動作しない場合：
- 入出力アイテムが存在するか確認
- 機械タイプが一致しているか確認

### Check Logs

Look at `logs/game.log` for error details.

エラーの詳細は `logs/game.log` を確認してください。

---

## Complete File Structure

```
mods/my_first_mod/
├── mod.toml        # Required
│                   # 必須
│
├── items.toml      # Optional
│                   # 任意
│
├── machines.toml   # Optional
│                   # 任意
│
├── recipes.toml    # Optional
│                   # 任意
│
├── textures/       # Optional: PNG files
│   └── diamond.png # 任意: PNGファイル
│
└── models/         # Optional: GLB files
    └── gem_polisher.glb
                    # 任意: GLBファイル
```

---

## See Also / 関連ドキュメント

- **[Mod Structure](Mod-Structure)** - Detailed mod.toml reference

  mod.tomlの詳細リファレンス

- **[Data Mod Guide](Data-Mod-Guide)** - Complete TOML documentation

  TOMLの完全ガイド

- **[TOML Schema](TOML-Schema)** - All fields reference

  全フィールドのリファレンス
