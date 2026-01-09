#![no_std]

#[cfg(feature = "panic_handler")]
use core::panic::PanicInfo;

#[cfg(feature = "panic_handler")]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// ホスト関数のextern宣言
extern "C" {
    pub fn host_log_info(ptr: *const u8, len: u32);
    pub fn host_log_error(ptr: *const u8, len: u32);
    pub fn host_get_machine_state(entity_id: u64) -> i32;
    pub fn host_set_machine_enabled(entity_id: u64, enabled: i32) -> i32;
    pub fn host_get_inventory_slot(entity_id: u64, slot: u32) -> u64; // item_id << 32 | count
    pub fn host_transfer_item(from_entity: u64, to_entity: u64, item_id: u32, count: u32) -> i32;
}

/// ログ出力（info）
pub fn log(msg: &str) {
    unsafe {
        host_log_info(msg.as_ptr(), msg.len() as u32);
    }
}

/// ログ出力（error）
pub fn log_error(msg: &str) {
    unsafe {
        host_log_error(msg.as_ptr(), msg.len() as u32);
    }
}

/// 機械の状態を取得
pub fn get_machine_state(entity_id: u64) -> i32 {
    unsafe { host_get_machine_state(entity_id) }
}

/// 機械の有効/無効を設定
pub fn set_machine_enabled(entity_id: u64, enabled: bool) -> i32 {
    unsafe { host_set_machine_enabled(entity_id, if enabled { 1 } else { 0 }) }
}

/// インベントリスロットを取得（item_id, count）
pub fn get_inventory_slot(entity_id: u64, slot: u32) -> (u32, u32) {
    let result = unsafe { host_get_inventory_slot(entity_id, slot) };
    ((result >> 32) as u32, result as u32)
}

/// アイテムを転送
pub fn transfer_item(from_entity: u64, to_entity: u64, item_id: u32, count: u32) -> i32 {
    unsafe { host_transfer_item(from_entity, to_entity, item_id, count) }
}
