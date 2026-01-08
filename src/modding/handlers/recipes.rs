//! Recipe-related JSON-RPC handlers
//!
//! Implements `recipe.list` and `recipe.add` methods.

use crate::game_spec::recipes::{get_recipes_for_machine, MachineType, ALL_RECIPES};
use crate::modding::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Recipe info returned by recipe.list
#[derive(Debug, Serialize)]
pub struct RecipeInfo {
    /// Recipe ID
    pub id: String,
    /// Machine type (furnace, crusher, assembler)
    pub machine_type: String,
    /// Input item IDs with counts
    pub inputs: Vec<RecipeItemInfo>,
    /// Output item IDs with counts
    pub outputs: Vec<RecipeItemInfo>,
    /// Processing time in seconds
    pub time: f32,
    /// Fuel requirement (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel: Option<FuelInfo>,
}

/// Item reference in a recipe (input or output)
#[derive(Debug, Serialize)]
pub struct RecipeItemInfo {
    /// Item ID (e.g., "base:iron_ore")
    pub item: String,
    /// Count
    pub count: u32,
}

/// Fuel requirement info
#[derive(Debug, Serialize)]
pub struct FuelInfo {
    /// Fuel item ID
    pub item: String,
    /// Amount consumed per processing
    pub amount: u32,
}

/// Parameters for recipe.list
#[derive(Debug, Deserialize)]
struct RecipeListParams {
    /// Optional filter by machine type
    #[serde(default)]
    machine_type: Option<String>,
}

/// Response for recipe.list
#[derive(Debug, Serialize)]
struct RecipeListResponse {
    recipes: Vec<RecipeInfo>,
}

/// Parameters for recipe.add
#[derive(Debug, Deserialize)]
struct RecipeAddParams {
    /// Recipe ID (e.g., "mymod:super_smelt")
    id: String,
    /// Input item IDs
    inputs: Vec<String>,
    /// Output item IDs
    outputs: Vec<String>,
    /// Processing time in seconds
    time: f32,
}

/// Response for recipe.add
#[derive(Debug, Serialize)]
struct RecipeAddResponse {
    success: bool,
    id: String,
}

/// Convert MachineType to string
fn machine_type_to_string(machine: MachineType) -> &'static str {
    match machine {
        MachineType::Furnace => "furnace",
        MachineType::Crusher => "crusher",
        MachineType::Assembler => "assembler",
    }
}

/// Parse machine type from string
fn parse_machine_type(s: &str) -> Option<MachineType> {
    match s.to_lowercase().as_str() {
        "furnace" => Some(MachineType::Furnace),
        "crusher" => Some(MachineType::Crusher),
        "assembler" => Some(MachineType::Assembler),
        _ => None,
    }
}

/// Convert RecipeSpec to RecipeInfo
fn recipe_to_info(recipe: &crate::game_spec::recipes::RecipeSpec) -> RecipeInfo {
    RecipeInfo {
        id: recipe.id.to_string(),
        machine_type: machine_type_to_string(recipe.machine).to_string(),
        inputs: recipe
            .inputs
            .iter()
            .map(|input| RecipeItemInfo {
                item: input.item_id().name().unwrap_or("unknown").to_string(),
                count: input.count,
            })
            .collect(),
        outputs: recipe
            .outputs
            .iter()
            .map(|output| RecipeItemInfo {
                item: output.item_id().name().unwrap_or("unknown").to_string(),
                count: output.count,
            })
            .collect(),
        time: recipe.craft_time,
        fuel: recipe.fuel.map(|f| FuelInfo {
            item: f.fuel_id().name().unwrap_or("unknown").to_string(),
            amount: f.amount,
        }),
    }
}

