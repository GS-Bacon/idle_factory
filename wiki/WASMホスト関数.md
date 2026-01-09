# WASMホスト関数リファレンス

[English version](WASM-Host-Functions)

Core Mod (WASM) で使用可能なホスト関数。

---

## 概要

ホスト関数はWASM Modとゲームの橋渡しです。

```rust
// ホスト関数の宣言
extern "C" {
    fn host_log_info(ptr: *const u8, len: usize);
    fn host_get_machine_state(entity_id: u64) -> i32;
    // ...
}

// 使用方法
unsafe {
    let msg = "Hello from WASM!";
    host_log_info(msg.as_ptr(), msg.len());
}
```

---

## 関数一覧

| 関数 | 説明 |
|------|------|
| [`host_log_info`](#host_log_info) | 情報ログ出力 |
| [`host_log_error`](#host_log_error) | エラーログ出力 |
| [`host_subscribe_event`](#host_subscribe_event) | イベント購読 |
| [`host_get_machine_state`](#host_get_machine_state) | 機械状態取得 |
| [`host_set_machine_enabled`](#host_set_machine_enabled) | 機械有効化/無効化 |
| [`host_get_inventory_slot`](#host_get_inventory_slot) | インベントリスロット取得 |

---

## ログ出力

### host_log_info

情報メッセージをログ出力します。`logs/game.log` に `[CoreMod]` プレフィックス付きで出力されます。

```rust
extern "C" {
    fn host_log_info(ptr: *const u8, len: usize);
}
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `ptr` | `*const u8` | UTF-8文字列へのポインタ |
| `len` | `usize` | バイト長 |

**戻り値:** なし (void)

**例:**
```rust
fn log_info(msg: &str) {
    unsafe { host_log_info(msg.as_ptr(), msg.len()); }
}

log_info("Machine processed item");
```

---

### host_log_error

エラーメッセージをログ出力します。

```rust
extern "C" {
    fn host_log_error(ptr: *const u8, len: usize);
}
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `ptr` | `*const u8` | UTF-8文字列へのポインタ |
| `len` | `usize` | バイト長 |

**戻り値:** なし (void)

**例:**
```rust
fn log_error(msg: &str) {
    unsafe { host_log_error(msg.as_ptr(), msg.len()); }
}

log_error("Failed to find machine");
```

---

## イベント

### host_subscribe_event

ゲームイベントを購読します。イベント発火時に `on_event()` が呼ばれます。

```rust
extern "C" {
    fn host_subscribe_event(event_type: u32) -> i32;
}
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `event_type` | `u32` | イベントタイプID（下表参照） |

**戻り値:**
| 値 | 意味 |
|----|------|
| `>= 0` | サブスクリプションID（成功） |
| `< 0` | エラー |

**イベントタイプ:**
| ID | 定数 | 説明 |
|----|------|------|
| `1` | `EVENT_MACHINE_COMPLETE` | 機械処理完了 |
| `2` | `EVENT_ITEM_CRAFTED` | アイテム作成 |
| `3` | `EVENT_BLOCK_PLACED` | ブロック設置 |
| `4` | `EVENT_BLOCK_REMOVED` | ブロック削除 |

**例:**
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

## 機械

### host_get_machine_state

機械の現在の状態を取得します。

```rust
extern "C" {
    fn host_get_machine_state(entity_id: u64) -> i32;
}
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `entity_id` | `u64` | 機械エンティティID |

**戻り値:**
| 値 | 状態 | 説明 |
|----|------|------|
| `0` | Idle | 処理中でない |
| `1` | Processing | レシピ処理中 |
| `2` | WaitingInput | アイテム・燃料待ち |
| `-1` | Error | エンティティ未発見 |

**例:**
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

機械を有効化または無効化します。

```rust
extern "C" {
    fn host_set_machine_enabled(entity_id: u64, enabled: i32) -> i32;
}
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `entity_id` | `u64` | 機械エンティティID |
| `enabled` | `i32` | `1` = 有効化, `0` = 無効化 |

**戻り値:**
| 値 | 意味 |
|----|------|
| `0` | 成功 |
| `-1` | エンティティ未発見 |
| `-2` | 権限なし |

**例:**
```rust
fn pause_machine(entity_id: u64) -> bool {
    unsafe { host_set_machine_enabled(entity_id, 0) == 0 }
}

fn resume_machine(entity_id: u64) -> bool {
    unsafe { host_set_machine_enabled(entity_id, 1) == 0 }
}
```

---

## インベントリ

### host_get_inventory_slot

インベントリスロットの内容を取得します。

```rust
extern "C" {
    fn host_get_inventory_slot(entity_id: u64, slot: u32) -> u64;
}
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `entity_id` | `u64` | インベントリを持つエンティティ |
| `slot` | `u32` | スロットインデックス（0始まり） |

**戻り値:**
- 上位32ビット: `item_id`（0 = 空）
- 下位32ビット: `count`

**例:**
```rust
fn get_slot(entity_id: u64, slot: u32) -> (u32, u32) {
    let result = unsafe { host_get_inventory_slot(entity_id, slot) };
    let item_id = (result >> 32) as u32;
    let count = (result & 0xFFFFFFFF) as u32;
    (item_id, count)
}

// スロットにアイテムがあるか確認
let (item_id, count) = get_slot(machine_id, 0);
if item_id > 0 {
    log_info(&format!("Slot 0: {} x {}", item_id, count));
}
```

---

## ライフサイクルコールバック

Modはこれらの関数をエクスポートする必要があります:

```rust
/// Modロード時に呼ばれる
#[no_mangle]
pub extern "C" fn on_init() {
    // 状態の初期化、イベント購読
}

/// 毎ゲームティック（20Hz）に呼ばれる
#[no_mangle]
pub extern "C" fn on_tick() {
    // 更新ロジック（高速に保つこと！）
}

/// 購読したイベント発火時に呼ばれる
#[no_mangle]
pub extern "C" fn on_event(event_type: u32, data: u64) {
    // イベント処理
}

/// Modアンロード時に呼ばれる
#[no_mangle]
pub extern "C" fn on_shutdown() {
    // クリーンアップ
}
```

---

## エラーハンドリング

失敗可能なホスト関数はエラー時に負の値を返します。

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

## 関連項目

- [Core Modガイド](Core-Mod-Guide) - WASMチュートリアル
- [はじめに](Getting-Started) - Data Modの基本
