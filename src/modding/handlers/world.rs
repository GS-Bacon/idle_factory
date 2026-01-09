//! World API handlers for E2E testing
//!
//! Provides APIs to:
//! - Query block at position
//! - Place/break blocks
//! - Get chunk info

use super::super::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use serde::{Deserialize, Serialize};

// Error codes
pub const POSITION_OUT_OF_BOUNDS: i32 = -32110;
pub const BLOCK_NOT_FOUND: i32 = -32111;
pub const CANNOT_PLACE: i32 = -32112;
pub const CANNOT_BREAK: i32 = -32113;

/// Block info for API responses
#[derive(Serialize, Clone, Default)]
pub struct BlockInfo {
    pub position: [i32; 3],
    pub item_id: String,
    pub metadata: serde_json::Value,
}

/// World state for E2E testing
#[derive(Default, Clone)]
pub struct WorldStateInfo {
    /// Function to query blocks (not stored, passed during request handling)
    /// For now, use a simple cache of recently queried blocks
    pub cached_blocks: Vec<BlockInfo>,
}

// === world.get_block ===

#[derive(Deserialize)]
pub struct GetBlockParams {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Handle world.get_block request
///
/// Returns info about the block at a position.
///
/// # Request
/// ```json
/// { "x": 0, "y": 10, "z": 0 }
/// ```
///
/// # Response
/// ```json
/// { "position": [0, 10, 0], "item_id": "base:stone", "metadata": {} }
/// ```
pub fn handle_world_get_block(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: GetBlockParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際のブロック取得はprocess_server_messagesで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "queued": true,
            "position": [params.x, params.y, params.z],
            "note": "Query queued for execution"
        }),
    )
}

// === world.place_block ===

#[derive(Deserialize)]
pub struct PlaceBlockParams {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub item_id: String,
    /// Optional facing direction (0-5 for +X, -X, +Y, -Y, +Z, -Z)
    pub facing: Option<u8>,
}

/// Handle world.place_block request
///
/// Places a block at a position.
///
/// # Request
/// ```json
/// { "x": 0, "y": 10, "z": 0, "item_id": "base:stone" }
/// ```
///
/// # Response
/// ```json
/// { "success": true, "position": [0, 10, 0] }
/// ```
pub fn handle_world_place_block(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: PlaceBlockParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際のブロック配置はprocess_server_messagesで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "position": [params.x, params.y, params.z],
            "item_id": params.item_id,
            "facing": params.facing,
            "note": "Command queued for execution"
        }),
    )
}

// === world.break_block ===

#[derive(Deserialize)]
pub struct BreakBlockParams {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Handle world.break_block request
///
/// Breaks a block at a position.
///
/// # Request
/// ```json
/// { "x": 0, "y": 10, "z": 0 }
/// ```
///
/// # Response
/// ```json
/// { "success": true, "position": [0, 10, 0], "dropped": "base:stone" }
/// ```
pub fn handle_world_break_block(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: BreakBlockParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際のブロック破壊はprocess_server_messagesで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "position": [params.x, params.y, params.z],
            "note": "Command queued for execution"
        }),
    )
}

// === world.raycast ===

#[derive(Deserialize)]
pub struct RaycastParams {
    /// Ray origin
    pub origin: [f32; 3],
    /// Ray direction (will be normalized)
    pub direction: [f32; 3],
    /// Max distance
    pub max_distance: Option<f32>,
}

/// Handle world.raycast request
///
/// Performs a raycast to find the first block hit.
///
/// # Request
/// ```json
/// { "origin": [0, 10, 0], "direction": [0, -1, 0], "max_distance": 100 }
/// ```
///
/// # Response
/// ```json
/// { "hit": true, "position": [0, 5, 0], "item_id": "base:stone", "distance": 5.0 }
/// ```
pub fn handle_world_raycast(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: RaycastParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際のレイキャストはprocess_server_messagesで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "queued": true,
            "origin": params.origin,
            "direction": params.direction,
            "max_distance": params.max_distance.unwrap_or(100.0),
            "note": "Query queued for execution"
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::protocol::JsonRpcRequest;

    #[test]
    fn test_handle_world_get_block() {
        let request = JsonRpcRequest::new(
            1,
            "world.get_block",
            serde_json::json!({ "x": 0, "y": 10, "z": 0 }),
        );
        let response = handle_world_get_block(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["queued"], true);
        assert_eq!(result["position"], serde_json::json!([0, 10, 0]));
    }

    #[test]
    fn test_handle_world_place_block() {
        let request = JsonRpcRequest::new(
            1,
            "world.place_block",
            serde_json::json!({ "x": 0, "y": 10, "z": 0, "item_id": "base:stone" }),
        );
        let response = handle_world_place_block(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["item_id"], "base:stone");
    }

    #[test]
    fn test_handle_world_place_block_with_facing() {
        let request = JsonRpcRequest::new(
            1,
            "world.place_block",
            serde_json::json!({ "x": 0, "y": 10, "z": 0, "item_id": "base:conveyor", "facing": 2 }),
        );
        let response = handle_world_place_block(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["facing"], 2);
    }

    #[test]
    fn test_handle_world_break_block() {
        let request = JsonRpcRequest::new(
            1,
            "world.break_block",
            serde_json::json!({ "x": 0, "y": 10, "z": 0 }),
        );
        let response = handle_world_break_block(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
    }

    #[test]
    fn test_handle_world_raycast() {
        let request = JsonRpcRequest::new(
            1,
            "world.raycast",
            serde_json::json!({
                "origin": [0.0, 10.0, 0.0],
                "direction": [0.0, -1.0, 0.0],
                "max_distance": 50.0
            }),
        );
        let response = handle_world_raycast(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["queued"], true);
    }

    #[test]
    fn test_handle_world_get_block_invalid_params() {
        let request = JsonRpcRequest::new(1, "world.get_block", serde_json::json!({}));
        let response = handle_world_get_block(&request);
        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, INVALID_PARAMS);
    }
}
