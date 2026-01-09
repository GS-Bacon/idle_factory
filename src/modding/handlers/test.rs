//! Test API handlers for E2E testing
//!
//! These handlers allow external test runners to:
//! - Query game state
//! - Inject virtual input
//! - Run assertions
//! - Subscribe to game events

use super::super::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use super::events::{EventSubscriptions, EventType};
use super::TestStateInfo;
use serde::{Deserialize, Serialize};

// === test.get_state ===

#[derive(Serialize)]
pub struct GameStateResult {
    pub ui_state: String, // "Gameplay", "Inventory", etc.
    pub player_position: [f32; 3],
    pub cursor_locked: bool,
    pub paused: bool,
}

/// Handle test.get_state request
///
/// Returns current game state for E2E testing.
///
/// # ja
/// E2Eテスト用に現在のゲーム状態を取得
///
/// # Response
/// ```json
/// { "ui_state": "Gameplay", "player_position": [0.0, 10.0, 0.0], "cursor_locked": true }
/// ```
pub fn handle_test_get_state(
    request: &JsonRpcRequest,
    test_state: &TestStateInfo,
) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "ui_state": test_state.ui_state,
            "player_position": test_state.player_position,
            "cursor_locked": test_state.cursor_locked,
        }),
    )
}

// === test.send_input ===

#[derive(Deserialize)]
pub struct SendInputParams {
    pub action: String, // "ToggleInventory", "MoveForward", etc.
}

/// Handle test.send_input request
///
/// Injects virtual input for E2E testing.
///
/// # ja
/// E2Eテスト用に仮想入力を注入
///
/// # Response
/// ```json
/// { "success": true, "action": "ToggleInventory" }
/// ```
pub fn handle_test_send_input(request: &JsonRpcRequest) -> JsonRpcResponse {
    // パラメータをパース
    let params: SendInputParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際の入力注入はprocess_server_messagesで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "action": params.action,
            "note": "Stub response - real implementation pending"
        }),
    )
}

// === test.assert ===

#[derive(Deserialize)]
pub struct AssertParams {
    pub condition: String, // "ui_state == Inventory"
}

/// Handle test.assert request
///
/// Evaluates a condition against game state for E2E testing.
///
/// # ja
/// E2Eテスト用に条件をゲーム状態と照合
///
/// # Response
/// ```json
/// { "success": true, "expected": "Inventory", "actual": "Inventory" }
/// ```
pub fn handle_test_assert(request: &JsonRpcRequest, test_state: &TestStateInfo) -> JsonRpcResponse {
    let params: AssertParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    let (success, expected, actual) = evaluate_condition(&params.condition, test_state);
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": success,
            "expected": expected,
            "actual": actual,
        }),
    )
}

/// 条件文字列を評価
/// "field == value" 形式の条件をパースして、状態と比較する
fn evaluate_condition(condition: &str, state: &TestStateInfo) -> (bool, String, String) {
    // パース: "field == value"
    let parts: Vec<&str> = condition.split(" == ").collect();
    if parts.len() != 2 {
        return (
            false,
            "valid condition (field == value)".into(),
            format!("invalid: {}", condition),
        );
    }

    let (field, expected) = (parts[0].trim(), parts[1].trim());

    // player_position の特別処理: JSON形式の配列と比較
    if field == "player_position" {
        let actual_json = serde_json::to_string(&state.player_position).unwrap_or_default();

        // 両方をJSON配列としてパースして比較
        let actual_parsed: Result<[f32; 3], _> = serde_json::from_str(&actual_json);
        let expected_parsed: Result<[f32; 3], _> = serde_json::from_str(expected);

        match (actual_parsed, expected_parsed) {
            (Ok(a), Ok(e)) => {
                // 許容誤差0.01で比較
                let success = (a[0] - e[0]).abs() < 0.01
                    && (a[1] - e[1]).abs() < 0.01
                    && (a[2] - e[2]).abs() < 0.01;
                return (success, expected.to_string(), actual_json);
            }
            _ => {
                return (false, expected.to_string(), actual_json);
            }
        }
    }

    let actual = match field {
        "ui_state" => state.ui_state.clone(),
        "cursor_locked" => state.cursor_locked.to_string(),
        _ => {
            return (
                false,
                format!("known field: {}", field),
                "unknown field".into(),
            )
        }
    };

    let success = actual == expected;
    (success, expected.to_string(), actual)
}

// === test.subscribe_event ===

#[derive(Deserialize)]
pub struct SubscribeEventParams {
    pub event_type: String, // "item.delivered", "block.placed", etc.
}

/// Subscribe to game events for test monitoring
pub fn handle_test_subscribe_event(
    request: &JsonRpcRequest,
    conn_id: u64,
    subscriptions: &mut EventSubscriptions,
) -> JsonRpcResponse {
    let params: SubscribeEventParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Parse event type
    let Some(event_type) = EventType::parse(&params.event_type) else {
        let valid_types: Vec<&str> = EventType::all().iter().map(|e| e.as_str()).collect();
        return JsonRpcResponse::error_with_data(
            request.id,
            INVALID_PARAMS,
            format!("Unknown event type: {}", params.event_type),
            serde_json::json!({ "valid_types": valid_types }),
        );
    };

    // Create subscription
    let subscription_id = subscriptions.subscribe(conn_id, event_type);

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "subscription_id": subscription_id,
            "event_type": params.event_type
        }),
    )
}

