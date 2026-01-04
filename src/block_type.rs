//! Block type definitions
//!
//! BlockType is the core enum for all items in the game.
//! Each block type belongs to a category for organization and behavior.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

// =============================================================================
// Block Categories
// =============================================================================

/// Category of a block type for organization and behavior
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum BlockCategory {
    /// Natural terrain blocks (Stone, Grass)
    Terrain,
    /// Raw ore blocks (IronOre, CopperOre, Coal)
    Ore,
    /// Machine blocks that can be placed and interacted with
    Machine,
    /// Processed materials (ingots, dust)
    Processed,
    /// Tools and equipment
    Tool,
}

impl BlockCategory {
    /// Get display name for this category
    pub fn name(&self) -> &'static str {
        match self {
            BlockCategory::Terrain => "地形",
            BlockCategory::Ore => "鉱石",
            BlockCategory::Machine => "機械",
            BlockCategory::Processed => "素材",
            BlockCategory::Tool => "道具",
        }
    }

    /// Check if items in this category can be placed in the world
    pub fn is_placeable(&self) -> bool {
        matches!(
            self,
            BlockCategory::Terrain | BlockCategory::Ore | BlockCategory::Machine
        )
    }

    /// Check if items in this category are consumable materials
    pub fn is_material(&self) -> bool {
        matches!(self, BlockCategory::Ore | BlockCategory::Processed)
    }
}

/// Types of blocks in the game
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Default,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    EnumIter,
)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum BlockType {
    #[default]
    Stone,
    Grass,
    #[strum(serialize = "iron_ore", serialize = "ironore", serialize = "iron")]
    IronOre,
    Coal,
    #[strum(serialize = "iron_ingot", serialize = "ironingot")]
    IronIngot,
    #[strum(
        serialize = "miner_block",
        serialize = "miner",
        serialize = "minerblock"
    )]
    MinerBlock,
    #[strum(
        serialize = "conveyor_block",
        serialize = "conveyor",
        serialize = "conveyorblock"
    )]
    ConveyorBlock,
    #[strum(
        serialize = "copper_ore",
        serialize = "copperore",
        serialize = "copper"
    )]
    CopperOre,
    #[strum(serialize = "copper_ingot", serialize = "copperingot")]
    CopperIngot,
    #[strum(
        serialize = "crusher_block",
        serialize = "crusher",
        serialize = "crusherblock"
    )]
    CrusherBlock,
    #[strum(
        serialize = "furnace_block",
        serialize = "furnace",
        serialize = "furnaceblock"
    )]
    FurnaceBlock,
    #[strum(
        serialize = "stone_pickaxe",
        serialize = "pickaxe",
        serialize = "stonepickaxe"
    )]
    StonePickaxe,
    #[strum(
        serialize = "assembler_block",
        serialize = "assembler",
        serialize = "assemblerblock"
    )]
    AssemblerBlock,
    #[strum(serialize = "iron_dust", serialize = "irondust")]
    IronDust,
    #[strum(serialize = "copper_dust", serialize = "copperdust")]
    CopperDust,
    #[strum(
        serialize = "platform_block",
        serialize = "platform",
        serialize = "platformblock"
    )]
    PlatformBlock,
}

impl BlockType {
    /// Get the color for this block type
    pub fn color(&self) -> Color {
        match self {
            BlockType::Stone => Color::srgb(0.5, 0.5, 0.5),
            BlockType::Grass => Color::srgb(0.2, 0.8, 0.2),
            BlockType::IronOre => Color::srgb(0.6, 0.5, 0.4),
            BlockType::Coal => Color::srgb(0.15, 0.15, 0.15),
            BlockType::IronIngot => Color::srgb(0.8, 0.8, 0.85),
            BlockType::MinerBlock => Color::srgb(0.8, 0.6, 0.2),
            BlockType::ConveyorBlock => Color::srgb(0.3, 0.3, 0.35),
            BlockType::CopperOre => Color::srgb(0.7, 0.4, 0.3),
            BlockType::CopperIngot => Color::srgb(0.9, 0.5, 0.3),
            BlockType::CrusherBlock => Color::srgb(0.4, 0.3, 0.5),
            BlockType::FurnaceBlock => Color::srgb(0.4, 0.3, 0.3),
            BlockType::StonePickaxe => Color::srgb(0.6, 0.6, 0.6),
            BlockType::AssemblerBlock => Color::srgb(0.3, 0.5, 0.4),
            BlockType::IronDust => Color::srgb(0.7, 0.7, 0.75),
            BlockType::CopperDust => Color::srgb(0.85, 0.55, 0.4),
            BlockType::PlatformBlock => Color::srgb(0.2, 0.5, 0.3),
        }
    }

