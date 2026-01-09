# WASM Host Functions Reference

Host functions available to Core Mods (WASM).

## Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `host_get_inventory_slot` | `_caller: Caller<'_, ModState>, entity_id: u64, slot: u32` | Upper 32 bits: item_id, Lower 32 bits: count (0 = empty slot) | Get inventory slot contents |
| `host_get_machine_state` | `_caller: Caller<'_, ModState>, entity_id: u64` | - 0: Idle | Get machine state by entity ID |
| `host_log_error` | `mut caller: Caller<'_, ModState>, ptr: u32, len: u32` | Nothing (void function) | Log an error message from WASM mod |
| `host_log_info` | `mut caller: Caller<'_, ModState>, ptr: u32, len: u32` | Nothing (void function) | Log an info message from WASM mod |
| `host_set_machine_enabled` | `_caller: Caller<'_, ModState>, entity_id: u64, enabled: i32` | - 0: Success | Enable or disable a machine |
| `host_subscribe_event` | `_caller: Caller<'_, ModState>, event_type: u32` | Subscription ID (>= 0 on success, negative on error) | Subscribe to game events |

## Details

### `host_get_inventory_slot`

Get inventory slot contents

**Parameters:**
- `_caller: Caller<'_`
- `ModState>`
- `entity_id: u64`
- `slot: u32`

**Returns:**
Upper 32 bits: item_id, Lower 32 bits: count (0 = empty slot)

### `host_get_machine_state`

Get machine state by entity ID

**Parameters:**
- `_caller: Caller<'_`
- `ModState>`
- `entity_id: u64`

**Returns:**
- 0: Idle
- 1: Processing
- 2: Waiting for input
- -1: Error (entity not found)

### `host_log_error`

Log an error message from WASM mod

**Parameters:**
- `mut caller: Caller<'_`
- `ModState>`
- `ptr: u32`
- `len: u32`

**Returns:**
Nothing (void function)

### `host_log_info`

Log an info message from WASM mod

**Parameters:**
- `mut caller: Caller<'_`
- `ModState>`
- `ptr: u32`
- `len: u32`

**Returns:**
Nothing (void function)

### `host_set_machine_enabled`

Enable or disable a machine

**Parameters:**
- `_caller: Caller<'_`
- `ModState>`
- `entity_id: u64`
- `enabled: i32`

**Returns:**
- 0: Success
- -1: Entity not found
- -2: Permission denied

### `host_subscribe_event`

Subscribe to game events

**Parameters:**
- `_caller: Caller<'_`
- `ModState>`
- `event_type: u32`

**Returns:**
Subscription ID (>= 0 on success, negative on error)
