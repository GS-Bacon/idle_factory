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
        }
    }

    /// Returns true if this block type is a machine (not a regular block)
    #[allow(dead_code)]
    pub fn is_machine(&self) -> bool {
        matches!(
            self,
            BlockType::MinerBlock | BlockType::ConveyorBlock | BlockType::CrusherBlock
        )
    }
}
