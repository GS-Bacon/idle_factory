//! Game specification as code
//!
//! This file is the Single Source of Truth for game design.
//! If you change the spec, update this file. Tests will verify implementation matches.

pub mod machines;
pub mod recipes;
pub mod registry;

// Re-exports for convenience
pub use machines::{
    get_input_ports, get_machine_spec, get_output_ports, IoPort, MachineSpec, MachineState,
    PortSide, ProcessType, UiSlotDef, UiSlotType, ALL_MACHINES, ASSEMBLER, CRUSHER, FURNACE, MINER,
};
#[allow(deprecated)]
pub use recipes::{
    find_recipe, find_recipe_by_id, get_recipes_for_machine, FuelRequirement, MachineType,
    RecipeInput, RecipeOutput, RecipeSpec, ALL_RECIPES, FURNACE_RECIPES, RECIPE_SMELT_COPPER,
    RECIPE_SMELT_IRON,
};
pub use registry::{GameRegistry, ItemDescriptor, RegistryPlugin, ITEM_DESCRIPTORS};

use crate::block_type::BlockType;
use crate::core::{items, ItemId};

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

/// Initial equipment (added to global inventory)
/// Deprecated: Use `initial_equipment_by_id()` instead for new code.
#[deprecated(since = "0.4.0", note = "Use initial_equipment_by_id() instead")]
pub const INITIAL_EQUIPMENT: &[(BlockType, u32)] = &[
    (BlockType::StonePickaxe, 1),
    (BlockType::MinerBlock, 2),
    (BlockType::ConveyorBlock, 90),
    (BlockType::FurnaceBlock, 1),
];

/// Initial equipment as ItemId (preferred for new code)
pub fn initial_equipment_by_id() -> Vec<(ItemId, u32)> {
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
// Quest System
// =============================================================================

/// Quest type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestType {
    Main,
    Sub,
}

/// Quest definition
pub struct QuestSpec {
    pub id: &'static str,
    #[allow(dead_code)]
    pub quest_type: QuestType,
    pub description: &'static str,
    pub required_items: &'static [(BlockType, u32)],
    pub rewards: &'static [(BlockType, u32)],
    pub unlocks: &'static [BlockType],
}

/// Main quests
pub const MAIN_QUESTS: &[QuestSpec] = &[
    QuestSpec {
        id: "main_1",
        quest_type: QuestType::Main,
        description: "鉄インゴットを10個納品せよ",
        required_items: &[(BlockType::IronIngot, 10)],
        rewards: &[
            (BlockType::AssemblerBlock, 1),
            (BlockType::ConveyorBlock, 20),
        ],
        unlocks: &[BlockType::AssemblerBlock], // Unlock Assembler (machine crafting)
    },
    QuestSpec {
        id: "main_2",
        quest_type: QuestType::Main,
        description: "銅インゴットを30個納品せよ",
        required_items: &[(BlockType::CopperIngot, 30)],
        rewards: &[
            (BlockType::CrusherBlock, 2),
            (BlockType::FurnaceBlock, 1), // Extra furnace for parallel production
        ],
        unlocks: &[BlockType::CrusherBlock], // Unlock Crusher (ore doubling)
    },
    QuestSpec {
        id: "main_3",
        quest_type: QuestType::Main,
        description: "鉄インゴット100個を納品せよ",
        required_items: &[(BlockType::IronIngot, 100)],
        rewards: &[
            (BlockType::MinerBlock, 4),
            (BlockType::ConveyorBlock, 50),
            (BlockType::FurnaceBlock, 2),
        ],
        unlocks: &[], // No new unlocks - player can now craft everything
    },
];

/// Sub quests (rewards are now more useful - machines instead of raw resources)
pub const SUB_QUESTS: &[QuestSpec] = &[
    QuestSpec {
        id: "sub_iron_100",
        quest_type: QuestType::Sub,
        description: "鉄インゴット100個を納品",
        required_items: &[(BlockType::IronIngot, 100)],
        rewards: &[(BlockType::MinerBlock, 2), (BlockType::ConveyorBlock, 30)],
        unlocks: &[],
    },
    QuestSpec {
        id: "sub_copper_100",
        quest_type: QuestType::Sub,
        description: "銅インゴット100個を納品",
        required_items: &[(BlockType::CopperIngot, 100)],
        rewards: &[(BlockType::FurnaceBlock, 2), (BlockType::ConveyorBlock, 30)],
        unlocks: &[],
    },
    QuestSpec {
        id: "sub_coal_200",
        quest_type: QuestType::Sub,
        description: "石炭200個を納品",
        required_items: &[(BlockType::Coal, 200)],
        rewards: &[(BlockType::CrusherBlock, 1)],
        unlocks: &[],
    },
];

#[allow(dead_code)]
#[deprecated(note = "Use MAIN_QUESTS instead")]
pub const QUESTS: &[QuestSpec] = MAIN_QUESTS;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_quest_progression() {
        let q1_total: u32 = MAIN_QUESTS[0].required_items.iter().map(|(_, n)| n).sum();
        assert!(q1_total <= 20, "Quest 1 should be easy for early game");

        // First two quests should unlock new mechanics
        assert!(
            !MAIN_QUESTS[0].unlocks.is_empty(),
            "Quest 1 should unlock Assembler"
        );
        assert!(
            !MAIN_QUESTS[1].unlocks.is_empty(),
            "Quest 2 should unlock Crusher"
        );
        // Quest 3 doesn't need to unlock anything - player can craft all machines now
    }

    #[test]
    fn test_quest_rewards_not_empty() {
        for quest in MAIN_QUESTS.iter().chain(SUB_QUESTS.iter()) {
            assert!(
                !quest.rewards.is_empty(),
                "Quest {} should have rewards",
                quest.id
            );
        }
    }

    #[test]
    fn test_initial_equipment_not_empty() {
        #[allow(deprecated)]
        let legacy_equipment = INITIAL_EQUIPMENT;
        assert!(!legacy_equipment.is_empty());

        // Test new ItemId-based function
        let equipment = initial_equipment_by_id();
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
}
