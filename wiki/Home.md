# Idle Factory Modding Wiki

Welcome to the official Modding documentation.

公式Moddingドキュメントへようこそ。

---

## What is Modding? / Moddingとは

Idle Factory supports three types of mods that extend the game in different ways.

Idle Factoryは3種類のModに対応しており、それぞれ異なる方法でゲームを拡張できます。

| Type | Difficulty | What You Can Do |
|------|------------|-----------------|
| **Data Mod** | Easy | Add items, machines, recipes |
| **Script Mod** | Medium | Build external tools, overlays |
| **Core Mod** | Advanced | Change game behavior |

| 種類 | 難易度 | できること |
|------|--------|-----------|
| **Data Mod** | 簡単 | アイテム・機械・レシピの追加 |
| **Script Mod** | 中級 | 外部ツール・オーバーレイの作成 |
| **Core Mod** | 上級 | ゲーム動作の変更 |

---

## Quick Start / クイックスタート

### Step 1: Choose Your Mod Type

**Want to add new content?**
→ Start with [Data Mod Guide](Data-Mod-Guide)

**Want to build external tools?**
→ See [Script Mod Guide](Script-Mod-Guide)

**Want to change game logic?**
→ Read [Core Mod Guide](Core-Mod-Guide)

### Step 1: Modの種類を選ぶ

**新しいコンテンツを追加したい**
→ [Data Mod Guide](Data-Mod-Guide) から始める

**外部ツールを作りたい**
→ [Script Mod Guide](Script-Mod-Guide) を参照

**ゲームロジックを変えたい**
→ [Core Mod Guide](Core-Mod-Guide) を読む

---

## Learning Path / 学習パス

### Beginner / 初心者

1. **[Getting Started](Getting-Started)** - Create your first mod in 5 minutes

   5分で最初のModを作成

2. **[Mod Structure](Mod-Structure)** - Understand mod.toml and folder layout

   mod.tomlとフォルダ構成を理解

### Intermediate / 中級者

3. **[Data Mod Guide](Data-Mod-Guide)** - Complete guide to items, machines, recipes

   アイテム・機械・レシピの完全ガイド

4. **[TOML Schema](TOML-Schema)** - All configuration fields reference

   全設定フィールドのリファレンス

### Advanced / 上級者

5. **[Script Mod Guide](Script-Mod-Guide)** - WebSocket API for external tools

   外部ツール用WebSocket API

6. **[Core Mod Guide](Core-Mod-Guide)** - WASM mods for game logic changes

   ゲームロジック変更用WASM Mod

---

## Example: Your First Data Mod

Create a new item in just 2 files:

たった2ファイルで新アイテムを作成:

**mods/my_mod/mod.toml**
```toml
[mod]
id = "my_mod"
name = "My Mod"
version = "1.0.0"

[mod.dependencies]
base = ">=0.3.0"
```

**mods/my_mod/items.toml**
```toml
[[item]]
id = "diamond"
name = "Diamond"
description = "A rare gem"
stack_size = 999
category = "ore"
```

Launch the game, and your diamond item appears automatically!

ゲームを起動すると、ダイヤモンドアイテムが自動的に追加されます！

---

## File Structure Overview / ファイル構成

```
mods/
└── my_mod/
    ├── mod.toml        # Required: Mod metadata
    │                   # 必須: Modメタデータ
    │
    ├── items.toml      # Optional: Item definitions
    │                   # 任意: アイテム定義
    │
    ├── machines.toml   # Optional: Machine definitions
    │                   # 任意: 機械定義
    │
    ├── recipes.toml    # Optional: Recipe definitions
    │                   # 任意: レシピ定義
    │
    ├── textures/       # Optional: PNG texture files
    │                   # 任意: PNGテクスチャ
    │
    └── models/         # Optional: GLB/GLTF 3D models
                        # 任意: GLB/GLTF 3Dモデル
```

---

## API Reference / APIリファレンス

| Document | Description |
|----------|-------------|
| [TOML Schema](TOML-Schema) | All item, machine, recipe fields |
| [WebSocket API](WebSocket-API) | JSON-RPC 2.0 methods for Script Mods |
| [WASM Host Functions](WASM-Host-Functions) | Host functions for Core Mods |

| ドキュメント | 説明 |
|-------------|------|
| [TOML Schema](TOML-Schema) | アイテム・機械・レシピの全フィールド |
| [WebSocket API](WebSocket-API) | Script Mod用JSON-RPC 2.0メソッド |
| [WASM Host Functions](WASM-Host-Functions) | Core Mod用ホスト関数 |

---

## Community / コミュニティ

- **[GitHub Issues](https://github.com/GS-Bacon/idle_factory/issues)** - Bug reports, feature requests

  バグ報告・機能要望

- **[Discussions](https://github.com/GS-Bacon/idle_factory/discussions)** - Questions, mod showcase

  質問・Mod紹介
