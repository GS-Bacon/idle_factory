# Mod Structure / Mod構造

Complete reference for mod.toml and folder organization.
mod.tomlとフォルダ構成の完全リファレンス。

---

## Folder Structure / フォルダ構成

```
mods/
└── your_mod_id/
    ├── mod.toml          # Required: Metadata / 必須: メタデータ
    ├── items.toml        # Item definitions / アイテム定義
    ├── machines.toml     # Machine definitions / 機械定義
    ├── recipes.toml      # Recipe definitions / レシピ定義
    ├── textures/         # PNG textures / PNGテクスチャ
    │   ├── my_item.png
    │   └── my_block_top.png
    ├── models/           # 3D models (GLB/GLTF) / 3Dモデル
    │   └── my_machine.glb
    └── scripts/          # Core Mod WASM / WASMスクリプト
        └── main.wasm
```

---

## mod.toml Reference / mod.tomlリファレンス

### Minimal Example / 最小例

```toml
[mod]
id = "my_mod"
name = "My Mod"
version = "1.0.0"
```

### Full Example / 完全例

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
base = ">=0.3.0"           # Base game version / ベースゲームバージョン
another_mod = ">=1.0.0"    # Other mod dependency / 他Mod依存

[mod.optional_dependencies]
optional_mod = ">=1.0.0"   # Loads if available / あれば読み込む

[mod.incompatible]
broken_mod = "*"           # Cannot coexist / 共存不可
```

---

## Field Reference / フィールドリファレンス

### [mod] Section

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier. Lowercase, underscores allowed. Used as namespace prefix. / 一意のID。小文字、アンダースコア可。名前空間のプレフィックスになる |
| `name` | string | Yes | Human-readable name shown in UI / UIに表示される名前 |
| `version` | string | Yes | Semantic versioning (MAJOR.MINOR.PATCH) / セマンティックバージョン |
| `author` | string | No | Creator name / 作者名 |
| `description` | string | No | Short description / 短い説明 |
| `license` | string | No | License (MIT, GPL, etc.) / ライセンス |
| `homepage` | string | No | URL to mod page / ModページのURL |
| `type` | string | No | `"data"` (default) or `"core"` (WASM) / デフォルトはdata |

### [mod.dependencies]

Version constraints: / バージョン制約:

| Syntax | Meaning |
|--------|---------|
| `">=1.0.0"` | 1.0.0 or higher / 1.0.0以上 |
| `"^1.2.0"` | Compatible with 1.2.0 (>=1.2.0, <2.0.0) / 1.2.0互換 |
| `"~1.2.0"` | Patch updates only (>=1.2.0, <1.3.0) / パッチ更新のみ |
| `"=1.2.3"` | Exact version / 完全一致 |
| `"*"` | Any version / 任意バージョン |

---

## ID Naming Rules / ID命名規則

### Mod ID

```
✓ my_mod
✓ advanced_machines
✓ bob_ores
✗ MyMod          (no uppercase / 大文字不可)
✗ my-mod         (no hyphens / ハイフン不可)
✗ 123mod         (cannot start with number / 数字始まり不可)
```

### Item/Machine/Recipe ID

Full ID format: `mod_id:item_id`
完全ID形式: `mod_id:item_id`

```toml
# In my_mod/items.toml
[[item]]
id = "super_ore"  # Full ID becomes: my_mod:super_ore

# Referencing in recipes
[recipe.inputs]
my_mod:super_ore = 1    # Explicit namespace / 明示的な名前空間
super_ore = 1           # Same mod, namespace optional / 同Mod内では省略可
base:iron_ore = 1       # Base game item / ベースゲームアイテム
```

---

## Load Order / 読み込み順序

1. `base` mod (always first / 常に最初)
2. Mods sorted by dependencies / 依存関係順にソート
3. Within same priority, alphabetical / 同優先度ならアルファベット順

### Debugging Load Order / 読み込み順デバッグ

Console command: / コンソールコマンド:
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

## Mod Types / Modタイプ

### Data Mod (Default)

- TOML files only / TOMLファイルのみ
- Add items, machines, recipes / アイテム・機械・レシピ追加
- No code execution / コード実行なし
- Safe, sandboxed / 安全、サンドボックス

```toml
[mod]
type = "data"  # Optional, this is the default / 省略可、デフォルト
```

### Core Mod (WASM)

- Rust compiled to WASM / RustをWASMにコンパイル
- Custom game logic / カスタムゲームロジック
- Access to host functions / ホスト関数アクセス可
- Requires explicit permission / 明示的許可が必要

```toml
[mod]
type = "core"
```

See [Core Mod Guide](Core-Mod-Guide) for WASM development.
WASM開発は[Core Modガイド](Core-Mod-Guide)を参照。

---

## Asset Paths / アセットパス

### Textures / テクスチャ

```
mods/my_mod/textures/
├── my_item.png           → my_mod:my_item
├── my_block.png          → my_mod:my_block (all sides / 全面)
├── my_block_top.png      → my_mod:my_block (top / 上面)
├── my_block_side.png     → my_mod:my_block (sides / 側面)
└── my_block_bottom.png   → my_mod:my_block (bottom / 底面)
```

**Requirements / 要件:**
- Format: PNG (RGBA) / 形式: PNG (RGBA)
- Size: 16x16 or 32x32 pixels / サイズ: 16x16 または 32x32
- Power of 2 recommended / 2の累乗推奨

### Models / モデル

```
mods/my_mod/models/
└── my_machine.glb        → Referenced in machines.toml
```

**Requirements / 要件:**
- Format: GLB or GLTF / 形式: GLB または GLTF
- Origin: Center bottom / 原点: 中央底面
- Scale: 1 unit = 1 block / スケール: 1単位 = 1ブロック

---

## Best Practices / ベストプラクティス

### 1. Use Descriptive IDs / 説明的なIDを使う

```toml
# Good
id = "titanium_drill"
id = "advanced_furnace"

# Bad
id = "item1"
id = "machine_new"
```

### 2. Namespace Your Tags / タグに名前空間を付ける

```toml
tags = ["mymod/special", "ore", "rare"]
```

### 3. Version Properly / 適切にバージョニング

- Bug fix: 1.0.0 → 1.0.1 / バグ修正
- New feature: 1.0.0 → 1.1.0 / 新機能
- Breaking change: 1.0.0 → 2.0.0 / 破壊的変更

### 4. Document Dependencies / 依存関係を文書化

```toml
[mod.dependencies]
base = ">=0.3.0"  # Required for new item tags / 新アイテムタグに必要
```

---

## See Also / 関連

- [Getting Started](Getting-Started) - Quick tutorial / クイックチュートリアル
- [Data Mod Guide](Data-Mod-Guide) - TOML deep dive / TOML詳細
- [TOML Schema](TOML-Schema) - All fields / 全フィールド
