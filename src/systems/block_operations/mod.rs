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

use crate::{Conveyor, Crusher, DeliveryPlatform, Furnace, Miner};

/// Bundled machine queries for block_break system (reduces parameter count)
#[derive(SystemParam)]
pub struct MachineBreakQueries<'w, 's> {
    pub conveyor: Query<'w, 's, (Entity, &'static Conveyor, &'static GlobalTransform)>,
    pub miner: Query<'w, 's, (Entity, &'static Miner, &'static GlobalTransform)>,
    pub crusher: Query<'w, 's, (Entity, &'static Crusher, &'static GlobalTransform)>,
    pub furnace: Query<'w, 's, (Entity, &'static Furnace, &'static GlobalTransform)>,
    pub platform: Query<'w, 's, &'static Transform, With<DeliveryPlatform>>,
}

/// Bundled machine queries for block_place system (reduces parameter count)
#[derive(SystemParam)]
pub struct MachinePlaceQueries<'w, 's> {
    pub conveyor: Query<'w, 's, &'static Conveyor>,
    pub miner: Query<'w, 's, &'static Miner>,
    pub crusher: Query<'w, 's, (&'static Crusher, &'static Transform)>,
    pub furnace: Query<'w, 's, &'static Transform, With<Furnace>>,
}

/// Bundled chunk render assets (reduces parameter count)
#[derive(SystemParam)]
pub struct ChunkAssets<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
}
