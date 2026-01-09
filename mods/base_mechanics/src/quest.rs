//! クエスト進行ロジック

use mod_sdk::log;

/// 現在のクエスト状態
#[derive(Clone, Copy)]
pub enum QuestState {
    NotStarted,
    InProgress,
    ReadyToComplete,
    Completed,
}

static mut CURRENT_QUEST_STATE: QuestState = QuestState::NotStarted;
static mut QUEST_TARGET_COUNT: u32 = 0;
static mut QUEST_CURRENT_COUNT: u32 = 0;
static mut QUEST_STATE_CHANGED: bool = false;

/// クエスト進行をチェック
pub fn check_quest_progress() {
    unsafe {
        match CURRENT_QUEST_STATE {
            QuestState::InProgress => {
                // 進行中のクエストの完了条件をチェック
                if QUEST_CURRENT_COUNT >= QUEST_TARGET_COUNT {
                    CURRENT_QUEST_STATE = QuestState::ReadyToComplete;
                    log("Quest ready to complete!");
                }
            }
            _ => {}
        }
    }
}

/// 新しいクエストを開始
#[allow(dead_code)]
pub fn start_quest(target_count: u32) {
    unsafe {
        QUEST_STATE_CHANGED = true;
        CURRENT_QUEST_STATE = QuestState::InProgress;
        QUEST_TARGET_COUNT = target_count;
        QUEST_CURRENT_COUNT = 0;
    }
    log("Quest started");
}

/// クエストにアイテムを追加
#[allow(dead_code)]
pub fn add_quest_item(count: u32) {
    unsafe {
        if matches!(CURRENT_QUEST_STATE, QuestState::InProgress) {
            QUEST_CURRENT_COUNT += count;
        }
    }
}

/// クエストを完了
#[allow(dead_code)]
pub fn complete_quest() -> bool {
    unsafe {
        if matches!(CURRENT_QUEST_STATE, QuestState::ReadyToComplete) {
            CURRENT_QUEST_STATE = QuestState::Completed;
            log("Quest completed!");
            true
        } else {
            false
        }
    }
}

/// クエストの進捗を取得
#[allow(dead_code)]
pub fn get_progress() -> (u32, u32) {
    unsafe { (QUEST_CURRENT_COUNT, QUEST_TARGET_COUNT) }
}

/// 状態変更があったか確認してリセット
#[allow(dead_code)]
pub fn poll_state_changed() -> bool {
    unsafe {
        let changed = QUEST_STATE_CHANGED;
        QUEST_STATE_CHANGED = false;
        changed
    }
}
