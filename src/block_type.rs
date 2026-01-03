//! Block type definitions

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

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
            BlockType::StonePickaxe | BlockType::IronIngot | BlockType::CopperIngot
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
        assert_eq!(BlockType::iter().count(), 12);
    }

    #[test]
    fn test_block_type_is_machine() {
        assert!(BlockType::MinerBlock.is_machine());
        assert!(BlockType::ConveyorBlock.is_machine());
        assert!(BlockType::CrusherBlock.is_machine());
        assert!(BlockType::FurnaceBlock.is_machine());

        assert!(!BlockType::Stone.is_machine());
        assert!(!BlockType::IronOre.is_machine());
        assert!(!BlockType::IronIngot.is_machine());
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

        assert_eq!(BlockType::Stone.smelt_result(), None);
        assert_eq!(BlockType::Coal.smelt_result(), None);
        assert_eq!(BlockType::IronIngot.smelt_result(), None);
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
}
