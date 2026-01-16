//! V2 Save Data Structures (String ID based)

use super::common::{
    ConveyorShapeSave, DirectionSave, GameModeSaveData, IVec3Save, PlayerSaveData,
};
use bevy::prelude::IVec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Item stack using string IDs
/// This allows for mod items and future extensibility
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemStackV2 {
    /// Item ID in "namespace:id" format (e.g., "base:iron_ore", "mymod:copper_plate")
    pub item_id: String,
    /// Number of items in this stack
    pub count: u32,
}

impl ItemStackV2 {
    /// Create a new item stack with the given ID and count
    pub fn new(item_id: impl Into<String>, count: u32) -> Self {
        Self {
            item_id: item_id.into(),
            count,
        }
    }
}

/// Inventory save data using string IDs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventorySaveDataV2 {
    pub selected_slot: usize,
    pub slots: Vec<Option<ItemStackV2>>,
}

/// Platform inventory save data using string IDs
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlatformInventorySaveDataV2 {
    /// Items stored: "namespace:id" -> count
    pub items: HashMap<String, u32>,
}

/// World save data using string IDs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldSaveDataV2 {
    /// Modified blocks: "x,y,z" -> Some("namespace:id") for placed, None for removed
    pub modified_blocks: HashMap<String, Option<String>>,
}

impl WorldSaveDataV2 {
    /// Convert IVec3 to string key for JSON serialization
    pub fn pos_to_key(pos: IVec3) -> String {
        format!("{},{},{}", pos.x, pos.y, pos.z)
    }

    /// Parse string key back to IVec3
    pub fn key_to_pos(key: &str) -> Option<IVec3> {
        let parts: Vec<&str> = key.split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(IVec3::new(
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    }
}

/// Single item on conveyor using string ID
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConveyorItemSaveV2 {
    pub item_id: String,
    pub progress: f32,
    pub lateral_offset: f32,
}

/// Miner save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinerSaveDataV2 {
    pub position: IVec3Save,
    pub progress: f32,
    pub buffer: Option<ItemStackV2>,
}

/// Conveyor save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConveyorSaveDataV2 {
    pub position: IVec3Save,
    pub direction: DirectionSave,
    pub shape: ConveyorShapeSave,
    pub items: Vec<ConveyorItemSaveV2>,
    pub last_output_index: usize,
    pub last_input_source: usize,
}

/// Furnace save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FurnaceSaveDataV2 {
    pub position: IVec3Save,
    pub fuel: u32,
    pub input: Option<ItemStackV2>,
    pub output: Option<ItemStackV2>,
    pub progress: f32,
}

/// Crusher save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrusherSaveDataV2 {
    pub position: IVec3Save,
    pub input: Option<ItemStackV2>,
    pub output: Option<ItemStackV2>,
    pub progress: f32,
}

/// Machine save data (all machine types)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MachineSaveDataV2 {
    Miner(MinerSaveDataV2),
    Conveyor(ConveyorSaveDataV2),
    Furnace(FurnaceSaveDataV2),
    Crusher(CrusherSaveDataV2),
}

/// Quest save data using string IDs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestSaveDataV2 {
    pub current_index: usize,
    pub completed: bool,
    pub rewards_claimed: bool,
    /// Items delivered: "namespace:id" -> count
    pub delivered: HashMap<String, u32>,
}

/// Main save data structure using string IDs throughout
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveDataV2 {
    /// Save format version (should be "0.2.0" or later)
    pub version: String,
    /// Timestamp when saved (Unix milliseconds)
    pub timestamp: u64,
    /// Player state
    pub player: PlayerSaveData,
    /// Inventory state
    pub inventory: InventorySaveDataV2,
    /// Global inventory
    #[serde(default)]
    pub platform_inventory: PlatformInventorySaveDataV2,
    /// World modifications
    pub world: WorldSaveDataV2,
    /// All machines in the world
    pub machines: Vec<MachineSaveDataV2>,
    /// Quest progress
    pub quests: QuestSaveDataV2,
    /// Game mode
    pub mode: GameModeSaveData,
}
