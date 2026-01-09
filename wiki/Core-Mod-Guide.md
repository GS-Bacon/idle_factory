# Core Mod Guide / Core Modガイド

Create advanced mods using Rust compiled to WASM.
RustをWASMにコンパイルして高度なModを作成。

---

## Overview / 概要

Core Mods are WebAssembly modules that:
Core ModはWebAssemblyモジュールで:

- Run inside the game process / ゲームプロセス内で実行
- Access game state via host functions / ホスト関数でゲーム状態にアクセス
- Subscribe to game events / ゲームイベントを購読
- Modify game behavior / ゲーム動作を変更

**Use cases / ユースケース:**
- Custom machine logic / カスタム機械ロジック
- Event-driven automation / イベント駆動の自動化
- Performance-critical mods / パフォーマンス重視のMod

---

## Prerequisites / 前提条件

- Rust toolchain with `wasm32-unknown-unknown` target
- Basic understanding of Rust

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown
```

---

## Quick Start / クイックスタート

### 1. Create project structure / プロジェクト構造を作成

```
mods/my_core_mod/
├── mod.toml
├── src/
│   └── lib.rs
├── Cargo.toml
└── scripts/
    └── main.wasm    ← Built output
```

### 2. Create mod.toml

```toml
[mod]
id = "my_core_mod"
name = "My Core Mod"
version = "1.0.0"
author = "Your Name"
description = "A custom WASM mod"
type = "core"

[mod.dependencies]
base = ">=0.3.0"
```

### 3. Create Cargo.toml

```toml
[package]
name = "my_core_mod"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# No dependencies required for basic mods
```

### 4. Create src/lib.rs

```rust
//! My Core Mod

// Host function declarations
extern "C" {
    fn host_log_info(ptr: *const u8, len: usize);
    fn host_log_error(ptr: *const u8, len: usize);
    fn host_subscribe_event(event_type: u32) -> i32;
    fn host_get_machine_state(entity_id: u64) -> i32;
    fn host_set_machine_enabled(entity_id: u64, enabled: i32) -> i32;
    fn host_get_inventory_slot(entity_id: u64, slot: u32) -> u64;
}

// Helper: Log info message
fn log_info(msg: &str) {
    unsafe {
        host_log_info(msg.as_ptr(), msg.len());
    }
}

// Helper: Log error message
fn log_error(msg: &str) {
    unsafe {
        host_log_error(msg.as_ptr(), msg.len());
    }
}

/// Called when mod is loaded
#[no_mangle]
pub extern "C" fn on_init() {
    log_info("My Core Mod initialized!");
}

/// Called every game tick (20Hz)
#[no_mangle]
pub extern "C" fn on_tick() {
    // Called every tick, keep this fast!
}

/// Called when mod is unloaded
#[no_mangle]
pub extern "C" fn on_shutdown() {
    log_info("My Core Mod shutting down");
}
```

### 5. Build / ビルド

```bash
cd mods/my_core_mod
cargo build --target wasm32-unknown-unknown --release

# Copy to scripts folder
mkdir -p scripts
cp target/wasm32-unknown-unknown/release/my_core_mod.wasm scripts/main.wasm
```

### 6. Test / テスト

Launch the game. Check `logs/game.log` for:
ゲームを起動。`logs/game.log` で確認:

```
[CoreMod] Loading: my_core_mod
[CoreMod] my_core_mod: My Core Mod initialized!
```

---

## Host Functions / ホスト関数

### Logging / ログ出力

```rust
extern "C" {
    /// Log info message
    fn host_log_info(ptr: *const u8, len: usize);

    /// Log error message
    fn host_log_error(ptr: *const u8, len: usize);
}
```

### Events / イベント

```rust
extern "C" {
    /// Subscribe to event type
    /// Returns: subscription ID (>= 0) or error (< 0)
    fn host_subscribe_event(event_type: u32) -> i32;
}

// Event types
const EVENT_MACHINE_COMPLETE: u32 = 1;
const EVENT_ITEM_CRAFTED: u32 = 2;
const EVENT_BLOCK_PLACED: u32 = 3;
const EVENT_BLOCK_REMOVED: u32 = 4;
```

### Machine Control / 機械制御

```rust
extern "C" {
    /// Get machine state
    /// Returns: 0=Idle, 1=Processing, 2=WaitingInput, -1=Error
    fn host_get_machine_state(entity_id: u64) -> i32;

    /// Enable/disable machine
    /// Returns: 0=Success, -1=NotFound, -2=PermissionDenied
    fn host_set_machine_enabled(entity_id: u64, enabled: i32) -> i32;
}
```

### Inventory / インベントリ

```rust
extern "C" {
    /// Get inventory slot contents
    /// Returns: upper 32 bits = item_id, lower 32 bits = count
    fn host_get_inventory_slot(entity_id: u64, slot: u32) -> u64;
}

