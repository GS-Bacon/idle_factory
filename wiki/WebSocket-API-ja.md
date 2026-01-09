# WebSocket APIリファレンス

[English version](WebSocket-API)

Script Mod用のJSON-RPC 2.0 APIリファレンスです。

---

## 接続

```
ws://127.0.0.1:9877
```

ゲーム起動時にWebSocketサーバーが自動的に開始されます。

---

## プロトコル

### リクエスト形式

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "category.action",
  "params": {}
}
```

| フィールド | 型 | 説明 |
|-------|------|-------------|
| `jsonrpc` | string | 常に`"2.0"` |
| `id` | int | リクエストID、レスポンスで返却 |
| `method` | string | メソッド名 |
| `params` | object | パラメータ |

### レスポンス形式

**成功:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { ... }
}
```

**エラー:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

### エラーコード

| コード | 意味 |
|------|---------|
| `-32700` | パースエラー（無効なJSON） |
| `-32600` | 無効なリクエスト |
| `-32601` | メソッドが見つからない |
| `-32602` | 無効なパラメータ |
| `-32603` | 内部エラー |
| `-32001` | アイテムが既に存在 |
| `-32002` | 無効なアイテムID |

---

## クイックスタート

### JavaScript

```javascript
const ws = new WebSocket("ws://127.0.0.1:9877");

// Helper function
function call(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = Date.now();
    ws.send(JSON.stringify({ jsonrpc: "2.0", id, method, params }));

    const handler = (event) => {
      const response = JSON.parse(event.data);
      if (response.id === id) {
        ws.removeEventListener("message", handler);
        response.error ? reject(response.error) : resolve(response.result);
      }
    };
    ws.addEventListener("message", handler);
  });
}

// Usage
ws.onopen = async () => {
  const version = await call("game.version");
  console.log("Game version:", version.version);

  const items = await call("item.list");
  console.log("Items:", items.items.length);
};
```

### Python

```python
import asyncio
import websockets
import json

async def main():
    async with websockets.connect("ws://127.0.0.1:9877") as ws:
        # Get game version
        await ws.send(json.dumps({
            "jsonrpc": "2.0", "id": 1,
            "method": "game.version", "params": {}
        }))
        response = json.loads(await ws.recv())
        print(f"Game version: {response['result']['version']}")

asyncio.run(main())
```

---

## メソッド

### ゲーム

#### `game.version`

ゲームとAPIのバージョンを取得。

**リクエスト:**
```json
{ "method": "game.version", "params": {} }
```

**レスポンス:**
```json
{
  "version": "0.3.136",
  "api_version": "1.0.0"
}
```

---

#### `game.state`

現在のゲーム状態を取得。

**リクエスト:**
```json
{ "method": "game.state", "params": {} }
```

**レスポンス:**
```json
{
  "paused": false,
  "tick": 12345,
  "player_count": 1
}
```

| フィールド | 型 | 説明 |
|-------|------|-------------|
| `paused` | bool | 一時停止中 |
| `tick` | int | ゲームtick |
| `player_count` | int | プレイヤー数 |

---

### アイテム

#### `item.list`

全アイテムを一覧。

**リクエスト:**
```json
{ "method": "item.list", "params": {} }
```

**オプション: Modでフィルタ:**
```json
{ "method": "item.list", "params": { "mod_id": "base" } }
```

**レスポンス:**
```json
{
  "items": [
    {
      "id": "base:iron_ore",
      "name": "Iron Ore",
      "stack_size": 999,
      "category": "ore"
    }
  ]
}
```

---

#### `item.add`

新しいアイテムを登録。

**リクエスト:**
```json
{
  "method": "item.add",
  "params": {
    "id": "mymod:super_ore",
    "name": "Super Ore",
    "description": "A very special ore",
    "stack_size": 999,
    "category": "ore"
  }
}
```

**レスポンス:**
```json
{ "success": true, "id": "mymod:super_ore" }
```

**エラー（重複）:**
```json
{ "error": { "code": -32001, "message": "Item already exists" } }
```

---

### 機械

#### `machine.list`

全機械を一覧。

**リクエスト:**
```json
{ "method": "machine.list", "params": {} }
```

