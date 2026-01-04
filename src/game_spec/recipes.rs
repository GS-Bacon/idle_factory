//! Recipe system specification
//!
//! All processing recipes are defined as `RecipeSpec`.

use crate::BlockType;

/// Machine type for recipes
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MachineType {
    Furnace,   // Smelter
    Crusher,   // Crusher
    Assembler, // Assembler (future)
}

/// Recipe input
#[derive(Clone, Copy, Debug)]
pub struct RecipeInput {
    /// Item type
    pub item: BlockType,
    /// Required count
    pub count: u32,
    /// Input slot ID (0 = main, 1+ = sub)
    pub slot: u8,
}

impl RecipeInput {
    pub const fn new(item: BlockType, count: u32, slot: u8) -> Self {
        Self { item, count, slot }
    }
}

/// Recipe output
#[derive(Clone, Copy, Debug)]
pub struct RecipeOutput {
    /// Item type
    pub item: BlockType,
    /// Output count
    pub count: u32,
    /// Output chance (0.0-1.0, 1.0 = guaranteed)
    pub chance: f32,
}

impl RecipeOutput {
    /// Create guaranteed output
    pub const fn guaranteed(item: BlockType, count: u32) -> Self {
        Self {
            item,
            count,
            chance: 1.0,
        }
    }

    /// Create chance-based output
    #[allow(dead_code)]
    pub const fn chance(item: BlockType, count: u32, chance: f32) -> Self {
        Self {
            item,
            count,
            chance,
        }
    }
}

/// Fuel requirement
#[derive(Clone, Copy, Debug)]
pub struct FuelRequirement {
    /// Fuel item type
    pub fuel_type: BlockType,
    /// Amount consumed per processing
    pub amount: u32,
}

impl FuelRequirement {
    pub const fn new(fuel_type: BlockType, amount: u32) -> Self {
        Self { fuel_type, amount }
    }
}

/// Recipe definition
#[derive(Clone, Debug)]
pub struct RecipeSpec {
    /// Recipe ID (unique)
    pub id: &'static str,
    /// Machine type
    pub machine: MachineType,
    /// Input materials list
    pub inputs: &'static [RecipeInput],
    /// Output items list (with chances)
    pub outputs: &'static [RecipeOutput],
    /// Processing time (seconds)
    pub craft_time: f32,
    /// Fuel requirement (None = no fuel needed)
    pub fuel: Option<FuelRequirement>,
}

impl RecipeSpec {
    /// Get guaranteed outputs only
    pub fn guaranteed_outputs(&self) -> impl Iterator<Item = &RecipeOutput> {
        self.outputs.iter().filter(|o| o.chance >= 1.0)
    }

    /// Get chance-based outputs
    pub fn chance_outputs(&self) -> impl Iterator<Item = &RecipeOutput> {
        self.outputs.iter().filter(|o| o.chance < 1.0)
    }
}

// =============================================================================
// Smelting Recipes
// =============================================================================

/// Iron ore -> Iron ingot (consumes 1 coal)
pub const RECIPE_SMELT_IRON: RecipeSpec = RecipeSpec {
    id: "smelt_iron",
    machine: MachineType::Furnace,
    inputs: &[RecipeInput::new(BlockType::IronOre, 1, 0)],
    outputs: &[RecipeOutput::guaranteed(BlockType::IronIngot, 1)],
    craft_time: 2.0,
    fuel: Some(FuelRequirement::new(BlockType::Coal, 1)),
};

/// Copper ore -> Copper ingot (consumes 1 coal)
pub const RECIPE_SMELT_COPPER: RecipeSpec = RecipeSpec {
    id: "smelt_copper",
    machine: MachineType::Furnace,
    inputs: &[RecipeInput::new(BlockType::CopperOre, 1, 0)],
    outputs: &[RecipeOutput::guaranteed(BlockType::CopperIngot, 1)],
    craft_time: 2.0,
    fuel: Some(FuelRequirement::new(BlockType::Coal, 1)),
};

/// All furnace recipes
pub const FURNACE_RECIPES: &[&RecipeSpec] = &[&RECIPE_SMELT_IRON, &RECIPE_SMELT_COPPER];

// =============================================================================
// Crusher Recipes (future)
// =============================================================================

/// All crusher recipes (currently empty)
pub const CRUSHER_RECIPES: &[&RecipeSpec] = &[];

// =============================================================================
// All Recipes
// =============================================================================

/// All recipes list
pub const ALL_RECIPES: &[&RecipeSpec] = &[&RECIPE_SMELT_IRON, &RECIPE_SMELT_COPPER];

/// Find recipe by input item and machine type
pub fn find_recipe(machine: MachineType, input: BlockType) -> Option<&'static RecipeSpec> {
    ALL_RECIPES
        .iter()
        .find(|r| r.machine == machine && r.inputs.iter().any(|i| i.item == input))
        .copied()
}

/// Get all recipes for a machine type
pub fn get_recipes_for_machine(machine: MachineType) -> impl Iterator<Item = &'static RecipeSpec> {
    ALL_RECIPES
        .iter()
        .filter(move |r| r.machine == machine)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_system() {
        for recipe in ALL_RECIPES {
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

        let iron_smelt = find_recipe(MachineType::Furnace, BlockType::IronOre);
        assert!(iron_smelt.is_some());
        assert_eq!(iron_smelt.unwrap().id, "smelt_iron");

        let copper_crush = find_recipe(MachineType::Crusher, BlockType::CopperOre);
        assert!(copper_crush.is_none());
    }

    #[test]
    fn test_fuel_requirements() {
        let iron_smelt = find_recipe(MachineType::Furnace, BlockType::IronOre).unwrap();
        assert!(iron_smelt.fuel.is_some());
        let fuel = iron_smelt.fuel.unwrap();
        assert_eq!(fuel.fuel_type, BlockType::Coal);
        assert_eq!(fuel.amount, 1);
    }

    #[test]
    fn test_output_chances() {
        for recipe in ALL_RECIPES {
            for output in recipe.outputs {
                assert!(
                    output.chance >= 1.0,
                    "Recipe {} output should be guaranteed",
                    recipe.id
                );
            }
        }

        let iron_smelt = find_recipe(MachineType::Furnace, BlockType::IronOre).unwrap();
        let guaranteed: Vec<_> = iron_smelt.guaranteed_outputs().collect();
        let chance: Vec<_> = iron_smelt.chance_outputs().collect();
        assert_eq!(guaranteed.len(), 1);
        assert_eq!(chance.len(), 0);
    }

    #[test]
    fn test_get_recipes_for_machine() {
        let furnace_recipes: Vec<_> = get_recipes_for_machine(MachineType::Furnace).collect();
        assert_eq!(furnace_recipes.len(), 2);

        let crusher_recipes: Vec<_> = get_recipes_for_machine(MachineType::Crusher).collect();
        assert_eq!(crusher_recipes.len(), 0);

        let assembler_recipes: Vec<_> = get_recipes_for_machine(MachineType::Assembler).collect();
        assert_eq!(assembler_recipes.len(), 0);
    }
}
