//! Game specification as code
//!
//! This file is the Single Source of Truth for game design.
//! If you change the spec, update this file. Tests will verify implementation matches.

pub mod machines;
pub mod recipes;
pub mod registry;

// Re-exports for convenience
pub use machines::{
    get_input_ports, get_machine_spec, get_machine_spec_by_id, get_output_ports, IoPort,
    MachineSpec, MachineState, PortSide, ProcessType, UiSlotDef, UiSlotType, ALL_MACHINES,
    ASSEMBLER, CRUSHER, FURNACE, MINER,
};
pub use recipes::{
    all_recipes, find_recipe, find_recipe_by_id, get_recipes_for_machine, FuelRequirement,
    MachineType, Recipe, RecipeInput, RecipeOutput,
};
pub use registry::{GameRegistry, ItemDescriptor, RegistryPlugin, ITEM_DESCRIPTORS};

use crate::block_type::BlockType;
use crate::core::{items, ItemId};
use std::sync::LazyLock;

// =============================================================================
// v0.2 New System Specs
// =============================================================================

/// Global Inventory System
#[allow(dead_code)]
pub mod global_inventory_spec {
    pub const STORAGE_LIMIT: u32 = 0;
    pub const RETURN_ON_DEMOLISH: bool = true;
    pub const CAN_PICKUP_FROM_CONVEYOR: bool = false;
}

/// Delivery Platform Spec
#[allow(dead_code)]
pub mod delivery_platform_spec {
    pub const INITIAL_COUNT: u32 = 1;
    pub const CAN_CRAFT_MORE: bool = true;
    pub const SHARE_INVENTORY: bool = true;
    pub const AUTO_DELIVER_ENABLED: bool = false;
}

/// Assembler Spec (synced with machines.rs ASSEMBLER definition)
#[allow(dead_code)]
pub mod assembler_spec {
    /// Output buffer size (matches ASSEMBLER.buffer_size)
    pub const OUTPUT_BUFFER_SIZE: u32 = 32;
    /// Base crafting time - actual time comes from recipe.craft_time
    pub const CRAFT_TIME_BASE: f32 = 3.0;
}

/// Quest System Spec
#[allow(dead_code)]
pub mod quest_system_spec {
    pub const MAX_ACTIVE_SUB_QUESTS: u32 = 5;
    pub const SUB_QUEST_AUTO_DELIVER: bool = true;
}

/// UI Spec
#[allow(dead_code)]
pub mod ui_spec {
    pub const INVENTORY_COLUMNS: u32 = 8;
    pub const INVENTORY_ROWS_PER_PAGE: u32 = 4;
    pub const CATEGORIES: &[&str] = &["全て", "素材", "機械", "部品"];
}

/// Block Breaking Spec
pub mod breaking_spec {
    use crate::block_type::BlockType;
    use crate::core::{items, ItemId};

    /// Multiplier when breaking with bare hands (slower)
    pub const BARE_HAND_MULTIPLIER: f32 = 2.0;
    /// Multiplier when breaking with stone pickaxe (normal speed)
    pub const STONE_PICKAXE_MULTIPLIER: f32 = 1.0;

    /// Get base break time from ItemDescriptor.hardness (via ItemId)
    /// This is now data-driven - each item has its own hardness value
    pub fn get_base_break_time(item_id: ItemId) -> f32 {
        // Convert to BlockType to access hardness (temporary until full migration)
        BlockType::try_from(item_id)
            .map(|bt| bt.hardness())
            .unwrap_or(1.0) // Default hardness for unknown items
    }

    /// Get tool multiplier (affects break time)
    pub fn get_tool_multiplier(tool: Option<ItemId>) -> f32 {
        match tool {
            Some(t) if t == items::stone_pickaxe() => STONE_PICKAXE_MULTIPLIER,
            _ => BARE_HAND_MULTIPLIER,
        }
    }
}

/// Biome Mining Spec
#[allow(dead_code)]
pub mod biome_mining_spec {
    use crate::BlockType;

    pub type MiningProbability = (BlockType, u32);