fn get_slot_contents(entity_id: u64, slot: u32) -> (u32, u32) {
    let result = unsafe { host_get_inventory_slot(entity_id, slot) };
    let item_id = (result >> 32) as u32;
    let count = result as u32;
    (item_id, count)
}
```

---

## Lifecycle / ライフサイクル

```rust
/// Called once when mod loads
#[no_mangle]
pub extern "C" fn on_init() {
    // Initialize state, subscribe to events
}

/// Called every game tick (50ms / 20Hz)
#[no_mangle]
pub extern "C" fn on_tick() {
    // Update logic (keep fast!)
}

/// Called when mod unloads
#[no_mangle]
pub extern "C" fn on_shutdown() {
    // Cleanup
}

/// Called when subscribed event fires
#[no_mangle]
pub extern "C" fn on_event(event_type: u32, data: u64) {
    // Handle event
}
```

---

## Example: Auto-Pause / 例: 自動一時停止

Pause machines when output is full:
出力が満杯になったら機械を一時停止:

```rust
static mut MONITORED_MACHINES: Vec<u64> = Vec::new();

#[no_mangle]
pub extern "C" fn on_init() {
    log_info("Auto-Pause mod loaded");
}

#[no_mangle]
pub extern "C" fn on_tick() {
    unsafe {
        for &entity_id in &MONITORED_MACHINES {
            let (item_id, count) = get_slot_contents(entity_id, 0);

            // If output slot has 64+ items, pause
            if count >= 64 {
                host_set_machine_enabled(entity_id, 0);
            } else {
                host_set_machine_enabled(entity_id, 1);
            }
        }
    }
}

/// Register a machine for monitoring
#[no_mangle]
pub extern "C" fn register_machine(entity_id: u64) {
    unsafe {
        if !MONITORED_MACHINES.contains(&entity_id) {
            MONITORED_MACHINES.push(entity_id);
        }
    }
}
```

---

## Best Practices / ベストプラクティス

### 1. Keep on_tick() fast / on_tick()を軽く

```rust
// Bad: Heavy computation every tick
#[no_mangle]
pub extern "C" fn on_tick() {
    for entity in all_entities() {  // Thousands of entities!
        complex_calculation(entity);
    }
}

// Good: Only process when needed
static mut TICK_COUNTER: u32 = 0;

#[no_mangle]
pub extern "C" fn on_tick() {
    unsafe {
        TICK_COUNTER += 1;
        if TICK_COUNTER % 20 == 0 {  // Every second
            do_periodic_work();
        }
    }
}
```

### 2. Use events instead of polling / ポーリングよりイベント

```rust
// Bad: Check every tick
#[no_mangle]
pub extern "C" fn on_tick() {
    if machine_completed() {
        handle_completion();
    }
}

// Good: Subscribe to events
#[no_mangle]
pub extern "C" fn on_init() {
    unsafe { host_subscribe_event(EVENT_MACHINE_COMPLETE); }
}

#[no_mangle]
pub extern "C" fn on_event(event_type: u32, data: u64) {
    if event_type == EVENT_MACHINE_COMPLETE {
        handle_completion(data);
    }
}
```

### 3. Handle errors gracefully / エラーを適切に処理

```rust
fn get_machine_state_safe(entity_id: u64) -> Option<MachineState> {
    let result = unsafe { host_get_machine_state(entity_id) };
    match result {
        0 => Some(MachineState::Idle),
        1 => Some(MachineState::Processing),
        2 => Some(MachineState::WaitingInput),
        _ => None,  // Error or unknown
    }
}
```

---

## Debugging / デバッグ

### Use log functions / ログ関数を使用

```rust
fn debug_machine(entity_id: u64) {
    let state = unsafe { host_get_machine_state(entity_id) };
    log_info(&format!("Machine {} state: {}", entity_id, state));
}
```

### Check game logs / ゲームログ確認

```
logs/game.log
```

Core Mod output is prefixed with `[CoreMod]`.
Core Mod出力は `[CoreMod]` プレフィックス付き。

---

## Limitations / 制限

| Limitation | Reason |
|------------|--------|
| No file I/O | Security sandbox / セキュリティ |
| No networking | Security sandbox / セキュリティ |
| Limited memory | 16MB default / デフォルト16MB |
| No threading | WASM single-threaded / シングルスレッド |

---

## See Also / 関連

- [WASM Host Functions](WASM-Host-Functions) - Full function reference / 関数リファレンス
- [Mod Structure](Mod-Structure) - mod.toml format / mod.toml形式
- [Getting Started](Getting-Started) - Data Mod basics / Data Modの基本
