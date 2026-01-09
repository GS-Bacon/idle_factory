//! UI API handlers for Mod support
//!
//! Provides methods for:
//! - ui.set_condition: Set custom UI visibility conditions
//! - ui.register: Register custom UI visibility rules
//! - ui.subscribe_visibility: Subscribe to UI visibility changes (future)

use super::super::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use crate::events::{UIConditionChanged, UIRegistration};
use crate::ui::visibility::{UIId, UILayer, VisibilityCondition, VisibilityRules};
use crate::InputState;
use serde::{Deserialize, Serialize};

/// Parameters for ui.set_condition
#[derive(Debug, Deserialize)]
pub struct SetConditionParams {
    /// Condition name (e.g., "my_mod:unlocked")
    pub name: String,
    /// Condition value
    pub value: bool,
}

/// Result for ui.set_condition
#[derive(Debug, Serialize)]
pub struct SetConditionResult {
    pub success: bool,
    pub name: String,
    pub value: bool,
}

/// Handle ui.set_condition - sets a custom condition for UI visibility
///
/// This returns an event that should be sent by the caller.
#[allow(clippy::result_large_err)]
pub fn handle_ui_set_condition(
    request: &JsonRpcRequest,
) -> Result<(JsonRpcResponse, UIConditionChanged), JsonRpcResponse> {
    let params: SetConditionParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return Err(JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            ));
        }
    };

    let event = UIConditionChanged::Custom {
        name: params.name.clone(),
        value: params.value,
    };

    let result = SetConditionResult {
        success: true,
        name: params.name,
        value: params.value,
    };

    Ok((
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap()),
        event,
    ))
}

// ============================================================================
// ui.register API
// ============================================================================

/// Parameters for ui.register
#[derive(Debug, Deserialize)]
pub struct RegisterUIParams {
    /// UI identifier (e.g., "my_mod:stats_panel")
    pub id: String,
    /// Visibility conditions
    pub conditions: Vec<ConditionDef>,
    /// UI layer (optional, defaults to "Content")
    #[serde(default = "default_layer")]
    pub layer: String,
}

fn default_layer() -> String {
    "Content".to_string()
}

/// Condition definition for JSON API
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ConditionDef {
    InputStateIs { state: String },
    InputStateIsNot { state: String },
    TutorialCompleted,
    TutorialNotCompleted,
    Custom { name: String },
    CustomNot { name: String },
    Always,
}

/// Result for ui.register
#[derive(Debug, Serialize)]
pub struct RegisterUIResult {
    pub success: bool,
    pub id: String,
}

/// Handle ui.register - registers visibility rules for a Mod UI
#[allow(clippy::result_large_err)]
pub fn handle_ui_register(
    request: &JsonRpcRequest,
) -> Result<(JsonRpcResponse, UIRegistration), JsonRpcResponse> {
    let params: RegisterUIParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return Err(JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            ));
        }
    };

    // Parse UIId
    let ui_id = UIId::from_string_id(&params.id).ok_or_else(|| {
        JsonRpcResponse::error(request.id, INVALID_PARAMS, "Invalid UI id format")
    })?;

    // Parse conditions
    let conditions = params
        .conditions
        .into_iter()
        .map(condition_from_def)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| JsonRpcResponse::error(request.id, INVALID_PARAMS, e))?;

    // Parse layer
    let layer = parse_layer(&params.layer).unwrap_or(UILayer::Content);

    let rules = VisibilityRules { conditions, layer };
    let event = UIRegistration { id: ui_id, rules };

    let result = RegisterUIResult {
        success: true,
        id: params.id,
    };

    Ok((
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap()),
        event,
    ))
}

fn condition_from_def(def: ConditionDef) -> Result<VisibilityCondition, String> {
    match def {
        ConditionDef::InputStateIs { state } => {
            let s = parse_input_state(&state)?;
            Ok(VisibilityCondition::InputStateIs(s))
        }
        ConditionDef::InputStateIsNot { state } => {
            let s = parse_input_state(&state)?;
            Ok(VisibilityCondition::InputStateIsNot(s))
        }
        ConditionDef::TutorialCompleted => Ok(VisibilityCondition::TutorialCompleted),
        ConditionDef::TutorialNotCompleted => Ok(VisibilityCondition::TutorialNotCompleted),
        ConditionDef::Custom { name } => Ok(VisibilityCondition::Custom(name)),
        ConditionDef::CustomNot { name } => Ok(VisibilityCondition::CustomNot(name)),
        ConditionDef::Always => Ok(VisibilityCondition::Always),
    }
}

fn parse_input_state(s: &str) -> Result<InputState, String> {
    match s.to_lowercase().as_str() {
        "gameplay" => Ok(InputState::Gameplay),
        "inventory" => Ok(InputState::Inventory),
        "machineui" | "machine_ui" => Ok(InputState::MachineUI),
        "command" => Ok(InputState::Command),
        "paused" => Ok(InputState::Paused),
        _ => Err(format!("Unknown InputState: {}", s)),
    }
}

