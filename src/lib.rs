//! Idle Factory - Game Library
//!
//! This library exposes the core game types and systems for use in tests and the main binary.

pub mod block_type;
pub mod components;
pub mod constants;
pub mod debug;
pub mod events;
pub mod game_spec;
pub mod logging;
pub mod meshes;
pub mod player;
pub mod plugins;
pub mod rng;
pub mod save;
pub mod setup;
pub mod systems;
pub mod ui;
pub mod updater;
pub mod utils;
pub mod vox_loader;
pub mod world;

// Re-export commonly used types at crate root
pub use block_type::BlockType;
pub use components::*;
pub use constants::*;
pub use player::Inventory;

// Re-export world types
pub use world::{BiomeMap, WorldData};

// Re-export events
pub use events::{
    BlockBreakEvent, BlockPlaceEvent, GameEventsPlugin, ItemTransferEvent, MachineAction,
    MachineInteractEvent, QuestProgressEvent, SpawnMachineEvent,
};

// Re-export systems for command/test use
pub use systems::{set_ui_open_state, DebugConveyorEvent, LookEvent, SetBlockEvent, TeleportEvent};

// Re-export utility functions
pub use utils::{parse_item_name, ray_aabb_intersection};

// Re-export plugins for testing
pub use plugins::{DebugPlugin, MachineSystemsPlugin, SavePlugin, UIPlugin};
pub use vox_loader::VoxLoaderPlugin;

// Re-export updater plugin
pub use updater::UpdaterPlugin;
