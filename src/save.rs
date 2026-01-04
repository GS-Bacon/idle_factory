//! Save/Load system for game data persistence

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::block_type::BlockType;

/// Save data version for compatibility checking
pub const SAVE_VERSION: &str = "0.1.0";

/// Auto-save interval in seconds
pub const AUTO_SAVE_INTERVAL: f32 = 60.0;

/// Save directory name
pub const SAVE_DIR: &str = "saves";

/// Main save data structure containing all game state
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveData {
    /// Save format version
    pub version: String,
    /// Timestamp when saved (Unix milliseconds)
    pub timestamp: u64,
    /// Player state
    pub player: PlayerSaveData,
    /// Inventory state
    pub inventory: InventorySaveData,
    /// Global inventory (machines and items) - v0.2 feature
    #[serde(default)]
    pub global_inventory: GlobalInventorySaveData,
    /// World modifications
    pub world: WorldSaveData,
    /// All machines in the world
    pub machines: Vec<MachineSaveData>,
    /// Quest progress
    pub quests: QuestSaveData,
    /// Game mode
    pub mode: GameModeSaveData,
}

/// Player position and rotation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerSaveData {
    pub position: Vec3Save,
    pub rotation: CameraRotation,
}

/// Vec3 wrapper for serialization
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Vec3Save {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Vec3Save {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vec3Save> for Vec3 {
    fn from(v: Vec3Save) -> Self {
        Vec3::new(v.x, v.y, v.z)
    }
}

/// IVec3 wrapper for serialization
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IVec3Save {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl From<IVec3> for IVec3Save {
    fn from(v: IVec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<IVec3Save> for IVec3 {
    fn from(v: IVec3Save) -> Self {
        IVec3::new(v.x, v.y, v.z)
    }
}

/// Camera rotation (pitch/yaw)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CameraRotation {
    pub pitch: f32,
    pub yaw: f32,
}

/// Inventory save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventorySaveData {
    pub selected_slot: usize,
    pub slots: Vec<Option<ItemStack>>,
}

/// Global inventory save data (v0.2)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GlobalInventorySaveData {
    /// Items stored in global inventory: BlockType -> count
    pub items: HashMap<BlockTypeSave, u32>,
}

/// Single item stack
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemStack {
    pub item_type: BlockTypeSave,
    pub count: u32,
}

/// BlockType wrapper for serialization (string-based for forward compatibility)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum BlockTypeSave {
    Stone,
    Grass,
    IronOre,
    Coal,
    IronIngot,
    MinerBlock,
    ConveyorBlock,
    CopperOre,
    CopperIngot,
    CrusherBlock,
    FurnaceBlock,
    StonePickaxe,
    AssemblerBlock,
    IronDust,
    CopperDust,
}

impl From<BlockType> for BlockTypeSave {
    fn from(bt: BlockType) -> Self {
        match bt {
            BlockType::Stone => BlockTypeSave::Stone,
            BlockType::Grass => BlockTypeSave::Grass,
            BlockType::IronOre => BlockTypeSave::IronOre,
            BlockType::Coal => BlockTypeSave::Coal,
            BlockType::IronIngot => BlockTypeSave::IronIngot,
            BlockType::MinerBlock => BlockTypeSave::MinerBlock,
            BlockType::ConveyorBlock => BlockTypeSave::ConveyorBlock,
            BlockType::CopperOre => BlockTypeSave::CopperOre,
            BlockType::CopperIngot => BlockTypeSave::CopperIngot,
            BlockType::CrusherBlock => BlockTypeSave::CrusherBlock,
            BlockType::FurnaceBlock => BlockTypeSave::FurnaceBlock,
            BlockType::StonePickaxe => BlockTypeSave::StonePickaxe,
            BlockType::AssemblerBlock => BlockTypeSave::AssemblerBlock,
            BlockType::IronDust => BlockTypeSave::IronDust,
            BlockType::CopperDust => BlockTypeSave::CopperDust,
        }
    }
}

impl From<BlockTypeSave> for BlockType {
    fn from(bt: BlockTypeSave) -> Self {
        match bt {
            BlockTypeSave::Stone => BlockType::Stone,
            BlockTypeSave::Grass => BlockType::Grass,
            BlockTypeSave::IronOre => BlockType::IronOre,
            BlockTypeSave::Coal => BlockType::Coal,
            BlockTypeSave::IronIngot => BlockType::IronIngot,
            BlockTypeSave::MinerBlock => BlockType::MinerBlock,
            BlockTypeSave::ConveyorBlock => BlockType::ConveyorBlock,
            BlockTypeSave::CopperOre => BlockType::CopperOre,
            BlockTypeSave::CopperIngot => BlockType::CopperIngot,
            BlockTypeSave::CrusherBlock => BlockType::CrusherBlock,
            BlockTypeSave::FurnaceBlock => BlockType::FurnaceBlock,
            BlockTypeSave::StonePickaxe => BlockType::StonePickaxe,
            BlockTypeSave::AssemblerBlock => BlockType::AssemblerBlock,
            BlockTypeSave::IronDust => BlockType::IronDust,
            BlockTypeSave::CopperDust => BlockType::CopperDust,
        }
    }
}

/// World save data (modified blocks only)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldSaveData {
    /// Modified blocks: position -> Some(block) for placed, None for removed
    pub modified_blocks: HashMap<String, Option<BlockTypeSave>>,
}

