//! Recipe system specification
//!
//! All processing recipes are defined using ItemId (no BlockType dependency).
//! Recipes are lazily initialized at runtime.

use crate::core::{items, ItemId};
use std::sync::LazyLock;

/// Machine type for recipes
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum MachineType {
    Furnace,   // Smelter
    Crusher,   // Crusher
    Assembler, // Assembler
}

/// Recipe input
#[derive(Clone, Debug)]
pub struct RecipeInput {
    /// Item type
    pub item: ItemId,
    /// Required count
    pub count: u32,
    /// Input slot ID (0 = main, 1+ = sub)
    pub slot: u8,
}

impl RecipeInput {
    pub fn new(item: ItemId, count: u32, slot: u8) -> Self {
        Self { item, count, slot }
    }
}

/// Recipe output
#[derive(Clone, Debug)]
pub struct RecipeOutput {
    /// Item type
    pub item: ItemId,
    /// Output count
    pub count: u32,
    /// Output chance (0.0-1.0, 1.0 = guaranteed)
    pub chance: f32,
}

impl RecipeOutput {
    /// Create guaranteed output
    pub fn guaranteed(item: ItemId, count: u32) -> Self {
        Self {
            item,
            count,
            chance: 1.0,
        }
    }

    /// Create chance-based output
    #[allow(dead_code)]
    pub fn chance(item: ItemId, count: u32, chance: f32) -> Self {
        Self {
            item,
            count,
            chance,
        }
    }
}

/// Fuel requirement
#[derive(Clone, Debug)]
pub struct FuelRequirement {
    /// Fuel item type
    pub fuel_type: ItemId,
    /// Amount consumed per processing
    pub amount: u32,
}

impl FuelRequirement {
    pub fn new(fuel_type: ItemId, amount: u32) -> Self {
        Self { fuel_type, amount }
    }
}

/// Recipe definition
#[derive(Clone, Debug)]
pub struct Recipe {
    /// Recipe ID (unique)
    pub id: &'static str,
    /// Machine type
    pub machine: MachineType,
    /// Input materials list
    pub inputs: Vec<RecipeInput>,
    /// Output items list (with chances)
    pub outputs: Vec<RecipeOutput>,
    /// Processing time (seconds)
    pub craft_time: f32,
    /// Fuel requirement (None = no fuel needed)
    pub fuel: Option<FuelRequirement>,
}

impl Recipe {
    /// Get guaranteed outputs only
    pub fn guaranteed_outputs(&self) -> impl Iterator<Item = &RecipeOutput> {
        self.outputs.iter().filter(|o| o.chance >= 1.0)
    }

    /// Get chance-based outputs
    pub fn chance_outputs(&self) -> impl Iterator<Item = &RecipeOutput> {
        self.outputs.iter().filter(|o| o.chance < 1.0)
    }

    /// Get input item as ItemId (for compatibility)
    pub fn input_item(&self, slot: u8) -> Option<ItemId> {
        self.inputs.iter().find(|i| i.slot == slot).map(|i| i.item)
    }

    /// Get primary output item
    pub fn output_item(&self) -> Option<ItemId> {
        self.outputs.first().map(|o| o.item)
    }
}

// =============================================================================
// Recipe Registry (LazyLock)
// =============================================================================

