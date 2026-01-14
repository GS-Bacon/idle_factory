//! Game systems organized by domain
//!
//! This module contains all Bevy system functions, organized by functionality.

pub mod block_operations;
pub mod chunk;
pub mod command;
pub mod cursor;
pub mod debug_ui;
pub mod hotbar;
pub mod invariants;
pub mod inventory_ui;
pub mod player;
pub mod quest;
pub mod targeting;
pub mod tutorial;
pub mod ui_navigation;
pub mod ui_visibility;

pub use block_operations::*;
pub use chunk::*;
pub use command::*;
pub use cursor::*;
pub use debug_ui::*;
pub use hotbar::*;
pub use invariants::*;
pub use inventory_ui::*;
pub use player::*;
pub use quest::*;
pub use targeting::*;
pub use tutorial::*;
pub use ui_navigation::*;
pub use ui_visibility::*;

// Re-export conveyor systems from logistics module
pub use crate::logistics::conveyor::*;

// Re-export save systems from save module
pub use crate::save::systems::*;
