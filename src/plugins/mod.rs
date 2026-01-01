//! Game plugins
//!
//! Plugins organize systems, resources, and events into logical groups.

mod machines;
mod save;
mod ui;

pub use machines::MachineSystemsPlugin;
pub use save::SavePlugin;
pub use ui::UIPlugin;
