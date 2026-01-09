//! Machine API handlers for E2E testing
//!
//! Provides APIs to:
//! - Get machine slots at a position
//! - Insert/extract items from machines

use super::super::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use serde::{Deserialize, Serialize};

// Error codes
pub const MACHINE_NOT_FOUND: i32 = -32140;
pub const SLOT_NOT_FOUND: i32 = -32141;
pub const OPERATION_FAILED: i32 = -32142;

/// Machine slot info for API responses
#[derive(Serialize, Clone)]
pub struct MachineSlotInfo {
    pub slot_type: String, // "input", "output", "fuel"
    pub slot_index: usize,
    pub item_id: Option<String>,
    pub amount: u32,
    pub max_amount: u32,
}

/// Machine info for API responses
#[derive(Serialize, Clone)]
pub struct MachineInfo {
    pub position: [i32; 3],
    pub machine_type: String,
    pub slots: Vec<MachineSlotInfo>,
    pub progress: f32, // 0.0 to 1.0
    pub is_working: bool,
}

// === machine.get_slots ===

#[derive(Deserialize)]
pub struct GetSlotsParams {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Handle machine.get_slots request
///
/// Returns slot info for a machine at a position.
///
/// # Request
/// ```json
/// { "x": 0, "y": 10, "z": 0 }
/// ```
///
/// # Response
/// ```json
/// {
///   "position": [0, 10, 0],
///   "machine_type": "furnace",
///   "slots": [...],
///   "progress": 0.5,
///   "is_working": true
/// }
/// ```
pub fn handle_machine_get_slots(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: GetSlotsParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際の機械取得はprocess_server_messagesで行う
    // ここではキューに入れてレスポンスを返す
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "queued": true,
            "position": [params.x, params.y, params.z],
            "note": "Query queued for execution"
        }),
    )
}

// === machine.insert_item ===

#[derive(Deserialize)]
pub struct InsertItemParams {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub slot: usize, // 0=input, 1=fuel (for furnace), etc.
    pub item_id: String,
    pub amount: u32,
}

/// Handle machine.insert_item request
///
/// Inserts an item into a machine slot.
///
/// # Request
/// ```json
/// { "x": 0, "y": 10, "z": 0, "slot": 0, "item_id": "base:iron_ore", "amount": 10 }
/// ```
pub fn handle_machine_insert_item(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: InsertItemParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際の挿入はprocess_server_messagesで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "position": [params.x, params.y, params.z],
            "slot": params.slot,
            "item_id": params.item_id,
            "amount": params.amount,
            "note": "Command queued for execution"
        }),
    )
}

// === machine.extract_item ===

#[derive(Deserialize)]
pub struct ExtractItemParams {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub slot: usize,
    pub amount: Option<u32>, // None = all
}

/// Handle machine.extract_item request
///
/// Extracts items from a machine slot.
///
/// # Request
/// ```json
/// { "x": 0, "y": 10, "z": 0, "slot": 2, "amount": 5 }
/// ```
pub fn handle_machine_extract_item(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: ExtractItemParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際の抽出はprocess_server_messagesで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "position": [params.x, params.y, params.z],
            "slot": params.slot,
            "amount": params.amount,
            "note": "Command queued for execution"
        }),
    )
}

// === machine.get_progress ===

#[derive(Deserialize)]
pub struct GetProgressParams {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Handle machine.get_progress request
///
/// Returns current processing progress for a machine.
///
/// # Response
/// ```json
/// { "progress": 0.5, "is_working": true, "current_recipe": "smelt_iron" }
/// ```
pub fn handle_machine_get_progress(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: GetProgressParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際の進捗取得はprocess_server_messagesで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "queued": true,
            "position": [params.x, params.y, params.z],
            "note": "Query queued for execution"
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::protocol::JsonRpcRequest;

    #[test]
    fn test_handle_machine_get_slots() {
        let request = JsonRpcRequest::new(
            1,
            "machine.get_slots",
            serde_json::json!({ "x": 0, "y": 10, "z": 0 }),
        );
        let response = handle_machine_get_slots(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["queued"], true);
        assert_eq!(result["position"], serde_json::json!([0, 10, 0]));
    }

    #[test]
    fn test_handle_machine_insert_item() {
        let request = JsonRpcRequest::new(
            1,
            "machine.insert_item",
            serde_json::json!({
                "x": 0, "y": 10, "z": 0,
                "slot": 0,
                "item_id": "base:iron_ore",
                "amount": 10
            }),
        );
        let response = handle_machine_insert_item(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["item_id"], "base:iron_ore");
    }

    #[test]
    fn test_handle_machine_extract_item() {
        let request = JsonRpcRequest::new(
            1,
            "machine.extract_item",
            serde_json::json!({
                "x": 0, "y": 10, "z": 0,
                "slot": 2,
                "amount": 5
            }),
        );
        let response = handle_machine_extract_item(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["slot"], 2);
    }

    #[test]
    fn test_handle_machine_get_progress() {
        let request = JsonRpcRequest::new(
            1,
            "machine.get_progress",
            serde_json::json!({ "x": 0, "y": 10, "z": 0 }),
        );
        let response = handle_machine_get_progress(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["queued"], true);
    }

    #[test]
    fn test_handle_machine_get_slots_invalid_params() {
        let request = JsonRpcRequest::new(1, "machine.get_slots", serde_json::json!({}));
        let response = handle_machine_get_slots(&request);
        assert!(response.is_error());
    }
}
