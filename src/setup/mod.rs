//! Setup systems for initial game state
//!
//! Contains all one-time setup functions that run at game start.
//!
//! ## Modules
//! - `lighting`: Directional light and ambient light setup
//! - `player`: Player entity with camera
//! - `ui_setup`: All UI panel creation (hotbar, machine UIs, inventory, etc.)
//! - `initial_items`: Initial equipment and furnace placement

pub mod lighting;
pub mod player;
pub mod ui_setup;
pub mod initial_items;

pub use lighting::*;
pub use player::*;
pub use ui_setup::*;
pub use initial_items::*;
