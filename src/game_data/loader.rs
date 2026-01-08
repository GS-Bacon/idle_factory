//! JSON file loader for game data

use super::*;
use std::fs;
use std::path::Path;

/// Load all game data from JSON files
pub fn load_all_data() -> Result<GameData, String> {
    let base_path = Path::new("assets/data");

    let mut data = GameData::default();

    // Load recipes
    let recipes_path = base_path.join("recipes.json");
    if recipes_path.exists() {
        let content = fs::read_to_string(&recipes_path)
            .map_err(|e| format!("Failed to read recipes.json: {}", e))?;
        let recipes_file: RecipesFile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse recipes.json: {}", e))?;
        data.recipes = recipes_file.recipes;
    }

    // Load quests
    let quests_path = base_path.join("quests.json");
    if quests_path.exists() {
        let content = fs::read_to_string(&quests_path)
            .map_err(|e| format!("Failed to read quests.json: {}", e))?;
        let quests_file: QuestsFile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse quests.json: {}", e))?;
        data.main_quests = quests_file.main_quests;
        data.sub_quests = quests_file.sub_quests;
        data.initial_equipment = GameData::parse_item_counts(&quests_file.initial_equipment);
    }

    // Load machines
    let machines_path = base_path.join("machines.json");
    if machines_path.exists() {
        let content = fs::read_to_string(&machines_path)
            .map_err(|e| format!("Failed to read machines.json: {}", e))?;
        let machines_file: MachinesFile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse machines.json: {}", e))?;
        data.machines = machines_file.machines;
        data.machine_constants = machines_file.constants;
    }

    // Build indices
    data.build_indices();

    Ok(data)
}

/// Load recipes only
#[allow(dead_code)]
pub fn load_recipes() -> Result<Vec<RecipeData>, String> {
    let path = Path::new("assets/data/recipes.json");
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read recipes.json: {}", e))?;
    let file: RecipesFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse recipes.json: {}", e))?;
    Ok(file.recipes)
}

/// Load quests only
#[allow(dead_code)]
pub fn load_quests() -> Result<QuestsFile, String> {
    let path = Path::new("assets/data/quests.json");
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read quests.json: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse quests.json: {}", e))
}

/// Load machines only
#[allow(dead_code)]
pub fn load_machines() -> Result<MachinesFile, String> {
    let path = Path::new("assets/data/machines.json");
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read machines.json: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse machines.json: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_data() {
        // This test requires the JSON files to exist
        let result = load_all_data();
        assert!(result.is_ok(), "Failed to load game data: {:?}", result);

        let data = result.unwrap();
        assert!(!data.recipes.is_empty(), "Recipes should not be empty");
        assert!(
            !data.main_quests.is_empty(),
            "Main quests should not be empty"
        );
        assert!(!data.machines.is_empty(), "Machines should not be empty");
    }

    #[test]
    fn test_recipe_lookup() {
        let result = load_all_data();
        assert!(result.is_ok());

        let data = result.unwrap();
        let furnace_recipes = data.get_recipes_for_machine("furnace");
        assert!(!furnace_recipes.is_empty(), "Furnace should have recipes");

        // Find iron smelting recipe
        let iron_recipe = data.find_recipe("furnace", crate::core::items::iron_ore());
        assert!(iron_recipe.is_some(), "Should find iron smelting recipe");
        assert_eq!(iron_recipe.unwrap().id, "smelt_iron");
    }

    #[test]
    fn test_machine_lookup() {
        let result = load_all_data();
        assert!(result.is_ok());

        let data = result.unwrap();
        let miner = data.get_machine(crate::core::items::miner_block());
        assert!(miner.is_some(), "Should find miner");
        assert_eq!(miner.unwrap().id, "miner");

        let stone = data.get_machine(crate::core::items::stone());
        assert!(stone.is_none(), "Stone is not a machine");
    }
}