    /// Get the display name for this block type
    pub fn name(&self) -> &'static str {
        match self {
            BlockType::Stone => "Stone",
            BlockType::Grass => "Grass",
            BlockType::IronOre => "Iron Ore",
            BlockType::Coal => "Coal",
            BlockType::IronIngot => "Iron Ingot",
            BlockType::MinerBlock => "Miner",
            BlockType::ConveyorBlock => "Conveyor",
            BlockType::CopperOre => "Copper Ore",
            BlockType::CopperIngot => "Copper Ingot",
            BlockType::CrusherBlock => "Crusher",
            BlockType::FurnaceBlock => "Furnace",
            BlockType::StonePickaxe => "Stone Pickaxe",
            BlockType::AssemblerBlock => "Assembler",
            BlockType::IronDust => "Iron Dust",
            BlockType::CopperDust => "Copper Dust",
            BlockType::PlatformBlock => "Platform",
        }
    }

    /// Get a short name for UI display (max 4 chars)
    pub fn short_name(&self) -> &'static str {
        match self {
            BlockType::Stone => "Stn",
            BlockType::Grass => "Grs",
            BlockType::IronOre => "FeO",
            BlockType::Coal => "C",
            BlockType::IronIngot => "Fe",
            BlockType::MinerBlock => "Min",
            BlockType::ConveyorBlock => "Conv",
            BlockType::CopperOre => "CuO",
            BlockType::CopperIngot => "Cu",
            BlockType::CrusherBlock => "Cru",
            BlockType::FurnaceBlock => "Fur",
            BlockType::StonePickaxe => "Pick",
            BlockType::AssemblerBlock => "Asm",
            BlockType::IronDust => "FeD",
            BlockType::CopperDust => "CuD",
            BlockType::PlatformBlock => "Plt",
        }
    }

    /// Returns true if this block type is a tool
    pub fn is_tool(&self) -> bool {
        matches!(self, BlockType::StonePickaxe)
    }

    /// Returns true if this block type can be placed in the world
    /// Tools and processed materials cannot be placed
    pub fn is_placeable(&self) -> bool {
        !matches!(
            self,
            BlockType::StonePickaxe
                | BlockType::IronIngot
                | BlockType::CopperIngot
                | BlockType::IronDust
                | BlockType::CopperDust
        )
    }

    /// Returns true if this block type is a machine (not a regular block)
    #[allow(dead_code)]
    pub fn is_machine(&self) -> bool {
        matches!(
            self,
            BlockType::MinerBlock
                | BlockType::ConveyorBlock
                | BlockType::CrusherBlock
                | BlockType::FurnaceBlock
                | BlockType::AssemblerBlock
        )
    }

    /// Returns true if this block type is a raw ore
    pub fn is_ore(&self) -> bool {
        matches!(
            self,
            BlockType::IronOre | BlockType::CopperOre | BlockType::Coal
        )
    }

    /// Returns true if this block type is a processed material
    pub fn is_ingot(&self) -> bool {
        matches!(self, BlockType::IronIngot | BlockType::CopperIngot)
    }

    /// Get the smelted result for this ore (if any)
    pub fn smelt_result(&self) -> Option<BlockType> {
        match self {
            BlockType::IronOre => Some(BlockType::IronIngot),
            BlockType::CopperOre => Some(BlockType::CopperIngot),
            BlockType::IronDust => Some(BlockType::IronIngot),
            BlockType::CopperDust => Some(BlockType::CopperIngot),
            _ => None,
        }
    }

    /// Returns true if this block type is dust (crushed ore)
    pub fn is_dust(&self) -> bool {
        matches!(self, BlockType::IronDust | BlockType::CopperDust)
    }

    /// Get the category of this block type
    pub fn category(&self) -> BlockCategory {
        match self {
            // Terrain
            BlockType::Stone | BlockType::Grass => BlockCategory::Terrain,
            // Ores
            BlockType::IronOre | BlockType::CopperOre | BlockType::Coal => BlockCategory::Ore,
            // Machines
            BlockType::MinerBlock
            | BlockType::ConveyorBlock
            | BlockType::CrusherBlock
            | BlockType::FurnaceBlock
            | BlockType::AssemblerBlock
            | BlockType::PlatformBlock => BlockCategory::Machine,
            // Processed materials
            BlockType::IronIngot
            | BlockType::CopperIngot
            | BlockType::IronDust
            | BlockType::CopperDust => BlockCategory::Processed,
            // Tools
            BlockType::StonePickaxe => BlockCategory::Tool,
        }
    }

    /// Returns true if this block type is in the given category
    pub fn is_category(&self, category: BlockCategory) -> bool {
        self.category() == category
    }

    /// Get all block types in a given category
    pub fn all_in_category(category: BlockCategory) -> Vec<BlockType> {
        use strum::IntoEnumIterator;
        BlockType::iter()
            .filter(|bt| bt.category() == category)
            .collect()
    }

    /// Convert to a "game item" representation (for future inventory system)
    /// This returns the same BlockType for now, but provides a semantic distinction
    pub fn as_item(&self) -> BlockType {
        *self
    }

    /// Get the base material for crafted items
    /// For example: IronIngot -> IronOre, CopperDust -> CopperOre
    pub fn base_ore(&self) -> Option<BlockType> {
        match self {
            BlockType::IronIngot | BlockType::IronDust => Some(BlockType::IronOre),
            BlockType::CopperIngot | BlockType::CopperDust => Some(BlockType::CopperOre),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use strum::IntoEnumIterator;

    #[test]
    fn test_block_type_color_returns_valid() {
        // Use EnumIter to iterate over all block types
        for bt in BlockType::iter() {
            let color = bt.color();
            let srgba = color.to_srgba();
            assert!(srgba.red >= 0.0 && srgba.red <= 1.0);
            assert!(srgba.green >= 0.0 && srgba.green <= 1.0);
            assert!(srgba.blue >= 0.0 && srgba.blue <= 1.0);
        }
    }

    #[test]
    fn test_block_type_name_not_empty() {
        // Use EnumIter to iterate over all block types
        for bt in BlockType::iter() {
            assert!(!bt.name().is_empty(), "Block type {:?} has empty name", bt);
        }
    }

    #[test]
    fn test_strum_enum_string_parsing() {
        // Test snake_case parsing
        assert_eq!(BlockType::from_str("stone").ok(), Some(BlockType::Stone));
        assert_eq!(
            BlockType::from_str("iron_ore").ok(),
            Some(BlockType::IronOre)
        );

        // Test case insensitivity
        assert_eq!(BlockType::from_str("STONE").ok(), Some(BlockType::Stone));
        assert_eq!(
            BlockType::from_str("IronOre").ok(),
            Some(BlockType::IronOre)
        );

        // Test aliases
        assert_eq!(BlockType::from_str("iron").ok(), Some(BlockType::IronOre));
        assert_eq!(
            BlockType::from_str("miner").ok(),
            Some(BlockType::MinerBlock)
        );
        assert_eq!(
            BlockType::from_str("conveyor").ok(),
            Some(BlockType::ConveyorBlock)
        );

        // Test invalid parsing
        assert!(BlockType::from_str("invalid_block").is_err());
    }

    #[test]
    fn test_strum_display() {
        // Test that Display uses snake_case
        assert_eq!(format!("{}", BlockType::Stone), "stone");
        assert_eq!(format!("{}", BlockType::IronOre), "iron_ore");
        assert_eq!(format!("{}", BlockType::MinerBlock), "miner_block");
    }

    #[test]
    fn test_strum_iter_count() {
        // Verify we have the expected number of block types
        assert_eq!(BlockType::iter().count(), 16);
    }

    #[test]
    fn test_block_type_is_machine() {
        assert!(BlockType::MinerBlock.is_machine());
        assert!(BlockType::ConveyorBlock.is_machine());
        assert!(BlockType::CrusherBlock.is_machine());
        assert!(BlockType::FurnaceBlock.is_machine());
        assert!(BlockType::AssemblerBlock.is_machine());

        assert!(!BlockType::Stone.is_machine());
        assert!(!BlockType::IronOre.is_machine());
        assert!(!BlockType::IronIngot.is_machine());
        assert!(!BlockType::IronDust.is_machine());
    }

    #[test]
    fn test_block_type_is_ore() {
        assert!(BlockType::IronOre.is_ore());
        assert!(BlockType::CopperOre.is_ore());
        assert!(BlockType::Coal.is_ore());

        assert!(!BlockType::Stone.is_ore());
        assert!(!BlockType::IronIngot.is_ore());
        assert!(!BlockType::MinerBlock.is_ore());
    }

    #[test]
    fn test_block_type_is_ingot() {
        assert!(BlockType::IronIngot.is_ingot());
        assert!(BlockType::CopperIngot.is_ingot());

        assert!(!BlockType::IronOre.is_ingot());
        assert!(!BlockType::Stone.is_ingot());
    }

    #[test]
    fn test_block_type_smelt_result() {
        assert_eq!(
            BlockType::IronOre.smelt_result(),
            Some(BlockType::IronIngot)
        );
        assert_eq!(
            BlockType::CopperOre.smelt_result(),
            Some(BlockType::CopperIngot)
        );
        assert_eq!(
            BlockType::IronDust.smelt_result(),
            Some(BlockType::IronIngot)
        );
        assert_eq!(
            BlockType::CopperDust.smelt_result(),
            Some(BlockType::CopperIngot)
        );

        assert_eq!(BlockType::Stone.smelt_result(), None);
        assert_eq!(BlockType::Coal.smelt_result(), None);
        assert_eq!(BlockType::IronIngot.smelt_result(), None);
    }

    #[test]
    fn test_block_type_is_dust() {
        assert!(BlockType::IronDust.is_dust());
        assert!(BlockType::CopperDust.is_dust());

        assert!(!BlockType::IronOre.is_dust());
        assert!(!BlockType::IronIngot.is_dust());
    }

    #[test]
    fn test_block_type_default() {
        assert_eq!(BlockType::default(), BlockType::Stone);
    }

    #[test]
    fn test_block_type_equality() {
        assert_eq!(BlockType::Stone, BlockType::Stone);
        assert_ne!(BlockType::Stone, BlockType::Grass);
    }

    #[test]
    fn test_block_category() {
        // Terrain
        assert_eq!(BlockType::Stone.category(), BlockCategory::Terrain);
        assert_eq!(BlockType::Grass.category(), BlockCategory::Terrain);

        // Ore
        assert_eq!(BlockType::IronOre.category(), BlockCategory::Ore);
        assert_eq!(BlockType::CopperOre.category(), BlockCategory::Ore);
        assert_eq!(BlockType::Coal.category(), BlockCategory::Ore);

        // Machine
        assert_eq!(BlockType::MinerBlock.category(), BlockCategory::Machine);
        assert_eq!(BlockType::FurnaceBlock.category(), BlockCategory::Machine);
        assert_eq!(BlockType::ConveyorBlock.category(), BlockCategory::Machine);

        // Processed
        assert_eq!(BlockType::IronIngot.category(), BlockCategory::Processed);
        assert_eq!(BlockType::IronDust.category(), BlockCategory::Processed);

        // Tool
        assert_eq!(BlockType::StonePickaxe.category(), BlockCategory::Tool);
    }

    #[test]
    fn test_is_category() {
        assert!(BlockType::Stone.is_category(BlockCategory::Terrain));
        assert!(!BlockType::Stone.is_category(BlockCategory::Machine));

        assert!(BlockType::MinerBlock.is_category(BlockCategory::Machine));
        assert!(!BlockType::MinerBlock.is_category(BlockCategory::Ore));
    }

    #[test]
    fn test_all_in_category() {
        let machines = BlockType::all_in_category(BlockCategory::Machine);
        assert!(machines.contains(&BlockType::MinerBlock));
        assert!(machines.contains(&BlockType::FurnaceBlock));
        assert!(machines.contains(&BlockType::ConveyorBlock));
        assert!(!machines.contains(&BlockType::Stone));

        let ores = BlockType::all_in_category(BlockCategory::Ore);
        assert_eq!(ores.len(), 3); // IronOre, CopperOre, Coal
    }

    #[test]
    fn test_base_ore() {
        assert_eq!(BlockType::IronIngot.base_ore(), Some(BlockType::IronOre));
        assert_eq!(BlockType::IronDust.base_ore(), Some(BlockType::IronOre));
        assert_eq!(
            BlockType::CopperIngot.base_ore(),
            Some(BlockType::CopperOre)
        );
        assert_eq!(BlockType::CopperDust.base_ore(), Some(BlockType::CopperOre));
        assert_eq!(BlockType::Stone.base_ore(), None);
        assert_eq!(BlockType::IronOre.base_ore(), None);
    }

    #[test]
    fn test_category_is_placeable() {
        assert!(BlockCategory::Terrain.is_placeable());
        assert!(BlockCategory::Ore.is_placeable());
        assert!(BlockCategory::Machine.is_placeable());
        assert!(!BlockCategory::Processed.is_placeable());
        assert!(!BlockCategory::Tool.is_placeable());
    }

    #[test]
    fn test_category_is_material() {
        assert!(BlockCategory::Ore.is_material());
        assert!(BlockCategory::Processed.is_material());
        assert!(!BlockCategory::Terrain.is_material());
        assert!(!BlockCategory::Machine.is_material());
        assert!(!BlockCategory::Tool.is_material());
    }
}