fn parse_layer(s: &str) -> Option<UILayer> {
    match s.to_lowercase().as_str() {
        "background" => Some(UILayer::Background),
        "gameplay" => Some(UILayer::Gameplay),
        "content" => Some(UILayer::Content),
        "overlay" => Some(UILayer::Overlay),
        "modal" => Some(UILayer::Modal),
        "notification" => Some(UILayer::Notification),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_condition_success() {
        let request = JsonRpcRequest::new(
            1,
            "ui.set_condition",
            serde_json::json!({
                "name": "my_mod:unlocked",
                "value": true
            }),
        );

        let result = handle_ui_set_condition(&request);
        assert!(result.is_ok());

        let (response, event) = result.unwrap();
        assert!(response.is_success());

        match event {
            UIConditionChanged::Custom { name, value } => {
                assert_eq!(name, "my_mod:unlocked");
                assert!(value);
            }
            _ => panic!("Expected Custom event"),
        }
    }

    #[test]
    fn test_set_condition_invalid_params() {
        let request = JsonRpcRequest::new(
            1,
            "ui.set_condition",
            serde_json::json!({
                "name": "test"
                // missing "value"
            }),
        );

        let result = handle_ui_set_condition(&request);
        assert!(result.is_err());

        let response = result.err().unwrap();
        assert!(response.is_error());
    }

    #[test]
    fn test_ui_register_success() {
        let request = JsonRpcRequest::new(
            1,
            "ui.register",
            serde_json::json!({
                "id": "my_mod:stats_panel",
                "conditions": [
                    { "type": "InputStateIs", "state": "Gameplay" },
                    { "type": "TutorialCompleted" }
                ],
                "layer": "Content"
            }),
        );

        let result = handle_ui_register(&request);
        assert!(result.is_ok());

        let (response, event) = result.unwrap();
        assert!(response.is_success());
        assert_eq!(
            event.id,
            UIId::Mod {
                namespace: "my_mod".to_string(),
                name: "stats_panel".to_string()
            }
        );
        assert_eq!(event.rules.conditions.len(), 2);
        assert_eq!(event.rules.layer, UILayer::Content);
    }

    #[test]
    fn test_ui_register_default_layer() {
        let request = JsonRpcRequest::new(
            1,
            "ui.register",
            serde_json::json!({
                "id": "my_mod:panel",
                "conditions": [{ "type": "Always" }]
            }),
        );

        let result = handle_ui_register(&request);
        assert!(result.is_ok());

        let (_, event) = result.unwrap();
        assert_eq!(event.rules.layer, UILayer::Content);
    }

    #[test]
    fn test_ui_register_all_condition_types() {
        let request = JsonRpcRequest::new(
            1,
            "ui.register",
            serde_json::json!({
                "id": "test:panel",
                "conditions": [
                    { "type": "InputStateIs", "state": "Gameplay" },
                    { "type": "InputStateIsNot", "state": "Paused" },
                    { "type": "TutorialCompleted" },
                    { "type": "TutorialNotCompleted" },
                    { "type": "Custom", "name": "my_mod:flag" },
                    { "type": "CustomNot", "name": "my_mod:other" },
                    { "type": "Always" }
                ]
            }),
        );

        let result = handle_ui_register(&request);
        assert!(result.is_ok());

        let (_, event) = result.unwrap();
        assert_eq!(event.rules.conditions.len(), 7);
    }

    #[test]
    fn test_ui_register_invalid_state() {
        let request = JsonRpcRequest::new(
            1,
            "ui.register",
            serde_json::json!({
                "id": "test:panel",
                "conditions": [
                    { "type": "InputStateIs", "state": "InvalidState" }
                ]
            }),
        );

        let result = handle_ui_register(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_ui_register_invalid_id() {
        let request = JsonRpcRequest::new(
            1,
            "ui.register",
            serde_json::json!({
                "id": "",
                "conditions": []
            }),
        );

        let result = handle_ui_register(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_input_state() {
        assert!(parse_input_state("gameplay").is_ok());
        assert!(parse_input_state("Gameplay").is_ok());
        assert!(parse_input_state("GAMEPLAY").is_ok());
        assert!(parse_input_state("inventory").is_ok());
        assert!(parse_input_state("machineui").is_ok());
        assert!(parse_input_state("machine_ui").is_ok());
        assert!(parse_input_state("command").is_ok());
        assert!(parse_input_state("paused").is_ok());
        assert!(parse_input_state("invalid").is_err());
    }

    #[test]
    fn test_parse_layer() {
        assert_eq!(parse_layer("background"), Some(UILayer::Background));
        assert_eq!(parse_layer("gameplay"), Some(UILayer::Gameplay));
        assert_eq!(parse_layer("content"), Some(UILayer::Content));
        assert_eq!(parse_layer("overlay"), Some(UILayer::Overlay));
        assert_eq!(parse_layer("modal"), Some(UILayer::Modal));
        assert_eq!(parse_layer("notification"), Some(UILayer::Notification));
        assert_eq!(parse_layer("Content"), Some(UILayer::Content)); // case insensitive
        assert_eq!(parse_layer("invalid"), None);
    }
}
