//! Block placement and breaking systems
//!
//! This module contains the core block interaction systems:
//! - block_break: Breaking world blocks and machines
//! - block_place: Placing blocks and machines

mod breaking;
mod placement;

pub use breaking::block_break;
pub use placement::block_place;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::components::Machine;
use crate::player::{LocalPlayer, PlayerInventory};
use crate::{Conveyor, DeliveryPlatform};

/// Bundled local player inventory access (reduces parameter count)
#[derive(SystemParam)]
pub struct LocalPlayerInventory<'w, 's> {
    local_player: Option<Res<'w, LocalPlayer>>,
    inventories: Query<'w, 's, &'static mut PlayerInventory>,
}

impl LocalPlayerInventory<'_, '_> {
    /// Get mutable access to the local player's inventory
    pub fn get_mut(&mut self) -> Option<Mut<'_, PlayerInventory>> {
        let local_player = self.local_player.as_ref()?;
        self.inventories.get_mut(local_player.0).ok()
    }

    /// Get read-only access to the local player's inventory
    pub fn get(&self) -> Option<&PlayerInventory> {
        let local_player = self.local_player.as_ref()?;
        self.inventories.get(local_player.0).ok()
    }

    /// Get the local player's entity
    pub fn entity(&self) -> Option<Entity> {
        self.local_player.as_ref().map(|lp| lp.0)
    }
}

/// Bundled machine queries for block_break system (reduces parameter count)
#[derive(SystemParam)]
pub struct MachineBreakQueries<'w, 's> {
    pub conveyor: Query<'w, 's, (Entity, &'static Conveyor, &'static GlobalTransform)>,
    pub machine: Query<'w, 's, (Entity, &'static Machine, &'static GlobalTransform)>,
    pub platform: Query<'w, 's, &'static Transform, With<DeliveryPlatform>>,
}

/// Bundled machine queries for block_place system (reduces parameter count)
#[derive(SystemParam)]
pub struct MachinePlaceQueries<'w, 's> {
    pub conveyor: Query<'w, 's, &'static Conveyor>,
    pub machine: Query<'w, 's, (&'static Machine, &'static Transform)>,
}

/// Bundled chunk render assets (reduces parameter count)
#[derive(SystemParam)]
pub struct ChunkAssets<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
}

/// Bundled block break events (reduces parameter count)
#[derive(SystemParam)]
pub struct BlockBreakEvents<'w> {
    pub tutorial: EventWriter<'w, crate::systems::TutorialEvent>,
    pub block_broken: EventWriter<'w, crate::events::game_events::BlockBroken>,
}

/// Bundled block place events (reduces parameter count)
#[derive(SystemParam)]
pub struct BlockPlaceEvents<'w> {
    pub tutorial: EventWriter<'w, crate::systems::TutorialEvent>,
    pub block_placed: EventWriter<'w, crate::events::game_events::BlockPlaced>,
    pub machine_spawned: EventWriter<'w, crate::events::game_events::MachineSpawned>,
}
