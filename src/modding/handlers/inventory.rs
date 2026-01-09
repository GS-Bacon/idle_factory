//! Inventory API handlers for E2E testing
//!
//! Provides APIs to:
//! - Query inventory slots
//! - Move items between slots
//! - Get player hotbar state

use super::super::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use serde::{Deserialize, Serialize};

// Error codes
pub const SLOT_OUT_OF_RANGE: i32 = -32100;
pub const ITEM_NOT_FOUND: i32 = -32101;
pub const SLOT_FULL: i32 = -32102;

/// Inventory slot info for API responses
#[derive(Serialize, Clone, Default)]
pub struct SlotInfo {
    pub index: usize,
    pub item_id: Option<String>,
    pub amount: u32,
}

/// Player inventory state for E2E testing
#[derive(Default, Clone)]
pub struct InventoryStateInfo {
    /// Main inventory slots (0-39)
    pub slots: Vec<SlotInfo>,
    /// Hotbar slots (subset of main, indices 0-8)
    pub hotbar: Vec<SlotInfo>,
    /// Currently selected hotbar index
    pub selected_hotbar: usize,
    /// Equipment slots
    pub equipment: Vec<SlotInfo>,
}

// === inventory.get_slot ===

#[derive(Deserialize)]
pub struct GetSlotParams {
    pub index: usize,
}

/// Handle inventory.get_slot request
///
/// Returns info about a specific inventory slot.
///
/// # Request
/// ```json
/// { "index": 0 }
/// ```
///
/// # Response
/// ```json
/// { "index": 0, "item_id": "base:iron_ore", "amount": 10 }
/// ```
pub fn handle_inventory_get_slot(
    request: &JsonRpcRequest,
    state: &InventoryStateInfo,
) -> JsonRpcResponse {
    let params: GetSlotParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    if params.index >= state.slots.len() {
        return JsonRpcResponse::error(
            request.id,
            SLOT_OUT_OF_RANGE,
            format!(
                "Slot index {} out of range (max {})",
                params.index,
                state.slots.len().saturating_sub(1)
            ),
        );
    }

    let slot = &state.slots[params.index];
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "index": slot.index,
            "item_id": slot.item_id,
            "amount": slot.amount,
        }),
    )
}

// === inventory.list ===

#[derive(Deserialize, Default)]
pub struct ListParams {
    /// If true, only return non-empty slots
    #[serde(default)]
    pub non_empty_only: bool,
}

/// Handle inventory.list request
///
/// Returns all inventory slots.
///
/// # Response
/// ```json
/// { "slots": [...], "selected_hotbar": 0 }
/// ```
pub fn handle_inventory_list(
    request: &JsonRpcRequest,
    state: &InventoryStateInfo,
) -> JsonRpcResponse {
    let params: ListParams = serde_json::from_value(request.params.clone()).unwrap_or_default();

    let slots: Vec<_> = if params.non_empty_only {
        state
            .slots
            .iter()
            .filter(|s| s.item_id.is_some())
            .cloned()
            .collect()
    } else {
        state.slots.clone()
    };

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "slots": slots,
            "hotbar": state.hotbar,
            "selected_hotbar": state.selected_hotbar,
            "equipment": state.equipment,
        }),
    )
}

// === inventory.move_item ===

#[derive(Deserialize)]
pub struct MoveItemParams {
    /// Source slot index
    pub from: usize,
    /// Destination slot index
    pub to: usize,
    /// Amount to move (None = all)
    pub amount: Option<u32>,
}

/// Handle inventory.move_item request
///
/// Moves items between inventory slots.
/// This is a command that will be queued for execution.
///
/// # Request
/// ```json
/// { "from": 0, "to": 5, "amount": 10 }
/// ```
///
/// # Response
/// ```json
/// { "success": true, "moved": 10 }
/// ```
pub fn handle_inventory_move_item(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: MoveItemParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際の移動はprocess_server_messagesで行う
    // ここではコマンドをキューに入れるだけ
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "from": params.from,
            "to": params.to,
            "amount": params.amount,
            "note": "Command queued for execution"
        }),
    )
}

