//! 納品プラットフォームロジック

use mod_sdk::log;

/// 納品プラットフォームの状態
static mut DELIVERED_ITEMS: u32 = 0;
static mut TOTAL_DELIVERED: u32 = 0;

/// プラットフォームの更新処理
pub fn update_platforms() {
    // TODO: 実際の実装では host_get_all_platforms() などを呼び出す
    // 現在はスタブ
}

/// アイテム納品時の処理
pub fn on_item_delivered() {
    unsafe {
        DELIVERED_ITEMS += 1;
        TOTAL_DELIVERED += 1;

        // 10個ごとにログ出力
        if TOTAL_DELIVERED % 10 == 0 {
            log("Milestone: Items delivered");
        }
    }
}

/// 今回のセッションで納品されたアイテム数
#[allow(dead_code)]
pub fn get_delivered_count() -> u32 {
    unsafe { DELIVERED_ITEMS }
}

/// 総納品数
#[allow(dead_code)]
pub fn get_total_delivered() -> u32 {
    unsafe { TOTAL_DELIVERED }
}

/// カウンターをリセット（新規クエスト開始時など）
#[allow(dead_code)]
pub fn reset_session_counter() {
    unsafe {
        DELIVERED_ITEMS = 0;
    }
}