    pub const IRON_BIOME: &[MiningProbability] = &[
        (BlockType::IronOre, 70),
        (BlockType::Stone, 22),
        (BlockType::Coal, 8),
    ];

    pub const COPPER_BIOME: &[MiningProbability] = &[
        (BlockType::CopperOre, 70),
        (BlockType::Stone, 22),
        (BlockType::IronOre, 8),
    ];

    pub const COAL_BIOME: &[MiningProbability] = &[
        (BlockType::Coal, 75),
        (BlockType::Stone, 20),
        (BlockType::IronOre, 5),
    ];

    pub const STONE_BIOME: &[MiningProbability] = &[
        (BlockType::Stone, 85),
        (BlockType::Coal, 10),
        (BlockType::IronOre, 5),
    ];

    pub const MIXED_BIOME: &[MiningProbability] = &[
        (BlockType::IronOre, 30),
        (BlockType::CopperOre, 25),
        (BlockType::Coal, 25),
        (BlockType::Stone, 20),
    ];

    pub const SPAWN_GUARANTEE_RADIUS: u32 = 10;
    pub const GUARANTEED_SPAWN_BIOMES: &[&str] = &["iron", "coal", "copper"];
    pub const UNMINEABLE_BIOMES: &[&str] = &["ocean", "lava", "void"];
}

// =============================================================================
// Initial Equipment
// =============================================================================

/// Initial equipment for new players
pub fn initial_equipment() -> Vec<(ItemId, u32)> {
    vec![
        (items::stone_pickaxe(), 1),
        (items::miner_block(), 2),
        (items::conveyor_block(), 90),
        (items::furnace_block(), 1),
    ]
}

/// Creative mode equipment (for debug/testing)
#[allow(dead_code)]
pub const CREATIVE_MODE_EQUIPMENT: &[(BlockType, u32)] = &[
    (BlockType::MinerBlock, 99),
    (BlockType::ConveyorBlock, 999),
    (BlockType::CrusherBlock, 99),
    (BlockType::FurnaceBlock, 99),
    (BlockType::IronOre, 999),
    (BlockType::Coal, 999),
    (BlockType::CopperOre, 999),
];

// =============================================================================
// Quest System (ItemId-based with LazyLock)
// =============================================================================

/// Quest type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestType {
    Main,
    Sub,
}

/// Quest definition (ItemId-based)
#[derive(Clone, Debug)]
pub struct Quest {
    pub id: &'static str,
    pub quest_type: QuestType,
    pub description: &'static str,
    pub required_items: Vec<(ItemId, u32)>,
    pub rewards: Vec<(ItemId, u32)>,
    pub unlocks: Vec<ItemId>,
}

impl Quest {
    /// Get required items (already ItemId)
    pub fn required_items_id(&self) -> &[(ItemId, u32)] {
        &self.required_items
    }

    /// Get rewards (already ItemId)
    pub fn rewards_id(&self) -> &[(ItemId, u32)] {
        &self.rewards
    }

    /// Get unlocks (already ItemId)
    pub fn unlocks_id(&self) -> &[ItemId] {
        &self.unlocks
    }
}

/// Main quests (LazyLock for runtime initialization with ItemId)
static MAIN_QUESTS: LazyLock<Vec<Quest>> = LazyLock::new(|| {
    vec![
        Quest {
            id: "main_1",
            quest_type: QuestType::Main,
            description: "鉄インゴットを10個納品せよ",
            required_items: vec![(items::iron_ingot(), 10)],
            rewards: vec![(items::assembler_block(), 1), (items::conveyor_block(), 20)],
            unlocks: vec![items::assembler_block()], // Unlock Assembler (machine crafting)
        },
        Quest {
            id: "main_2",
            quest_type: QuestType::Main,
            description: "銅インゴットを30個納品せよ",
            required_items: vec![(items::copper_ingot(), 30)],
            rewards: vec![
                (items::crusher_block(), 2),
                (items::furnace_block(), 1), // Extra furnace for parallel production
            ],
            unlocks: vec![items::crusher_block()], // Unlock Crusher (ore doubling)
        },
        Quest {
            id: "main_3",
            quest_type: QuestType::Main,
            description: "鉄インゴット100個を納品せよ",
            required_items: vec![(items::iron_ingot(), 100)],
            rewards: vec![
                (items::miner_block(), 4),
                (items::conveyor_block(), 50),
                (items::furnace_block(), 2),
            ],
            unlocks: vec![], // No new unlocks - player can now craft everything
        },
    ]
});

