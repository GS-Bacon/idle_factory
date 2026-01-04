//! UI components and systems
//!
//! This module contains UI definitions and logic.

pub mod machine_ui;
pub mod storage_ui;
pub mod widgets;

pub use storage_ui::{
    global_inventory_category_click, global_inventory_page_nav, global_inventory_search_input,
    global_inventory_toggle, setup_global_inventory_ui, update_global_inventory_ui,
};

pub use widgets::{
    spawn_button, spawn_slot, spawn_slot_row, ButtonConfig, ButtonWidget, SlotConfig,
    SlotCountText, SlotItemImage, SlotWidget,
};

pub use machine_ui::{setup_crusher_ui, setup_furnace_ui, setup_miner_ui};
