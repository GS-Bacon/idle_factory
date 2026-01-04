//! Pure recipe evaluation logic (Bevy-independent)
//!
//! This module provides recipe lookup and evaluation without Bevy dependencies.

use crate::BlockType;

/// A recipe definition
#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    /// Unique recipe ID
    pub id: &'static str,
    /// Input items required
    pub inputs: &'static [(BlockType, u32)],
    /// Output items produced
    pub outputs: &'static [(BlockType, u32)],
    /// Time to craft (seconds)
    pub time: f32,
    /// Category for recipe selection UI
    pub category: RecipeCategory,
}

/// Recipe category for filtering
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecipeCategory {
    /// Smelting recipes (furnace)
    Smelting,
    /// Crushing recipes (crusher)
    Crushing,
    /// Crafting recipes (assembler, future)
    Crafting,
}

impl Recipe {
    /// Check if a recipe can be crafted with given inputs
    pub fn can_craft(&self, available: &[(BlockType, u32)]) -> bool {
        self.inputs.iter().all(|(required_item, required_count)| {
            available
                .iter()
                .any(|(item, count)| item == required_item && count >= required_count)
        })
    }

    /// Get the primary output item (first output)
    pub fn primary_output(&self) -> Option<(BlockType, u32)> {
        self.outputs.first().copied()
    }
}

/// Find a recipe that can be crafted with available input
pub fn find_matching_recipe(
    recipes: &[Recipe],
    input: BlockType,
    category: RecipeCategory,
) -> Option<&Recipe> {
    recipes
        .iter()
        .find(|r| r.category == category && r.inputs.iter().any(|(item, _)| *item == input))
}

/// Standard smelting recipes (furnace)
pub static SMELTING_RECIPES: &[Recipe] = &[
    Recipe {
        id: "smelt_iron",
        inputs: &[(BlockType::IronOre, 1)],
        outputs: &[(BlockType::IronIngot, 1)],
        time: 3.0,
        category: RecipeCategory::Smelting,
    },
    Recipe {
        id: "smelt_copper",
        inputs: &[(BlockType::CopperOre, 1)],
        outputs: &[(BlockType::CopperIngot, 1)],
        time: 3.0,
        category: RecipeCategory::Smelting,
    },
];

/// Standard crushing recipes (crusher)
pub static CRUSHING_RECIPES: &[Recipe] = &[
    Recipe {
        id: "crush_iron",
        inputs: &[(BlockType::IronOre, 1)],
        outputs: &[(BlockType::IronOre, 2)],
        time: 4.0,
        category: RecipeCategory::Crushing,
    },
    Recipe {
        id: "crush_copper",
        inputs: &[(BlockType::CopperOre, 1)],
        outputs: &[(BlockType::CopperOre, 2)],
        time: 4.0,
        category: RecipeCategory::Crushing,
    },
    Recipe {
        id: "crush_coal",
        inputs: &[(BlockType::Coal, 1)],
        outputs: &[(BlockType::Coal, 2)],
        time: 4.0,
        category: RecipeCategory::Crushing,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_can_craft() {
        let recipe = &SMELTING_RECIPES[0]; // Iron ore -> Iron ingot
        assert!(recipe.can_craft(&[(BlockType::IronOre, 1)]));
        assert!(recipe.can_craft(&[(BlockType::IronOre, 5)]));
        assert!(!recipe.can_craft(&[(BlockType::CopperOre, 1)]));
        assert!(!recipe.can_craft(&[]));
    }

    #[test]
    fn test_find_matching_recipe() {
        let recipe = find_matching_recipe(
            SMELTING_RECIPES,
            BlockType::IronOre,
            RecipeCategory::Smelting,
        );
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().id, "smelt_iron");

        let recipe =
            find_matching_recipe(SMELTING_RECIPES, BlockType::Stone, RecipeCategory::Smelting);
        assert!(recipe.is_none());
    }
}
