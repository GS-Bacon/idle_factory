# Mod API Documentation

Welcome to the Mod API documentation for the game.

## API Types

| Type | Description | Link |
|------|-------------|------|
| **WebSocket API** | JSON-RPC 2.0 API for Script Mods | [WebSocket API](WebSocket-API) |
| **WASM Host Functions** | Host functions for Core Mods | [WASM Host Functions](WASM-Host-Functions) |

## Getting Started

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

See [WASM Host Functions](WASM-Host-Functions) for available functions.

---

*Auto-generated from source code.*
