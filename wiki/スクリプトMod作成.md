# Script Modガイド

[English version](Script-Mod-Guide)

WebSocketでゲームと通信する外部ツールを作成するガイドです。

---

## 概要

Script Modは外部プログラムで、以下のことができます:

- WebSocketでゲームに接続
- ゲーム状態を照会（アイテム、機械、レシピ）
- 実行時にデータを変更可能

**ユースケース:**
- マップビューア
- 統計ダッシュボード
- アイテムデータベース
- 自動化スクリプト

---

## クイックスタート

### 1. ゲームを起動

WebSocketサーバーは `ws://127.0.0.1:9877` で動作します。

### 2. 接続して照会

**JavaScript (Node.js またはブラウザ):**

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

## ヘルパーライブラリ

### JavaScript非同期ラッパー

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

## 例: アイテムデータベース

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

## 例: レシピ分析

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

## 例: リアルタイム統計

```javascript
async function monitorGame() {
  setInterval(async () => {
    const state = await client.call('game.state');

    console.log(`Tick: ${state.tick}, Paused: ${state.paused}`);
  }, 1000);
}
```

---

## ベストプラクティス

### 1. 切断を処理

```javascript
ws.on('close', () => {
  console.log('Disconnected, reconnecting...');
  setTimeout(() => connect(), 5000);
});

ws.on('error', (err) => {
  console.error('WebSocket error:', err);
});
```

### 2. リクエストをバッチ処理

```javascript
// Bad: Many sequential requests
for (const id of itemIds) {
  await client.call('item.info', { id });
}

// Good: Single list request
const { items } = await client.call('item.list');
const filtered = items.filter(i => itemIds.includes(i.id));
```

### 3. データをキャッシュ

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

## 利用可能なメソッド

完全なリファレンスは[WebSocket API](WebSocket-API-ja)を参照してください。

| メソッド | 説明 |
|--------|-------------|
| `game.version` | ゲーム/APIバージョンを取得 |
| `game.state` | 現在のゲーム状態を取得 |
| `item.list` | 全アイテムを一覧 |
| `item.add` | 新しいアイテムを登録 |
| `machine.list` | 全機械を一覧 |
| `machine.add` | 新しい機械を登録 |
| `recipe.list` | 全レシピを一覧 |
| `recipe.add` | 新しいレシピを登録 |
| `mod.list` | ロード済みModを一覧 |
| `mod.info` | Mod詳細を取得 |
| `mod.enable/disable` | Modの有効/無効を切替 |
| `texture.list` | テクスチャを一覧 |

---

## デバッグ

### 接続確認

```bash
# Using websocat (install: cargo install websocat)
echo '{"jsonrpc":"2.0","id":1,"method":"game.version","params":{}}' | websocat ws://127.0.0.1:9877
```

### ゲームログ確認

```
logs/game.log
```

WebSocketエラーは `[WebSocket]` プレフィックスでログ出力されます。

---

## 関連

- [WebSocket API](WebSocket-API-ja) - メソッドリファレンス
- [Getting Started](Getting-Started) - クイックチュートリアル
