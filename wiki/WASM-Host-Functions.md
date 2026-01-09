# WASM Host Functions Reference / WASMホスト関数リファレンス

Host functions available to Core Mods (WASM).
Core Mod (WASM) で使用可能なホスト関数。

---

## Overview / 概要

Host functions are the bridge between your WASM mod and the game.
ホスト関数はWASM Modとゲームの橋渡し。

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

## Function Summary / 関数一覧

| Function | Description |
|----------|-------------|
| [`host_log_info`](#host_log_info) | Log info message / 情報ログ |
| [`host_log_error`](#host_log_error) | Log error message / エラーログ |
| [`host_subscribe_event`](#host_subscribe_event) | Subscribe to events / イベント購読 |
| [`host_get_machine_state`](#host_get_machine_state) | Get machine state / 機械状態取得 |
| [`host_set_machine_enabled`](#host_set_machine_enabled) | Enable/disable machine / 機械有効化/無効化 |
| [`host_get_inventory_slot`](#host_get_inventory_slot) | Get inventory slot / インベントリスロット取得 |

---

## Logging / ログ出力

### host_log_info

Log an info message. Appears in `logs/game.log` with `[CoreMod]` prefix.
情報メッセージをログ出力。`logs/game.log` に `[CoreMod]` プレフィックス付きで出力。

```rust
extern "C" {
    fn host_log_info(ptr: *const u8, len: usize);
}
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `ptr` | `*const u8` | Pointer to UTF-8 string / UTF-8文字列へのポインタ |
| `len` | `usize` | String length in bytes / バイト長 |

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
エラーメッセージをログ出力。

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

## Events / イベント

### host_subscribe_event

Subscribe to game events. When the event fires, `on_event()` is called.
ゲームイベントを購読。イベント発火時に `on_event()` が呼ばれる。

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
| `1` | `EVENT_MACHINE_COMPLETE` | Machine finished processing / 機械処理完了 |
| `2` | `EVENT_ITEM_CRAFTED` | Item was crafted / アイテム作成 |
| `3` | `EVENT_BLOCK_PLACED` | Block placed in world / ブロック設置 |
| `4` | `EVENT_BLOCK_REMOVED` | Block removed from world / ブロック削除 |

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

## Machines / 機械

### host_get_machine_state

Get the current state of a machine.
機械の現在の状態を取得。

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
| `0` | Idle | Not processing / 処理中でない |
| `1` | Processing | Working on recipe / レシピ処理中 |
| `2` | WaitingInput | Needs items/fuel / アイテム・燃料待ち |
| `-1` | Error | Entity not found / エンティティ未発見 |

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
機械を有効化または無効化。

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

## Inventory / インベントリ

### host_get_inventory_slot

Get contents of an inventory slot.
インベントリスロットの内容を取得。

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

## Lifecycle Callbacks / ライフサイクルコールバック

Your mod must export these functions:
Modはこれらの関数をエクスポートする必要がある:

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

## Error Handling / エラーハンドリング

All host functions that can fail return negative values on error.
失敗可能なホスト関数はエラー時に負の値を返す。

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

## See Also / 関連

- [Core Mod Guide](Core-Mod-Guide) - Complete WASM tutorial / WASMチュートリアル
- [Getting Started](Getting-Started) - Data Mod basics / Data Modの基本
