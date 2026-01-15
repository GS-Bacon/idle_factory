//! Test API handlers for E2E testing
//!
//! These handlers allow external test runners to:
//! - Query game state
//! - Inject virtual input
//! - Run assertions
//! - Check input permissions per UI state
//! - Get/clear event history

use super::super::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use super::{InputFlags, TestStateInfo};
use crate::events::TestEvent;
use serde::{Deserialize, Serialize};

// === test.get_ui_elements ===

/// Information about a single UI element
#[derive(Debug, Clone, Serialize)]
pub struct UIElementInfo {
    /// Element ID string (e.g., "base:hotbar")
    pub id: String,
    /// Whether the element is currently visible
    pub visible: bool,
    /// Whether the element can be interacted with
    pub interactable: bool,
}

/// Handle test.get_ui_elements request
///
/// Returns a list of all UI elements with their current visibility and interactability.
pub fn handle_test_get_ui_elements(
    request: &JsonRpcRequest,
    elements: &[UIElementInfo],
) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "elements": elements,
        }),
    )
}

// === test.get_state ===

#[derive(Serialize)]
pub struct GameStateResult {
    pub ui_state: String, // "Gameplay", "Inventory", etc.
    pub player_position: [f32; 3],
    pub cursor_locked: bool,
    pub paused: bool,
}

pub fn handle_test_get_state(
    request: &JsonRpcRequest,
    test_state: &TestStateInfo,
) -> JsonRpcResponse {
    // Build hotbar JSON
    let hotbar_json: Vec<_> = test_state
        .hotbar
        .iter()
        .map(|slot| {
            serde_json::json!({
                "item_id": slot.item_id,
                "count": slot.count,
            })
        })
        .collect();

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "ui_state": test_state.ui_state,
            "player_position": test_state.player_position,
            "cursor_locked": test_state.cursor_locked,
            "target_block": test_state.target_block,
            "breaking_progress": test_state.breaking_progress,
            "input_flags": {
                "allows_block_actions": test_state.input_flags.allows_block_actions,
                "allows_movement": test_state.input_flags.allows_movement,
                "allows_camera": test_state.input_flags.allows_camera,
                "allows_hotbar": test_state.input_flags.allows_hotbar,
            },
            "ui_stack": test_state.ui_stack,
            "stack_depth": test_state.stack_depth,
            "hotbar": hotbar_json,
            "selected_slot": test_state.selected_slot,
        }),
    )
}

// === test.get_input_state ===

/// Returns input permission flags for the current UI state
pub fn handle_test_get_input_state(
    request: &JsonRpcRequest,
    input_flags: &InputFlags,
) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "allows_block_actions": input_flags.allows_block_actions,
            "allows_movement": input_flags.allows_movement,
            "allows_camera": input_flags.allows_camera,
            "allows_hotbar": input_flags.allows_hotbar,
        }),
    )
}

// === test.get_events / test.clear_events ===

/// Returns recorded test events
pub fn handle_test_get_events(request: &JsonRpcRequest, events: &[TestEvent]) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "events": events,
        }),
    )
}

/// Returns the count of cleared events (actual clearing done in server.rs)
pub fn handle_test_clear_events(request: &JsonRpcRequest, cleared_count: usize) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "cleared": cleared_count,
        }),
    )
}

// === test.send_input ===

#[derive(Deserialize)]
pub struct SendInputParams {
    pub action: String, // "ToggleInventory", "MoveForward", etc.
}

// === test.set_ui_state ===

#[derive(Deserialize)]
pub struct SetUiStateParams {
    pub state: String, // "Gameplay", "Inventory", "MachineUI", "PauseMenu"
}

/// Valid UI states for test.set_ui_state
pub const VALID_UI_STATES: &[&str] = &[
    "Gameplay",
    "Inventory",
    "MachineUI",
    "PauseMenu",
    "GlobalInventory",
    "Command",
    "Settings",
];

pub fn handle_test_set_ui_state(request: &JsonRpcRequest) -> JsonRpcResponse {
    // Parse params
    let params: SetUiStateParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Validate state string
    if !VALID_UI_STATES.contains(&params.state.as_str()) {
        return JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            format!(
                "Invalid state: {}. Valid states: {:?}",
                params.state, VALID_UI_STATES
            ),
        );
    }

    // Return success - actual state change happens in process_server_messages
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "state": params.state,
        }),
    )
}

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

