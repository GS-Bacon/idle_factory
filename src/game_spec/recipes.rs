//! Recipe system specification
//!
//! All processing recipes are defined as `RecipeSpec`.
//!
//! ## BlockType vs ItemId
//!
//! Recipe definitions use `BlockType` for `const` compatibility.
//! For ItemId-based lookups, use `find_recipe()`.

use crate::core::ItemId;
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
// Crusher Recipes
// =============================================================================

/// Iron ore -> Iron dust x2 (doubles output)
pub const RECIPE_CRUSH_IRON: RecipeSpec = RecipeSpec {
    id: "crush_iron",
    machine: MachineType::Crusher,
    inputs: &[RecipeInput::new(BlockType::IronOre, 1, 0)],
    outputs: &[RecipeOutput::guaranteed(BlockType::IronDust, 2)],
    craft_time: 1.5,
    fuel: None,
};

/// Copper ore -> Copper dust x2 (doubles output)
pub const RECIPE_CRUSH_COPPER: RecipeSpec = RecipeSpec {
    id: "crush_copper",
    machine: MachineType::Crusher,
    inputs: &[RecipeInput::new(BlockType::CopperOre, 1, 0)],
    outputs: &[RecipeOutput::guaranteed(BlockType::CopperDust, 2)],
    craft_time: 1.5,
    fuel: None,
};

/// All crusher recipes
pub const CRUSHER_RECIPES: &[&RecipeSpec] = &[&RECIPE_CRUSH_IRON, &RECIPE_CRUSH_COPPER];

// =============================================================================
// Dust Smelting Recipes (faster than ore smelting - 1.5s vs 2.0s)
// =============================================================================

/// Iron dust -> Iron ingot (requires fuel, faster than ore)
pub const RECIPE_SMELT_IRON_DUST: RecipeSpec = RecipeSpec {
    id: "smelt_iron_dust",
    machine: MachineType::Furnace,
    inputs: &[RecipeInput::new(BlockType::IronDust, 1, 0)],
    outputs: &[RecipeOutput::guaranteed(BlockType::IronIngot, 1)],
    craft_time: 1.5, // Faster than ore
    fuel: Some(FuelRequirement::new(BlockType::Coal, 1)),
};

/// Copper dust -> Copper ingot (requires fuel, faster than ore)
pub const RECIPE_SMELT_COPPER_DUST: RecipeSpec = RecipeSpec {
    id: "smelt_copper_dust",
    machine: MachineType::Furnace,
    inputs: &[RecipeInput::new(BlockType::CopperDust, 1, 0)],
    outputs: &[RecipeOutput::guaranteed(BlockType::CopperIngot, 1)],
    craft_time: 1.5, // Faster than ore
    fuel: Some(FuelRequirement::new(BlockType::Coal, 1)),
};

/// All dust smelting recipes
pub const DUST_SMELT_RECIPES: &[&RecipeSpec] =
    &[&RECIPE_SMELT_IRON_DUST, &RECIPE_SMELT_COPPER_DUST];

// =============================================================================
// Assembler Recipes
// =============================================================================

/// Conveyor crafting: Iron ingot x2 -> Conveyor x5
pub const RECIPE_CRAFT_CONVEYOR: RecipeSpec = RecipeSpec {
    id: "craft_conveyor",
    machine: MachineType::Assembler,
    inputs: &[RecipeInput::new(BlockType::IronIngot, 2, 0)],
    outputs: &[RecipeOutput::guaranteed(BlockType::ConveyorBlock, 5)],
    craft_time: 2.0,
    fuel: None,
};

/// Miner crafting: Iron ingot x5 + Stone x10 -> Miner x1
pub const RECIPE_CRAFT_MINER: RecipeSpec = RecipeSpec {
    id: "craft_miner",
    machine: MachineType::Assembler,
    inputs: &[
        RecipeInput::new(BlockType::IronIngot, 5, 0),
        RecipeInput::new(BlockType::Stone, 10, 1),
    ],
    outputs: &[RecipeOutput::guaranteed(BlockType::MinerBlock, 1)],
    craft_time: 5.0,
    fuel: None,
};

/// Furnace crafting: Iron ingot x8 + Stone x20 -> Furnace x1
pub const RECIPE_CRAFT_FURNACE: RecipeSpec = RecipeSpec {
    id: "craft_furnace",
    machine: MachineType::Assembler,
    inputs: &[
        RecipeInput::new(BlockType::IronIngot, 8, 0),
        RecipeInput::new(BlockType::Stone, 20, 1),
    ],
    outputs: &[RecipeOutput::guaranteed(BlockType::FurnaceBlock, 1)],
    craft_time: 6.0,
    fuel: None,
};

/// Crusher crafting: Iron ingot x10 + Copper ingot x5 -> Crusher x1
pub const RECIPE_CRAFT_CRUSHER: RecipeSpec = RecipeSpec {
    id: "craft_crusher",
    machine: MachineType::Assembler,
    inputs: &[
        RecipeInput::new(BlockType::IronIngot, 10, 0),
        RecipeInput::new(BlockType::CopperIngot, 5, 1),
    ],
    outputs: &[RecipeOutput::guaranteed(BlockType::CrusherBlock, 1)],
    craft_time: 8.0,
    fuel: None,
};

