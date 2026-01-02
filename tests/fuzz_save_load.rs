//! Fuzz-like tests for save/load parsing using proptest
//!
//! Tests that the save/load system handles malformed and edge-case inputs gracefully.

use proptest::prelude::*;
use serde_json;

// Re-export save types for testing
mod save_types {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct SaveData {
        pub version: String,
        pub timestamp: u64,
        pub player: PlayerSaveData,
        pub inventory: InventorySaveData,
        #[serde(default)]
        pub global_inventory: GlobalInventorySaveData,
        pub world: WorldSaveData,
        pub machines: Vec<MachineSaveData>,
        pub quests: QuestSaveData,
        pub mode: GameModeSaveData,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct PlayerSaveData {
        pub position: Vec3Save,
        pub rotation: CameraRotation,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    pub struct Vec3Save {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    pub struct CameraRotation {
        pub pitch: f32,
        pub yaw: f32,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct InventorySaveData {
        pub selected_slot: usize,
        pub slots: Vec<Option<ItemStack>>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Default)]
    pub struct GlobalInventorySaveData {
        pub items: HashMap<BlockTypeSave, u32>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ItemStack {
        pub item_type: BlockTypeSave,
        pub count: u32,
    }

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
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct WorldSaveData {
        pub modified_blocks: HashMap<String, Option<BlockTypeSave>>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type")]
    pub enum MachineSaveData {
        Miner(MinerSaveData),
        Conveyor(ConveyorSaveData),
        Furnace(FurnaceSaveData),
        Crusher(CrusherSaveData),
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DirectionSave {
        North,
        South,
        East,
        West,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ConveyorShapeSave {
        Straight,
        CornerLeft,
        CornerRight,
        TJunction,
        Splitter,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MinerSaveData {
        pub position: IVec3Save,
        pub progress: f32,
        pub buffer: Option<ItemStack>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct IVec3Save {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ConveyorSaveData {
        pub position: IVec3Save,
        pub direction: DirectionSave,
        pub shape: ConveyorShapeSave,
        pub items: Vec<ConveyorItemSave>,
        pub last_output_index: usize,
        pub last_input_source: usize,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ConveyorItemSave {
        pub item_type: BlockTypeSave,
        pub progress: f32,
        pub lateral_offset: f32,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct FurnaceSaveData {
        pub position: IVec3Save,
        pub fuel: u32,
        pub input: Option<ItemStack>,
        pub output: Option<ItemStack>,
        pub progress: f32,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct CrusherSaveData {
        pub position: IVec3Save,
        pub input: Option<ItemStack>,
        pub output: Option<ItemStack>,
        pub progress: f32,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct QuestSaveData {
        pub current_index: usize,
        pub completed: bool,
        pub rewards_claimed: bool,
        pub delivered: HashMap<BlockTypeSave, u32>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GameModeSaveData {
        pub creative: bool,
    }
}

use save_types::*;

// Generators for save types
// Note: Using POSITIVE range to avoid NaN/Infinity which aren't valid JSON
fn arb_vec3save() -> impl Strategy<Value = Vec3Save> {
    (
        -1000000.0f32..1000000.0f32,
        -1000000.0f32..1000000.0f32,
        -1000000.0f32..1000000.0f32,
    )
        .prop_map(|(x, y, z)| Vec3Save { x, y, z })
}

fn arb_ivec3save() -> impl Strategy<Value = IVec3Save> {
    (any::<i32>(), any::<i32>(), any::<i32>()).prop_map(|(x, y, z)| IVec3Save { x, y, z })
}

fn arb_block_type() -> impl Strategy<Value = BlockTypeSave> {
    prop_oneof![
        Just(BlockTypeSave::Stone),
        Just(BlockTypeSave::Grass),
        Just(BlockTypeSave::IronOre),
        Just(BlockTypeSave::Coal),
        Just(BlockTypeSave::IronIngot),
        Just(BlockTypeSave::MinerBlock),
        Just(BlockTypeSave::ConveyorBlock),
        Just(BlockTypeSave::CopperOre),
        Just(BlockTypeSave::CopperIngot),
        Just(BlockTypeSave::CrusherBlock),
        Just(BlockTypeSave::FurnaceBlock),
    ]
}

fn arb_item_stack() -> impl Strategy<Value = ItemStack> {
    (arb_block_type(), any::<u32>()).prop_map(|(item_type, count)| ItemStack { item_type, count })
}

fn arb_direction() -> impl Strategy<Value = DirectionSave> {
    prop_oneof![
        Just(DirectionSave::North),
        Just(DirectionSave::South),
        Just(DirectionSave::East),
        Just(DirectionSave::West),
    ]
}

fn arb_conveyor_shape() -> impl Strategy<Value = ConveyorShapeSave> {
    prop_oneof![
        Just(ConveyorShapeSave::Straight),
        Just(ConveyorShapeSave::CornerLeft),
        Just(ConveyorShapeSave::CornerRight),
        Just(ConveyorShapeSave::TJunction),
        Just(ConveyorShapeSave::Splitter),
    ]
}

fn arb_miner_save() -> impl Strategy<Value = MachineSaveData> {
    (
        arb_ivec3save(),
        0.0f32..1.0f32,
        proptest::option::of(arb_item_stack()),
    )
        .prop_map(|(position, progress, buffer)| {
            MachineSaveData::Miner(MinerSaveData {
                position,
                progress,
                buffer,
            })
        })
}

fn arb_conveyor_save() -> impl Strategy<Value = MachineSaveData> {
    (
        arb_ivec3save(),
        arb_direction(),
        arb_conveyor_shape(),
        prop::collection::vec(
            (arb_block_type(), 0.0f32..0.999f32, -0.5f32..0.5f32).prop_map(
                |(item_type, progress, lateral_offset)| ConveyorItemSave {
                    item_type,
                    progress,
                    lateral_offset,
                },
            ),
            0..5,
        ),
        0usize..10,
        0usize..10,
    )
        .prop_map(
            |(position, direction, shape, items, last_output_index, last_input_source)| {
                MachineSaveData::Conveyor(ConveyorSaveData {
                    position,
                    direction,
                    shape,
                    items,
                    last_output_index,
                    last_input_source,
                })
            },
        )
}

fn arb_furnace_save() -> impl Strategy<Value = MachineSaveData> {
    (
        arb_ivec3save(),
        any::<u32>(),
        proptest::option::of(arb_item_stack()),
        proptest::option::of(arb_item_stack()),
        0.0f32..1.0f32,
    )
        .prop_map(|(position, fuel, input, output, progress)| {
            MachineSaveData::Furnace(FurnaceSaveData {
                position,
                fuel,
                input,
                output,
                progress,
            })
        })
}

fn arb_crusher_save() -> impl Strategy<Value = MachineSaveData> {
    (
        arb_ivec3save(),
        proptest::option::of(arb_item_stack()),
        proptest::option::of(arb_item_stack()),
        0.0f32..1.0f32,
    )
        .prop_map(|(position, input, output, progress)| {
            MachineSaveData::Crusher(CrusherSaveData {
                position,
                input,
                output,
                progress,
            })
        })
}

fn arb_machine_save() -> impl Strategy<Value = MachineSaveData> {
    prop_oneof![
        arb_miner_save(),
        arb_conveyor_save(),
        arb_furnace_save(),
        arb_crusher_save(),
    ]
}

fn arb_save_data() -> impl Strategy<Value = SaveData> {
    (
        "0\\.1\\.0|0\\.2\\.0|[0-9]+\\.[0-9]+\\.[0-9]+", // version
        any::<u64>(),                                   // timestamp
        (arb_vec3save(), (-90.0f32..90.0f32, -180.0f32..180.0f32)), // player
        (
            0usize..9,
            prop::collection::vec(proptest::option::of(arb_item_stack()), 0..36),
        ), // inventory
        prop::collection::hash_map(arb_block_type(), any::<u32>(), 0..10), // global_inventory
        prop::collection::hash_map("[0-9,-]+", proptest::option::of(arb_block_type()), 0..100), // world
        prop::collection::vec(arb_machine_save(), 0..50), // machines
        (
            any::<usize>(),
            any::<bool>(),
            any::<bool>(),
            prop::collection::hash_map(arb_block_type(), any::<u32>(), 0..5),
        ), // quests
        any::<bool>(),                                    // mode.creative
    )
        .prop_map(
            |(
                version,
                timestamp,
                (position, (pitch, yaw)),
                (selected_slot, slots),
                global_items,
                modified_blocks,
                machines,
                (current_index, completed, rewards_claimed, delivered),
                creative,
            )| {
                SaveData {
                    version,
                    timestamp,
                    player: PlayerSaveData {
                        position,
                        rotation: CameraRotation { pitch, yaw },
                    },
                    inventory: InventorySaveData {
                        selected_slot,
                        slots,
                    },
                    global_inventory: GlobalInventorySaveData {
                        items: global_items,
                    },
                    world: WorldSaveData { modified_blocks },
                    machines,
                    quests: QuestSaveData {
                        current_index,
                        completed,
                        rewards_claimed,
                        delivered,
                    },
                    mode: GameModeSaveData { creative },
                }
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Fuzz: Save data round-trip (serialize -> deserialize)
    #[test]
    fn fuzz_save_roundtrip(save_data in arb_save_data()) {
        // Serialize
        let json = serde_json::to_string(&save_data);
        prop_assert!(json.is_ok(), "Serialization should not fail");
        let json_str = json.unwrap();

        // Deserialize
        let parsed: Result<SaveData, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "Deserialization should not fail for valid data");

        let loaded = parsed.unwrap();

        // Check key fields are preserved
        prop_assert_eq!(save_data.version, loaded.version);
        prop_assert_eq!(save_data.timestamp, loaded.timestamp);
        prop_assert_eq!(save_data.mode.creative, loaded.mode.creative);
        prop_assert_eq!(save_data.quests.completed, loaded.quests.completed);
    }

    /// Fuzz: Malformed JSON should not panic
    #[test]
    fn fuzz_malformed_json(random_bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
        let maybe_string = String::from_utf8(random_bytes);
        if let Ok(s) = maybe_string {
            // Should not panic, just return error
            let result: Result<SaveData, _> = serde_json::from_str(&s);
            // We expect most random strings to fail parsing, but it should not panic
            let _ = result;
        }
    }

    /// Fuzz: Truncated JSON should not panic
    #[test]
    fn fuzz_truncated_json(save_data in arb_save_data(), cut_point in 0usize..500) {
        let json = serde_json::to_string(&save_data).unwrap();
        let truncated = if cut_point < json.len() {
            &json[..cut_point]
        } else {
            &json
        };

        // Should not panic
        let result: Result<SaveData, _> = serde_json::from_str(truncated);
        let _ = result;
    }

    /// Fuzz: Extra fields should be ignored
    #[test]
    fn fuzz_extra_fields(save_data in arb_save_data()) {
        let mut json_value: serde_json::Value = serde_json::to_value(&save_data).unwrap();

        // Add random extra fields
        if let serde_json::Value::Object(ref mut map) = json_value {
            map.insert("extra_field_1".to_string(), serde_json::json!(12345));
            map.insert("unknown_thing".to_string(), serde_json::json!("random"));
            map.insert("nested_extra".to_string(), serde_json::json!({"foo": "bar"}));
        }

        let json_str = serde_json::to_string(&json_value).unwrap();
        let result: Result<SaveData, _> = serde_json::from_str(&json_str);

        // Should succeed (extra fields ignored by serde default)
        prop_assert!(result.is_ok(), "Extra fields should be ignored");
    }

    /// Fuzz: Position key parsing
    #[test]
    fn fuzz_position_key_parse(x in any::<i32>(), y in any::<i32>(), z in any::<i32>()) {
        let key = format!("{},{},{}", x, y, z);
        let parts: Vec<&str> = key.split(',').collect();

        prop_assert_eq!(parts.len(), 3);
        prop_assert_eq!(parts[0].parse::<i32>().ok(), Some(x));
        prop_assert_eq!(parts[1].parse::<i32>().ok(), Some(y));
        prop_assert_eq!(parts[2].parse::<i32>().ok(), Some(z));
    }

    /// Fuzz: Invalid position keys should not panic
    #[test]
    fn fuzz_invalid_position_key(key in ".*") {
        fn key_to_pos(key: &str) -> Option<(i32, i32, i32)> {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() != 3 {
                return None;
            }
            Some((
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                parts[2].parse().ok()?,
            ))
        }

        // Should not panic for any input
        let _ = key_to_pos(&key);
    }

    /// Fuzz: Machine save data serialization
    #[test]
    fn fuzz_machine_roundtrip(machine in arb_machine_save()) {
        let json = serde_json::to_string(&machine);
        prop_assert!(json.is_ok());

        let loaded: Result<MachineSaveData, _> = serde_json::from_str(&json.unwrap());
        prop_assert!(loaded.is_ok());
    }

    /// Fuzz: Item stack with extreme values
    #[test]
    fn fuzz_extreme_item_counts(count in prop::num::u32::ANY) {
        let stack = ItemStack {
            item_type: BlockTypeSave::Stone,
            count,
        };

        let json = serde_json::to_string(&stack).unwrap();
        let loaded: ItemStack = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(loaded.count, count);
    }

    /// Fuzz: Float edge cases in positions
    #[test]
    fn fuzz_float_positions(x in prop::num::f32::ANY, y in prop::num::f32::ANY, z in prop::num::f32::ANY) {
        let pos = Vec3Save { x, y, z };
        let json = serde_json::to_string(&pos);

        // NaN and Infinity don't serialize to valid JSON numbers in serde_json
        if x.is_finite() && y.is_finite() && z.is_finite() {
            prop_assert!(json.is_ok());
            let loaded: Vec3Save = serde_json::from_str(&json.unwrap()).unwrap();
            prop_assert_eq!(loaded.x, x);
            prop_assert_eq!(loaded.y, y);
            prop_assert_eq!(loaded.z, z);
        }
    }
}

#[test]
fn test_specific_malformed_inputs() {
    // Test specific edge cases
    let test_cases = [
        "",
        "{}",
        "null",
        "[]",
        r#"{"version": "0.1.0"}"#,
        r#"{"version": null}"#,
        r#"{"timestamp": "not a number"}"#,
        r#"{"player": {"position": {"x": "nan"}}}"#,
        "\\x00\\x00\\x00",
        "{\"",
        "}{",
        r#"{"machines": [{"type": "Unknown"}]}"#,
    ];

    for case in test_cases {
        // Should not panic
        let result: Result<SaveData, _> = serde_json::from_str(case);
        // We expect these to fail, but not panic
        assert!(
            result.is_err(),
            "Expected error for malformed input: {case}"
        );
    }
}

#[test]
fn test_large_save_data() {
    // Test with large numbers of machines
    let mut machines = Vec::new();
    for i in 0..1000 {
        machines.push(MachineSaveData::Miner(MinerSaveData {
            position: IVec3Save { x: i, y: 8, z: 0 },
            progress: 0.5,
            buffer: None,
        }));
    }

    let save_data = SaveData {
        version: "0.1.0".to_string(),
        timestamp: 0,
        player: PlayerSaveData {
            position: Vec3Save {
                x: 0.0,
                y: 10.0,
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
            modified_blocks: std::collections::HashMap::new(),
        },
        machines,
        quests: QuestSaveData {
            current_index: 0,
            completed: false,
            rewards_claimed: false,
            delivered: std::collections::HashMap::new(),
        },
        mode: GameModeSaveData { creative: false },
    };

    let json = serde_json::to_string(&save_data).unwrap();
    let loaded: SaveData = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.machines.len(), 1000);
}