/// Sub quests (rewards are now more useful - machines instead of raw resources)
static SUB_QUESTS: LazyLock<Vec<Quest>> = LazyLock::new(|| {
    vec![
        Quest {
            id: "sub_iron_100",
            quest_type: QuestType::Sub,
            description: "鉄インゴット100個を納品",
            required_items: vec![(items::iron_ingot(), 100)],
            rewards: vec![(items::miner_block(), 2), (items::conveyor_block(), 30)],
            unlocks: vec![],
        },
        Quest {
            id: "sub_copper_100",
            quest_type: QuestType::Sub,
            description: "銅インゴット100個を納品",
            required_items: vec![(items::copper_ingot(), 100)],
            rewards: vec![(items::furnace_block(), 2), (items::conveyor_block(), 30)],
            unlocks: vec![],
        },
        Quest {
            id: "sub_coal_200",
            quest_type: QuestType::Sub,
            description: "石炭200個を納品",
            required_items: vec![(items::coal(), 200)],
            rewards: vec![(items::crusher_block(), 1)],
            unlocks: vec![],
        },
    ]
});

/// Get all main quests
pub fn main_quests() -> &'static [Quest] {
    &MAIN_QUESTS
}

/// Get all sub quests
pub fn sub_quests() -> &'static [Quest] {
    &SUB_QUESTS
}

/// Find quest by ID
pub fn find_quest(id: &str) -> Option<&'static Quest> {
    MAIN_QUESTS
        .iter()
        .chain(SUB_QUESTS.iter())
        .find(|q| q.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_quest_progression() {
        let quests = main_quests();
        let q1_total: u32 = quests[0].required_items.iter().map(|(_, n)| n).sum();
        assert!(q1_total <= 20, "Quest 1 should be easy for early game");

        // First two quests should unlock new mechanics
        assert!(
            !quests[0].unlocks.is_empty(),
            "Quest 1 should unlock Assembler"
        );
        assert!(
            !quests[1].unlocks.is_empty(),
            "Quest 2 should unlock Crusher"
        );
        // Quest 3 doesn't need to unlock anything - player can craft all machines now
    }

    #[test]
    fn test_quest_rewards_not_empty() {
        for quest in main_quests().iter().chain(sub_quests().iter()) {
            assert!(
                !quest.rewards.is_empty(),
                "Quest {} should have rewards",
                quest.id
            );
        }
    }

    #[test]
    fn test_initial_equipment_not_empty() {
        let equipment = initial_equipment();
        assert!(!equipment.is_empty());
        assert_eq!(equipment.len(), 4); // StonePickaxe, Miner, Conveyor, Furnace
    }

    #[test]
    fn test_spec_constants() {
        assert_eq!(global_inventory_spec::STORAGE_LIMIT, 0);
        assert!(global_inventory_spec::RETURN_ON_DEMOLISH);
        assert_eq!(delivery_platform_spec::INITIAL_COUNT, 1);
        assert!(quest_system_spec::MAX_ACTIVE_SUB_QUESTS >= 3);
    }

    #[test]
    fn test_find_quest() {
        let quest = find_quest("main_1");
        assert!(quest.is_some());
        assert_eq!(quest.unwrap().quest_type, QuestType::Main);

        let sub_quest = find_quest("sub_iron_100");
        assert!(sub_quest.is_some());
        assert_eq!(sub_quest.unwrap().quest_type, QuestType::Sub);

        let not_found = find_quest("nonexistent");
        assert!(not_found.is_none());
    }
}
