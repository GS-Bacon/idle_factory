# Getting Started / はじめに

Create your first mod in 5 minutes.
5分で最初のModを作成しましょう。

---

## Prerequisites / 必要なもの

- Idle Factory game installed / Idle Factoryゲーム本体
- Text editor (VS Code, Notepad++, etc.) / テキストエディタ

---

## Tutorial: Add a Diamond Item / チュートリアル: ダイヤモンドを追加

### Step 1: Create Mod Folder / Modフォルダを作成

```
mods/
└── my_first_mod/
    ├── mod.toml      ← Mod metadata / メタデータ
    └── items.toml    ← Item definitions / アイテム定義
```

Create folder `mods/my_first_mod/` in the game directory.
ゲームディレクトリに `mods/my_first_mod/` フォルダを作成。

### Step 2: Create mod.toml / mod.tomlを作成

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
| `id` | Yes | Unique identifier (lowercase, underscores OK) / 一意の識別子 |
| `name` | Yes | Display name / 表示名 |
| `version` | Yes | Semantic versioning / セマンティックバージョン |
| `dependencies` | No | Required mods / 依存Mod |

### Step 3: Create items.toml / items.tomlを作成

```toml
# Diamond Ore / ダイヤモンド鉱石
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

# Diamond / ダイヤモンド
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

### Step 4: Launch / ゲーム起動

1. Start Idle Factory / Idle Factoryを起動
2. Mod loads automatically / Modが自動読み込み
3. Press E to open inventory / Eキーでインベントリを開く

### Step 5: Verify / 確認

Open console (T key) and type: / コンソール（Tキー）で入力:
```
/item list my_first_mod
```

Expected output / 期待される出力:
```
my_first_mod:diamond_ore - Diamond Ore
my_first_mod:diamond - Diamond
```

---

## Next: Add Recipe / 次: レシピ追加

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

---

## Next: Add Machine / 次: 機械追加

Create `machines.toml`:

```toml
[[machine]]
id = "gem_polisher"
name = "Gem Polisher"
name_en = "Gem Polisher"
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

## Troubleshooting / トラブルシューティング

| Problem | Solution |
|---------|----------|
| Mod not loading / Modが読み込まれない | Check `mod.toml` syntax, ensure `id` is unique lowercase / 構文確認、idを一意の小文字に |
| Items missing / アイテムがない | Verify `category` is valid: `terrain`, `ore`, `processed`, `machine`, `tool` |
| Recipe not working / レシピが動かない | Ensure input/output items exist, machine type matches / 入出力アイテムと機械タイプを確認 |

Check logs: `logs/game.log` / ログ確認: `logs/game.log`

---

## File Structure / ファイル構造

```
mods/my_first_mod/
├── mod.toml        # Required / 必須
├── items.toml      # Optional / 任意
├── machines.toml   # Optional / 任意
├── recipes.toml    # Optional / 任意
├── textures/       # Optional / 任意
│   └── diamond.png
└── models/         # Optional / 任意
    └── gem_polisher.glb
```

---

## See Also / 関連

- [Mod Structure](Mod-Structure) - mod.toml reference / mod.toml詳細
- [Data Mod Guide](Data-Mod-Guide) - Complete TOML docs / TOML完全ガイド
- [TOML Schema](TOML-Schema) - All fields / 全フィールド
