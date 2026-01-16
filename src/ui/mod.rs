//! UI components and systems
//!
//! This module contains UI definitions and logic.

pub mod machine_ui;
pub mod widgets;

pub use widgets::{
    spawn_button, spawn_slot, spawn_slot_row, ButtonConfig, ButtonWidget, SlotConfig,
    SlotCountText, SlotItemImage, SlotWidget,
};

pub use machine_ui::setup_generic_machine_ui;