/// Assembler crafting: Iron ingot x15 + Copper ingot x10 -> Assembler x1
pub const RECIPE_CRAFT_ASSEMBLER: RecipeSpec = RecipeSpec {
    id: "craft_assembler",
    machine: MachineType::Assembler,
    inputs: &[
        RecipeInput::new(BlockType::IronIngot, 15, 0),
        RecipeInput::new(BlockType::CopperIngot, 10, 1),
    ],
    outputs: &[RecipeOutput::guaranteed(BlockType::AssemblerBlock, 1)],
    craft_time: 10.0,
    fuel: None,
};

/// All assembler recipes
pub const ASSEMBLER_RECIPES: &[&RecipeSpec] = &[
    &RECIPE_CRAFT_CONVEYOR,
    &RECIPE_CRAFT_MINER,
    &RECIPE_CRAFT_FURNACE,
    &RECIPE_CRAFT_CRUSHER,
    &RECIPE_CRAFT_ASSEMBLER,
];

// =============================================================================
// All Recipes
// =============================================================================

/// All recipes list
pub const ALL_RECIPES: &[&RecipeSpec] = &[
    // Furnace - ore smelting
    &RECIPE_SMELT_IRON,
    &RECIPE_SMELT_COPPER,
    // Furnace - dust smelting
    &RECIPE_SMELT_IRON_DUST,
    &RECIPE_SMELT_COPPER_DUST,
    // Crusher
    &RECIPE_CRUSH_IRON,
    &RECIPE_CRUSH_COPPER,
    // Assembler
    &RECIPE_CRAFT_CONVEYOR,
    &RECIPE_CRAFT_MINER,
    &RECIPE_CRAFT_FURNACE,
    &RECIPE_CRAFT_CRUSHER,
    &RECIPE_CRAFT_ASSEMBLER,
];

/// Find recipe by input item ID and machine type
pub fn find_recipe(machine: MachineType, input: ItemId) -> Option<&'static RecipeSpec> {
    let block_type: BlockType = input.try_into().ok()?;
    ALL_RECIPES
        .iter()
        .find(|r| r.machine == machine && r.inputs.iter().any(|i| i.item == block_type))
        .copied()
}

/// Get all recipes for a machine type
pub fn get_recipes_for_machine(machine: MachineType) -> impl Iterator<Item = &'static RecipeSpec> {
    ALL_RECIPES
        .iter()
        .filter(move |r| r.machine == machine)
        .copied()
}

// =============================================================================
// ItemId Helpers
// =============================================================================

impl RecipeInput {
    /// Get input item as ItemId
    pub fn item_id(&self) -> ItemId {
        self.item.into()
    }
}

impl RecipeOutput {
    /// Get output item as ItemId
    pub fn item_id(&self) -> ItemId {
        self.item.into()
    }
}

impl FuelRequirement {
    /// Get fuel type as ItemId
    pub fn fuel_id(&self) -> ItemId {
        self.fuel_type.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

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

        let iron_smelt = find_recipe(MachineType::Furnace, items::iron_ore());
        assert!(iron_smelt.is_some());
        assert_eq!(iron_smelt.unwrap().id, "smelt_iron");

        // Crusher recipes now exist
        let copper_crush = find_recipe(MachineType::Crusher, items::copper_ore());
        assert!(copper_crush.is_some());
        assert_eq!(copper_crush.unwrap().id, "crush_copper");
    }

    #[test]
    fn test_fuel_requirements() {
        let iron_smelt = find_recipe(MachineType::Furnace, items::iron_ore()).unwrap();
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

    // =========================================================================
    // ItemId API tests
    // =========================================================================

    #[test]
    fn test_find_recipe() {
        // Furnace recipe lookup
        let iron_smelt = find_recipe(MachineType::Furnace, items::iron_ore());
        assert!(iron_smelt.is_some());
        assert_eq!(iron_smelt.unwrap().id, "smelt_iron");

        // Crusher recipe lookup
        let copper_crush = find_recipe(MachineType::Crusher, items::copper_ore());
        assert!(copper_crush.is_some());
        assert_eq!(copper_crush.unwrap().id, "crush_copper");

        // Non-matching lookup
        let no_recipe = find_recipe(MachineType::Furnace, items::stone());
        assert!(no_recipe.is_none());
    }

    #[test]
    fn test_recipe_item_id_helpers() {
        let recipe = &RECIPE_SMELT_IRON;

        // Input item_id
        let input_id = recipe.inputs[0].item_id();
        assert_eq!(input_id.name(), Some("base:iron_ore"));

        // Output item_id
        let output_id = recipe.outputs[0].item_id();
        assert_eq!(output_id.name(), Some("base:iron_ingot"));

        // Fuel item_id
        let fuel_id = recipe.fuel.unwrap().fuel_id();
        assert_eq!(fuel_id.name(), Some("base:coal"));
    }
}
