//! Craft API handlers for E2E testing
//!
//! Provides APIs to:
//! - List all crafting recipes
//! - Check if a recipe can be crafted
//! - Get recipe details

use super::super::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use serde::{Deserialize, Serialize};

// Error codes
pub const RECIPE_NOT_FOUND: i32 = -32130;

/// Get default crafting recipes (same as setup_default_recipes in craft/mod.rs)
fn get_default_recipes() -> crate::craft::CraftingRegistry {
    use crate::core::items;
    use crate::craft::{CraftingRecipe, CraftingRegistry, CraftingStation};

    let mut registry = CraftingRegistry::new();

    // 手持ちクラフト - 石のツール
    registry.register(
        CraftingRecipe::builder("stone_pickaxe", CraftingStation::Hand, 2.0)
            .input(items::stone(), 3)
            .output(items::stone(), 1)
            .build(),
    );

    // 手持ちクラフト - 松明（石炭使用）
    registry.register(
        CraftingRecipe::builder("torch", CraftingStation::Hand, 1.0)
            .input(items::coal(), 1)
            .input(items::stone(), 1)
            .output(items::coal(), 4)
            .build(),
    );

    // 作業台クラフト - 鉄プレート
    registry.register(
        CraftingRecipe::builder("iron_plate", CraftingStation::Workbench, 3.0)
            .input(items::iron_ingot(), 2)
            .output(items::iron_ingot(), 1)
            .build(),
    );

    // 作業台クラフト - 銅線
    registry.register(
        CraftingRecipe::builder("copper_wire", CraftingStation::Workbench, 2.0)
            .input(items::copper_ingot(), 1)
            .output(items::copper_ingot(), 2)
            .build(),
    );

    registry
}

/// Crafting recipe info for API responses
#[derive(Serialize, Clone)]
pub struct CraftRecipeInfo {
    pub name: String,
    pub station: String,
    pub craft_time: f32,
    pub inputs: Vec<CraftItemInfo>,
    pub outputs: Vec<CraftItemInfo>,
    pub unlocked: bool,
}

/// Craft item info
#[derive(Serialize, Clone)]
pub struct CraftItemInfo {
    pub item_id: String,
    pub count: u32,
}

// === craft.list ===

#[derive(Deserialize, Default)]
pub struct CraftListParams {
    /// Filter by station (optional)
    pub station: Option<String>,
}

/// Handle craft.list request
///
/// Returns all crafting recipes.
///
/// # Request
/// ```json
/// { "station": "Hand" }  // optional filter
/// ```
///
/// # Response
/// ```json
/// {
///   "recipes": [
///     {
///       "name": "stone_pickaxe",
///       "station": "Hand",
///       "craft_time": 2.0,
///       "inputs": [...],
///       "outputs": [...],
///       "unlocked": true
///     }
///   ]
/// }
/// ```
pub fn handle_craft_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    use crate::craft::CraftingStation;

    let params: CraftListParams =
        serde_json::from_value(request.params.clone()).unwrap_or_default();

    let registry = get_default_recipes();

    let station_filter = params.station.as_ref().and_then(|s| match s.as_str() {
        "Hand" => Some(CraftingStation::Hand),
        "Workbench" => Some(CraftingStation::Workbench),
        "Forge" => Some(CraftingStation::Forge),
        _ => None,
    });

    let recipes: Vec<CraftRecipeInfo> = registry
        .all()
        .filter(|r| station_filter.is_none() || Some(r.station) == station_filter)
        .map(|r| {
            let inputs: Vec<CraftItemInfo> = r
                .inputs
                .iter()
                .map(|i| CraftItemInfo {
                    item_id: i.item.name().unwrap_or("unknown").to_string(),
                    count: i.count,
                })
                .collect();

            let outputs: Vec<CraftItemInfo> = r
                .outputs
                .iter()
                .map(|o| CraftItemInfo {
                    item_id: o.item.name().unwrap_or("unknown").to_string(),
                    count: o.count,
                })
                .collect();

            CraftRecipeInfo {
                name: r.name.to_string(),
                station: format!("{:?}", r.station),
                craft_time: r.craft_time,
                inputs,
                outputs,
                unlocked: r.unlocked,
            }
        })
        .collect();

    JsonRpcResponse::success(request.id, serde_json::json!({ "recipes": recipes }))
}

// === craft.get ===

#[derive(Deserialize)]
pub struct CraftGetParams {
    pub name: String,
}

