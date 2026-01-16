//! MachineModels resource for loaded 3D model handles

use bevy::prelude::*;

use crate::core::{items, ItemId};

use super::ConveyorShape;

/// Resource to hold loaded 3D model handles for machines and conveyors
#[derive(Resource, Default)]
pub struct MachineModels {
    /// Conveyor models by shape (glTF scenes)
    pub conveyor_straight: Option<Handle<Scene>>,
    pub conveyor_corner_left: Option<Handle<Scene>>,
    pub conveyor_corner_right: Option<Handle<Scene>>,
    pub conveyor_t_junction: Option<Handle<Scene>>,
    pub conveyor_splitter: Option<Handle<Scene>>,
    /// Machine models (glTF scenes)
    pub miner: Option<Handle<Scene>>,
    pub furnace: Option<Handle<Scene>>,
    pub crusher: Option<Handle<Scene>>,
    /// Item models (for conveyor display)
    pub item_iron_ore: Option<Handle<Scene>>,
    pub item_copper_ore: Option<Handle<Scene>>,
    pub item_coal: Option<Handle<Scene>>,
    pub item_stone: Option<Handle<Scene>>,
    pub item_iron_ingot: Option<Handle<Scene>>,
    pub item_copper_ingot: Option<Handle<Scene>>,
    /// Whether models are loaded (fallback to procedural if not)
    pub loaded: bool,
    /// VOX mesh handles (direct mesh, for hot reload)
    pub vox_miner: Option<Handle<Mesh>>,
    pub vox_conveyor_straight: Option<Handle<Mesh>>,
    /// Generation counter for hot reload (increment to trigger respawn)
    pub vox_generation: u32,
}

impl MachineModels {
    /// Get conveyor model handle for a given shape
    pub fn get_conveyor_model(&self, shape: ConveyorShape) -> Option<Handle<Scene>> {
        match shape {
            ConveyorShape::Straight => self.conveyor_straight.clone(),
            // No swap - logic correctly identifies turn direction
            ConveyorShape::CornerLeft => self.conveyor_corner_left.clone(),
            ConveyorShape::CornerRight => self.conveyor_corner_right.clone(),
            ConveyorShape::TJunction => self.conveyor_t_junction.clone(),
            ConveyorShape::Splitter => self.conveyor_splitter.clone(),
        }
    }

    /// Get item model handle for a given ItemId
    pub fn get_item_model(&self, item_id: ItemId) -> Option<Handle<Scene>> {
        if item_id == items::iron_ore() {
            self.item_iron_ore.clone()
        } else if item_id == items::copper_ore() {
            self.item_copper_ore.clone()
        } else if item_id == items::coal() {
            self.item_coal.clone()
        } else if item_id == items::stone() {
            self.item_stone.clone()
        } else if item_id == items::iron_ingot() {
            self.item_iron_ingot.clone()
        } else if item_id == items::copper_ingot() {
            self.item_copper_ingot.clone()
        } else {
            None // Other item types don't have item models
        }
    }

    /// Get scene handle for held item display (machines)
    pub fn get_held_item_scene(&self, item_id: ItemId) -> Option<Handle<Scene>> {
        if item_id == items::miner_block() {
            self.miner.clone()
        } else if item_id == items::furnace_block() {
            self.furnace.clone()
        } else if item_id == items::crusher_block() {
            self.crusher.clone()
        } else if item_id == items::conveyor_block() {
            self.conveyor_straight.clone()
        } else {
            None
        }
    }
}