impl WorldSaveData {
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

/// Machine save data (all machine types)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MachineSaveData {
    Miner(MinerSaveData),
    Conveyor(ConveyorSaveData),
    Furnace(FurnaceSaveData),
    Crusher(CrusherSaveData),
}

/// Direction for conveyors
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionSave {
    North,
    South,
    East,
    West,
}

/// Conveyor shape
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConveyorShapeSave {
    Straight,
    CornerLeft,
    CornerRight,
    TJunction,
    Splitter,
}

/// Miner save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinerSaveData {
    pub position: IVec3Save,
    pub progress: f32,
    pub buffer: Option<ItemStack>,
}

/// Conveyor save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConveyorSaveData {
    pub position: IVec3Save,
    pub direction: DirectionSave,
    pub shape: ConveyorShapeSave,
    pub items: Vec<ConveyorItemSave>,
    pub last_output_index: usize,
    pub last_input_source: usize,
}

/// Single item on conveyor
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConveyorItemSave {
    pub item_type: BlockTypeSave,
    pub progress: f32,
    pub lateral_offset: f32,
}

/// Furnace save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FurnaceSaveData {
    pub position: IVec3Save,
    pub fuel: u32,
    pub input: Option<ItemStack>,
    pub output: Option<ItemStack>,
    pub progress: f32,
}

/// Crusher save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrusherSaveData {
    pub position: IVec3Save,
    pub input: Option<ItemStack>,
    pub output: Option<ItemStack>,
    pub progress: f32,
}

/// Quest progress save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestSaveData {
    pub current_index: usize,
    pub completed: bool,
    pub rewards_claimed: bool,
    /// Items delivered to delivery platform
    pub delivered: HashMap<BlockTypeSave, u32>,
}

/// Game mode save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameModeSaveData {
    pub creative: bool,
}

/// Auto-save timer resource
#[derive(Resource)]
pub struct AutoSaveTimer {
    pub timer: Timer,
}

impl Default for AutoSaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(AUTO_SAVE_INTERVAL, TimerMode::Repeating),
        }
    }
}

