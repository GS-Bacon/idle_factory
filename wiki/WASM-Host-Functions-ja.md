# WASMホスト関数リファレンス

Core Mod (WASM) で使用可能なホスト関数

## 関数一覧

| 関数 | パラメータ | 戻り値 | 説明 |
|------|------------|--------|------|
| `host_get_inventory_slot` | `_caller: Caller<'_, ModState>, entity_id: u64, slot: u32` | Upper 32 bits: item_id, Lower 32 bits: count (0 = empty slot) | インベントリスロットの内容を取得 |
| `host_get_machine_state` | `_caller: Caller<'_, ModState>, entity_id: u64` | - 0: Idle | Entity IDで機械の状態を取得 |
| `host_log_error` | `mut caller: Caller<'_, ModState>, ptr: u32, len: u32` | Nothing (void function) | WASM Modからのエラーログを出力 |
| `host_log_info` | `mut caller: Caller<'_, ModState>, ptr: u32, len: u32` | Nothing (void function) | WASM Modからの情報ログを出力 |
| `host_set_machine_enabled` | `_caller: Caller<'_, ModState>, entity_id: u64, enabled: i32` | - 0: Success | 機械の有効/無効を設定 |
| `host_subscribe_event` | `_caller: Caller<'_, ModState>, event_type: u32` | Subscription ID (>= 0 on success, negative on error) | ゲームイベントを購読 |

## 詳細

### `host_get_inventory_slot`

インベントリスロットの内容を取得

**パラメータ:**
- `_caller: Caller<'_`
- `ModState>`
- `entity_id: u64`
- `slot: u32`

**戻り値:**
Upper 32 bits: item_id, Lower 32 bits: count (0 = empty slot)

### `host_get_machine_state`

Entity IDで機械の状態を取得

**パラメータ:**
- `_caller: Caller<'_`
- `ModState>`
- `entity_id: u64`

**戻り値:**
- 0: Idle
- 1: Processing
- 2: Waiting for input
- -1: Error (entity not found)

### `host_log_error`

WASM Modからのエラーログを出力

**パラメータ:**
- `mut caller: Caller<'_`
- `ModState>`
- `ptr: u32`
- `len: u32`

**戻り値:**
Nothing (void function)

### `host_log_info`

WASM Modからの情報ログを出力

**パラメータ:**
- `mut caller: Caller<'_`
- `ModState>`
- `ptr: u32`
- `len: u32`

**戻り値:**
Nothing (void function)

### `host_set_machine_enabled`

機械の有効/無効を設定

**パラメータ:**
- `_caller: Caller<'_`
- `ModState>`
- `entity_id: u64`
- `enabled: i32`

**戻り値:**
- 0: Success
- -1: Entity not found
- -2: Permission denied

### `host_subscribe_event`

ゲームイベントを購読

**パラメータ:**
- `_caller: Caller<'_`
- `ModState>`
- `event_type: u32`

**戻り値:**
Subscription ID (>= 0 on success, negative on error)
