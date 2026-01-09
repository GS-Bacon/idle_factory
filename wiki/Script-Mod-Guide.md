# Script Mod Guide / Script Modガイド

Build external tools that communicate with the game via WebSocket.
WebSocketでゲームと通信する外部ツールを作成。

---

## Overview / 概要

Script Mods are external programs that:
Script Modは外部プログラムで:

- Connect to the game via WebSocket / WebSocketでゲームに接続
- Query game state (items, machines, recipes) / ゲーム状態を照会
- Optionally modify data at runtime / 実行時にデータを変更可能

**Use cases / ユースケース:**
- Map viewers / マップビューア
- Statistics dashboards / 統計ダッシュボード
- Item databases / アイテムデータベース
- Automation scripts / 自動化スクリプト

---

## Quick Start / クイックスタート

### 1. Start the game / ゲームを起動

WebSocket server runs on `ws://127.0.0.1:9877`.
WebSocketサーバーは `ws://127.0.0.1:9877` で動作。

### 2. Connect and query / 接続して照会

**JavaScript (Node.js or browser):**

```javascript
const WebSocket = require('ws'); // Node.js only

const ws = new WebSocket('ws://127.0.0.1:9877');

ws.on('open', () => {
  console.log('Connected to game');

  // Request game version
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'game.version',
    params: {}
  }));
});

ws.on('message', (data) => {
  const response = JSON.parse(data);
  console.log('Response:', response.result);
});
```

**Python:**

```python
import asyncio
import websockets
import json

async def main():
    uri = "ws://127.0.0.1:9877"
    async with websockets.connect(uri) as ws:
        # Request game version
        await ws.send(json.dumps({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "game.version",
            "params": {}
        }))

        response = json.loads(await ws.recv())
        print(f"Game version: {response['result']['version']}")

asyncio.run(main())
```

---

## Helper Library / ヘルパーライブラリ

### JavaScript async wrapper / JavaScript非同期ラッパー

```javascript
class GameClient {
  constructor(url = 'ws://127.0.0.1:9877') {
    this.ws = new WebSocket(url);
    this.pending = new Map();
    this.nextId = 1;

    this.ws.onmessage = (event) => {
      const response = JSON.parse(event.data);
      const resolve = this.pending.get(response.id);
      if (resolve) {
        this.pending.delete(response.id);
        resolve(response);
      }
    };
  }

  async ready() {
    return new Promise(resolve => {
      if (this.ws.readyState === 1) resolve();
      else this.ws.onopen = resolve;
    });
  }

  async call(method, params = {}) {
    await this.ready();
    const id = this.nextId++;

    return new Promise((resolve, reject) => {
      this.pending.set(id, (response) => {
        if (response.error) reject(response.error);
        else resolve(response.result);
      });

      this.ws.send(JSON.stringify({
        jsonrpc: '2.0',
        id,
        method,
        params
      }));
    });
  }
}

// Usage
const client = new GameClient();
const version = await client.call('game.version');
const items = await client.call('item.list');
```

---

## Example: Item Database / 例: アイテムデータベース

```javascript
const client = new GameClient();

async function buildItemDatabase() {
  const { items } = await client.call('item.list');

  const database = {
    byId: {},
    byCategory: {},
    byTag: {}
  };

  for (const item of items) {
    // Index by ID
    database.byId[item.id] = item;

    // Index by category
    if (!database.byCategory[item.category]) {
      database.byCategory[item.category] = [];
    }
    database.byCategory[item.category].push(item);

    // Index by tags (if available)
    for (const tag of item.tags || []) {
      if (!database.byTag[tag]) {
        database.byTag[tag] = [];
      }
      database.byTag[tag].push(item);
    }
  }

  return database;
}

// Usage
const db = await buildItemDatabase();
console.log('Ores:', db.byCategory['ore']);
console.log('Smeltable:', db.byTag['smeltable']);
```

---

## Example: Recipe Analyzer / 例: レシピ分析

```javascript
async function analyzeRecipes() {
  const { recipes } = await client.call('recipe.list');
  const { items } = await client.call('item.list');

  const itemMap = Object.fromEntries(items.map(i => [i.id, i]));

  for (const recipe of recipes) {
    const inputCost = recipe.inputs.reduce((sum, i) => sum + i.count, 0);
    const outputYield = recipe.outputs.reduce((sum, o) => sum + o.count, 0);
    const efficiency = outputYield / inputCost;

    console.log(`${recipe.id}: ${efficiency.toFixed(2)}x efficiency`);
  }
}
```

---

## Example: Live Statistics / 例: リアルタイム統計

```javascript
async function monitorGame() {
  setInterval(async () => {
    const state = await client.call('game.state');

    console.log(`Tick: ${state.tick}, Paused: ${state.paused}`);
  }, 1000);
}
```

---

## Best Practices / ベストプラクティス

### 1. Handle disconnection / 切断を処理

```javascript
ws.on('close', () => {
  console.log('Disconnected, reconnecting...');
  setTimeout(() => connect(), 5000);
});

ws.on('error', (err) => {
  console.error('WebSocket error:', err);
});
```

### 2. Batch requests / リクエストをバッチ処理

```javascript
// Bad: Many sequential requests
for (const id of itemIds) {
  await client.call('item.info', { id });
}

// Good: Single list request
const { items } = await client.call('item.list');
const filtered = items.filter(i => itemIds.includes(i.id));
```

### 3. Cache data / データをキャッシュ

```javascript
let cachedItems = null;
let cacheTime = 0;

async function getItems() {
  if (!cachedItems || Date.now() - cacheTime > 60000) {
    cachedItems = await client.call('item.list');
    cacheTime = Date.now();
  }
  return cachedItems;
}
```

---

## Available Methods / 利用可能なメソッド

See [WebSocket API](WebSocket-API) for full reference.
完全なリファレンスは[WebSocket API](WebSocket-API)を参照。

| Method | Description |
|--------|-------------|
| `game.version` | Get game/API version |
| `game.state` | Get current game state |
| `item.list` | List all items |
| `item.add` | Register new item |
| `machine.list` | List all machines |
| `machine.add` | Register new machine |
| `recipe.list` | List all recipes |
| `recipe.add` | Register new recipe |
| `mod.list` | List loaded mods |
| `mod.info` | Get mod details |
| `mod.enable/disable` | Toggle mod state |
| `texture.list` | List textures |

---

## Debugging / デバッグ

### Check connection / 接続確認

```bash
# Using websocat (install: cargo install websocat)
echo '{"jsonrpc":"2.0","id":1,"method":"game.version","params":{}}' | websocat ws://127.0.0.1:9877
```

### Check game logs / ゲームログ確認

```
logs/game.log
```

WebSocket errors are logged with `[WebSocket]` prefix.
WebSocketエラーは `[WebSocket]` プレフィックスでログ出力。

---

## See Also / 関連

- [WebSocket API](WebSocket-API) - Complete method reference / メソッドリファレンス
- [Getting Started](Getting-Started) - Quick tutorial / クイックチュートリアル
