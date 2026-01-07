//! Game Registry - Single Source of Truth for all game data
//!
//! All descriptors (blocks, items, machines, recipes) are registered here
//! and can be accessed via O(1) lookup.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::block_type::{BlockCategory, BlockType};

use super::machines::MachineSpec;
use super::recipes::RecipeSpec;

// =============================================================================
// Item Descriptor (unified block/item definition)
// =============================================================================

/// Unified descriptor for all blocks and items
#[derive(Debug, Clone)]
pub struct ItemDescriptor {
    /// Display name
    pub name: &'static str,
    /// Short name for UI (max 4 chars)
    pub short_name: &'static str,
    /// Display color
    pub color: Color,
    /// Category
    pub category: BlockCategory,
    /// Max stack size (1 for tools, 999 for materials)
    pub stack_size: u32,
    /// Can be placed in world
    pub is_placeable: bool,
    /// Base break time in seconds (before tool multiplier)
    pub hardness: f32,
    /// What this block drops when broken (None = drops itself)
    pub drops: Option<BlockType>,
}

impl ItemDescriptor {
    pub const fn new(
        name: &'static str,
        short_name: &'static str,
        color: (f32, f32, f32),
        category: BlockCategory,
        stack_size: u32,
        is_placeable: bool,
    ) -> Self {
        Self {
            name,
            short_name,
            color: Color::srgb(color.0, color.1, color.2),
            category,
            stack_size,
            is_placeable,
            hardness: 1.0, // Default hardness
            drops: None,   // Default: drops itself
        }
    }

    /// Create with custom hardness
    pub const fn with_hardness(mut self, hardness: f32) -> Self {
        self.hardness = hardness;
        self
    }

    /// Create with custom drops
    pub const fn with_drops(mut self, drops: BlockType) -> Self {
        self.drops = Some(drops);
        self
    }

    /// Get what this block drops (self if None)
    pub fn get_drops(&self, block_type: BlockType) -> BlockType {
        self.drops.unwrap_or(block_type)
    }
}

// =============================================================================
// Static Item Definitions
// =============================================================================

/// All item descriptors (indexed by BlockType)
/// Hardness values: 1.0 = terrain, 0.5 = machines, 0.0 = instant
pub const ITEM_DESCRIPTORS: &[(BlockType, ItemDescriptor)] = &[
    // Terrain (hardness 1.0)
    (
        BlockType::Stone,
        ItemDescriptor::new(
            "Stone",
            "Stn",
            (0.5, 0.5, 0.5),
            BlockCategory::Terrain,
            999,
            true,
        )
        .with_hardness(1.0),
    ),
    (
        BlockType::Grass,
        ItemDescriptor::new(
            "Grass",
            "Grs",
            (0.2, 0.8, 0.2),
            BlockCategory::Terrain,
            999,
            true,
        )
        .with_hardness(0.8),
    ),
    // Ores (hardness 1.2 - slightly harder than stone)
    (
        BlockType::IronOre,
        ItemDescriptor::new(
            "Iron Ore",
            "FeO",
            (0.6, 0.5, 0.4),
            BlockCategory::Ore,
            999,
            true,
        )
        .with_hardness(1.2),
    ),
    (
        BlockType::CopperOre,
        ItemDescriptor::new(
            "Copper Ore",
            "CuO",
            (0.7, 0.4, 0.3),
            BlockCategory::Ore,
            999,
            true,
        )
        .with_hardness(1.2),
    ),
    (
        BlockType::Coal,
        ItemDescriptor::new(
            "Coal",
            "C",
            (0.15, 0.15, 0.15),
            BlockCategory::Ore,
            999,
            true,
        )
        .with_hardness(1.0),
    ),
    // Processed (not placeable, no hardness needed)
    (
        BlockType::IronIngot,
        ItemDescriptor::new(
            "Iron Ingot",
            "Fe",
            (0.8, 0.8, 0.85),
            BlockCategory::Processed,
            999,
            false,
        ),
    ),
    (
        BlockType::CopperIngot,
        ItemDescriptor::new(
            "Copper Ingot",
            "Cu",
            (0.9, 0.5, 0.3),
            BlockCategory::Processed,
            999,
            false,
        ),
    ),
    (
        BlockType::IronDust,
        ItemDescriptor::new(
            "Iron Dust",
            "FeD",
            (0.7, 0.7, 0.75),
            BlockCategory::Processed,
            999,
            false,
        ),
    ),
    (
        BlockType::CopperDust,
        ItemDescriptor::new(
            "Copper Dust",
            "CuD",
            (0.85, 0.55, 0.4),
            BlockCategory::Processed,
            999,
            false,
        ),
    ),
    // Machines (hardness 0.5 - easier to break)
    (
        BlockType::MinerBlock,
        ItemDescriptor::new(
            "Miner",
            "Min",
            (0.8, 0.6, 0.2),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.5),
    ),
    (
        BlockType::ConveyorBlock,
        ItemDescriptor::new(
            "Conveyor",
            "Conv",
            (0.3, 0.3, 0.35),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.3),
    ),
    (
        BlockType::FurnaceBlock,
        ItemDescriptor::new(
            "Furnace",
            "Fur",
            (0.4, 0.3, 0.3),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.5),
    ),
    (
        BlockType::CrusherBlock,
        ItemDescriptor::new(
            "Crusher",
            "Cru",
            (0.4, 0.3, 0.5),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.5),
    ),
    (
        BlockType::AssemblerBlock,
        ItemDescriptor::new(
            "Assembler",
            "Asm",
            (0.3, 0.5, 0.4),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.5),
    ),
    (
        BlockType::PlatformBlock,
        ItemDescriptor::new(
            "Platform",
            "Plt",
            (0.2, 0.5, 0.3),
            BlockCategory::Machine,
            999,
            true,
        )
        .with_hardness(0.5),
    ),
    // Tools (not placeable)
    (
        BlockType::StonePickaxe,
        ItemDescriptor::new(
            "Stone Pickaxe",
            "Pick",
            (0.6, 0.6, 0.6),
            BlockCategory::Tool,
            1,
            false,
        ),
    ),
];

