//! Command input system
//!
//! Provides in-game command input for debugging and E2E testing:
//! - UI layer: Text input box with T/slash key toggle
//! - Executor: Command parsing and event dispatch
//! - Handlers: Event processing (teleport, spawn, etc.)

mod executor;
mod handlers;
mod ui;

use crate::BlockType;
use bevy::prelude::*;

// Re-export public items
pub use handlers::{
    handle_teleport_event,
    handle_look_event,
    handle_setblock_event,
    handle_spawn_machine_event,
    handle_debug_conveyor_event,
};
pub use ui::{command_input_toggle, command_input_handler};

/// E2E test command events
#[derive(Event)]
pub struct TeleportEvent {
    pub position: Vec3,
}

#[derive(Event)]
pub struct LookEvent {
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Event)]
pub struct SetBlockEvent {
    pub position: IVec3,
    pub block_type: BlockType,
}

/// Debug conveyor event (for /debug_conveyor command)
#[derive(Event)]
pub struct DebugConveyorEvent;