**レスポンス:**
```json
{
  "machines": [
    {
      "id": "furnace",
      "name": "精錬炉",
      "input_slots": 2,
      "output_slots": 1,
      "requires_fuel": true
    }
  ]
}
```

---

#### `machine.add`

新しい機械タイプを登録。

**リクエスト:**
```json
{
  "method": "machine.add",
  "params": {
    "id": "mymod:super_furnace",
    "name": "Super Furnace"
  }
}
```

**レスポンス:**
```json
{ "success": true, "id": "mymod:super_furnace" }
```

---

### レシピ

#### `recipe.list`

全レシピを一覧。

**リクエスト:**
```json
{ "method": "recipe.list", "params": {} }
```

**レスポンス:**
```json
{
  "recipes": [
    {
      "id": "smelt_iron",
      "machine_type": "furnace",
      "inputs": [{ "item": "iron_ore", "count": 1 }],
      "outputs": [{ "item": "iron_ingot", "count": 1 }],
      "time": 2.0
    }
  ]
}
```

---

#### `recipe.add`

新しいレシピを登録。

**リクエスト:**
```json
{
  "method": "recipe.add",
  "params": {
    "id": "mymod:super_smelt",
    "machine_type": "furnace",
    "inputs": [{ "item": "base:iron_ore", "count": 1 }],
    "outputs": [{ "item": "base:iron_ingot", "count": 2 }],
    "time": 1.0
  }
}
```

**レスポンス:**
```json
{ "success": true, "id": "mymod:super_smelt" }
```

---

### Mod管理

#### `mod.list`

全Modを一覧。

**リクエスト:**
```json
{ "method": "mod.list", "params": {} }
```

**レスポンス:**
```json
{
  "mods": [
    { "id": "base", "name": "Base Game", "version": "0.3.136", "enabled": true }
  ]
}
```

---

#### `mod.info`

Mod詳細を取得。

**リクエスト:**
```json
{ "method": "mod.info", "params": { "id": "base" } }
```

**レスポンス:**
```json
{
  "id": "base",
  "name": "Base Game",
  "version": "0.3.136",
  "author": "Idle Factory Team",
  "description": "Core game content",
  "enabled": true
}
```

---

#### `mod.enable` / `mod.disable`

Modを有効化/無効化。

**リクエスト:**
```json
{ "method": "mod.enable", "params": { "id": "my_mod" } }
{ "method": "mod.disable", "params": { "id": "my_mod" } }
```

**レスポンス:**
```json
{ "success": true }
```

---

### テクスチャ

#### `texture.list`

全テクスチャを一覧。

**レスポンス:**
```json
{
  "textures": [
    { "name": "stone", "uv": [0.0, 0.0, 0.0625, 0.0625], "is_mod": false }
  ],
  "atlas_size": [256, 256]
}
```

---

#### `texture.get_atlas_info`

テクスチャアトラス情報を取得。

**レスポンス:**
```json
{
  "size": [256, 256],
  "tile_size": 16,
  "texture_count": 12,
  "generation": 1
}
```

---

#### `texture.register_resolver`

カスタムテクスチャリゾルバを登録。

**リクエスト:**
```json
{ "method": "texture.register_resolver", "params": { "pattern": "mymod:*" } }
```

**レスポンス:**
```json
{ "success": true, "resolver_id": 1 }
```

---

### テスト (E2E用)

自動テスト用であり、Mod向けではありません。

#### `test.get_state`

```json
{ "ui_state": "Gameplay", "player_position": [0.0, 10.0, 0.0], "cursor_locked": true }
```

#### `test.send_input`

```json
{ "method": "test.send_input", "params": { "action": "ToggleInventory" } }
```

利用可能なアクション: `MoveForward`, `MoveBackward`, `MoveLeft`, `MoveRight`, `Jump`, `ToggleInventory`, `TogglePause`, `ToggleQuest`, `OpenCommand`, `CloseUI`, `PrimaryAction`, `SecondaryAction`, `RotateBlock`, `Hotbar1`-`Hotbar9`

#### `test.assert`

```json
{ "method": "test.assert", "params": { "condition": "ui_state == Inventory" } }
```

---

## 関連

- [Script Modガイド](スクリプトMod作成) - 外部ツール開発
- [Getting Started](Getting-Started) - クイックチュートリアル