static RECIPES: LazyLock<Vec<Recipe>> = LazyLock::new(|| {
    vec![
        // =================================================================
        // Furnace - ore smelting
        // =================================================================
        Recipe {
            id: "smelt_iron",
            machine: MachineType::Furnace,
            inputs: vec![RecipeInput::new(items::iron_ore(), 1, 0)],
            outputs: vec![RecipeOutput::guaranteed(items::iron_ingot(), 1)],
            craft_time: 2.0,
            fuel: Some(FuelRequirement::new(items::coal(), 1)),
        },
        Recipe {
            id: "smelt_copper",
            machine: MachineType::Furnace,
            inputs: vec![RecipeInput::new(items::copper_ore(), 1, 0)],
            outputs: vec![RecipeOutput::guaranteed(items::copper_ingot(), 1)],
            craft_time: 2.0,
            fuel: Some(FuelRequirement::new(items::coal(), 1)),
        },
        // =================================================================
        // Furnace - dust smelting (faster than ore)
        // =================================================================
        Recipe {
            id: "smelt_iron_dust",
            machine: MachineType::Furnace,
            inputs: vec![RecipeInput::new(items::iron_dust(), 1, 0)],
            outputs: vec![RecipeOutput::guaranteed(items::iron_ingot(), 1)],
            craft_time: 1.5,
            fuel: Some(FuelRequirement::new(items::coal(), 1)),
        },
        Recipe {
            id: "smelt_copper_dust",
            machine: MachineType::Furnace,
            inputs: vec![RecipeInput::new(items::copper_dust(), 1, 0)],
            outputs: vec![RecipeOutput::guaranteed(items::copper_ingot(), 1)],
            craft_time: 1.5,
            fuel: Some(FuelRequirement::new(items::coal(), 1)),
        },
        // =================================================================
        // Crusher
        // =================================================================
        Recipe {
            id: "crush_iron",
            machine: MachineType::Crusher,
            inputs: vec![RecipeInput::new(items::iron_ore(), 1, 0)],
            outputs: vec![RecipeOutput::guaranteed(items::iron_dust(), 2)],
            craft_time: 1.5,
            fuel: None,
        },
        Recipe {
            id: "crush_copper",
            machine: MachineType::Crusher,
            inputs: vec![RecipeInput::new(items::copper_ore(), 1, 0)],
            outputs: vec![RecipeOutput::guaranteed(items::copper_dust(), 2)],
            craft_time: 1.5,
            fuel: None,
        },
        // =================================================================
        // Assembler
        // =================================================================
        Recipe {
            id: "craft_conveyor",
            machine: MachineType::Assembler,
            inputs: vec![RecipeInput::new(items::iron_ingot(), 2, 0)],
            outputs: vec![RecipeOutput::guaranteed(items::conveyor_block(), 5)],
            craft_time: 2.0,
            fuel: None,
        },
        Recipe {
            id: "craft_miner",
            machine: MachineType::Assembler,
            inputs: vec![
                RecipeInput::new(items::iron_ingot(), 5, 0),
                RecipeInput::new(items::stone(), 10, 1),
            ],
            outputs: vec![RecipeOutput::guaranteed(items::miner_block(), 1)],
            craft_time: 5.0,
            fuel: None,
        },
        Recipe {
            id: "craft_furnace",
            machine: MachineType::Assembler,
            inputs: vec![
                RecipeInput::new(items::iron_ingot(), 8, 0),
                RecipeInput::new(items::stone(), 20, 1),
            ],
            outputs: vec![RecipeOutput::guaranteed(items::furnace_block(), 1)],
            craft_time: 6.0,
            fuel: None,
        },
        Recipe {
            id: "craft_crusher",
            machine: MachineType::Assembler,
            inputs: vec![
                RecipeInput::new(items::iron_ingot(), 10, 0),
                RecipeInput::new(items::copper_ingot(), 5, 1),
            ],
            outputs: vec![RecipeOutput::guaranteed(items::crusher_block(), 1)],
            craft_time: 8.0,
            fuel: None,
        },
        Recipe {
            id: "craft_assembler",
            machine: MachineType::Assembler,
            inputs: vec![
                RecipeInput::new(items::iron_ingot(), 15, 0),
                RecipeInput::new(items::copper_ingot(), 10, 1),
            ],
            outputs: vec![RecipeOutput::guaranteed(items::assembler_block(), 1)],
            craft_time: 10.0,
            fuel: None,
        },
    ]
});

// =============================================================================
// Public API
// =============================================================================

/// Get all recipes
pub fn all_recipes() -> &'static [Recipe] {
    &RECIPES
}

/// Find recipe by input item ID and machine type
pub fn find_recipe(machine: MachineType, input: ItemId) -> Option<&'static Recipe> {
    RECIPES
        .iter()
        .find(|r| r.machine == machine && r.inputs.iter().any(|i| i.item == input))
}

/// Get all recipes for a machine type
pub fn get_recipes_for_machine(machine: MachineType) -> impl Iterator<Item = &'static Recipe> {
    RECIPES.iter().filter(move |r| r.machine == machine)
}

