# WebSocket API リファレンス

Mod連携用 JSON-RPC 2.0 API

**接続先**: `ws://127.0.0.1:9877`

## メソッド一覧

### Game

#### `game.state`

現在のゲーム状態を取得

**レスポンス:**
```json
{
"paused": false,
"tick": 12345,
"player_count": 1
}
```

#### `game.version`

ゲームバージョンとAPIバージョンを取得

**レスポンス:**
```json
{
"version": "0.3.78",
"api_version": "1.0.0"
}
```

### Item

#### `item.add`

新しいアイテムを登録（現在はスタブ実装）

**レスポンス:**
```json
{ "success": true, "id": "mymod:super_ingot" }
```

#### `item.list`

登録済みアイテム一覧を取得（namespace指定でフィルタ可能）

**レスポンス:**
```json
{ "items": [{ "id": "base:iron_ore", "name": "Iron Ore", "stack_size": 999 }] }
```

### Machine

#### `machine.add`

新しい機械タイプを登録（スタブ実装）

**レスポンス:**
```json
{ "success": true, "id": "mymod:super_furnace" }
```

#### `machine.list`

登録済み機械の一覧を取得

**レスポンス:**
```json
{ "machines": [{ "id": "furnace", "name": "精錬炉", "input_slots": 1, "output_slots": 1, "requires_fuel": true }] }
```

### Mod

#### `mod.disable`

有効なModを無効化

**レスポンス:**
```json
{ "success": true }
```

#### `mod.enable`

無効化されたModを有効化

**レスポンス:**
```json
{ "success": true }
```

#### `mod.info`

指定Modの詳細情報を取得

**レスポンス:**
```json
{ "id": "base", "name": "Base Game", "version": "0.3.78", "description": "...", "enabled": true }
```

#### `mod.list`

登録済みMod一覧を取得

**レスポンス:**
```json
{ "mods": [{ "id": "base", "name": "Base Game", "version": "0.3.78", "enabled": true }] }
```

### Recipe

#### `recipe.add`

新しいレシピを登録（現在はスタブ実装）

**レスポンス:**
```json
{ "success": true, "id": "mymod:super_smelt" }
```

#### `recipe.list`

レシピ一覧を取得（machine_typeでフィルタ可能）

**レスポンス:**
```json
{ "recipes": [{ "id": "smelt_iron", "machine_type": "furnace", "inputs": [{ "item": "iron_ore", "count": 1 }], "outputs": [{ "item": "iron_ingot", "count": 1 }], "time": 2.0 }] }
```

### Test

#### `test.assert`

E2Eテスト用に条件をゲーム状態と照合

**レスポンス:**
```json
{ "success": true, "expected": "Inventory", "actual": "Inventory" }
```

#### `test.get_state`

E2Eテスト用に現在のゲーム状態を取得

**レスポンス:**
```json
{ "ui_state": "Gameplay", "player_position": [0.0, 10.0, 0.0], "cursor_locked": true }
```

#### `test.send_input`

E2Eテスト用に仮想入力を注入

**レスポンス:**
```json
{ "success": true, "action": "ToggleInventory" }
```

### Texture

#### `texture.get_atlas_info`

テクスチャアトラス情報を取得

**レスポンス:**
```json
{ "size": [256, 256], "tile_size": 16, "texture_count": 12, "generation": 1 }
```

#### `texture.list`

登録済みテクスチャ一覧を取得（UV座標付き）

**レスポンス:**
```json
{ "textures": [{ "name": "stone", "uv": [0.0, 0.0, 0.0625, 0.0625], "is_mod": false }], "atlas_size": [256, 256] }
```

#### `texture.register_resolver`

カスタムテクスチャリゾルバを登録（接続テクスチャ等用）

**レスポンス:**
```json
{ "success": true, "resolver_id": 1 }
```
