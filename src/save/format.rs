//! Save/Load system for game data persistence
//!
//! This module uses V2 format exclusively (string IDs like "base:iron_ore").
//! BlockType::to_save_string_id() and from_save_string_id() are used for conversion.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Save version with string ID format
pub const SAVE_VERSION: &str = "0.2.0";

/// Auto-save interval in seconds
pub const AUTO_SAVE_INTERVAL: f32 = 60.0;

/// Save directory name
pub const SAVE_DIR: &str = "saves";

// =============================================================================
// Common Structures (used by all versions)
// =============================================================================

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

/// Game mode save data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameModeSaveData {
    pub creative: bool,
}

// =============================================================================
// V2 Save Data Structures (String ID based)
// =============================================================================

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

/// Global inventory save data using string IDs
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GlobalInventorySaveDataV2 {
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
    pub global_inventory: GlobalInventorySaveDataV2,
    /// World modifications
    pub world: WorldSaveDataV2,
    /// All machines in the world
    pub machines: Vec<MachineSaveDataV2>,
    /// Quest progress
    pub quests: QuestSaveDataV2,
    /// Game mode
    pub mode: GameModeSaveData,
}

// =============================================================================
// Utilities
// =============================================================================

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

// =============================================================================
// Native Save/Load Functions
// =============================================================================

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

    /// Save game data in V2 format
    pub fn save_game_v2(data: &SaveDataV2, filename: &str) -> Result<(), String> {
        ensure_save_dir().map_err(|e| format!("Failed to create save directory: {}", e))?;

        let path = get_save_dir().join(format!("{}.json", filename));
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize save data: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write save file: {}", e))?;

        Ok(())
    }

    /// Load game data in V2 format
    pub fn load_game_v2(filename: &str) -> Result<SaveDataV2, String> {
        let path = get_save_dir().join(format!("{}.json", filename));

        if !path.exists() {
            return Err(format!("Save file not found: {}", filename));
        }

        let json =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read save file: {}", e))?;

        serde_json::from_str(&json).map_err(|e| format!("Failed to parse save data: {}", e))
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
            if path.extension().is_some_and(|e| e == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Try to read timestamp from file
                    if let Ok(json) = fs::read_to_string(&path) {
                        if let Ok(data) = serde_json::from_str::<SaveDataV2>(&json) {
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

/// List all save files
#[allow(dead_code)]
pub fn list_saves() -> Result<Vec<SaveSlotInfo>, String> {
    native::list_saves()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_key_conversion() {
        let pos = IVec3::new(10, -5, 20);
        let key = WorldSaveDataV2::pos_to_key(pos);
        assert_eq!(key, "10,-5,20");

        let parsed = WorldSaveDataV2::key_to_pos(&key).expect("key_to_pos should succeed");
        assert_eq!(parsed, pos);
    }

    #[test]
    fn test_key_to_pos_invalid_formats() {
        // Too few parts
        assert!(WorldSaveDataV2::key_to_pos("10,20").is_none());

        // Too many parts
        assert!(WorldSaveDataV2::key_to_pos("10,20,30,40").is_none());

        // Non-numeric
        assert!(WorldSaveDataV2::key_to_pos("abc,20,30").is_none());
        assert!(WorldSaveDataV2::key_to_pos("10,xyz,30").is_none());
        assert!(WorldSaveDataV2::key_to_pos("10,20,!!!").is_none());

        // Empty
        assert!(WorldSaveDataV2::key_to_pos("").is_none());

        // Partial empty
        assert!(WorldSaveDataV2::key_to_pos(",20,30").is_none());
        assert!(WorldSaveDataV2::key_to_pos("10,,30").is_none());
    }

    #[test]
    fn test_key_to_pos_boundary_values() {
        // Large positive values
        let big_pos = IVec3::new(i32::MAX, i32::MAX, i32::MAX);
        let key = WorldSaveDataV2::pos_to_key(big_pos);
        let restored =
            WorldSaveDataV2::key_to_pos(&key).expect("key_to_pos should succeed for big values");
        assert_eq!(restored, big_pos);

        // Large negative values
        let small_pos = IVec3::new(i32::MIN, i32::MIN, i32::MIN);
        let key = WorldSaveDataV2::pos_to_key(small_pos);
        let restored =
            WorldSaveDataV2::key_to_pos(&key).expect("key_to_pos should succeed for small values");
        assert_eq!(restored, small_pos);

        // Zero
        let zero_pos = IVec3::ZERO;
        let key = WorldSaveDataV2::pos_to_key(zero_pos);
        let restored =
            WorldSaveDataV2::key_to_pos(&key).expect("key_to_pos should succeed for zero");
        assert_eq!(restored, zero_pos);
    }

    #[test]
    fn test_item_stack_v2_new() {
        let stack = ItemStackV2::new("base:iron_ore", 64);
        assert_eq!(stack.item_id, "base:iron_ore");
        assert_eq!(stack.count, 64);
    }

    #[test]
    fn test_item_stack_v2_serialization() {
        let stack = ItemStackV2::new("base:iron_ore", 64);

        let json = serde_json::to_string(&stack).expect("serialization should succeed");
        assert!(json.contains("base:iron_ore"));
        assert!(json.contains("64"));

        let restored: ItemStackV2 =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(restored, stack);
    }

    #[test]
    fn test_save_data_v2_serialization() {
        let v2 = SaveDataV2 {
            version: SAVE_VERSION.to_string(),
            timestamp: 1704067200000,
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
            inventory: InventorySaveDataV2 {
                selected_slot: 0,
                slots: vec![Some(ItemStackV2::new("base:iron_ore", 64))],
            },
            global_inventory: GlobalInventorySaveDataV2::default(),
            world: WorldSaveDataV2 {
                modified_blocks: HashMap::new(),
            },
            machines: vec![],
            quests: QuestSaveDataV2 {
                current_index: 0,
                completed: false,
                rewards_claimed: false,
                delivered: HashMap::new(),
            },
            mode: GameModeSaveData { creative: false },
        };

        // Serialize and deserialize
        let json = serde_json::to_string_pretty(&v2).expect("serialization should succeed");

        // JSON should contain string IDs
        assert!(json.contains("base:iron_ore"));
        assert!(json.contains("0.2.0"));

        // Deserialize back
        let restored: SaveDataV2 =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(restored.version, SAVE_VERSION);
        assert_eq!(
            restored.inventory.slots[0].as_ref().unwrap().item_id,
            "base:iron_ore"
        );
    }

    #[test]
    fn test_machine_save_data_v2_all_types() {
        // Test all machine types serialize correctly
        let machines = vec![
            MachineSaveDataV2::Miner(MinerSaveDataV2 {
                position: IVec3Save { x: 0, y: 0, z: 0 },
                progress: 0.5,
                buffer: Some(ItemStackV2::new("base:iron_ore", 1)),
            }),
            MachineSaveDataV2::Conveyor(ConveyorSaveDataV2 {
                position: IVec3Save { x: 1, y: 0, z: 0 },
                direction: DirectionSave::East,
                shape: ConveyorShapeSave::Straight,
                items: vec![ConveyorItemSaveV2 {
                    item_id: "base:coal".to_string(),
                    progress: 0.3,
                    lateral_offset: 0.0,
                }],
                last_output_index: 0,
                last_input_source: 0,
            }),
            MachineSaveDataV2::Furnace(FurnaceSaveDataV2 {
                position: IVec3Save { x: 2, y: 0, z: 0 },
                fuel: 10,
                input: Some(ItemStackV2::new("base:iron_ore", 5)),
                output: Some(ItemStackV2::new("base:iron_ingot", 3)),
                progress: 0.75,
            }),
            MachineSaveDataV2::Crusher(CrusherSaveDataV2 {
                position: IVec3Save { x: 3, y: 0, z: 0 },
                input: Some(ItemStackV2::new("base:copper_ore", 10)),
                output: None,
                progress: 0.25,
            }),
        ];

        for machine in machines {
            let json = serde_json::to_string(&machine).expect("serialization should succeed");
            let restored: MachineSaveDataV2 =
                serde_json::from_str(&json).expect("deserialization should succeed");

            // Verify the type matches
            match (&machine, &restored) {
                (MachineSaveDataV2::Miner(_), MachineSaveDataV2::Miner(_)) => {}
                (MachineSaveDataV2::Conveyor(_), MachineSaveDataV2::Conveyor(_)) => {}
                (MachineSaveDataV2::Furnace(_), MachineSaveDataV2::Furnace(_)) => {}
                (MachineSaveDataV2::Crusher(_), MachineSaveDataV2::Crusher(_)) => {}
                _ => panic!("Machine type mismatch after roundtrip"),
            }
        }
    }

    #[test]
    fn test_empty_save_data_v2() {
        let data = SaveDataV2 {
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
            inventory: InventorySaveDataV2 {
                selected_slot: 0,
                slots: vec![],
            },
            global_inventory: GlobalInventorySaveDataV2::default(),
            world: WorldSaveDataV2 {
                modified_blocks: HashMap::new(),
            },
            machines: vec![],
            quests: QuestSaveDataV2 {
                current_index: 0,
                completed: false,
                rewards_claimed: false,
                delivered: HashMap::new(),
            },
            mode: GameModeSaveData { creative: false },
        };

        let json = serde_json::to_string(&data).expect("serialization should succeed");
        let restored: SaveDataV2 =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert!(restored.inventory.slots.is_empty());
        assert!(restored.machines.is_empty());
        assert!(restored.world.modified_blocks.is_empty());
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
            let conveyor = ConveyorSaveDataV2 {
                position: IVec3Save { x: 0, y: 0, z: 0 },
                direction: DirectionSave::North,
                shape,
                items: vec![],
                last_output_index: 0,
                last_input_source: 0,
            };

            let json = serde_json::to_string(&conveyor).expect("serialization should succeed");
            let restored: ConveyorSaveDataV2 =
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
            let conveyor = ConveyorSaveDataV2 {
                position: IVec3Save { x: 0, y: 0, z: 0 },
                direction: dir,
                shape: ConveyorShapeSave::Straight,
                items: vec![],
                last_output_index: 0,
                last_input_source: 0,
            };

            let json = serde_json::to_string(&conveyor).expect("serialization should succeed");
            let restored: ConveyorSaveDataV2 =
                serde_json::from_str(&json).expect("deserialization should succeed");

            assert_eq!(restored.direction, dir);
        }
    }

    #[test]
    fn test_item_stack_v2_json_format() {
        // Verify the JSON format is what we expect
        let stack = ItemStackV2::new("base:iron_ore", 64);
        let json = serde_json::to_string_pretty(&stack).expect("serialization should succeed");

        // The JSON should be human-readable with string IDs
        assert!(json.contains(r#""item_id": "base:iron_ore""#));
        assert!(json.contains(r#""count": 64"#));
    }

    #[test]
    fn test_save_data_v2_round_trip_with_machines() {
        // Create comprehensive save data
        let mut modified_blocks = HashMap::new();
        modified_blocks.insert("5,10,5".to_string(), Some("base:stone".to_string()));
        modified_blocks.insert("6,10,5".to_string(), None); // Removed block

        let mut delivered = HashMap::new();
        delivered.insert("base:iron_ingot".to_string(), 5);

        let mut global_items = HashMap::new();
        global_items.insert("base:iron_ingot".to_string(), 100);
        global_items.insert("base:copper_ore".to_string(), 50);

        let data = SaveDataV2 {
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
            inventory: InventorySaveDataV2 {
                selected_slot: 3,
                slots: vec![
                    Some(ItemStackV2::new("base:iron_ore", 64)),
                    Some(ItemStackV2::new("base:coal", 32)),
                    None,
                    Some(ItemStackV2::new("base:miner_block", 5)),
                ],
            },
            global_inventory: GlobalInventorySaveDataV2 {
                items: global_items,
            },
            world: WorldSaveDataV2 { modified_blocks },
            machines: vec![
                MachineSaveDataV2::Miner(MinerSaveDataV2 {
                    position: IVec3Save { x: 10, y: 5, z: 10 },
                    progress: 0.5,
                    buffer: Some(ItemStackV2::new("base:iron_ore", 1)),
                }),
                MachineSaveDataV2::Conveyor(ConveyorSaveDataV2 {
                    position: IVec3Save { x: 11, y: 5, z: 10 },
                    direction: DirectionSave::East,
                    shape: ConveyorShapeSave::Straight,
                    items: vec![ConveyorItemSaveV2 {
                        item_id: "base:iron_ore".to_string(),
                        progress: 0.3,
                        lateral_offset: 0.0,
                    }],
                    last_output_index: 0,
                    last_input_source: 0,
                }),
                MachineSaveDataV2::Furnace(FurnaceSaveDataV2 {
                    position: IVec3Save { x: 12, y: 5, z: 10 },
                    fuel: 10,
                    input: Some(ItemStackV2::new("base:iron_ore", 5)),
                    output: Some(ItemStackV2::new("base:iron_ingot", 3)),
                    progress: 0.75,
                }),
                MachineSaveDataV2::Crusher(CrusherSaveDataV2 {
                    position: IVec3Save { x: 13, y: 5, z: 10 },
                    input: Some(ItemStackV2::new("base:copper_ore", 10)),
                    output: Some(ItemStackV2::new("base:copper_dust", 6)),
                    progress: 0.25,
                }),
            ],
            quests: QuestSaveDataV2 {
                current_index: 2,
                completed: false,
                rewards_claimed: false,
                delivered,
            },
            mode: GameModeSaveData { creative: true },
        };

        // Serialize and deserialize
        let json = serde_json::to_string(&data).expect("serialization should succeed");
        let restored: SaveDataV2 =
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
            restored.global_inventory.items.get("base:iron_ingot"),
            Some(&100)
        );

        // World
        assert_eq!(restored.world.modified_blocks.len(), 2);

        // Machines
        assert_eq!(restored.machines.len(), 4);
        match &restored.machines[0] {
            MachineSaveDataV2::Miner(m) => assert!((m.progress - 0.5).abs() < 0.001),
            _ => panic!("Expected Miner"),
        }
        match &restored.machines[1] {
            MachineSaveDataV2::Conveyor(c) => {
                assert_eq!(c.direction, DirectionSave::East);
                assert_eq!(c.items.len(), 1);
            }
            _ => panic!("Expected Conveyor"),
        }
        match &restored.machines[2] {
            MachineSaveDataV2::Furnace(f) => assert_eq!(f.fuel, 10),
            _ => panic!("Expected Furnace"),
        }
        match &restored.machines[3] {
            MachineSaveDataV2::Crusher(c) => assert_eq!(
                c.input.as_ref().expect("crusher input should exist").count,
                10
            ),
            _ => panic!("Expected Crusher"),
        }

        // Quests
        assert_eq!(restored.quests.current_index, 2);
        assert_eq!(restored.quests.delivered.get("base:iron_ingot"), Some(&5));

        // Mode
        assert!(restored.mode.creative);
    }
}
