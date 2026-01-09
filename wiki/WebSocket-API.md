# WebSocket API Reference

JSON-RPC 2.0 API for Mod integration.

**Connection**: `ws://127.0.0.1:9877`

## Methods

### Game

#### `game.state`

Handle `game.state` request

**Response:**
```json
{
"paused": false,
"tick": 12345,
"player_count": 1
}
```

#### `game.version`

Handle `game.version` request

**Response:**
```json
{
"version": "0.3.78",
"api_version": "1.0.0"
}
```

### Item

#### `item.add`

Handle item.add request

**Response:**
```json
{ "success": true, "id": "mymod:super_ingot" }
```

#### `item.list`

Handle item.list request

**Response:**
```json
{ "items": [{ "id": "base:iron_ore", "name": "Iron Ore", "stack_size": 999 }] }
```

### Machine

#### `machine.add`

Handle machine.add request

**Response:**
```json
{ "success": true, "id": "mymod:super_furnace" }
```

#### `machine.list`

Handle machine.list request

**Response:**
```json
{ "machines": [{ "id": "furnace", "name": "精錬炉", "input_slots": 1, "output_slots": 1, "requires_fuel": true }] }
```

### Mod

#### `mod.disable`

mod.disable handler

**Response:**
```json
{ "success": true }
```

#### `mod.enable`

mod.enable handler

**Response:**
```json
{ "success": true }
```

#### `mod.info`

mod.info handler

**Response:**
```json
{ "id": "base", "name": "Base Game", "version": "0.3.78", "description": "...", "enabled": true }
```

#### `mod.list`

mod.list handler

**Response:**
```json
{ "mods": [{ "id": "base", "name": "Base Game", "version": "0.3.78", "enabled": true }] }
```

### Recipe

#### `recipe.add`

Handle recipe.add method (stub)

**Response:**
```json
{ "success": true, "id": "mymod:super_smelt" }
```

#### `recipe.list`

Handle recipe.list method

**Response:**
```json
{ "recipes": [{ "id": "smelt_iron", "machine_type": "furnace", "inputs": [{ "item": "iron_ore", "count": 1 }], "outputs": [{ "item": "iron_ingot", "count": 1 }], "time": 2.0 }] }
```

### Test

#### `test.assert`

Handle test.assert request

**Response:**
```json
{ "success": true, "expected": "Inventory", "actual": "Inventory" }
```

#### `test.get_state`

Handle test.get_state request

**Response:**
```json
{ "ui_state": "Gameplay", "player_position": [0.0, 10.0, 0.0], "cursor_locked": true }
```

#### `test.send_input`

Handle test.send_input request

**Response:**
```json
{ "success": true, "action": "ToggleInventory" }
```

### Texture

#### `texture.get_atlas_info`

Handle texture.get_atlas_info request

**Response:**
```json
{ "size": [256, 256], "tile_size": 16, "texture_count": 12, "generation": 1 }
```

#### `texture.list`

Handle texture.list request

**Response:**
```json
{ "textures": [{ "name": "stone", "uv": [0.0, 0.0, 0.0625, 0.0625], "is_mod": false }], "atlas_size": [256, 256] }
```

#### `texture.register_resolver`

Handle texture.register_resolver request

**Response:**
```json
{ "success": true, "resolver_id": 1 }
```
