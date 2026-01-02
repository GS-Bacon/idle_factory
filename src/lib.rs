//! Idle Factory - Game Library
//!
//! This library exposes the core game types and systems for use in tests and the main binary.

pub mod block_type;
pub mod components;
pub mod constants;
pub mod events;
pub mod game_spec;
pub mod logging;
pub mod meshes;
pub mod player;
pub mod plugins;
pub mod save;
pub mod setup;
pub mod systems;
pub mod ui;
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
    GameEventsPlugin, SpawnMachineEvent,
    BlockPlaceEvent, BlockBreakEvent, QuestProgressEvent,
    ItemTransferEvent, MachineInteractEvent, MachineAction,
};

// Re-export systems for command/test use
pub use systems::{
    TeleportEvent, LookEvent, SetBlockEvent, DebugConveyorEvent,
    set_ui_open_state,
};

// Re-export utility functions
pub use utils::{ray_aabb_intersection, parse_item_name};

// Re-export plugins for testing
pub use plugins::{DebugPlugin, MachineSystemsPlugin, SavePlugin, UIPlugin};
pub use vox_loader::VoxLoaderPlugin;
