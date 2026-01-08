//! Machine API handlers for Mod API
//!
//! Provides JSON-RPC methods for machine queries and management.

use crate::game_spec::machines::ALL_MACHINES;
use crate::modding::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use serde::{Deserialize, Serialize};

/// Machine info for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineInfo {
    /// Machine ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Number of input slots
    pub input_slots: u8,
    /// Number of output slots
    pub output_slots: u8,
    /// Requires fuel
    pub requires_fuel: bool,
}

/// Response for machine.list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineListResponse {
    pub machines: Vec<MachineInfo>,
}

/// Parameters for machine.add
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineAddParams {
    /// Machine ID (namespace:id format)
    pub id: String,
    /// Display name
    pub name: String,
    /// Number of input slots (default: 1)
    #[serde(default = "default_slots")]
    pub input_slots: u8,
    /// Number of output slots (default: 1)
    #[serde(default = "default_slots")]
    pub output_slots: u8,
    /// Requires fuel (default: false)
    #[serde(default)]
    pub requires_fuel: bool,
}

fn default_slots() -> u8 {
    1
}

/// Response for machine.add
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineAddResponse {
    pub success: bool,
    pub id: String,
}

/// Handle machine.list request
pub fn handle_machine_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    let machines: Vec<MachineInfo> = ALL_MACHINES
        .iter()
        .map(|spec| {
            // Count input/output slots from ui_slots
            let input_slots = spec
                .ui_slots
                .iter()
                .filter(|s| matches!(s.slot_type, crate::game_spec::UiSlotType::Input))
                .count() as u8;
            let output_slots = spec
                .ui_slots
                .iter()
                .filter(|s| matches!(s.slot_type, crate::game_spec::UiSlotType::Output))
                .count() as u8;

            MachineInfo {
                id: spec.id.to_string(),
                name: spec.name.to_string(),
                input_slots,
                output_slots,
                requires_fuel: spec.requires_fuel,
            }
        })
        .collect();

    let response = MachineListResponse { machines };
    JsonRpcResponse::success(request.id, serde_json::to_value(response).unwrap())
}

/// Handle machine.add request
pub fn handle_machine_add(request: &JsonRpcRequest) -> JsonRpcResponse {
    // Parse parameters
    let params: MachineAddParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Validate ID format (should be namespace:id)
    if !params.id.contains(':') {
        return JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            "ID must be in namespace:id format (e.g., 'mymod:super_furnace')",
        );
    }

    // TODO: In the future, dynamically register the machine to the registry.
    // For now, just log and return success as a stub.
    tracing::info!(
        "machine.add stub: id={}, name={}, input_slots={}, output_slots={}, requires_fuel={}",
        params.id,
        params.name,
        params.input_slots,
        params.output_slots,
        params.requires_fuel
    );

    let response = MachineAddResponse {
        success: true,
        id: params.id,
    };
    JsonRpcResponse::success(request.id, serde_json::to_value(response).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_handle_machine_list() {
        let request = JsonRpcRequest::new(1, "machine.list", json!({}));
        let response = handle_machine_list(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let list: MachineListResponse = serde_json::from_value(result).unwrap();

        // Should have at least 4 machines (miner, furnace, crusher, assembler)
        assert!(list.machines.len() >= 4);

        // Check miner exists
        let miner = list.machines.iter().find(|m| m.id == "miner");
        assert!(miner.is_some());
        let miner = miner.unwrap();
        assert_eq!(miner.name, "採掘機");
        assert!(!miner.requires_fuel);

        // Check furnace exists
        let furnace = list.machines.iter().find(|m| m.id == "furnace");
        assert!(furnace.is_some());
        let furnace = furnace.unwrap();
        assert_eq!(furnace.name, "精錬炉");
        assert!(furnace.requires_fuel);
    }

    #[test]
    fn test_handle_machine_add_success() {
        let request = JsonRpcRequest::new(
            1,
            "machine.add",
            json!({
                "id": "mymod:super_furnace",
                "name": "Super Furnace",
                "input_slots": 2,
                "output_slots": 1
            }),
        );
        let response = handle_machine_add(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let add_response: MachineAddResponse = serde_json::from_value(result).unwrap();

        assert!(add_response.success);
        assert_eq!(add_response.id, "mymod:super_furnace");
    }

    #[test]
    fn test_handle_machine_add_default_slots() {
        let request = JsonRpcRequest::new(
            1,
            "machine.add",
            json!({
                "id": "mymod:basic_machine",
                "name": "Basic Machine"
            }),
        );
        let response = handle_machine_add(&request);

        assert!(response.is_success());
    }

    #[test]
    fn test_handle_machine_add_invalid_id_format() {
        let request = JsonRpcRequest::new(
            1,
            "machine.add",
            json!({
                "id": "invalid_id_without_namespace",
                "name": "Invalid Machine"
            }),
        );
        let response = handle_machine_add(&request);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, INVALID_PARAMS);
        assert!(error.message.contains("namespace:id"));
    }

    #[test]
    fn test_handle_machine_add_missing_params() {
        let request = JsonRpcRequest::new(1, "machine.add", json!({}));
        let response = handle_machine_add(&request);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, INVALID_PARAMS);
    }

    #[test]
    fn test_machine_info_serialization() {
        let info = MachineInfo {
            id: "test:machine".to_string(),
            name: "Test Machine".to_string(),
            input_slots: 2,
            output_slots: 1,
            requires_fuel: true,
        };

        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["id"], "test:machine");
        assert_eq!(json["name"], "Test Machine");
        assert_eq!(json["input_slots"], 2);
        assert_eq!(json["output_slots"], 1);
        assert_eq!(json["requires_fuel"], true);
    }
}
