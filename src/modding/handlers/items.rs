//! Item-related JSON-RPC handlers
//!
//! Handles `item.list` and `item.add` methods.

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::core::items;
use crate::game_spec::item_descriptors;
use crate::modding::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};

/// Item info returned by item.list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemInfo {
    /// Full item ID (e.g., "base:iron_ore")
    pub id: String,
    /// Display name
    pub name: String,
    /// Max stack size
    pub stack_size: u32,
}

/// Parameters for item.list
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemListParams {
    /// Optional namespace filter (e.g., "base")
    #[serde(default)]
    pub namespace: Option<String>,
}

/// Result of item.list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemListResult {
    /// List of items
    pub items: Vec<ItemInfo>,
}

/// Parameters for item.add
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAddParams {
    /// Item ID (e.g., "mymod:super_ingot")
    pub id: String,
    /// Display name
    pub name: String,
    /// Stack size (optional, defaults to 64)
    #[serde(default = "default_stack_size")]
    pub stack_size: u32,
}

fn default_stack_size() -> u32 {
    64
}

/// Result of item.add
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAddResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// The registered item ID
    pub id: String,
}

/// Error codes for item operations
pub const ITEM_ALREADY_EXISTS: i32 = -32010;
pub const INVALID_ITEM_ID: i32 = -32011;

/// Handle item.list request
///
/// Returns a list of all registered items, optionally filtered by namespace.
pub fn handle_item_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    // Parse params (optional)
    let list_params: ItemListParams =
        serde_json::from_value(request.params.clone()).unwrap_or_default();

    let interner = items::interner();
    let mut items_list = Vec::new();

    // Get all registered items from item_descriptors()
    for (item_id, descriptor) in item_descriptors() {
        let Some(string_id) = item_id.to_string_id(interner) else {
            continue;
        };

        // Filter by namespace if specified
        if let Some(ref ns) = list_params.namespace {
            if let Some(item_ns) = item_id.namespace(interner) {
                if item_ns != ns {
                    continue;
                }
            } else {
                continue;
            }
        }

        items_list.push(ItemInfo {
            id: string_id.to_string(),
            name: descriptor.name.to_string(),
            stack_size: descriptor.stack_size,
        });
    }

    let result = ItemListResult { items: items_list };
    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
}

/// Handle item.add request
///
/// Currently a stub - logs the request but does not actually add items.
/// Dynamic item addition will be implemented in a future version.
pub fn handle_item_add(request: &JsonRpcRequest) -> JsonRpcResponse {
    // Parse params
    let add_params: ItemAddParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Validate ID format (namespace:name)
    if !add_params.id.contains(':') {
        return JsonRpcResponse::error(
            request.id,
            INVALID_ITEM_ID,
            "Item ID must be in 'namespace:name' format",
        );
    }

    // Check for duplicate ID
    let interner = items::interner();
    if interner.get(&add_params.id).is_some() {
        return JsonRpcResponse::error(
            request.id,
            ITEM_ALREADY_EXISTS,
            format!("Item already exists: {}", add_params.id),
        );
    }

    // Stub: Log the request but don't actually add the item yet
    info!(
        "item.add stub: id={}, name={}, stack_size={}",
        add_params.id, add_params.name, add_params.stack_size
    );
    warn!("Dynamic item addition not yet implemented - item.add is currently a stub");

    let result = ItemAddResult {
        success: true,
        id: add_params.id,
    };

    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_handle_item_list_all() {
        let request = JsonRpcRequest::new(1, "item.list", json!({}));
        let response = handle_item_list(&request);

        assert!(response.is_success());
        let result: ItemListResult = serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(!result.items.is_empty());

        // Check that iron_ore is in the list
        let iron_ore = result.items.iter().find(|i| i.id == "base:iron_ore");
        assert!(iron_ore.is_some());
        assert_eq!(iron_ore.unwrap().name, "Iron Ore");
        assert_eq!(iron_ore.unwrap().stack_size, 999);
    }

    #[test]
    fn test_handle_item_list_with_namespace() {
        let request = JsonRpcRequest::new(1, "item.list", json!({"namespace": "base"}));
        let response = handle_item_list(&request);

        assert!(response.is_success());
        let result: ItemListResult = serde_json::from_value(response.result.unwrap()).unwrap();

        // All items should be in base namespace
        for item in &result.items {
            assert!(
                item.id.starts_with("base:"),
                "Item {} not in base namespace",
                item.id
            );
        }
    }

    #[test]
    fn test_handle_item_list_empty_namespace() {
        // Request items from a namespace that doesn't exist
        let request = JsonRpcRequest::new(1, "item.list", json!({"namespace": "nonexistent"}));
        let response = handle_item_list(&request);

        assert!(response.is_success());
        let result: ItemListResult = serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(result.items.is_empty());
    }

    #[test]
    fn test_handle_item_add_stub() {
        let request = JsonRpcRequest::new(
            1,
            "item.add",
            json!({
                "id": "mymod:super_ingot",
                "name": "Super Ingot",
                "stack_size": 64
            }),
        );
        let response = handle_item_add(&request);

        assert!(response.is_success());
        let result: ItemAddResult = serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(result.success);
        assert_eq!(result.id, "mymod:super_ingot");
    }

    #[test]
    fn test_handle_item_add_default_stack_size() {
        let request = JsonRpcRequest::new(
            1,
            "item.add",
            json!({
                "id": "mymod:another_item",
                "name": "Another Item"
            }),
        );
        let response = handle_item_add(&request);

        assert!(response.is_success());
        let result: ItemAddResult = serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_handle_item_add_invalid_id_format() {
        let request = JsonRpcRequest::new(
            1,
            "item.add",
            json!({
                "id": "invalid_no_namespace",
                "name": "Bad Item"
            }),
        );
        let response = handle_item_add(&request);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, INVALID_ITEM_ID);
    }

    #[test]
    fn test_handle_item_add_missing_required_params() {
        let request = JsonRpcRequest::new(
            1,
            "item.add",
            json!({
                "id": "mymod:item"
                // missing "name"
            }),
        );
        let response = handle_item_add(&request);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, INVALID_PARAMS);
    }

    #[test]
    fn test_handle_item_add_duplicate_id() {
        // Try to add an item that already exists
        let request = JsonRpcRequest::new(
            1,
            "item.add",
            json!({
                "id": "base:stone",
                "name": "Duplicate Stone"
            }),
        );
        let response = handle_item_add(&request);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, ITEM_ALREADY_EXISTS);
    }

    #[test]
    fn test_item_info_serialization() {
        let info = ItemInfo {
            id: "base:test".to_string(),
            name: "Test".to_string(),
            stack_size: 64,
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ItemInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, info.id);
        assert_eq!(parsed.name, info.name);
        assert_eq!(parsed.stack_size, info.stack_size);
    }
}
