//! Game plugins
//!
//! Plugins organize systems, resources, and events into logical groups.

mod debug;
mod game;
mod machines;
mod save;
mod ui;

pub use debug::DebugPlugin;
pub use game::GamePlugin;
pub use machines::MachineSystemsPlugin;
pub use save::SavePlugin;
pub use ui::UIPlugin;
