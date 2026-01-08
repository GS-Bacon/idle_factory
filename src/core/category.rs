//! Block/Item Categories
//!
//! Defines categories for blocks and items.
//! This is separate from BlockType to allow BlockType enum to be removed
//! while keeping category functionality.

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_category_names() {
        assert_eq!(BlockCategory::Terrain.name(), "地形");
        assert_eq!(BlockCategory::Ore.name(), "鉱石");
        assert_eq!(BlockCategory::Machine.name(), "機械");
        assert_eq!(BlockCategory::Processed.name(), "素材");
        assert_eq!(BlockCategory::Tool.name(), "道具");
    }
}
