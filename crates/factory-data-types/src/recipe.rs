//! Recipe type definitions

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Work type for machines (Game format)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum WorkType {
    #[default]
    Assembling,
    Pressing,
    Crushing,
    Cutting,
    Mixing,
    WireDrawing,
    Washing,
    Smelting,
}

impl WorkType {
    /// Parse a string into a WorkType
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "assembling" => Some(Self::Assembling),
            "pressing" => Some(Self::Pressing),
            "crushing" => Some(Self::Crushing),
            "cutting" => Some(Self::Cutting),
            "mixing" => Some(Self::Mixing),
            "wiredrawing" | "wire_drawing" => Some(Self::WireDrawing),
            "washing" => Some(Self::Washing),
            "smelting" => Some(Self::Smelting),
            _ => None,
        }
    }
}

/// Machine type for editor (Editor format)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub enum MachineType {
    #[default]
    Assembler,
    Mixer,
    Press,
    Furnace,
    Crusher,
    Centrifuge,
    ChemicalReactor,
    Packager,
    Custom(String),
}

impl MachineType {
    /// Convert MachineType to WorkType
    pub fn to_work_type(&self) -> WorkType {
        match self {
            Self::Assembler => WorkType::Assembling,
            Self::Press => WorkType::Pressing,
            Self::Crusher => WorkType::Crushing,
            Self::Mixer => WorkType::Mixing,
            Self::Furnace => WorkType::Smelting,
            Self::Centrifuge => WorkType::Washing,
            Self::ChemicalReactor => WorkType::Mixing,
            Self::Packager => WorkType::Assembling,
            Self::Custom(_) => WorkType::Assembling,
        }
    }
}

/// Ingredient type (Item or Tag)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(tag = "type", content = "value")]
pub enum IngredientType {
    Item(String),
    Tag(String),
}

/// Recipe ingredient (Editor format)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct Ingredient {
    pub ingredient_type: IngredientType,
    pub amount: u32,
}

/// Product type (Item or Fluid)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(tag = "type", content = "value")]
pub enum ProductType {
    Item(String),
    Fluid(String),
}

/// Recipe product (Editor format)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct Product {
    pub product_type: ProductType,
    pub amount: u32,
    #[serde(default = "default_chance")]
    pub chance: f32,
}

fn default_chance() -> f32 {
    1.0
}

/// Recipe definition (Editor format)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct RecipeDef {
    pub id: String,
    pub machine_type: MachineType,
    pub ingredients: Vec<Ingredient>,
    pub results: Vec<Product>,
    pub process_time: f32,
    #[serde(default)]
    pub stress_impact: f32,
    #[serde(default)]
    pub i18n_key: String,
}

impl RecipeDef {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            i18n_key: format!("recipe.{}", &id),
            id,
            machine_type: MachineType::default(),
            ingredients: Vec::new(),
            results: Vec::new(),
            process_time: 1.0,
            stress_impact: 0.0,
        }
    }
}

/// Item I/O for game (Game format)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ItemIO {
    pub item: String,
    pub count: u32,
}

/// Fluid I/O for game (Game format)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct FluidIO {
    pub fluid: String,
    pub amount: f32,
}

/// Game-compatible recipe (Game format)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct GameRecipe {
    pub id: String,
    pub name: String,
    pub inputs: Vec<ItemIO>,
    #[serde(default)]
    pub input_fluid: Option<FluidIO>,
    pub outputs: Vec<ItemIO>,
    #[serde(default)]
    pub output_fluid: Option<FluidIO>,
    pub craft_time: f32,
    pub work_type: WorkType,
}

impl From<&RecipeDef> for GameRecipe {
    fn from(def: &RecipeDef) -> Self {
        let inputs = def
            .ingredients
            .iter()
            .map(|ing| {
                let item = match &ing.ingredient_type {
                    IngredientType::Item(id) => id.clone(),
                    IngredientType::Tag(tag) => tag.clone(),
                };
                ItemIO {
                    item,
                    count: ing.amount,
                }
            })
            .collect();

        let outputs = def
            .results
            .iter()
            .filter_map(|prod| match &prod.product_type {
                ProductType::Item(id) => Some(ItemIO {
                    item: id.clone(),
                    count: prod.amount,
                }),
                ProductType::Fluid(_) => None,
            })
            .collect();

        Self {
            id: def.id.clone(),
            name: def.i18n_key.replace("recipe.", ""),
            inputs,
            input_fluid: None,
            outputs,
            output_fluid: None,
            craft_time: def.process_time,
            work_type: def.machine_type.to_work_type(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_to_work_type() {
        assert_eq!(MachineType::Press.to_work_type(), WorkType::Pressing);
        assert_eq!(MachineType::Crusher.to_work_type(), WorkType::Crushing);
        assert_eq!(MachineType::Mixer.to_work_type(), WorkType::Mixing);
    }

    #[test]
    fn test_recipe_conversion() {
        let def = RecipeDef {
            id: "iron_plate".to_string(),
            machine_type: MachineType::Press,
            ingredients: vec![Ingredient {
                ingredient_type: IngredientType::Item("iron_ingot".to_string()),
                amount: 1,
            }],
            results: vec![Product {
                product_type: ProductType::Item("iron_plate".to_string()),
                amount: 1,
                chance: 1.0,
            }],
            process_time: 2.0,
            stress_impact: 4.0,
            i18n_key: "recipe.iron_plate".to_string(),
        };

        let game_recipe = GameRecipe::from(&def);
        assert_eq!(game_recipe.id, "iron_plate");
        assert_eq!(game_recipe.work_type, WorkType::Pressing);
        assert_eq!(game_recipe.inputs.len(), 1);
        assert_eq!(game_recipe.outputs.len(), 1);
    }
}