/// Save slot info for listing saves
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SaveSlotInfo {
    pub filename: String,
    pub timestamp: u64,
}

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use super::*;
    use std::fs;

    /// Get the saves directory path
    pub fn get_save_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(SAVE_DIR)
    }

    /// Ensure save directory exists
    pub fn ensure_save_dir() -> std::io::Result<()> {
        let dir = get_save_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(())
    }

    /// Save game data to a file
    pub fn save_game(data: &SaveData, filename: &str) -> Result<(), String> {
        ensure_save_dir().map_err(|e| format!("Failed to create save directory: {}", e))?;

        let path = get_save_dir().join(format!("{}.json", filename));
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize save data: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write save file: {}", e))?;

        Ok(())
    }

    /// Load game data from a file
    pub fn load_game(filename: &str) -> Result<SaveData, String> {
        let path = get_save_dir().join(format!("{}.json", filename));

        if !path.exists() {
            return Err(format!("Save file not found: {}", filename));
        }

        let json =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read save file: {}", e))?;

        let data: SaveData =
            serde_json::from_str(&json).map_err(|e| format!("Failed to parse save data: {}", e))?;

        Ok(data)
    }

    /// List all save files
    #[allow(dead_code)]
    pub fn list_saves() -> Result<Vec<SaveSlotInfo>, String> {
        let dir = get_save_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut saves = Vec::new();
        let entries =
            fs::read_dir(&dir).map_err(|e| format!("Failed to read save directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Try to read timestamp from file
                    if let Ok(json) = fs::read_to_string(&path) {
                        if let Ok(data) = serde_json::from_str::<SaveData>(&json) {
                            saves.push(SaveSlotInfo {
                                filename: stem.to_string(),
                                timestamp: data.timestamp,
                            });
                        }
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(saves)
    }

    /// Delete a save file
    #[allow(dead_code)]
    pub fn delete_save(filename: &str) -> Result<(), String> {
        let path = get_save_dir().join(format!("{}.json", filename));

        if !path.exists() {
            return Err(format!("Save file not found: {}", filename));
        }

        fs::remove_file(&path).map_err(|e| format!("Failed to delete save file: {}", e))?;

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use web_sys::window;

    const SAVE_PREFIX: &str = "idle_factory_save_";

    /// Save game data to localStorage
    pub fn save_game(data: &SaveData, filename: &str) -> Result<(), String> {
        let win = window().ok_or("No window available")?;
        let storage = win
            .local_storage()
            .map_err(|_| "Failed to access localStorage")?
            .ok_or("localStorage not available")?;

        let json =
            serde_json::to_string(data).map_err(|e| format!("Failed to serialize: {}", e))?;

        let key = format!("{}{}", SAVE_PREFIX, filename);
        storage
            .set_item(&key, &json)
            .map_err(|_| "Failed to save to localStorage")?;

        Ok(())
    }

    /// Load game data from localStorage
    pub fn load_game(filename: &str) -> Result<SaveData, String> {
        let win = window().ok_or("No window available")?;
        let storage = win
            .local_storage()
            .map_err(|_| "Failed to access localStorage")?
            .ok_or("localStorage not available")?;

        let key = format!("{}{}", SAVE_PREFIX, filename);
        let json = storage
            .get_item(&key)
            .map_err(|_| "Failed to read from localStorage")?
            .ok_or_else(|| format!("Save not found: {}", filename))?;

        let data: SaveData =
            serde_json::from_str(&json).map_err(|e| format!("Failed to parse save: {}", e))?;

        Ok(data)
    }

    /// List all saves in localStorage
    pub fn list_saves() -> Result<Vec<SaveSlotInfo>, String> {
        let win = window().ok_or("No window available")?;
        let storage = win
            .local_storage()
            .map_err(|_| "Failed to access localStorage")?
            .ok_or("localStorage not available")?;

        let mut saves = Vec::new();
        let len = storage
            .length()
            .map_err(|_| "Failed to get storage length")?;

        for i in 0..len {
            if let Ok(Some(key)) = storage.key(i) {
                if key.starts_with(SAVE_PREFIX) {
                    let filename = key.trim_start_matches(SAVE_PREFIX).to_string();
                    if let Ok(Some(json)) = storage.get_item(&key) {
                        if let Ok(data) = serde_json::from_str::<SaveData>(&json) {
                            saves.push(SaveSlotInfo {
                                filename,
                                timestamp: data.timestamp,
                            });
                        }
                    }
                }
            }
        }

        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(saves)
    }

    /// Delete a save from localStorage
    pub fn delete_save(filename: &str) -> Result<(), String> {
        let win = window().ok_or("No window available")?;
        let storage = win
            .local_storage()
            .map_err(|_| "Failed to access localStorage")?
            .ok_or("localStorage not available")?;

        let key = format!("{}{}", SAVE_PREFIX, filename);
        storage
            .remove_item(&key)
            .map_err(|_| "Failed to delete save")?;

        Ok(())
    }
}

/// Platform-agnostic save function
pub fn save_game(data: &SaveData, filename: &str) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::save_game(data, filename)
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm::save_game(data, filename)
    }
}

/// Platform-agnostic load function
pub fn load_game(filename: &str) -> Result<SaveData, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::load_game(filename)
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm::load_game(filename)
    }
}

