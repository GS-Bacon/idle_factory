//! Inventory UI systems
//!
//! This module handles all inventory UI interactions including:
//! - Visibility toggling
//! - Slot interactions (click, shift-click)
//! - Slot display updates
//! - Tooltip display
//! - Breaking progress bar
//! - Upper panel (creative/platform inventory)

mod breaking_bar;
mod slot_display;
mod slot_interaction;
mod tooltip;
mod upper_panel;
mod visibility;

// Re-export public systems
pub use breaking_bar::{spawn_breaking_progress_ui, update_breaking_progress_ui};
pub use slot_display::{inventory_update_slots, update_held_item_display};
pub use slot_interaction::{
    creative_inventory_click, inventory_continuous_shift_click, inventory_slot_click,
    trash_slot_click,
};
pub use tooltip::update_inventory_tooltip;
pub use upper_panel::{
    update_creative_catalog_sprites, update_upper_panel_slots, upper_panel_category_click,
    upper_panel_page_nav, upper_panel_slot_click,
};
pub use visibility::{set_ui_open_state, update_inventory_visibility};
