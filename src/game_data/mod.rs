//! Game data loader
//!
//! Loads game configuration from JSON files at runtime.
//! This allows for easy modding and data-driven game design.

use crate::BlockType;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

mod loader;

pub use loader::*;

// =============================================================================
// Recipe Data Structures
// =============================================================================

/// Recipe input from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct RecipeInputData {
    pub item: String,
    pub count: u32,
    pub slot: u8,
}

/// Recipe output from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct RecipeOutputData {
    pub item: String,
    pub count: u32,
    pub chance: f32,
}

/// Fuel requirement from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct FuelData {
    pub fuel_type: String,
    pub amount: u32,
}

/// Recipe from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct RecipeData {
    pub id: String,
    pub machine: String,
    pub inputs: Vec<RecipeInputData>,
    pub outputs: Vec<RecipeOutputData>,
    pub craft_time: f32,
    pub fuel: Option<FuelData>,
}

/// Recipes file structure
#[derive(Debug, Clone, Deserialize)]
pub struct RecipesFile {
    pub recipes: Vec<RecipeData>,
}

// =============================================================================
// Quest Data Structures
// =============================================================================

/// Item count pair from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct ItemCountData {
    pub item: String,
    pub count: u32,
}

/// Quest from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct QuestData {
    pub id: String,
    pub description: String,
    pub required_items: Vec<ItemCountData>,
    pub rewards: Vec<ItemCountData>,
    pub unlocks: Vec<String>,
}

/// Quests file structure
#[derive(Debug, Clone, Deserialize)]
pub struct QuestsFile {
    pub main_quests: Vec<QuestData>,
    pub sub_quests: Vec<QuestData>,
    pub initial_equipment: Vec<ItemCountData>,
}

// =============================================================================
// Machine Data Structures
// =============================================================================

/// I/O port from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct PortData {
    pub side: String,
    pub is_input: bool,
    pub slot_id: u8,
}

/// Machine from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct MachineData {
    pub id: String,
    pub name: String,
    pub block_type: String,
    pub ports: Vec<PortData>,
    pub buffer_size: u32,
    pub process_time: f32,
    pub requires_fuel: bool,
    pub auto_generate: bool,
}

/// Machine constants from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct MachineConstants {
    pub miner_interval: f32,
    pub furnace_smelt_time: f32,
    pub crusher_crush_time: f32,
    pub assembler_base_time: f32,
    pub conveyor_speed: f32,
}

/// Machines file structure
#[derive(Debug, Clone, Deserialize)]
pub struct MachinesFile {
    pub machines: Vec<MachineData>,
    pub constants: MachineConstants,
}

// =============================================================================
// Loaded Game Data Resource
// =============================================================================

/// All game data loaded from JSON files
#[derive(Resource, Debug, Clone)]
pub struct GameData {
    pub recipes: Vec<RecipeData>,
    pub main_quests: Vec<QuestData>,
    pub sub_quests: Vec<QuestData>,
    pub initial_equipment: Vec<(BlockType, u32)>,
    pub machines: Vec<MachineData>,
    pub machine_constants: MachineConstants,
    /// Recipe lookup by machine type
    pub recipes_by_machine: HashMap<String, Vec<usize>>,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            recipes: Vec::new(),
            main_quests: Vec::new(),
            sub_quests: Vec::new(),
            initial_equipment: Vec::new(),
            machines: Vec::new(),
            machine_constants: MachineConstants {
                miner_interval: 1.5,
                furnace_smelt_time: 2.0,
                crusher_crush_time: 1.5,
                assembler_base_time: 3.0,
                conveyor_speed: 2.0,
            },
            recipes_by_machine: HashMap::new(),
        }
    }
}

impl GameData {
    /// Build recipe lookup index
    pub fn build_indices(&mut self) {
        self.recipes_by_machine.clear();
        for (idx, recipe) in self.recipes.iter().enumerate() {
            self.recipes_by_machine
                .entry(recipe.machine.clone())
                .or_default()
                .push(idx);
        }
    }

    /// Find recipes for a machine type
    pub fn get_recipes_for_machine(&self, machine: &str) -> Vec<&RecipeData> {
        self.recipes_by_machine
            .get(machine)
            .map(|indices| indices.iter().map(|&i| &self.recipes[i]).collect())
            .unwrap_or_default()
    }

    /// Find a recipe by input item and machine
    pub fn find_recipe(&self, machine: &str, input: BlockType) -> Option<&RecipeData> {
        let input_str = input.to_string();
        self.get_recipes_for_machine(machine)
            .into_iter()
            .find(|r| r.inputs.iter().any(|i| i.item == input_str))
    }

    /// Get machine data by block type
    pub fn get_machine(&self, block_type: BlockType) -> Option<&MachineData> {
        let block_str = block_type.to_string();
        self.machines.iter().find(|m| m.block_type == block_str)
    }

    /// Convert item count data to BlockType pairs
    pub fn parse_item_counts(items: &[ItemCountData]) -> Vec<(BlockType, u32)> {
        items
            .iter()
            .filter_map(|item| {
                BlockType::from_str(&item.item)
                    .ok()
                    .map(|bt| (bt, item.count))
            })
            .collect()
    }
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that loads game data from JSON files
pub struct GameDataPlugin;

impl Plugin for GameDataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameData>()
            .add_systems(PreStartup, load_game_data);
    }
}

/// System to load game data at startup
fn load_game_data(mut game_data: ResMut<GameData>) {
    match loader::load_all_data() {
        Ok(data) => {
            *game_data = data;
            tracing::info!(
                "Loaded game data: {} recipes, {} main quests, {} machines",
                game_data.recipes.len(),
                game_data.main_quests.len(),
                game_data.machines.len()
            );
        }
        Err(e) => {
            tracing::warn!("Failed to load game data from JSON: {}. Using defaults.", e);
            // Keep default values
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_data_default() {
        let data = GameData::default();
        assert!(data.recipes.is_empty());
        assert!(data.machines.is_empty());
    }

    #[test]
    fn test_parse_item_counts() {
        let items = vec![
            ItemCountData {
                item: "iron_ingot".to_string(),
                count: 10,
            },
            ItemCountData {
                item: "coal".to_string(),
                count: 5,
            },
        ];

        let parsed = GameData::parse_item_counts(&items);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], (BlockType::IronIngot, 10));
        assert_eq!(parsed[1], (BlockType::Coal, 5));
    }
}