/// Handle recipe.list method
///
/// Parameters:
/// - `machine_type` (optional): Filter by machine type ("furnace", "crusher", "assembler")
///
/// Returns:
/// - `recipes`: Array of recipe info objects
pub fn handle_recipe_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    // Parse parameters (allow null/empty)
    let params: RecipeListParams = if request.params.is_null() {
        RecipeListParams { machine_type: None }
    } else {
        match serde_json::from_value(request.params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    request.id,
                    INVALID_PARAMS,
                    format!("Invalid params: {}", e),
                );
            }
        }
    };

    // Get recipes (optionally filtered by machine type)
    let recipes: Vec<RecipeInfo> = match &params.machine_type {
        Some(machine_str) => {
            match parse_machine_type(machine_str) {
                Some(machine_type) => get_recipes_for_machine(machine_type)
                    .map(recipe_to_info)
                    .collect(),
                None => {
                    // Invalid machine type - return error
                    return JsonRpcResponse::error(
                        request.id,
                        INVALID_PARAMS,
                        format!(
                            "Unknown machine_type: {}. Valid values: furnace, crusher, assembler",
                            machine_str
                        ),
                    );
                }
            }
        }
        None => {
            // No filter - return all recipes
            ALL_RECIPES.iter().map(|r| recipe_to_info(r)).collect()
        }
    };

    let response = RecipeListResponse { recipes };

    JsonRpcResponse::success(
        request.id,
        serde_json::to_value(response).unwrap_or_default(),
    )
}