// === inventory.get_hotbar ===

/// Handle inventory.get_hotbar request
///
/// Returns hotbar state.
///
/// # Response
/// ```json
/// { "slots": [...], "selected": 0 }
/// ```
pub fn handle_inventory_get_hotbar(
    request: &JsonRpcRequest,
    state: &InventoryStateInfo,
) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "slots": state.hotbar,
            "selected": state.selected_hotbar,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::protocol::JsonRpcRequest;

    fn make_test_state() -> InventoryStateInfo {
        InventoryStateInfo {
            slots: vec![
                SlotInfo {
                    index: 0,
                    item_id: Some("base:iron_ore".to_string()),
                    amount: 10,
                },
                SlotInfo {
                    index: 1,
                    item_id: None,
                    amount: 0,
                },
                SlotInfo {
                    index: 2,
                    item_id: Some("base:coal".to_string()),
                    amount: 5,
                },
            ],
            hotbar: vec![SlotInfo {
                index: 0,
                item_id: Some("base:iron_ore".to_string()),
                amount: 10,
            }],
            selected_hotbar: 0,
            equipment: vec![],
        }
    }

    #[test]
    fn test_handle_inventory_get_slot() {
        let state = make_test_state();
        let request =
            JsonRpcRequest::new(1, "inventory.get_slot", serde_json::json!({ "index": 0 }));
        let response = handle_inventory_get_slot(&request, &state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["index"], 0);
        assert_eq!(result["item_id"], "base:iron_ore");
        assert_eq!(result["amount"], 10);
    }

    #[test]
    fn test_handle_inventory_get_slot_empty() {
        let state = make_test_state();
        let request =
            JsonRpcRequest::new(1, "inventory.get_slot", serde_json::json!({ "index": 1 }));
        let response = handle_inventory_get_slot(&request, &state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["index"], 1);
        assert!(result["item_id"].is_null());
        assert_eq!(result["amount"], 0);
    }

    #[test]
    fn test_handle_inventory_get_slot_out_of_range() {
        let state = make_test_state();
        let request =
            JsonRpcRequest::new(1, "inventory.get_slot", serde_json::json!({ "index": 100 }));
        let response = handle_inventory_get_slot(&request, &state);
        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, SLOT_OUT_OF_RANGE);
    }

    #[test]
    fn test_handle_inventory_list() {
        let state = make_test_state();
        let request = JsonRpcRequest::new(1, "inventory.list", serde_json::Value::Null);
        let response = handle_inventory_list(&request, &state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        let slots = result["slots"].as_array().unwrap();
        assert_eq!(slots.len(), 3);
    }

    #[test]
    fn test_handle_inventory_list_non_empty() {
        let state = make_test_state();
        let request = JsonRpcRequest::new(
            1,
            "inventory.list",
            serde_json::json!({ "non_empty_only": true }),
        );
        let response = handle_inventory_list(&request, &state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        let slots = result["slots"].as_array().unwrap();
        assert_eq!(slots.len(), 2); // Only iron_ore and coal
    }

    #[test]
    fn test_handle_inventory_move_item() {
        let request = JsonRpcRequest::new(
            1,
            "inventory.move_item",
            serde_json::json!({ "from": 0, "to": 5, "amount": 10 }),
        );
        let response = handle_inventory_move_item(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["queued"], true);
    }

    #[test]
    fn test_handle_inventory_get_hotbar() {
        let state = make_test_state();
        let request = JsonRpcRequest::new(1, "inventory.get_hotbar", serde_json::Value::Null);
        let response = handle_inventory_get_hotbar(&request, &state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["selected"], 0);
        let slots = result["slots"].as_array().unwrap();
        assert_eq!(slots.len(), 1);
    }
}