// =============================================================================
// Game Registry (Bevy Resource)
// =============================================================================

/// Central registry for all game data
#[derive(Resource)]
pub struct GameRegistry {
    items: HashMap<BlockType, &'static ItemDescriptor>,
    machines: HashMap<BlockType, &'static MachineSpec>,
    recipes: Vec<&'static RecipeSpec>,
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GameRegistry {
    /// Create a new registry with all static data
    pub fn new() -> Self {
        let mut items = HashMap::new();
        for (block_type, descriptor) in ITEM_DESCRIPTORS {
            items.insert(*block_type, descriptor);
        }

        let mut machines = HashMap::new();
        for spec in super::machines::ALL_MACHINES {
            machines.insert(spec.block_type, *spec);
        }

        let recipes = super::recipes::ALL_RECIPES.to_vec();

        Self {
            items,
            machines,
            recipes,
        }
    }

    /// Get item descriptor by BlockType
    pub fn item(&self, block_type: BlockType) -> Option<&ItemDescriptor> {
        self.items.get(&block_type).copied()
    }

    /// Get machine spec by BlockType
    pub fn machine(&self, block_type: BlockType) -> Option<&MachineSpec> {
        self.machines.get(&block_type).copied()
    }

    /// Get all recipes
    pub fn recipes(&self) -> &[&'static RecipeSpec] {
        &self.recipes
    }

    /// Check if a BlockType is registered
    pub fn is_registered(&self, block_type: BlockType) -> bool {
        self.items.contains_key(&block_type)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameRegistry>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_all_block_types_registered() {
        let registry = GameRegistry::new();
        for block_type in BlockType::iter() {
            assert!(
                registry.is_registered(block_type),
                "BlockType::{:?} is not registered in ITEM_DESCRIPTORS",
                block_type
            );
        }
    }

    #[test]
    fn test_item_lookup() {
        let registry = GameRegistry::new();

        let stone = registry.item(BlockType::Stone).unwrap();
        assert_eq!(stone.name, "Stone");
        assert_eq!(stone.category, BlockCategory::Terrain);

        let iron_ingot = registry.item(BlockType::IronIngot).unwrap();
        assert_eq!(iron_ingot.name, "Iron Ingot");
        assert!(!iron_ingot.is_placeable);
    }

    #[test]
    fn test_machine_lookup() {
        let registry = GameRegistry::new();

        let miner = registry.machine(BlockType::MinerBlock);
        assert!(miner.is_some());
        assert_eq!(miner.unwrap().id, "miner");

        let furnace = registry.machine(BlockType::FurnaceBlock);
        assert!(furnace.is_some());
        assert!(furnace.unwrap().requires_fuel);

        // Non-machine should return None
        let stone = registry.machine(BlockType::Stone);
        assert!(stone.is_none());
    }

    #[test]
    fn test_stack_sizes() {
        let registry = GameRegistry::new();

        // Tools have stack size 1
        let pickaxe = registry.item(BlockType::StonePickaxe).unwrap();
        assert_eq!(pickaxe.stack_size, 1);

        // Materials have stack size 999
        let iron = registry.item(BlockType::IronIngot).unwrap();
        assert_eq!(iron.stack_size, 999);
    }

    #[test]
    fn test_recipes_loaded() {
        let registry = GameRegistry::new();
        assert!(!registry.recipes().is_empty());
    }

    #[test]
    fn test_hardness_values() {
        let registry = GameRegistry::new();

        // Machines have lower hardness (easier to break)
        let miner = registry.item(BlockType::MinerBlock).unwrap();
        assert_eq!(miner.hardness, 0.5);

        let conveyor = registry.item(BlockType::ConveyorBlock).unwrap();
        assert!(conveyor.hardness < 0.5); // Conveyor is even easier

        // Ores are harder than stone
        let iron_ore = registry.item(BlockType::IronOre).unwrap();
        let stone = registry.item(BlockType::Stone).unwrap();
        assert!(iron_ore.hardness > stone.hardness);
    }

    #[test]
    fn test_block_type_hardness_method() {
        // Test that BlockType.hardness() delegates to ItemDescriptor
        assert_eq!(BlockType::Stone.hardness(), 1.0);
        assert_eq!(BlockType::MinerBlock.hardness(), 0.5);
        assert!(BlockType::IronOre.hardness() > BlockType::Stone.hardness());
    }

    #[test]
    fn test_block_drops() {
        // Most blocks drop themselves
        assert_eq!(BlockType::Stone.drops(), BlockType::Stone);
        assert_eq!(BlockType::IronOre.drops(), BlockType::IronOre);
        assert_eq!(BlockType::MinerBlock.drops(), BlockType::MinerBlock);
    }
}
