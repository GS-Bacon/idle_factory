//! Block type definitions

use bevy::prelude::*;

/// Types of blocks in the game
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum BlockType {
    #[default]
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
        }
    }

    /// Returns true if this block type is a machine (not a regular block)
    #[allow(dead_code)]
    pub fn is_machine(&self) -> bool {
        matches!(
            self,
            BlockType::MinerBlock | BlockType::ConveyorBlock | BlockType::CrusherBlock | BlockType::FurnaceBlock
        )
    }

    /// Returns true if this block type is a raw ore
    pub fn is_ore(&self) -> bool {
        matches!(self, BlockType::IronOre | BlockType::CopperOre | BlockType::Coal)
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

    #[test]
    fn test_block_type_color_returns_valid() {
        // All block types should return a color
        let types = [
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
        ];
        for bt in types {
            let color = bt.color();
            let srgba = color.to_srgba();
            assert!(srgba.red >= 0.0 && srgba.red <= 1.0);
            assert!(srgba.green >= 0.0 && srgba.green <= 1.0);
            assert!(srgba.blue >= 0.0 && srgba.blue <= 1.0);
        }
    }

    #[test]
    fn test_block_type_name_not_empty() {
        let types = [
            BlockType::Stone,
            BlockType::Grass,
            BlockType::IronOre,
            BlockType::Coal,
        ];
        for bt in types {
            assert!(!bt.name().is_empty());
        }
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
        assert_eq!(BlockType::IronOre.smelt_result(), Some(BlockType::IronIngot));
        assert_eq!(BlockType::CopperOre.smelt_result(), Some(BlockType::CopperIngot));

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
