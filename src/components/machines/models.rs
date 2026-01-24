//! MachineModels resource for loaded 3D model handles

use bevy::prelude::*;

use crate::core::{items, ItemId};

use super::ConveyorShape;

/// Resource to hold loaded 3D model handles for machines and conveyors
#[derive(Resource, Default)]
pub struct MachineModels {
    /// Conveyor models by shape (glTF scenes, fallback)
    pub conveyor_straight: Option<Handle<Scene>>,
    pub conveyor_corner_left: Option<Handle<Scene>>,
    pub conveyor_corner_right: Option<Handle<Scene>>,
    pub conveyor_t_junction: Option<Handle<Scene>>,
    pub conveyor_splitter: Option<Handle<Scene>>,
    /// Machine models (glTF scenes, fallback)
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

    // === VOX mesh handles (priority over GLB) ===
    /// Machine VOX meshes
    pub vox_miner: Option<Handle<Mesh>>,
    pub vox_furnace: Option<Handle<Mesh>>,
    pub vox_crusher: Option<Handle<Mesh>>,
    pub vox_assembler: Option<Handle<Mesh>>,
    pub vox_generator: Option<Handle<Mesh>>,
    pub vox_inserter: Option<Handle<Mesh>>,
    pub vox_storage: Option<Handle<Mesh>>,
    pub vox_splitter_machine: Option<Handle<Mesh>>,
    /// Conveyor VOX meshes
    pub vox_conveyor_straight: Option<Handle<Mesh>>,
    pub vox_conveyor_corner_left: Option<Handle<Mesh>>,
    pub vox_conveyor_corner_right: Option<Handle<Mesh>>,
    pub vox_conveyor_t_junction: Option<Handle<Mesh>>,
    pub vox_conveyor_splitter: Option<Handle<Mesh>>,
    /// Shared material for VOX models (vertex color)
    pub vox_material: Option<Handle<StandardMaterial>>,
    /// Generation counter for hot reload (increment to trigger respawn)
    pub vox_generation: u32,
}

/// Model type returned - either VOX mesh or GLB scene
#[derive(Clone)]
pub enum ModelHandle {
    Vox(Handle<Mesh>),
    Glb(Handle<Scene>),
}

impl MachineModels {
    /// Get conveyor VOX mesh for a given shape (priority)
    pub fn get_conveyor_vox(&self, shape: ConveyorShape) -> Option<Handle<Mesh>> {
        match shape {
            ConveyorShape::Straight => self.vox_conveyor_straight.clone(),
            ConveyorShape::CornerLeft => self.vox_conveyor_corner_left.clone(),
            ConveyorShape::CornerRight => self.vox_conveyor_corner_right.clone(),
            ConveyorShape::TJunction => self.vox_conveyor_t_junction.clone(),
            ConveyorShape::Splitter => self.vox_conveyor_splitter.clone(),
        }
    }

    /// Get conveyor model handle for a given shape (GLB fallback)
    pub fn get_conveyor_model(&self, shape: ConveyorShape) -> Option<Handle<Scene>> {
        match shape {
            ConveyorShape::Straight => self.conveyor_straight.clone(),
            ConveyorShape::CornerLeft => self.conveyor_corner_left.clone(),
            ConveyorShape::CornerRight => self.conveyor_corner_right.clone(),
            ConveyorShape::TJunction => self.conveyor_t_junction.clone(),
            ConveyorShape::Splitter => self.conveyor_splitter.clone(),
        }
    }

    /// Get conveyor model - VOX priority, GLB fallback
    pub fn get_conveyor(&self, shape: ConveyorShape) -> Option<ModelHandle> {
        if let Some(mesh) = self.get_conveyor_vox(shape) {
            Some(ModelHandle::Vox(mesh))
        } else {
            self.get_conveyor_model(shape).map(ModelHandle::Glb)
        }
    }

    /// Get machine VOX mesh by name
    pub fn get_machine_vox(&self, name: &str) -> Option<Handle<Mesh>> {
        match name {
            "miner" => self.vox_miner.clone(),
            "furnace" => self.vox_furnace.clone(),
            "crusher" => self.vox_crusher.clone(),
            "assembler" => self.vox_assembler.clone(),
            "generator" => self.vox_generator.clone(),
            "inserter" => self.vox_inserter.clone(),
            "storage" => self.vox_storage.clone(),
            "splitter_machine" => self.vox_splitter_machine.clone(),
            _ => None,
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

    /// Check if VOX models are available
    pub fn has_vox_models(&self) -> bool {
        self.vox_miner.is_some() || self.vox_conveyor_straight.is_some()
    }
}