/// Platform-agnostic list saves function
#[allow(dead_code)]
pub fn list_saves() -> Result<Vec<SaveSlotInfo>, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::list_saves()
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm::list_saves()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_type_serialization() {
        let bt = BlockTypeSave::IronOre;
        let json = serde_json::to_string(&bt).expect("serialization should succeed");
        assert!(json.contains("IronOre"));

        let parsed: BlockTypeSave =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(parsed, bt);
    }

    #[test]
    fn test_block_type_conversion() {
        for bt in [
            BlockType::Stone,
            BlockType::Grass,
            BlockType::IronOre,
            BlockType::Coal,
            BlockType::IronIngot,
            BlockType::MinerBlock,
            BlockType::ConveyorBlock,
            BlockType::CopperOre,
            BlockType::CopperIngot,
            BlockType::CrusherBlock,
            BlockType::FurnaceBlock,
        ] {
            let save: BlockTypeSave = bt.into();
            let restored: BlockType = save.into();
            assert_eq!(bt, restored);
        }
    }

    #[test]
    fn test_pos_key_conversion() {
        let pos = IVec3::new(10, -5, 20);
        let key = WorldSaveData::pos_to_key(pos);
        assert_eq!(key, "10,-5,20");

        let parsed = WorldSaveData::key_to_pos(&key).expect("key_to_pos should succeed");
        assert_eq!(parsed, pos);
    }

    #[test]
    fn test_save_data_serialization() {
        let data = SaveData {
            version: SAVE_VERSION.to_string(),
            timestamp: 1704067200000,
            player: PlayerSaveData {
                position: Vec3Save {
                    x: 8.0,
                    y: 12.0,
                    z: 20.0,
                },
                rotation: CameraRotation {
                    pitch: 0.0,
                    yaw: 0.0,
                },
            },
            inventory: InventorySaveData {
                selected_slot: 0,
                slots: vec![
                    Some(ItemStack {
                        item_type: BlockTypeSave::Stone,
                        count: 10,
                    }),
                    None,
                ],
            },
            global_inventory: GlobalInventorySaveData::default(),
            world: WorldSaveData {
                modified_blocks: HashMap::new(),
            },
            machines: vec![],
            quests: QuestSaveData {
                current_index: 0,
                completed: false,
                rewards_claimed: false,
                delivered: HashMap::new(),
            },
            mode: GameModeSaveData { creative: false },
        };

        let json = serde_json::to_string_pretty(&data).expect("serialization should succeed");
        let parsed: SaveData = serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(parsed.version, data.version);
        assert_eq!(parsed.inventory.selected_slot, 0);
        assert!(parsed.inventory.slots[0].is_some());
    }

    // === Save/Load Round-trip Tests ===

    #[test]
    fn test_save_data_round_trip_with_machines() {
        // Create save data with all machine types
        let mut modified_blocks = HashMap::new();
        modified_blocks.insert("5,10,5".to_string(), Some(BlockTypeSave::Stone));
        modified_blocks.insert("6,10,5".to_string(), None); // Removed block

        let mut delivered = HashMap::new();
        delivered.insert(BlockTypeSave::IronIngot, 5);

        let data = SaveData {
            version: SAVE_VERSION.to_string(),
            timestamp: 1704067200000,
            player: PlayerSaveData {
                position: Vec3Save {
                    x: 100.0,
                    y: 50.0,
                    z: -200.0,
                },
                rotation: CameraRotation {
                    pitch: -0.5,
                    yaw: 3.14,
                },
            },
            inventory: InventorySaveData {
                selected_slot: 3,
                slots: vec![
                    Some(ItemStack {
                        item_type: BlockTypeSave::IronOre,
                        count: 64,
                    }),
                    Some(ItemStack {
                        item_type: BlockTypeSave::Coal,
                        count: 32,
                    }),
                    None,
                    Some(ItemStack {
                        item_type: BlockTypeSave::MinerBlock,
                        count: 5,
                    }),
                ],
            },
            global_inventory: GlobalInventorySaveData {
                items: {
                    let mut m = HashMap::new();
                    m.insert(BlockTypeSave::IronIngot, 100);
                    m.insert(BlockTypeSave::CopperOre, 50);
                    m
                },
            },
            world: WorldSaveData { modified_blocks },
            machines: vec![
                MachineSaveData::Miner(MinerSaveData {
                    position: IVec3Save { x: 10, y: 5, z: 10 },
                    progress: 0.5,
                    buffer: Some(ItemStack {
                        item_type: BlockTypeSave::IronOre,
                        count: 1,
                    }),
                }),
                MachineSaveData::Conveyor(ConveyorSaveData {
                    position: IVec3Save { x: 11, y: 5, z: 10 },
                    direction: DirectionSave::East,
                    shape: ConveyorShapeSave::Straight,
                    items: vec![ConveyorItemSave {
                        item_type: BlockTypeSave::IronOre,
                        progress: 0.3,
                        lateral_offset: 0.0,
                    }],
                    last_output_index: 0,
                    last_input_source: 0,
                }),
                MachineSaveData::Furnace(FurnaceSaveData {
                    position: IVec3Save { x: 12, y: 5, z: 10 },
                    fuel: 10,
                    input: Some(ItemStack {
                        item_type: BlockTypeSave::IronOre,
                        count: 5,
                    }),
                    output: Some(ItemStack {
                        item_type: BlockTypeSave::IronIngot,
                        count: 3,
                    }),
                    progress: 0.75,
                }),
                MachineSaveData::Crusher(CrusherSaveData {
                    position: IVec3Save { x: 13, y: 5, z: 10 },
                    input: Some(ItemStack {
                        item_type: BlockTypeSave::CopperOre,
                        count: 10,
                    }),
                    output: Some(ItemStack {
                        item_type: BlockTypeSave::CopperOre,
                        count: 6,
                    }),
                    progress: 0.25,
                }),
            ],
            quests: QuestSaveData {
                current_index: 2,
                completed: false,
                rewards_claimed: false,
                delivered,
            },
            mode: GameModeSaveData { creative: true },
        };

        // Serialize and deserialize
        let json = serde_json::to_string(&data).expect("serialization should succeed");
        let restored: SaveData =
            serde_json::from_str(&json).expect("deserialization should succeed");

        // Verify all fields
        assert_eq!(restored.version, data.version);
        assert_eq!(restored.timestamp, data.timestamp);

        // Player
        assert!((restored.player.position.x - 100.0).abs() < 0.001);
        assert!((restored.player.rotation.yaw - 3.14).abs() < 0.001);

        // Inventory
        assert_eq!(restored.inventory.selected_slot, 3);
        assert_eq!(restored.inventory.slots.len(), 4);
        assert_eq!(
            restored.inventory.slots[0]
                .as_ref()
                .expect("slot 0 should exist")
                .count,
            64
        );

        // Global inventory
        assert_eq!(
            restored
                .global_inventory
                .items
                .get(&BlockTypeSave::IronIngot),
            Some(&100)
        );

        // World
        assert_eq!(restored.world.modified_blocks.len(), 2);

        // Machines
        assert_eq!(restored.machines.len(), 4);
        match &restored.machines[0] {
            MachineSaveData::Miner(m) => assert!((m.progress - 0.5).abs() < 0.001),
            _ => panic!("Expected Miner"),
        }
        match &restored.machines[1] {
            MachineSaveData::Conveyor(c) => {
                assert_eq!(c.direction, DirectionSave::East);
                assert_eq!(c.items.len(), 1);
            }
            _ => panic!("Expected Conveyor"),
        }
        match &restored.machines[2] {
            MachineSaveData::Furnace(f) => assert_eq!(f.fuel, 10),
            _ => panic!("Expected Furnace"),
        }
        match &restored.machines[3] {
            MachineSaveData::Crusher(c) => assert_eq!(
                c.input.as_ref().expect("crusher input should exist").count,
                10
            ),
            _ => panic!("Expected Crusher"),
        }

        // Quests
        assert_eq!(restored.quests.current_index, 2);
        assert_eq!(
            restored.quests.delivered.get(&BlockTypeSave::IronIngot),
            Some(&5)
        );

        // Mode
        assert!(restored.mode.creative);
    }

    // === Edge Case Tests ===

    #[test]
    fn test_key_to_pos_invalid_formats() {
        // Too few parts
        assert!(WorldSaveData::key_to_pos("10,20").is_none());

        // Too many parts
        assert!(WorldSaveData::key_to_pos("10,20,30,40").is_none());

        // Non-numeric
        assert!(WorldSaveData::key_to_pos("abc,20,30").is_none());
        assert!(WorldSaveData::key_to_pos("10,xyz,30").is_none());
        assert!(WorldSaveData::key_to_pos("10,20,!!!").is_none());

        // Empty
        assert!(WorldSaveData::key_to_pos("").is_none());

        // Partial empty
        assert!(WorldSaveData::key_to_pos(",20,30").is_none());
        assert!(WorldSaveData::key_to_pos("10,,30").is_none());
    }

    #[test]
    fn test_key_to_pos_boundary_values() {
        // Large positive values
        let big_pos = IVec3::new(i32::MAX, i32::MAX, i32::MAX);
        let key = WorldSaveData::pos_to_key(big_pos);
        let restored =
            WorldSaveData::key_to_pos(&key).expect("key_to_pos should succeed for big values");
        assert_eq!(restored, big_pos);

        // Large negative values
        let small_pos = IVec3::new(i32::MIN, i32::MIN, i32::MIN);
        let key = WorldSaveData::pos_to_key(small_pos);
        let restored =
            WorldSaveData::key_to_pos(&key).expect("key_to_pos should succeed for small values");
        assert_eq!(restored, small_pos);

        // Zero
        let zero_pos = IVec3::ZERO;
        let key = WorldSaveData::pos_to_key(zero_pos);
        let restored = WorldSaveData::key_to_pos(&key).expect("key_to_pos should succeed for zero");
        assert_eq!(restored, zero_pos);
    }

    #[test]
    fn test_empty_save_data() {
        let data = SaveData {
            version: SAVE_VERSION.to_string(),
            timestamp: 0,
            player: PlayerSaveData {
                position: Vec3Save {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                rotation: CameraRotation {
                    pitch: 0.0,
                    yaw: 0.0,
                },
            },
            inventory: InventorySaveData {
                selected_slot: 0,
                slots: vec![],
            },
            global_inventory: GlobalInventorySaveData::default(),
            world: WorldSaveData {
                modified_blocks: HashMap::new(),
            },
            machines: vec![],
            quests: QuestSaveData {
                current_index: 0,
                completed: false,
                rewards_claimed: false,
                delivered: HashMap::new(),
            },
            mode: GameModeSaveData { creative: false },
        };

        let json = serde_json::to_string(&data).expect("serialization should succeed");
        let restored: SaveData =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert!(restored.inventory.slots.is_empty());
        assert!(restored.machines.is_empty());
        assert!(restored.world.modified_blocks.is_empty());
    }

    #[test]
    fn test_max_stack_values() {
        let stack = ItemStack {
            item_type: BlockTypeSave::Stone,
            count: u32::MAX,
        };

        let json = serde_json::to_string(&stack).expect("serialization should succeed");
        let restored: ItemStack =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(restored.count, u32::MAX);
    }

    #[test]
    fn test_conveyor_all_shapes() {
        for shape in [
            ConveyorShapeSave::Straight,
            ConveyorShapeSave::CornerLeft,
            ConveyorShapeSave::CornerRight,
            ConveyorShapeSave::TJunction,
            ConveyorShapeSave::Splitter,
        ] {
            let conveyor = ConveyorSaveData {
                position: IVec3Save { x: 0, y: 0, z: 0 },
                direction: DirectionSave::North,
                shape,
                items: vec![],
                last_output_index: 0,
                last_input_source: 0,
            };

            let json = serde_json::to_string(&conveyor).expect("serialization should succeed");
            let restored: ConveyorSaveData =
                serde_json::from_str(&json).expect("deserialization should succeed");

            assert_eq!(restored.shape, shape);
        }
    }

    #[test]
    fn test_direction_all_values() {
        for dir in [
            DirectionSave::North,
            DirectionSave::South,
            DirectionSave::East,
            DirectionSave::West,
        ] {
            let conveyor = ConveyorSaveData {
                position: IVec3Save { x: 0, y: 0, z: 0 },
                direction: dir,
                shape: ConveyorShapeSave::Straight,
                items: vec![],
                last_output_index: 0,
                last_input_source: 0,
            };

            let json = serde_json::to_string(&conveyor).expect("serialization should succeed");
            let restored: ConveyorSaveData =
                serde_json::from_str(&json).expect("deserialization should succeed");

            assert_eq!(restored.direction, dir);
        }
    }
}
