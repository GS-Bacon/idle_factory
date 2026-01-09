//! Player API handlers for E2E testing
//!
//! Provides APIs to:
//! - Get player state (position, health, etc.)
//! - Teleport player
//! - Query looking at block

use super::super::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use serde::{Deserialize, Serialize};

/// Player state for E2E testing
#[derive(Default, Clone, Serialize)]
pub struct PlayerStateInfo {
    /// Player position [x, y, z]
    pub position: [f32; 3],
    /// Player rotation (yaw, pitch)
    pub rotation: [f32; 2],
    /// Player velocity [x, y, z]
    pub velocity: [f32; 3],
    /// Is player on ground
    pub on_ground: bool,
    /// Is player flying (creative mode)
    pub flying: bool,
    /// Currently selected hotbar slot (0-8)
    pub selected_slot: usize,
    /// Block player is looking at (if any)
    pub looking_at: Option<[i32; 3]>,
    /// Distance to looking_at block
    pub looking_distance: Option<f32>,
}

// === player.get_state ===

/// Handle player.get_state request
///
/// Returns comprehensive player state.
///
/// # Response
/// ```json
/// {
///   "position": [0.0, 10.0, 0.0],
///   "rotation": [0.0, 0.0],
///   "velocity": [0.0, 0.0, 0.0],
///   "on_ground": true,
///   "flying": false,
///   "selected_slot": 0,
///   "looking_at": [0, 9, 1],
///   "looking_distance": 2.5
/// }
/// ```
pub fn handle_player_get_state(
    request: &JsonRpcRequest,
    state: &PlayerStateInfo,
) -> JsonRpcResponse {
    JsonRpcResponse::success(request.id, serde_json::to_value(state).unwrap())
}

// === player.teleport ===

#[derive(Deserialize)]
pub struct TeleportParams {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Optional rotation
    pub yaw: Option<f32>,
    pub pitch: Option<f32>,
}

/// Handle player.teleport request
///
/// Teleports player to a position.
///
/// # Request
/// ```json
/// { "x": 0, "y": 100, "z": 0 }
/// ```
///
/// # Response
/// ```json
/// { "success": true, "position": [0, 100, 0] }
/// ```
pub fn handle_player_teleport(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: TeleportParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際のテレポートはprocess_server_messagesで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "position": [params.x, params.y, params.z],
            "rotation": [params.yaw, params.pitch],
            "note": "Command queued for execution"
        }),
    )
}

// === player.get_looking_at ===

/// Handle player.get_looking_at request
///
/// Returns the block the player is looking at.
///
/// # Response
/// ```json
/// { "hit": true, "position": [0, 9, 1], "item_id": "base:stone", "distance": 2.5 }
/// ```
pub fn handle_player_get_looking_at(
    request: &JsonRpcRequest,
    state: &PlayerStateInfo,
) -> JsonRpcResponse {
    if let Some(pos) = state.looking_at {
        JsonRpcResponse::success(
            request.id,
            serde_json::json!({
                "hit": true,
                "position": pos,
                "distance": state.looking_distance,
            }),
        )
    } else {
        JsonRpcResponse::success(
            request.id,
            serde_json::json!({
                "hit": false,
            }),
        )
    }
}

// === player.set_selected_slot ===

#[derive(Deserialize)]
pub struct SetSelectedSlotParams {
    pub slot: usize,
}

/// Handle player.set_selected_slot request
///
/// Changes the selected hotbar slot.
///
/// # Request
/// ```json
/// { "slot": 3 }
/// ```
///
/// # Response
/// ```json
/// { "success": true, "slot": 3 }
/// ```
pub fn handle_player_set_selected_slot(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: SetSelectedSlotParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    if params.slot > 8 {
        return JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            format!("Slot must be 0-8, got {}", params.slot),
        );
    }

    // Note: 実際の選択変更はprocess_server_messagesで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "slot": params.slot,
            "note": "Command queued for execution"
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::protocol::JsonRpcRequest;

    fn make_test_state() -> PlayerStateInfo {
        PlayerStateInfo {
            position: [10.0, 50.0, 20.0],
            rotation: [45.0, -10.0],
            velocity: [0.0, -0.5, 0.0],
            on_ground: false,
            flying: false,
            selected_slot: 2,
            looking_at: Some([10, 49, 21]),
            looking_distance: Some(1.5),
        }
    }

    #[test]
    fn test_handle_player_get_state() {
        let state = make_test_state();
        let request = JsonRpcRequest::new(1, "player.get_state", serde_json::Value::Null);
        let response = handle_player_get_state(&request, &state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["position"], serde_json::json!([10.0, 50.0, 20.0]));
        assert_eq!(result["on_ground"], false);
        assert_eq!(result["selected_slot"], 2);
    }

    #[test]
    fn test_handle_player_teleport() {
        let request = JsonRpcRequest::new(
            1,
            "player.teleport",
            serde_json::json!({ "x": 100.0, "y": 200.0, "z": 300.0 }),
        );
        let response = handle_player_teleport(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["position"], serde_json::json!([100.0, 200.0, 300.0]));
    }

    #[test]
    fn test_handle_player_teleport_with_rotation() {
        let request = JsonRpcRequest::new(
            1,
            "player.teleport",
            serde_json::json!({ "x": 0.0, "y": 100.0, "z": 0.0, "yaw": 90.0, "pitch": -45.0 }),
        );
        let response = handle_player_teleport(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["rotation"], serde_json::json!([90.0, -45.0]));
    }

    #[test]
    fn test_handle_player_get_looking_at_hit() {
        let state = make_test_state();
        let request = JsonRpcRequest::new(1, "player.get_looking_at", serde_json::Value::Null);
        let response = handle_player_get_looking_at(&request, &state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["hit"], true);
        assert_eq!(result["position"], serde_json::json!([10, 49, 21]));
        assert_eq!(result["distance"], 1.5);
    }

    #[test]
    fn test_handle_player_get_looking_at_miss() {
        let mut state = make_test_state();
        state.looking_at = None;
        state.looking_distance = None;

        let request = JsonRpcRequest::new(1, "player.get_looking_at", serde_json::Value::Null);
        let response = handle_player_get_looking_at(&request, &state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["hit"], false);
    }

    #[test]
    fn test_handle_player_set_selected_slot() {
        let request = JsonRpcRequest::new(
            1,
            "player.set_selected_slot",
            serde_json::json!({ "slot": 5 }),
        );
        let response = handle_player_set_selected_slot(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["slot"], 5);
    }

    #[test]
    fn test_handle_player_set_selected_slot_invalid() {
        let request = JsonRpcRequest::new(
            1,
            "player.set_selected_slot",
            serde_json::json!({ "slot": 10 }),
        );
        let response = handle_player_set_selected_slot(&request);
        assert!(response.is_error());
    }
}
