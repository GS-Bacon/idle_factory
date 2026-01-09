[English version](Mod-Structure)

# Mod構造

mod.tomlとフォルダ構成の完全リファレンス。

---

## フォルダ構成

```
mods/
└── your_mod_id/
    ├── mod.toml          # 必須: メタデータ
    ├── items.toml        # アイテム定義
    ├── machines.toml     # 機械定義
    ├── recipes.toml      # レシピ定義
    ├── textures/         # PNGテクスチャ
    │   ├── my_item.png
    │   └── my_block_top.png
    ├── models/           # 3Dモデル (GLB/GLTF)
    │   └── my_machine.glb
    └── scripts/          # WASMスクリプト
        └── main.wasm
```

---

## mod.tomlリファレンス

### 最小例

```toml
[mod]
id = "my_mod"
name = "My Mod"
version = "1.0.0"
```

### 完全例

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
base = ">=0.3.0"           # ベースゲームバージョン
another_mod = ">=1.0.0"    # 他Mod依存

[mod.optional_dependencies]
optional_mod = ">=1.0.0"   # あれば読み込む

[mod.incompatible]
broken_mod = "*"           # 共存不可
```

---

## フィールドリファレンス

### [mod] セクション

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | string | はい | 一意のID。小文字、アンダースコア可。名前空間のプレフィックスになる |
| `name` | string | はい | UIに表示される名前 |
| `version` | string | はい | セマンティックバージョン (MAJOR.MINOR.PATCH) |
| `author` | string | いいえ | 作者名 |
| `description` | string | いいえ | 短い説明 |
| `license` | string | いいえ | ライセンス (MIT, GPL等) |
| `homepage` | string | いいえ | ModページのURL |
| `type` | string | いいえ | `"data"` (デフォルト) or `"core"` (WASM) |

### [mod.dependencies]

バージョン制約:

| 構文 | 意味 |
|------|------|
| `">=1.0.0"` | 1.0.0以上 |
| `"^1.2.0"` | 1.2.0互換 (>=1.2.0, <2.0.0) |
| `"~1.2.0"` | パッチ更新のみ (>=1.2.0, <1.3.0) |
| `"=1.2.3"` | 完全一致 |
| `"*"` | 任意バージョン |

---

## ID命名規則

### Mod ID

```
○ my_mod
○ advanced_machines
○ bob_ores
× MyMod          (大文字不可)
× my-mod         (ハイフン不可)
× 123mod         (数字始まり不可)
```

### Item/Machine/Recipe ID

完全ID形式: `mod_id:item_id`

```toml
# my_mod/items.toml内
[[item]]
id = "super_ore"  # 完全IDは: my_mod:super_ore

# レシピでの参照
[recipe.inputs]
my_mod:super_ore = 1    # 明示的な名前空間
super_ore = 1           # 同Mod内では省略可
base:iron_ore = 1       # ベースゲームアイテム
```

---

## 読み込み順序

1. `base` mod (常に最初)
2. 依存関係順にソート
3. 同優先度ならアルファベット順

### 読み込み順デバッグ

コンソールコマンド:
```
/mod list
```

出力:
```
[1] base v0.3.78 (enabled)
[2] my_first_mod v1.0.0 (enabled)
[3] advanced_machines v2.1.0 (enabled)
```

---

## Modタイプ

### Data Mod (デフォルト)

- TOMLファイルのみ
- アイテム・機械・レシピ追加
- コード実行なし
- 安全、サンドボックス

```toml
[mod]
type = "data"  # 省略可、デフォルト
```

### Core Mod (WASM)

- RustをWASMにコンパイル
- カスタムゲームロジック
- ホスト関数アクセス可
- 明示的許可が必要

```toml
[mod]
type = "core"
```

WASM開発は[Core Modガイド](Core-Mod-Guide)を参照。

---

## アセットパス

### テクスチャ

```
mods/my_mod/textures/
├── my_item.png           → my_mod:my_item
├── my_block.png          → my_mod:my_block (全面)
├── my_block_top.png      → my_mod:my_block (上面)
├── my_block_side.png     → my_mod:my_block (側面)
└── my_block_bottom.png   → my_mod:my_block (底面)
```

**要件:**
- 形式: PNG (RGBA)
- サイズ: 16x16 または 32x32
- 2の累乗推奨

### モデル

```
mods/my_mod/models/
└── my_machine.glb        → machines.tomlで参照
```

**要件:**
- 形式: GLB または GLTF
- 原点: 中央底面
- スケール: 1単位 = 1ブロック

---

## ベストプラクティス

### 1. 説明的なIDを使う

```toml
# 良い例
id = "titanium_drill"
id = "advanced_furnace"

# 悪い例
id = "item1"
id = "machine_new"
```

### 2. タグに名前空間を付ける

```toml
tags = ["mymod/special", "ore", "rare"]
```

### 3. 適切にバージョニング

- バグ修正: 1.0.0 → 1.0.1
- 新機能: 1.0.0 → 1.1.0
- 破壊的変更: 1.0.0 → 2.0.0

### 4. 依存関係を文書化

```toml
[mod.dependencies]
base = ">=0.3.0"  # 新アイテムタグに必要
```

---

## 関連ドキュメント

- [はじめに](はじめに) - クイックチュートリアル
- [Data Modガイド](データMod作成) - TOML詳細
- [TOML Schema](TOML-Schema) - 全フィールド