/// Handle recipe.add method (stub)
///
/// Parameters:
/// - `id`: Recipe ID (e.g., "mymod:super_smelt")
/// - `inputs`: Array of input item IDs
/// - `outputs`: Array of output item IDs
/// - `time`: Processing time in seconds
///
/// Returns:
/// - `success`: true
/// - `id`: The registered recipe ID
///
/// Note: Currently a stub that only logs the request.
/// Dynamic recipe registration will be implemented in a future phase.
pub fn handle_recipe_add(request: &JsonRpcRequest) -> JsonRpcResponse {
    // Parse parameters
    let params: RecipeAddParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Validate required fields
    if params.id.is_empty() {
        return JsonRpcResponse::error(request.id, INVALID_PARAMS, "Recipe id is required");
    }
    if params.inputs.is_empty() {
        return JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            "At least one input is required",
        );
    }
    if params.outputs.is_empty() {
        return JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            "At least one output is required",
        );
    }
    if params.time <= 0.0 {
        return JsonRpcResponse::error(request.id, INVALID_PARAMS, "time must be positive");
    }

    // TODO: Actually register the recipe when dynamic recipe system is implemented
    info!(
        "recipe.add stub: id={}, inputs={:?}, outputs={:?}, time={}",
        params.id, params.inputs, params.outputs, params.time
    );

    let response = RecipeAddResponse {
        success: true,
        id: params.id,
    };

    JsonRpcResponse::success(
        request.id,
        serde_json::to_value(response).unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_recipe_list_all() {
        let request = JsonRpcRequest::new(1, "recipe.list", serde_json::Value::Null);
        let response = handle_recipe_list(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let recipes = result["recipes"].as_array().unwrap();

        // Should return all recipes (11 total)
        assert_eq!(recipes.len(), 11);
    }

    #[test]
    fn test_recipe_list_furnace_only() {
        let request = JsonRpcRequest::new(1, "recipe.list", json!({ "machine_type": "furnace" }));
        let response = handle_recipe_list(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let recipes = result["recipes"].as_array().unwrap();

        // Furnace has 4 recipes (2 ore + 2 dust)
        assert_eq!(recipes.len(), 4);

        // All should be furnace type
        for recipe in recipes {
            assert_eq!(recipe["machine_type"], "furnace");
        }
    }

    #[test]
    fn test_recipe_list_crusher_only() {
        let request = JsonRpcRequest::new(1, "recipe.list", json!({ "machine_type": "crusher" }));
        let response = handle_recipe_list(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let recipes = result["recipes"].as_array().unwrap();

        // Crusher has 2 recipes
        assert_eq!(recipes.len(), 2);
    }

    #[test]
    fn test_recipe_list_assembler_only() {
        let request = JsonRpcRequest::new(1, "recipe.list", json!({ "machine_type": "assembler" }));
        let response = handle_recipe_list(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let recipes = result["recipes"].as_array().unwrap();

        // Assembler has 5 recipes
        assert_eq!(recipes.len(), 5);
    }

    #[test]
    fn test_recipe_list_invalid_machine_type() {
        let request = JsonRpcRequest::new(1, "recipe.list", json!({ "machine_type": "invalid" }));
        let response = handle_recipe_list(&request);

        assert!(response.is_error());
        assert_eq!(response.error.as_ref().unwrap().code, INVALID_PARAMS);
    }

    #[test]
    fn test_recipe_list_response_format() {
        let request = JsonRpcRequest::new(1, "recipe.list", json!({ "machine_type": "furnace" }));
        let response = handle_recipe_list(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let recipes = result["recipes"].as_array().unwrap();

        // Check first recipe structure
        let recipe = &recipes[0];
        assert!(recipe["id"].is_string());
        assert!(recipe["machine_type"].is_string());
        assert!(recipe["inputs"].is_array());
        assert!(recipe["outputs"].is_array());
        assert!(recipe["time"].is_number());

        // Check input structure
        let input = &recipe["inputs"][0];
        assert!(input["item"].is_string());
        assert!(input["count"].is_number());
    }

    #[test]
    fn test_recipe_add_success() {
        let request = JsonRpcRequest::new(
            1,
            "recipe.add",
            json!({
                "id": "mymod:super_smelt",
                "inputs": ["base:iron_ore"],
                "outputs": ["base:iron_ingot"],
                "time": 1.0
            }),
        );
        let response = handle_recipe_add(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["id"], "mymod:super_smelt");
    }

    #[test]
    fn test_recipe_add_missing_id() {
        let request = JsonRpcRequest::new(
            1,
            "recipe.add",
            json!({
                "id": "",
                "inputs": ["base:iron_ore"],
                "outputs": ["base:iron_ingot"],
                "time": 1.0
            }),
        );
        let response = handle_recipe_add(&request);

        assert!(response.is_error());
        assert!(response
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("id is required"));
    }

    #[test]
    fn test_recipe_add_empty_inputs() {
        let request = JsonRpcRequest::new(
            1,
            "recipe.add",
            json!({
                "id": "mymod:test",
                "inputs": [],
                "outputs": ["base:iron_ingot"],
                "time": 1.0
            }),
        );
        let response = handle_recipe_add(&request);

        assert!(response.is_error());
        assert!(response.error.as_ref().unwrap().message.contains("input"));
    }

    #[test]
    fn test_recipe_add_empty_outputs() {
        let request = JsonRpcRequest::new(
            1,
            "recipe.add",
            json!({
                "id": "mymod:test",
                "inputs": ["base:iron_ore"],
                "outputs": [],
                "time": 1.0
            }),
        );
        let response = handle_recipe_add(&request);

        assert!(response.is_error());
        assert!(response.error.as_ref().unwrap().message.contains("output"));
    }

    #[test]
    fn test_recipe_add_invalid_time() {
        let request = JsonRpcRequest::new(
            1,
            "recipe.add",
            json!({
                "id": "mymod:test",
                "inputs": ["base:iron_ore"],
                "outputs": ["base:iron_ingot"],
                "time": 0.0
            }),
        );
        let response = handle_recipe_add(&request);

        assert!(response.is_error());
        assert!(response.error.as_ref().unwrap().message.contains("time"));
    }

    #[test]
    fn test_recipe_add_invalid_params() {
        let request = JsonRpcRequest::new(1, "recipe.add", json!("invalid"));
        let response = handle_recipe_add(&request);

        assert!(response.is_error());
        assert_eq!(response.error.as_ref().unwrap().code, INVALID_PARAMS);
    }

    #[test]
    fn test_machine_type_to_string() {
        assert_eq!(machine_type_to_string(MachineType::Furnace), "furnace");
        assert_eq!(machine_type_to_string(MachineType::Crusher), "crusher");
        assert_eq!(machine_type_to_string(MachineType::Assembler), "assembler");
    }

    #[test]
    fn test_parse_machine_type() {
        assert_eq!(parse_machine_type("furnace"), Some(MachineType::Furnace));
        assert_eq!(parse_machine_type("FURNACE"), Some(MachineType::Furnace));
        assert_eq!(parse_machine_type("crusher"), Some(MachineType::Crusher));
        assert_eq!(
            parse_machine_type("assembler"),
            Some(MachineType::Assembler)
        );
        assert_eq!(parse_machine_type("invalid"), None);
    }
}