/// Handle craft.get request
///
/// Returns details for a specific recipe.
///
/// # Request
/// ```json
/// { "name": "stone_pickaxe" }
/// ```
pub fn handle_craft_get(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: CraftGetParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    let registry = get_default_recipes();

    match registry.get(&params.name) {
        Some(r) => {
            let inputs: Vec<CraftItemInfo> = r
                .inputs
                .iter()
                .map(|i| CraftItemInfo {
                    item_id: i.item.name().unwrap_or("unknown").to_string(),
                    count: i.count,
                })
                .collect();

            let outputs: Vec<CraftItemInfo> = r
                .outputs
                .iter()
                .map(|o| CraftItemInfo {
                    item_id: o.item.name().unwrap_or("unknown").to_string(),
                    count: o.count,
                })
                .collect();

            let info = CraftRecipeInfo {
                name: r.name.to_string(),
                station: format!("{:?}", r.station),
                craft_time: r.craft_time,
                inputs,
                outputs,
                unlocked: r.unlocked,
            };

            JsonRpcResponse::success(request.id, serde_json::to_value(info).unwrap())
        }
        None => JsonRpcResponse::error(
            request.id,
            RECIPE_NOT_FOUND,
            format!("Recipe not found: {}", params.name),
        ),
    }
}

// === craft.can_craft ===

#[derive(Deserialize)]
pub struct CanCraftParams {
    pub name: String,
}

/// Handle craft.can_craft request
///
/// Checks if a recipe can be crafted with current inventory.
///
/// # Request
/// ```json
/// { "name": "stone_pickaxe" }
/// ```
///
/// # Response
/// ```json
/// { "can_craft": true, "missing": [] }
/// ```
pub fn handle_craft_can_craft(
    request: &JsonRpcRequest,
    inventory_state: &super::inventory::InventoryStateInfo,
) -> JsonRpcResponse {
    use crate::core::items;
    use std::collections::HashMap;

    let params: CanCraftParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    let registry = get_default_recipes();

    let recipe = match registry.get(&params.name) {
        Some(r) => r,
        None => {
            return JsonRpcResponse::error(
                request.id,
                RECIPE_NOT_FOUND,
                format!("Recipe not found: {}", params.name),
            );
        }
    };

    // Build inventory map from state
    let mut inventory: HashMap<crate::core::ItemId, u32> = HashMap::new();
    for slot in &inventory_state.slots {
        if let Some(ref item_id_str) = slot.item_id {
            if let Some(item_id) = items::by_name(item_id_str) {
                *inventory.entry(item_id).or_insert(0) += slot.amount;
            }
        }
    }

    let can_craft = recipe.can_craft(&inventory);

    // Find missing items
    let mut missing: Vec<serde_json::Value> = Vec::new();
    for input in &recipe.inputs {
        let have = inventory.get(&input.item).copied().unwrap_or(0);
        if have < input.count {
            missing.push(serde_json::json!({
                "item_id": input.item.name().unwrap_or("unknown"),
                "required": input.count,
                "have": have,
            }));
        }
    }

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "can_craft": can_craft,
            "missing": missing,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::protocol::JsonRpcRequest;

    #[test]
    fn test_handle_craft_list() {
        let request = JsonRpcRequest::new(1, "craft.list", serde_json::Value::Null);
        let response = handle_craft_list(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        let recipes = result["recipes"].as_array().unwrap();
        assert!(!recipes.is_empty());
    }

    #[test]
    fn test_handle_craft_list_with_station_filter() {
        let request =
            JsonRpcRequest::new(1, "craft.list", serde_json::json!({ "station": "Hand" }));
        let response = handle_craft_list(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        let recipes = result["recipes"].as_array().unwrap();
        // All returned recipes should be Hand station
        for r in recipes {
            assert_eq!(r["station"], "Hand");
        }
    }

    #[test]
    fn test_handle_craft_get() {
        let request = JsonRpcRequest::new(
            1,
            "craft.get",
            serde_json::json!({ "name": "stone_pickaxe" }),
        );
        let response = handle_craft_get(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["name"], "stone_pickaxe");
    }

    #[test]
    fn test_handle_craft_get_not_found() {
        let request =
            JsonRpcRequest::new(1, "craft.get", serde_json::json!({ "name": "nonexistent" }));
        let response = handle_craft_get(&request);
        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, RECIPE_NOT_FOUND);
    }

    #[test]
    fn test_handle_craft_can_craft_empty_inventory() {
        use super::super::inventory::{InventoryStateInfo, SlotInfo};

        let inventory_state = InventoryStateInfo {
            slots: vec![SlotInfo {
                index: 0,
                item_id: None,
                amount: 0,
            }],
            ..Default::default()
        };

        let request = JsonRpcRequest::new(
            1,
            "craft.can_craft",
            serde_json::json!({ "name": "stone_pickaxe" }),
        );
        let response = handle_craft_can_craft(&request, &inventory_state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["can_craft"], false);
        assert!(!result["missing"].as_array().unwrap().is_empty());
    }
}