/// Find recipe by ID
pub fn find_recipe_by_id(id: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.id == id)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_system() {
        for recipe in all_recipes() {
            assert!(
                !recipe.inputs.is_empty(),
                "Recipe {} should have inputs",
                recipe.id
            );
            assert!(
                !recipe.outputs.is_empty(),
                "Recipe {} should have outputs",
                recipe.id
            );
            assert!(
                recipe.craft_time > 0.0,
                "Recipe {} should have positive craft time",
                recipe.id
            );
        }

        let iron_smelt = find_recipe(MachineType::Furnace, items::iron_ore());
        assert!(iron_smelt.is_some());
        assert_eq!(iron_smelt.unwrap().id, "smelt_iron");

        let copper_crush = find_recipe(MachineType::Crusher, items::copper_ore());
        assert!(copper_crush.is_some());
        assert_eq!(copper_crush.unwrap().id, "crush_copper");
    }

    #[test]
    fn test_fuel_requirements() {
        let iron_smelt = find_recipe(MachineType::Furnace, items::iron_ore()).unwrap();
        assert!(iron_smelt.fuel.is_some());
        let fuel = iron_smelt.fuel.as_ref().unwrap();
        assert_eq!(fuel.fuel_type, items::coal());
        assert_eq!(fuel.amount, 1);
    }

    #[test]
    fn test_output_chances() {
        for recipe in all_recipes() {
            for output in &recipe.outputs {
                assert!(
                    output.chance >= 1.0,
                    "Recipe {} output should be guaranteed",
                    recipe.id
                );
            }
        }

        let iron_smelt = find_recipe(MachineType::Furnace, items::iron_ore()).unwrap();
        let guaranteed: Vec<_> = iron_smelt.guaranteed_outputs().collect();
        let chance: Vec<_> = iron_smelt.chance_outputs().collect();
        assert_eq!(guaranteed.len(), 1);
        assert_eq!(chance.len(), 0);
    }

    #[test]
    fn test_get_recipes_for_machine() {
        // Furnace: 4 recipes (ore x2 + dust x2)
        let furnace_recipes: Vec<_> = get_recipes_for_machine(MachineType::Furnace).collect();
        assert_eq!(furnace_recipes.len(), 4);

        // Crusher: 2 recipes (iron + copper)
        let crusher_recipes: Vec<_> = get_recipes_for_machine(MachineType::Crusher).collect();
        assert_eq!(crusher_recipes.len(), 2);

        // Assembler: 5 recipes (conveyor, miner, furnace, crusher, assembler)
        let assembler_recipes: Vec<_> = get_recipes_for_machine(MachineType::Assembler).collect();
        assert_eq!(assembler_recipes.len(), 5);
    }

    #[test]
    fn test_crusher_doubles_output() {
        let iron_crush = find_recipe(MachineType::Crusher, items::iron_ore()).unwrap();
        assert_eq!(iron_crush.outputs[0].count, 2); // Ore -> Dust x2
        assert!(iron_crush.fuel.is_none()); // No fuel needed
    }

    #[test]
    fn test_dust_smelting_faster() {
        let iron_dust_smelt = find_recipe(MachineType::Furnace, items::iron_dust()).unwrap();
        assert!(iron_dust_smelt.fuel.is_some()); // Dust also needs fuel
        assert!(iron_dust_smelt.craft_time < 2.0); // Faster than ore (1.5s vs 2.0s)
    }

    #[test]
    fn test_find_recipe_by_id() {
        let recipe = find_recipe_by_id("smelt_iron");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().machine, MachineType::Furnace);

        let no_recipe = find_recipe_by_id("nonexistent");
        assert!(no_recipe.is_none());
    }

    #[test]
    fn test_recipe_item_accessors() {
        let recipe = find_recipe_by_id("smelt_iron").unwrap();

        // Input item
        let input = recipe.input_item(0);
        assert_eq!(input, Some(items::iron_ore()));

        // Output item
        let output = recipe.output_item();
        assert_eq!(output, Some(items::iron_ingot()));
    }

    #[test]
    fn test_all_recipes_count() {
        // Total: 4 furnace + 2 crusher + 5 assembler = 11
        assert_eq!(all_recipes().len(), 11);
    }
}