// === test.unsubscribe_event ===

#[derive(Deserialize)]
pub struct UnsubscribeEventParams {
    pub subscription_id: String,
}

/// Unsubscribe from game events
pub fn handle_test_unsubscribe_event(
    request: &JsonRpcRequest,
    subscriptions: &mut EventSubscriptions,
) -> JsonRpcResponse {
    let params: UnsubscribeEventParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    let removed = subscriptions.unsubscribe(&params.subscription_id);

    if removed {
        JsonRpcResponse::success(
            request.id,
            serde_json::json!({
                "success": true
            }),
        )
    } else {
        JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            format!("Subscription not found: {}", params.subscription_id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_test_subscribe_event() {
        let mut subs = EventSubscriptions::default();
        let request = JsonRpcRequest::new(
            1,
            "test.subscribe_event",
            serde_json::json!({ "event_type": "item.delivered" }),
        );
        let response = handle_test_subscribe_event(&request, 42, &mut subs);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert!(result["subscription_id"]
            .as_str()
            .unwrap()
            .starts_with("sub_"));
        assert_eq!(result["event_type"], "item.delivered");
    }

    #[test]
    fn test_handle_test_subscribe_event_invalid_type() {
        let mut subs = EventSubscriptions::default();
        let request = JsonRpcRequest::new(
            1,
            "test.subscribe_event",
            serde_json::json!({ "event_type": "invalid.event" }),
        );
        let response = handle_test_subscribe_event(&request, 42, &mut subs);
        assert!(response.is_error());
        assert!(response
            .error
            .unwrap()
            .message
            .contains("Unknown event type"));
    }

    #[test]
    fn test_handle_test_unsubscribe_event() {
        let mut subs = EventSubscriptions::default();
        let sub_id = subs.subscribe(42, EventType::ItemDelivered);

        let request = JsonRpcRequest::new(
            1,
            "test.unsubscribe_event",
            serde_json::json!({ "subscription_id": sub_id }),
        );
        let response = handle_test_unsubscribe_event(&request, &mut subs);
        assert!(response.is_success());
        assert_eq!(subs.count(), 0);
    }

    #[test]
    fn test_handle_test_get_state() {
        let test_state = TestStateInfo {
            ui_state: "Gameplay".to_string(),
            player_position: [1.0, 2.0, 3.0],
            cursor_locked: true,
        };
        let request = JsonRpcRequest::new(1, "test.get_state", serde_json::Value::Null);
        let response = handle_test_get_state(&request, &test_state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["ui_state"], "Gameplay");
        assert_eq!(result["cursor_locked"], true);
    }

    #[test]
    fn test_handle_test_send_input() {
        let request = JsonRpcRequest::new(
            1,
            "test.send_input",
            serde_json::json!({ "action": "ToggleInventory" }),
        );
        let response = handle_test_send_input(&request);
        assert!(response.is_success());
    }

    #[test]
    fn test_handle_test_send_input_invalid_params() {
        let request = JsonRpcRequest::new(1, "test.send_input", serde_json::json!({}));
        let response = handle_test_send_input(&request);
        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, INVALID_PARAMS);
    }

    #[test]
    fn test_handle_test_assert_success() {
        let test_state = TestStateInfo {
            ui_state: "Inventory".to_string(),
            player_position: [0.0, 0.0, 0.0],
            cursor_locked: false,
        };
        let request = JsonRpcRequest::new(
            1,
            "test.assert",
            serde_json::json!({ "condition": "ui_state == Inventory" }),
        );
        let response = handle_test_assert(&request, &test_state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["expected"], "Inventory");
        assert_eq!(result["actual"], "Inventory");
    }

    #[test]
    fn test_handle_test_assert_failure() {
        let test_state = TestStateInfo {
            ui_state: "Gameplay".to_string(),
            player_position: [0.0, 0.0, 0.0],
            cursor_locked: false,
        };
        let request = JsonRpcRequest::new(
            1,
            "test.assert",
            serde_json::json!({ "condition": "ui_state == Inventory" }),
        );
        let response = handle_test_assert(&request, &test_state);
        assert!(response.is_success()); // Response is success, but assertion failed

        let result = response.result.unwrap();
        assert_eq!(result["success"], false);
        assert_eq!(result["expected"], "Inventory");
        assert_eq!(result["actual"], "Gameplay");
    }

    #[test]
    fn test_handle_test_assert_invalid_params() {
        let test_state = TestStateInfo::default();
        let request = JsonRpcRequest::new(1, "test.assert", serde_json::json!({}));
        let response = handle_test_assert(&request, &test_state);
        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, INVALID_PARAMS);
    }

    #[test]
    fn test_evaluate_condition() {
        let state = TestStateInfo {
            ui_state: "Gameplay".to_string(),
            player_position: [1.0, 2.0, 3.0],
            cursor_locked: true,
        };

        // Test ui_state
        let (success, _, _) = evaluate_condition("ui_state == Gameplay", &state);
        assert!(success);

        // Test cursor_locked
        let (success, _, _) = evaluate_condition("cursor_locked == true", &state);
        assert!(success);

        // Test invalid condition
        let (success, _, _) = evaluate_condition("invalid", &state);
        assert!(!success);

        // Test unknown field
        let (success, _, _) = evaluate_condition("unknown == value", &state);
        assert!(!success);
    }
}
