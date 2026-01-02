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
    handle_assert_machine_event, handle_debug_event, handle_look_event, handle_screenshot_event,
    handle_setblock_event, handle_spawn_machine_event, handle_teleport_event,
};
pub use ui::{command_input_handler, command_input_toggle};

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

/// Debug event types
#[derive(Clone, Copy, Debug)]
pub enum DebugEventType {
    /// Dump all conveyor states
    Conveyor,
    /// Dump all machine states
    Machine,
    /// Show machine input/output port connections
    Connection,
}

/// Debug event (for /debug_* commands)
#[derive(Event)]
pub struct DebugEvent {
    pub debug_type: DebugEventType,
}

// Legacy aliases for backward compatibility
pub type DebugConveyorEvent = DebugEvent;
pub type DebugMachineEvent = DebugEvent;
pub type DebugConnectionEvent = DebugEvent;

/// Machine assertion types
#[derive(Clone, Copy, Debug)]
pub enum MachineAssertType {
    /// Check if any miner is actively mining (progress > 0)
    MinerWorking,
    /// Check if any conveyor has items
    ConveyorHasItems,
    /// Check total count of a specific machine type
    MachineCount { machine: BlockType, min_count: u32 },
}

/// Assert machine event for E2E testing
#[derive(Event)]
pub struct AssertMachineEvent {
    pub assert_type: MachineAssertType,
}

/// Screenshot event for capturing game screen
#[derive(Event)]
pub struct ScreenshotEvent {
    pub filename: String,
}
