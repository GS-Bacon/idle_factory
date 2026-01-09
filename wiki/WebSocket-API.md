# WebSocket API Reference / WebSocket APIリファレンス

JSON-RPC 2.0 API for Script Mods.
Script Mod用のJSON-RPC 2.0 API。

---

## Connection / 接続

```
ws://127.0.0.1:9877
```

The WebSocket server starts automatically when the game runs.
ゲーム起動時にWebSocketサーバーが自動的に開始。

---

## Protocol / プロトコル

### Request Format / リクエスト形式

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "category.action",
  "params": {}
}
```

| Field | Type | Description |
|-------|------|-------------|
| `jsonrpc` | string | Always `"2.0"` / 常に`"2.0"` |
| `id` | int | Request ID, returned in response / リクエストID |
| `method` | string | Method name / メソッド名 |
| `params` | object | Method parameters / パラメータ |

### Response Format / レスポンス形式

**Success / 成功:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { ... }
}
```

**Error / エラー:**
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

### Error Codes / エラーコード

| Code | Meaning |
|------|---------|
| `-32700` | Parse error (invalid JSON) / パースエラー |
| `-32600` | Invalid request / 無効なリクエスト |
| `-32601` | Method not found / メソッドが見つからない |
| `-32602` | Invalid params / 無効なパラメータ |
| `-32603` | Internal error / 内部エラー |
| `-32001` | Item already exists / アイテムが既に存在 |
| `-32002` | Invalid item ID / 無効なアイテムID |

---

## Quick Start / クイックスタート

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

## Methods / メソッド

### Game / ゲーム

#### `game.version`

Get game and API version. / ゲームとAPIのバージョンを取得。

**Request:**
```json
{ "method": "game.version", "params": {} }
```

**Response:**
```json
{
  "version": "0.3.136",
  "api_version": "1.0.0"
}
```

---

#### `game.state`

Get current game state. / 現在のゲーム状態を取得。

**Request:**
```json
{ "method": "game.state", "params": {} }
```

**Response:**
```json
{
  "paused": false,
  "tick": 12345,
  "player_count": 1
}
```

| Field | Type | Description |
|-------|------|-------------|
| `paused` | bool | Game is paused / 一時停止中 |
| `tick` | int | Current game tick / ゲームtick |
| `player_count` | int | Number of players / プレイヤー数 |

---

### Items / アイテム

#### `item.list`

List all registered items. / 全アイテムを一覧。

**Request:**
```json
{ "method": "item.list", "params": {} }
```

**Optional: Filter by mod / Modでフィルタ:**
```json
{ "method": "item.list", "params": { "mod_id": "base" } }
```

**Response:**
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

Register a new item. / 新しいアイテムを登録。

**Request:**
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

**Response:**
```json
{ "success": true, "id": "mymod:super_ore" }
```

**Error (duplicate):**
```json
{ "error": { "code": -32001, "message": "Item already exists" } }
```

---

### Machines / 機械

#### `machine.list`

List all machines. / 全機械を一覧。

**Request:**
```json
{ "method": "machine.list", "params": {} }
```

**Response:**
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

Register a new machine type. / 新しい機械タイプを登録。

**Request:**
```json
{
  "method": "machine.add",
  "params": {
    "id": "mymod:super_furnace",
    "name": "Super Furnace"
  }
}
```

**Response:**
```json
{ "success": true, "id": "mymod:super_furnace" }
```

---

### Recipes / レシピ

#### `recipe.list`

List all recipes. / 全レシピを一覧。

**Request:**
```json
{ "method": "recipe.list", "params": {} }
```

**Response:**
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

Register a new recipe. / 新しいレシピを登録。

**Request:**
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

**Response:**
```json
{ "success": true, "id": "mymod:super_smelt" }
```

---

### Mods / Mod管理

#### `mod.list`

List all mods. / 全Modを一覧。

**Request:**
```json
{ "method": "mod.list", "params": {} }
```

**Response:**
```json
{
  "mods": [
    { "id": "base", "name": "Base Game", "version": "0.3.136", "enabled": true }
  ]
}
```

---

#### `mod.info`

Get mod details. / Mod詳細を取得。

**Request:**
```json
{ "method": "mod.info", "params": { "id": "base" } }
```

**Response:**
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

Enable or disable a mod. / Modを有効化/無効化。

**Request:**
```json
{ "method": "mod.enable", "params": { "id": "my_mod" } }
{ "method": "mod.disable", "params": { "id": "my_mod" } }
```

**Response:**
```json
{ "success": true }
```

---

### Textures / テクスチャ

#### `texture.list`

List all textures. / 全テクスチャを一覧。

**Response:**
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

Get texture atlas info. / テクスチャアトラス情報を取得。

**Response:**
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

Register custom texture resolver. / カスタムテクスチャリゾルバを登録。

**Request:**
```json
{ "method": "texture.register_resolver", "params": { "pattern": "mymod:*" } }
```

**Response:**
```json
{ "success": true, "resolver_id": 1 }
```

---

### Test / テスト (E2E用)

For automated testing, not mods. / 自動テスト用、Mod向けではない。

#### `test.get_state`

```json
{ "ui_state": "Gameplay", "player_position": [0.0, 10.0, 0.0], "cursor_locked": true }
```

#### `test.send_input`

```json
{ "method": "test.send_input", "params": { "action": "ToggleInventory" } }
```

Available actions: `MoveForward`, `MoveBackward`, `MoveLeft`, `MoveRight`, `Jump`, `ToggleInventory`, `TogglePause`, `ToggleQuest`, `OpenCommand`, `CloseUI`, `PrimaryAction`, `SecondaryAction`, `RotateBlock`, `Hotbar1`-`Hotbar9`

#### `test.assert`

```json
{ "method": "test.assert", "params": { "condition": "ui_state == Inventory" } }
```

---

## See Also / 関連

- [Script Mod Guide](Script-Mod-Guide) - External tool development / 外部ツール開発
- [Getting Started](Getting-Started) - Quick tutorial / クイックチュートリアル
