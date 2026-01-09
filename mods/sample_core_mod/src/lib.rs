#![no_std]

use mod_sdk::*;

static mut TICK_COUNT: u64 = 0;

/// Mod初期化
#[no_mangle]
pub extern "C" fn mod_init() -> i32 {
    log("Hello from Sample Core Mod!");
    log("Mod initialized successfully");
    0 // 成功
}

/// 毎tick呼ばれる
#[no_mangle]
pub extern "C" fn mod_tick(tick: u64) {
    unsafe {
        TICK_COUNT = tick;

        // 20tickごとにログ出力（約1秒）
        if tick % 20 == 0 {
            // 注: no_std環境なのでformat!は使えない
            log("Tick milestone reached");
        }
    }
}

/// Mod終了時
#[no_mangle]
pub extern "C" fn mod_cleanup() {
    log("Sample Core Mod cleanup");
}