// === test.send_command ===

#[derive(Deserialize)]
pub struct SendCommandParams {
    pub command: String, // "/setblock 0 10 0 base:iron_ore", "/tp 0 10 0"
}

/// Handle test.send_command request
/// Queues a command for execution. Actual execution happens in process_test_command_queue.
pub fn handle_test_send_command(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: SendCommandParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Note: 実際のコマンド実行はprocess_test_command_queueで行う
    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "command": params.command,
        }),
    )
}

// === test.assert ===

#[derive(Deserialize)]
pub struct AssertParams {
    pub condition: String, // "ui_state == Inventory"
}

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
/// "field op value" 形式の条件をパースして、状態と比較する
/// 対応演算子: ==, !=, <, >, <=, >=, contains, not_contains
fn evaluate_condition(condition: &str, state: &TestStateInfo) -> (bool, String, String) {
    // Try different operators (order matters: longer operators first)
    let (field, op, expected) = if let Some((f, v)) = condition.split_once(" == ") {
        (f.trim(), "==", v.trim())
    } else if let Some((f, v)) = condition.split_once(" != ") {
        (f.trim(), "!=", v.trim())
    } else if let Some((f, v)) = condition.split_once(" <= ") {
        (f.trim(), "<=", v.trim())
    } else if let Some((f, v)) = condition.split_once(" >= ") {
        (f.trim(), ">=", v.trim())
    } else if let Some((f, v)) = condition.split_once(" < ") {
        (f.trim(), "<", v.trim())
    } else if let Some((f, v)) = condition.split_once(" > ") {
        (f.trim(), ">", v.trim())
    } else if let Some((f, v)) = condition.split_once(" contains ") {
        (f.trim(), "contains", v.trim())
    } else if let Some((f, v)) = condition.split_once(" not_contains ") {
        (f.trim(), "not_contains", v.trim())
    } else {
        return (
            false,
            "valid condition (field op value)".into(),
            format!("invalid: {}", condition),
        );
    };

    // player_position の特別処理: JSON形式の配列と比較
    if field == "player_position" && op == "==" {
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

    // ui_stack の特別処理: 配列として contains/not_contains をチェック
    if field == "ui_stack" {
        let actual_str = format!("{:?}", state.ui_stack);
        let contains = state.ui_stack.iter().any(|s| s == expected);
        let success = match op {
            "contains" => contains,
            "not_contains" => !contains,
            "==" => actual_str == expected,
            "!=" => actual_str != expected,
            _ => false,
        };
        return (success, expected.to_string(), actual_str);
    }

    // stack_depth の特別処理: 数値比較
    if field == "stack_depth" {
        let actual = state.stack_depth.to_string();
        let success = match op {
            "==" => actual == expected,
            "!=" => actual != expected,
            _ => false,
        };
        return (success, expected.to_string(), actual);
    }

    // selected_slot の特別処理: 数値比較
    if field == "selected_slot" {
        let actual = state.selected_slot.to_string();
        let success = match op {
            "==" => actual == expected,
            "!=" => actual != expected,
            _ => false,
        };
        return (success, expected.to_string(), actual);
    }

    // player_y の特別処理: Y座標の数値比較
    if field == "player_y" {
        let actual_y = state.player_position[1];
        let actual_str = format!("{:.2}", actual_y);
        if let Ok(expected_f) = expected.parse::<f32>() {
            let success = match op {
                "==" => (actual_y - expected_f).abs() < 0.01,
                "!=" => (actual_y - expected_f).abs() >= 0.01,
                "<" => actual_y < expected_f,
                ">" => actual_y > expected_f,
                "<=" => actual_y <= expected_f,
                ">=" => actual_y >= expected_f,
                _ => false,
            };
            return (success, expected.to_string(), actual_str);
        } else {
            return (false, expected.to_string(), actual_str);
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

    let success = match op {
        "==" => actual == expected,
        "!=" => actual != expected,
        _ => false,
    };
    (success, expected.to_string(), actual)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_state() -> TestStateInfo {
        TestStateInfo {
            ui_state: "Gameplay".to_string(),
            player_position: [1.0, 2.0, 3.0],
            cursor_locked: true,
            target_block: None,
            breaking_progress: 0.0,
            input_flags: InputFlags {
                allows_block_actions: true,
                allows_movement: true,
                allows_camera: true,
                allows_hotbar: true,
            },
            ui_stack: vec![],
            stack_depth: 0,
            hotbar: vec![],
            selected_slot: 0,
        }
    }

    #[test]
    fn test_handle_test_get_state() {
        let test_state = make_test_state();
        let request = JsonRpcRequest::new(1, "test.get_state", serde_json::Value::Null);
        let response = handle_test_get_state(&request, &test_state);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["ui_state"], "Gameplay");
        assert_eq!(result["cursor_locked"], true);
        assert!(result["input_flags"]["allows_block_actions"]
            .as_bool()
            .unwrap());
    }

    #[test]
    fn test_handle_test_get_input_state() {
        let input_flags = InputFlags {
            allows_block_actions: true,
            allows_movement: false,
            allows_camera: true,
            allows_hotbar: false,
        };
        let request = JsonRpcRequest::new(1, "test.get_input_state", serde_json::Value::Null);
        let response = handle_test_get_input_state(&request, &input_flags);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["allows_block_actions"], true);
        assert_eq!(result["allows_movement"], false);
        assert_eq!(result["allows_camera"], true);
        assert_eq!(result["allows_hotbar"], false);
    }

    #[test]
    fn test_handle_test_get_events() {
        let events = vec![
            TestEvent {
                event_type: "BlockBroken".to_string(),
                position: Some([1, 2, 3]),
                item_id: Some("stone".to_string()),
            },
            TestEvent {
                event_type: "BlockPlaced".to_string(),
                position: Some([4, 5, 6]),
                item_id: Some("conveyor".to_string()),
            },
        ];
        let request = JsonRpcRequest::new(1, "test.get_events", serde_json::Value::Null);
        let response = handle_test_get_events(&request, &events);
        assert!(response.is_success());

        let result = response.result.unwrap();
        let events_arr = result["events"].as_array().unwrap();
        assert_eq!(events_arr.len(), 2);
        assert_eq!(events_arr[0]["type"], "BlockBroken");
    }

    #[test]
    fn test_handle_test_clear_events() {
        let request = JsonRpcRequest::new(1, "test.clear_events", serde_json::Value::Null);
        let response = handle_test_clear_events(&request, 5);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["cleared"], 5);
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
        let mut test_state = make_test_state();
        test_state.ui_state = "Inventory".to_string();
        test_state.cursor_locked = false;
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
        let test_state = make_test_state();
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
        let test_state = make_test_state();
        let request = JsonRpcRequest::new(1, "test.assert", serde_json::json!({}));
        let response = handle_test_assert(&request, &test_state);
        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, INVALID_PARAMS);
    }

    #[test]
    fn test_evaluate_condition() {
        let state = make_test_state();

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

    #[test]
    fn test_handle_test_set_ui_state_valid() {
        let request = JsonRpcRequest::new(
            1,
            "test.set_ui_state",
            serde_json::json!({ "state": "Inventory" }),
        );
        let response = handle_test_set_ui_state(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["state"], "Inventory");
    }

    #[test]
    fn test_handle_test_set_ui_state_all_valid_states() {
        for state in VALID_UI_STATES {
            let request = JsonRpcRequest::new(
                1,
                "test.set_ui_state",
                serde_json::json!({ "state": state }),
            );
            let response = handle_test_set_ui_state(&request);
            assert!(response.is_success(), "State {} should be valid", state);
        }
    }

    #[test]
    fn test_handle_test_set_ui_state_invalid_state() {
        let request = JsonRpcRequest::new(
            1,
            "test.set_ui_state",
            serde_json::json!({ "state": "InvalidState" }),
        );
        let response = handle_test_set_ui_state(&request);
        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, INVALID_PARAMS);
    }

    #[test]
    fn test_handle_test_set_ui_state_missing_params() {
        let request = JsonRpcRequest::new(1, "test.set_ui_state", serde_json::json!({}));
        let response = handle_test_set_ui_state(&request);
        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, INVALID_PARAMS);
    }
}
