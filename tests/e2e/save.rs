//! Save/Load system tests

use bevy::prelude::*;
use idle_factory::core::items;
use idle_factory::ItemId;
use std::collections::HashMap;

// ============================================================================
// Save Tests
// ============================================================================

#[test]
fn test_save_directory_creation() {
    let save_dir = std::path::Path::new("saves");

    if save_dir.exists() {
        // Don't delete existing saves
    } else {
        std::fs::create_dir_all(save_dir).expect("Should create saves directory");
    }

    assert!(save_dir.exists() || std::fs::create_dir_all(save_dir).is_ok());
}

#[test]
fn test_save_file_json_structure() {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestSaveData {
        version: String,
        timestamp: u64,
        player: TestPlayerData,
        inventory: TestInventoryData,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestPlayerData {
        position: (f32, f32, f32),
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestInventoryData {
        selected_slot: usize,
        slots: Vec<Option<(String, u32)>>,
    }

    let data = TestSaveData {
        version: "0.1.0".to_string(),
        timestamp: 1704067200000,
        player: TestPlayerData {
            position: (8.0, 12.0, 20.0),
        },
        inventory: TestInventoryData {
            selected_slot: 0,
            slots: vec![
                Some(("Stone".to_string(), 10)),
                None,
                Some(("IronOre".to_string(), 5)),
            ],
        },
    };

    let json = serde_json::to_string_pretty(&data).expect("Should serialize");
    assert!(json.contains("version"));
    assert!(json.contains("0.1.0"));
    assert!(json.contains("player"));
    assert!(json.contains("inventory"));

    let parsed: TestSaveData = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(parsed.version, "0.1.0");
    assert_eq!(parsed.player.position.0, 8.0);
    assert_eq!(parsed.inventory.selected_slot, 0);
}

#[test]
fn test_position_key_conversion() {
    fn pos_to_key(pos: IVec3) -> String {
        format!("{},{},{}", pos.x, pos.y, pos.z)
    }

    fn key_to_pos(key: &str) -> Option<IVec3> {
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

    let pos = IVec3::new(10, 8, 20);
    let key = pos_to_key(pos);
    assert_eq!(key, "10,8,20");
    assert_eq!(key_to_pos(&key), Some(pos));

    let pos_neg = IVec3::new(-5, 0, -10);
    let key_neg = pos_to_key(pos_neg);
    assert_eq!(key_neg, "-5,0,-10");
    assert_eq!(key_to_pos(&key_neg), Some(pos_neg));

    let pos_zero = IVec3::ZERO;
    let key_zero = pos_to_key(pos_zero);
    assert_eq!(key_zero, "0,0,0");
    assert_eq!(key_to_pos(&key_zero), Some(pos_zero));

    assert_eq!(key_to_pos("invalid"), None);
    assert_eq!(key_to_pos("1,2"), None);
    assert_eq!(key_to_pos("a,b,c"), None);
}

#[test]
fn test_modified_blocks_save_load() {
    let mut modified_blocks: HashMap<IVec3, Option<ItemId>> = HashMap::new();

    modified_blocks.insert(IVec3::new(10, 8, 20), Some(items::stone()));
    modified_blocks.insert(IVec3::new(12, 8, 20), None);
    modified_blocks.insert(IVec3::new(5, 8, 5), Some(items::miner_block()));

    fn pos_to_key(pos: IVec3) -> String {
        format!("{},{},{}", pos.x, pos.y, pos.z)
    }

    let save_data: HashMap<String, Option<String>> = modified_blocks
        .iter()
        .map(|(pos, block)| {
            let key = pos_to_key(*pos);
            let value = block.and_then(|b| b.name().map(|s| s.to_string()));
            (key, value)
        })
        .collect();

    assert_eq!(save_data.len(), 3);
    assert!(save_data.contains_key("10,8,20"));
    assert!(save_data.contains_key("12,8,20"));
    assert!(save_data.contains_key("5,8,5"));

    assert_eq!(
        save_data.get("10,8,20"),
        Some(&Some("base:stone".to_string()))
    );
    assert_eq!(save_data.get("12,8,20"), Some(&None));
    assert_eq!(
        save_data.get("5,8,5"),
        Some(&Some("base:miner_block".to_string()))
    );
}

#[test]
fn test_machine_state_serialization() {
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestMiner {
        position: (i32, i32, i32),
        progress: f32,
        buffer: Option<(String, u32)>,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestConveyor {
        position: (i32, i32, i32),
        direction: String,
        items: Vec<(String, f32)>,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestFurnace {
        position: (i32, i32, i32),
        fuel: u32,
        input: Option<(String, u32)>,
        output: Option<(String, u32)>,
        progress: f32,
    }

    let miner = TestMiner {
        position: (10, 8, 10),
        progress: 0.5,
        buffer: Some(("IronOre".to_string(), 3)),
    };
    let json = serde_json::to_string(&miner).unwrap();
    let parsed: TestMiner = serde_json::from_str(&json).unwrap();
    assert_eq!(miner, parsed);

    let conveyor = TestConveyor {
        position: (11, 8, 10),
        direction: "East".to_string(),
        items: vec![("IronOre".to_string(), 0.3), ("IronOre".to_string(), 0.7)],
    };
    let json = serde_json::to_string(&conveyor).unwrap();
    let parsed: TestConveyor = serde_json::from_str(&json).unwrap();
    assert_eq!(conveyor, parsed);

    let furnace = TestFurnace {
        position: (12, 8, 10),
        fuel: 5,
        input: Some(("IronOre".to_string(), 10)),
        output: Some(("IronIngot".to_string(), 2)),
        progress: 0.75,
    };
    let json = serde_json::to_string(&furnace).unwrap();
    let parsed: TestFurnace = serde_json::from_str(&json).unwrap();
    assert_eq!(furnace, parsed);
}

#[test]
fn test_quest_progress_serialization() {
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestQuestData {
        current_index: usize,
        completed: bool,
        rewards_claimed: bool,
        delivered: HashMap<String, u32>,
    }

    let mut delivered = HashMap::new();
    delivered.insert("IronIngot".to_string(), 50);

    let quest = TestQuestData {
        current_index: 1,
        completed: false,
        rewards_claimed: false,
        delivered,
    };

    let json = serde_json::to_string(&quest).unwrap();
    let parsed: TestQuestData = serde_json::from_str(&json).unwrap();
    assert_eq!(quest, parsed);

    let quest_done = TestQuestData {
        current_index: 2,
        completed: true,
        rewards_claimed: true,
        delivered: HashMap::new(),
    };

    let json = serde_json::to_string(&quest_done).unwrap();
    let parsed: TestQuestData = serde_json::from_str(&json).unwrap();
    assert_eq!(quest_done, parsed);
}

#[test]
fn test_inventory_serialization_edge_cases() {
    const NUM_SLOTS: usize = 36;
    const MAX_STACK: u32 = 999;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestInventory {
        selected_slot: usize,
        slots: Vec<Option<(String, u32)>>,
    }

    let empty = TestInventory {
        selected_slot: 0,
        slots: vec![None; NUM_SLOTS],
    };
    let json = serde_json::to_string(&empty).unwrap();
    let parsed: TestInventory = serde_json::from_str(&json).unwrap();
    assert_eq!(empty.slots.len(), NUM_SLOTS);
    assert!(parsed.slots.iter().all(|s| s.is_none()));

    let mut full_slots = Vec::new();
    for _ in 0..NUM_SLOTS {
        full_slots.push(Some(("Stone".to_string(), MAX_STACK)));
    }
    let full = TestInventory {
        selected_slot: 8,
        slots: full_slots,
    };
    let json = serde_json::to_string(&full).unwrap();
    let parsed: TestInventory = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.selected_slot, 8);
    assert!(parsed
        .slots
        .iter()
        .all(|s| { s.as_ref().map(|(_, c)| *c == MAX_STACK).unwrap_or(false) }));
}

#[test]
fn test_auto_save_timer() {
    const AUTO_SAVE_INTERVAL: f32 = 60.0;

    struct MockTimer {
        elapsed: f32,
        duration: f32,
        just_finished: bool,
    }

    impl MockTimer {
        fn new(duration: f32) -> Self {
            Self {
                elapsed: 0.0,
                duration,
                just_finished: false,
            }
        }

        fn tick(&mut self, delta: f32) {
            self.elapsed += delta;
            self.just_finished = false;
            if self.elapsed >= self.duration {
                self.elapsed -= self.duration;
                self.just_finished = true;
            }
        }
    }

    let mut timer = MockTimer::new(AUTO_SAVE_INTERVAL);

    for _ in 0..59 {
        timer.tick(1.0);
        assert!(!timer.just_finished);
    }

    timer.tick(1.0);
    assert!(timer.just_finished);

    timer.tick(0.1);
    assert!(!timer.just_finished);

    timer.tick(60.0);
    assert!(timer.just_finished);
}

#[test]
fn test_save_command_parsing() {
    fn parse_save_command(input: &str) -> Option<String> {
        let lowered = input.trim().to_lowercase();
        let parts: Vec<&str> = lowered.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "/save" | "save" => Some(parts.get(1).unwrap_or(&"quicksave").to_string()),
            _ => None,
        }
    }

    assert_eq!(parse_save_command("/save"), Some("quicksave".to_string()));
    assert_eq!(parse_save_command("save"), Some("quicksave".to_string()));
    assert_eq!(
        parse_save_command("/save myworld"),
        Some("myworld".to_string())
    );
    assert_eq!(
        parse_save_command("/save test_save"),
        Some("test_save".to_string())
    );
    assert_eq!(parse_save_command("/creative"), None);
    assert_eq!(parse_save_command("help"), None);
}

#[test]
fn test_load_command_parsing() {
    fn parse_load_command(input: &str) -> Option<String> {
        let lowered = input.trim().to_lowercase();
        let parts: Vec<&str> = lowered.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "/load" | "load" => Some(parts.get(1).unwrap_or(&"quicksave").to_string()),
            _ => None,
        }
    }

    assert_eq!(parse_load_command("/load"), Some("quicksave".to_string()));
    assert_eq!(parse_load_command("load"), Some("quicksave".to_string()));
    assert_eq!(
        parse_load_command("/load myworld"),
        Some("myworld".to_string())
    );
    assert_eq!(
        parse_load_command("/load autosave"),
        Some("autosave".to_string())
    );
    assert_eq!(parse_load_command("/save"), None);
    assert_eq!(parse_load_command("clear"), None);
}
