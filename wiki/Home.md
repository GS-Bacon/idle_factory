# Idle Factory Modding Wiki

Welcome to the official Modding documentation.
公式Moddingドキュメントへようこそ。

---

## Quick Links / クイックリンク

| Guide | Description |
|-------|-------------|
| [Getting Started](Getting-Started) | Create your first mod in 5 minutes / 5分でMod作成 |
| [Mod Structure](Mod-Structure) | mod.toml format and folder organization / mod.tomlとフォルダ構成 |
| [Data Mod Guide](Data-Mod-Guide) | Add items, machines, recipes via TOML / TOMLでアイテム追加 |
| [Script Mod Guide](Script-Mod-Guide) | External tools via WebSocket API / WebSocketで外部ツール |
| [Core Mod Guide](Core-Mod-Guide) | Advanced mods via WASM / WASMで高度なMod |

## API Reference / APIリファレンス

| Reference | Description |
|-----------|-------------|
| [WebSocket API](WebSocket-API) | JSON-RPC 2.0 methods for Script Mods |
| [WASM Host Functions](WASM-Host-Functions) | Host functions for Core Mods |
| [TOML Schema](TOML-Schema) | Item, machine, recipe field reference |

---

## Mod Types / Modの種類

| Type | Language | Use Case | Complexity |
|------|----------|----------|------------|
| **Data Mod** | TOML | Add items, machines, recipes / アイテム追加 | Easy / 簡単 |
| **Script Mod** | JS, Python, etc. | External tools, overlays / 外部ツール | Medium / 中級 |
| **Core Mod** | Rust (WASM) | Custom game logic / ゲームロジック変更 | Advanced / 上級 |

### Which should I use? / どれを使うべき？

```
Want to add new items/machines/recipes?
新しいアイテム・機械・レシピを追加したい
  → Data Mod (TOML only, no code / TOMLだけ、コード不要)

Want to build external tools (map viewer, statistics)?
外部ツールを作りたい（マップビューア、統計等）
  → Script Mod (WebSocket API)

Want to change game behavior?
ゲームの動作を変えたい
  → Core Mod (WASM)
```

---

## Example Mods / サンプル

### Data Mod: Add a new item / 新アイテム追加

```toml
# mods/mymod/items.toml
[[item]]
id = "diamond"
name = "Diamond"
description = "A rare gem"
stack_size = 999
category = "ore"
is_placeable = true
hardness = 2.0
color = [0.6, 0.95, 1.0]
tags = ["gem", "rare"]
```

### Script Mod: Get game version / ゲームバージョン取得

```javascript
const ws = new WebSocket("ws://127.0.0.1:9877");
ws.onopen = () => {
  ws.send(JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "game.version",
    params: {}
  }));
};
ws.onmessage = (e) => {
  const result = JSON.parse(e.data).result;
  console.log("Version:", result.version);
};
```

---

## File Structure Overview / ファイル構造概要

```
mods/
└── my_mod/
    ├── mod.toml        # Required: Metadata / 必須
    ├── items.toml      # Optional: Items / 任意
    ├── machines.toml   # Optional: Machines / 任意
    ├── recipes.toml    # Optional: Recipes / 任意
    ├── textures/       # Optional: PNG files / 任意
    └── models/         # Optional: GLB files / 任意
```

### Minimal mod.toml / 最小のmod.toml

```toml
[mod]
id = "my_mod"
name = "My Mod"
version = "1.0.0"

[mod.dependencies]
base = ">=0.3.0"
```

---

## Community / コミュニティ

- [GitHub Issues](https://github.com/user/idle_factory/issues) - Bug reports, feature requests / バグ報告・機能要望
- [Discussions](https://github.com/user/idle_factory/discussions) - Questions, mod showcase / 質問・Mod紹介
