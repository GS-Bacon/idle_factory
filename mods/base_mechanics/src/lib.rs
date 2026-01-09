#![no_std]

mod platform;
mod quest;

use mod_sdk::log;

/// Mod初期化
#[no_mangle]
pub extern "C" fn mod_init() -> i32 {
    log("Base Mechanics Mod initialized");
    log("- Delivery Platform logic enabled");
    log("- Quest progression logic enabled");
    0
}

/// 毎tick処理
#[no_mangle]
pub extern "C" fn mod_tick(tick: u64) {
    // 納品プラットフォームの更新（20tickごと=1秒）
    if tick % 20 == 0 {
        platform::update_platforms();
    }

    // クエスト進行チェック（100tickごと=5秒）
    if tick % 100 == 0 {
        quest::check_quest_progress();
    }
}

/// イベントハンドラ
#[no_mangle]
pub extern "C" fn mod_on_event(event_type: u32, _data_ptr: u32, _data_len: u32) {
    match event_type {
        2 => {
            // ItemDeliver イベント
            platform::on_item_delivered();
        }
        3 => {
            // MachineComplete イベント
            // 特に処理なし
        }
        _ => {}
    }
}

/// Mod終了
#[no_mangle]
pub extern "C" fn mod_cleanup() {
    log("Base Mechanics Mod cleanup");
}
