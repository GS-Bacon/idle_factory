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
    /// Get the item descriptor for this block type
    /// This is the single source of truth for all block/item metadata
    pub fn descriptor(&self) -> &'static crate::game_spec::ItemDescriptor {
        use crate::core::ItemId;
        use crate::game_spec::item_descriptors;
        let item_id: ItemId = (*self).into();
        item_descriptors()
            .iter()
            .find(|(id, _)| *id == item_id)
            .map(|(_, desc)| desc)
            .expect("All BlockTypes must be registered in item_descriptors")
    }

    /// Get the color for this block type
    pub fn color(&self) -> Color {
        self.descriptor().color
    }

    /// Get the display name for this block type
    pub fn name(&self) -> &'static str {
        self.descriptor().name
    }

    /// Get a short name for UI display (max 4 chars)
    pub fn short_name(&self) -> &'static str {
        self.descriptor().short_name
    }

    /// Returns true if this block type is a tool
    pub fn is_tool(&self) -> bool {
        self.category() == BlockCategory::Tool
    }

    /// Returns true if this block type can be placed in the world
    /// Tools and processed materials cannot be placed
    pub fn is_placeable(&self) -> bool {
        self.descriptor().is_placeable
    }

    /// Returns true if this block type is a machine (not a regular block)
    #[allow(dead_code)]
    pub fn is_machine(&self) -> bool {
        self.category() == BlockCategory::Machine
    }

    /// Returns true if this block type is a raw ore
    pub fn is_ore(&self) -> bool {
        self.category() == BlockCategory::Ore
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
        self.descriptor().category
    }

    /// Get the hardness (base break time) for this block type
    pub fn hardness(&self) -> f32 {
        self.descriptor().hardness
    }

    /// Get what this block drops when broken
    pub fn drops(&self) -> BlockType {
        use crate::core::ItemId;
        let item_id: ItemId = (*self).into();
        let drop_id = self.descriptor().get_drops(item_id);
        drop_id.try_into().unwrap_or(*self)
    }

    /// Get the max stack size for this block type
    pub fn stack_size(&self) -> u32 {
        self.descriptor().stack_size
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

    // =============================================================================
    // Save String ID Format (V2)
    // =============================================================================

    /// Default namespace for base game items
    pub const SAVE_NAMESPACE: &'static str = "base";

    /// Convert to save string ID format ("base:stone", "base:iron_ore", etc.)
    /// Uses strum Display which outputs snake_case
    pub fn to_save_string_id(&self) -> String {
        format!("{}:{}", Self::SAVE_NAMESPACE, self)
    }

    /// Parse from save string ID format ("base:stone", "base:iron_ore", etc.)
    /// Supports aliases defined in strum (e.g., "miner" -> MinerBlock)
    pub fn from_save_string_id(s: &str) -> Option<Self> {
        let (namespace, id) = if let Some(colon_pos) = s.find(':') {
            (&s[..colon_pos], &s[colon_pos + 1..])
        } else {
            // Fallback: treat as just ID with default namespace
            (Self::SAVE_NAMESPACE, s)
        };

        // Only support base namespace for now
        if namespace != Self::SAVE_NAMESPACE {
            return None;
        }

        std::str::FromStr::from_str(id).ok()
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

    // === Save String ID Tests ===

    #[test]
    fn test_to_save_string_id() {
        assert_eq!(BlockType::Stone.to_save_string_id(), "base:stone");
        assert_eq!(BlockType::IronOre.to_save_string_id(), "base:iron_ore");
        assert_eq!(
            BlockType::MinerBlock.to_save_string_id(),
            "base:miner_block"
        );
        assert_eq!(
            BlockType::ConveyorBlock.to_save_string_id(),
            "base:conveyor_block"
        );
        assert_eq!(
            BlockType::StonePickaxe.to_save_string_id(),
            "base:stone_pickaxe"
        );
    }

    #[test]
    fn test_from_save_string_id() {
        // Test basic parsing
        assert_eq!(
            BlockType::from_save_string_id("base:stone"),
            Some(BlockType::Stone)
        );
        assert_eq!(
            BlockType::from_save_string_id("base:iron_ore"),
            Some(BlockType::IronOre)
        );
        assert_eq!(
            BlockType::from_save_string_id("base:miner_block"),
            Some(BlockType::MinerBlock)
        );

        // Test aliases (from strum)
        assert_eq!(
            BlockType::from_save_string_id("base:miner"),
            Some(BlockType::MinerBlock)
        );
        assert_eq!(
            BlockType::from_save_string_id("base:conveyor"),
            Some(BlockType::ConveyorBlock)
        );
        assert_eq!(
            BlockType::from_save_string_id("base:furnace"),
            Some(BlockType::FurnaceBlock)
        );
        assert_eq!(
            BlockType::from_save_string_id("base:pickaxe"),
            Some(BlockType::StonePickaxe)
        );
        assert_eq!(
            BlockType::from_save_string_id("base:iron"),
            Some(BlockType::IronOre)
        );

        // Test fallback (no namespace)
        assert_eq!(
            BlockType::from_save_string_id("stone"),
            Some(BlockType::Stone)
        );
        assert_eq!(
            BlockType::from_save_string_id("iron_ore"),
            Some(BlockType::IronOre)
        );

        // Test invalid cases
        assert_eq!(BlockType::from_save_string_id("unknown:stone"), None);
        assert_eq!(BlockType::from_save_string_id("base:unknown_item"), None);
        assert_eq!(BlockType::from_save_string_id("mod:custom_item"), None);
    }

    #[test]
    fn test_save_string_id_roundtrip() {
        // Test all BlockType variants can be converted to string and back
        for bt in BlockType::iter() {
            let string_id = bt.to_save_string_id();
            let restored = BlockType::from_save_string_id(&string_id)
                .unwrap_or_else(|| panic!("Failed to parse string ID: {}", string_id));
            assert_eq!(bt, restored, "Roundtrip failed for {:?}", bt);
        }
    }
}
