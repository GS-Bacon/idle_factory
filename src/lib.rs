//! Idle Factory - Game Library
//!
//! This library exposes the core game types and systems for use in tests and the main binary.

pub mod achievements;
pub mod audio;
pub mod block_type;
pub mod blockbench;
pub mod components;
pub mod constants;
pub mod core;
pub mod debug;
pub mod events;
pub mod game_data;
pub mod game_spec;
pub mod logging;
pub mod logistics;
pub mod machines;
pub mod meshes;
pub mod player;
pub mod plugins;
pub mod rng;
pub mod save;
pub mod settings;
pub mod setup;
pub mod systems;
pub mod ui;
#[cfg(feature = "updater")]
pub mod updater;
pub mod utils;
pub mod vox_loader;
pub mod world;

// Re-export commonly used types at crate root
pub use block_type::{BlockCategory, BlockType};
pub use components::*;
pub use constants::*;

// Re-export core types (dynamic ID system)
pub use core::{FluidId, ItemId, MachineId, RecipeId, SharedInterner, StringInterner};

// Re-export world types
pub use world::{BiomeMap, WorldData};

// Re-export events
pub use events::{
    BlockBreakEvent, BlockPlaceEvent, EventDepth, EventError, EventSystemConfig, EventsPlugin,
    GameEventsPlugin, GuardedEventWriter, ItemTransferEvent, MachineAction, MachineInteractEvent,
    QuestProgressEvent, SpawnMachineEvent,
};

// Re-export systems for command/test use
pub use systems::{set_ui_open_state, DebugConveyorEvent, LookEvent, SetBlockEvent, TeleportEvent};

// Re-export utility functions
pub use utils::{
    grid_to_world, grid_to_world_center, parse_item_name, ray_aabb_intersection, world_to_grid,
    GridPos, WorldPos,
};

// Re-export achievements
pub use achievements::{AchievementUnlocked, AchievementsPlugin, PlayerAchievements};

// Re-export plugins for testing
pub use audio::{AudioPlugin, SoundCategory, SoundSettings};
pub use blockbench::BlockbenchPlugin;
pub use plugins::{DebugPlugin, MachineSystemsPlugin, SavePlugin, UIPlugin};
pub use vox_loader::VoxLoaderPlugin;

// Re-export updater plugin
#[cfg(feature = "updater")]
pub use updater::UpdaterPlugin;
