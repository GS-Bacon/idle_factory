# Mod API ドキュメント

ゲームのMod API ドキュメントへようこそ。

[English](Home) | **日本語**

## API種別

| 種別 | 説明 | リンク |
|------|------|--------|
| **WebSocket API** | Script Mod用 JSON-RPC 2.0 API | [WebSocket API](WebSocket-API-ja) |
| **WASMホスト関数** | Core Mod用ホスト関数 | [WASMホスト関数](WASM-Host-Functions-ja) |

## はじめに

### Script Mod (WebSocket)

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

ws.onmessage = (event) => {
  console.log(JSON.parse(event.data));
};
```

### Core Mod (WASM)

利用可能な関数は [WASMホスト関数](WASM-Host-Functions-ja) を参照してください。

---

*ソースコードから自動生成*
