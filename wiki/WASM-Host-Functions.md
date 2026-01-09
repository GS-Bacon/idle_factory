[日本語版はこちら](WASMホスト関数)

# WASM Host Functions Reference

Host functions available to Core Mods (WASM).

---

## Overview

Host functions are the bridge between your WASM mod and the game.

```rust
// Declare host functions
extern "C" {
    fn host_log_info(ptr: *const u8, len: usize);
    fn host_get_machine_state(entity_id: u64) -> i32;
    // ...
}

// Use them
unsafe {
    let msg = "Hello from WASM!";
    host_log_info(msg.as_ptr(), msg.len());
}
```

---

## Function Summary

| Function | Description |
|----------|-------------|
| [`host_log_info`](#host_log_info) | Log info message |
| [`host_log_error`](#host_log_error) | Log error message |
| [`host_subscribe_event`](#host_subscribe_event) | Subscribe to events |
| [`host_get_machine_state`](#host_get_machine_state) | Get machine state |
| [`host_set_machine_enabled`](#host_set_machine_enabled) | Enable/disable machine |
| [`host_get_inventory_slot`](#host_get_inventory_slot) | Get inventory slot |

---

## Logging

### host_log_info

Log an info message. Appears in `logs/game.log` with `[CoreMod]` prefix.

```rust
extern "C" {
    fn host_log_info(ptr: *const u8, len: usize);
}
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `ptr` | `*const u8` | Pointer to UTF-8 string |
| `len` | `usize` | String length in bytes |

**Returns:** Nothing (void)

**Example:**
```rust
fn log_info(msg: &str) {
    unsafe { host_log_info(msg.as_ptr(), msg.len()); }
}

log_info("Machine processed item");
```

---

### host_log_error

Log an error message.

```rust
extern "C" {
    fn host_log_error(ptr: *const u8, len: usize);
}
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `ptr` | `*const u8` | Pointer to UTF-8 string |
| `len` | `usize` | String length in bytes |

**Returns:** Nothing (void)

**Example:**
```rust
fn log_error(msg: &str) {
    unsafe { host_log_error(msg.as_ptr(), msg.len()); }
}

log_error("Failed to find machine");
```

---

## Events

### host_subscribe_event

Subscribe to game events. When the event fires, `on_event()` is called.

```rust
extern "C" {
    fn host_subscribe_event(event_type: u32) -> i32;
}
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `event_type` | `u32` | Event type ID (see table below) |

**Returns:**
| Value | Meaning |
|-------|---------|
| `>= 0` | Subscription ID (success) |
| `< 0` | Error |

**Event Types:**
| ID | Constant | Description |
|----|----------|-------------|
| `1` | `EVENT_MACHINE_COMPLETE` | Machine finished processing |
| `2` | `EVENT_ITEM_CRAFTED` | Item was crafted |
| `3` | `EVENT_BLOCK_PLACED` | Block placed in world |
| `4` | `EVENT_BLOCK_REMOVED` | Block removed from world |

**Example:**
```rust
const EVENT_MACHINE_COMPLETE: u32 = 1;

#[no_mangle]
pub extern "C" fn on_init() {
    let sub_id = unsafe { host_subscribe_event(EVENT_MACHINE_COMPLETE) };
    if sub_id >= 0 {
        log_info(&format!("Subscribed with ID: {}", sub_id));
    }
}

#[no_mangle]
pub extern "C" fn on_event(event_type: u32, data: u64) {
    if event_type == EVENT_MACHINE_COMPLETE {
        let entity_id = data;
        log_info(&format!("Machine {} completed", entity_id));
    }
}
```

---

## Machines

### host_get_machine_state

Get the current state of a machine.

```rust
extern "C" {
    fn host_get_machine_state(entity_id: u64) -> i32;
}
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `entity_id` | `u64` | Machine entity ID |

**Returns:**
| Value | State | Description |
|-------|-------|-------------|
| `0` | Idle | Not processing |
| `1` | Processing | Working on recipe |
| `2` | WaitingInput | Needs items/fuel |
| `-1` | Error | Entity not found |

**Example:**
```rust
enum MachineState {
    Idle,
    Processing,
    WaitingInput,
    Unknown,
}

fn get_state(entity_id: u64) -> MachineState {
    match unsafe { host_get_machine_state(entity_id) } {
        0 => MachineState::Idle,
        1 => MachineState::Processing,
        2 => MachineState::WaitingInput,
        _ => MachineState::Unknown,
    }
}
```

---

### host_set_machine_enabled

Enable or disable a machine.

```rust
extern "C" {
    fn host_set_machine_enabled(entity_id: u64, enabled: i32) -> i32;
}
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `entity_id` | `u64` | Machine entity ID |
| `enabled` | `i32` | `1` = enable, `0` = disable |

**Returns:**
| Value | Meaning |
|-------|---------|
| `0` | Success |
| `-1` | Entity not found |
| `-2` | Permission denied |

**Example:**
```rust
fn pause_machine(entity_id: u64) -> bool {
    unsafe { host_set_machine_enabled(entity_id, 0) == 0 }
}

fn resume_machine(entity_id: u64) -> bool {
    unsafe { host_set_machine_enabled(entity_id, 1) == 0 }
}
```

---

## Inventory

### host_get_inventory_slot

Get contents of an inventory slot.

```rust
extern "C" {
    fn host_get_inventory_slot(entity_id: u64, slot: u32) -> u64;
}
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `entity_id` | `u64` | Entity with inventory |
| `slot` | `u32` | Slot index (0-based) |

**Returns:**
- Upper 32 bits: `item_id` (0 = empty)
- Lower 32 bits: `count`

**Example:**
```rust
fn get_slot(entity_id: u64, slot: u32) -> (u32, u32) {
    let result = unsafe { host_get_inventory_slot(entity_id, slot) };
    let item_id = (result >> 32) as u32;
    let count = (result & 0xFFFFFFFF) as u32;
    (item_id, count)
}

// Check if slot has items
let (item_id, count) = get_slot(machine_id, 0);
if item_id > 0 {
    log_info(&format!("Slot 0: {} x {}", item_id, count));
}
```

---

## Lifecycle Callbacks

Your mod must export these functions:

```rust
/// Called when mod loads
#[no_mangle]
pub extern "C" fn on_init() {
    // Initialize state, subscribe to events
}

/// Called every game tick (20Hz)
#[no_mangle]
pub extern "C" fn on_tick() {
    // Update logic (keep fast!)
}

/// Called when subscribed event fires
#[no_mangle]
pub extern "C" fn on_event(event_type: u32, data: u64) {
    // Handle event
}

/// Called when mod unloads
#[no_mangle]
pub extern "C" fn on_shutdown() {
    // Cleanup
}
```

---

## Error Handling

All host functions that can fail return negative values on error.

```rust
fn safe_get_state(entity_id: u64) -> Option<MachineState> {
    let result = unsafe { host_get_machine_state(entity_id) };
    if result < 0 {
        log_error(&format!("Failed to get state for {}", entity_id));
        return None;
    }
    Some(match result {
        0 => MachineState::Idle,
        1 => MachineState::Processing,
        2 => MachineState::WaitingInput,
        _ => return None,
    })
}
```

---

## See Also

- [Core Mod Guide](Core-Mod-Guide) - Complete WASM tutorial
- [Getting Started](Getting-Started) - Data Mod basics
