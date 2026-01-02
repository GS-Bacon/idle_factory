//! UI components and systems
//!
//! This module contains UI definitions and logic.

pub mod storage_ui;

pub use storage_ui::{
    global_inventory_category_click, global_inventory_page_nav, global_inventory_search_input,
    global_inventory_toggle, setup_global_inventory_ui, update_global_inventory_ui,
};
