//! Save/Load system for game data persistence
//!
//! This module uses V2 format exclusively (string IDs like "base:iron_ore").
//! BlockType::to_save_string_id() and from_save_string_id() are used for conversion.

mod common;
pub mod native;
mod timer;
mod v2;

// Re-export constants
/// Save version with string ID format
pub const SAVE_VERSION: &str = "0.2.0";

/// Auto-save interval in seconds
pub const AUTO_SAVE_INTERVAL: f32 = 60.0;

/// Save directory name
pub const SAVE_DIR: &str = "saves";

// Re-export common types
pub use common::{
    CameraRotation, ConveyorShapeSave, DirectionSave, GameModeSaveData, IVec3Save, PlayerSaveData,
    Vec3Save,
};

// Re-export timer types
pub use timer::{AutoSaveTimer, SaveSlotInfo};

// Re-export V2 types
pub use v2::{
    ConveyorItemSaveV2, ConveyorSaveDataV2, CrusherSaveDataV2, FurnaceSaveDataV2,
    InventorySaveDataV2, ItemStackV2, MachineSaveDataV2, MinerSaveDataV2,
    PlatformInventorySaveDataV2, QuestSaveDataV2, SaveDataV2, WorldSaveDataV2,
};

/// List all save files
#[allow(dead_code)]
pub fn list_saves() -> Result<Vec<SaveSlotInfo>, String> {
    native::list_saves()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::IVec3;
    use std::collections::HashMap;

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
            platform_inventory: PlatformInventorySaveDataV2::default(),
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
            platform_inventory: PlatformInventorySaveDataV2::default(),
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
            platform_inventory: PlatformInventorySaveDataV2 {
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
            restored.platform_inventory.items.get("base:iron_ingot"),
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
