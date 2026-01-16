//! Generic machine systems (Phase C Data-Driven Design)
//!
//! These systems work with the generic `Machine` component,
//! using `MachineSpec` to determine behavior.

pub(crate) mod auto_generate;
mod cleanup;
mod interact;
mod output;
mod recipe;
mod tick;
mod ui;

// Re-export public systems
pub use cleanup::cleanup_invalid_interacting_machine;
pub use cleanup::machine_visual_feedback;
pub use interact::generic_machine_interact;
pub use tick::generic_machine_tick;
pub use ui::generic_machine_ui_input;
pub use ui::update_generic_machine_ui;

#[cfg(test)]
mod tests;
