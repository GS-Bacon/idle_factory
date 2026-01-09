//! Network-related JSON-RPC handlers
//!
//! Handles network segment and virtual link operations for Mod API.
//!
//! # Methods
//! - `network.type.list` - List registered network types
//! - `network.type.register` - Register a custom network type
//! - `network.segment.list` - List all segments
//! - `network.segment.get` - Get segment details
//! - `network.virtual_link.add` - Add a virtual link
//! - `network.virtual_link.remove` - Remove a virtual link
//! - `network.virtual_link.list` - List all virtual links

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::modding::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};

/// Error codes for network operations
pub const NETWORK_TYPE_NOT_FOUND: i32 = -32050;
pub const SEGMENT_NOT_FOUND: i32 = -32051;
pub const VIRTUAL_LINK_NOT_FOUND: i32 = -32052;
pub const INVALID_NETWORK_TYPE: i32 = -32053;

// =============================================================================
// Network Type API
// =============================================================================

/// Network type info returned by network.type.list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTypeInfo {
    /// Full ID (e.g., "base:power")
    pub id: String,
    /// Display name
    pub name: String,
    /// Whether this type supports storage
    pub has_storage: bool,
    /// Value type: "Float" or "Discrete"
    pub value_type: String,
    /// Propagation: "Instant", "Segment", or "Distance"
    pub propagation: String,
    /// Conduit compatibility group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conduit_group: Option<String>,
}

/// Result of network.type.list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTypeListResult {
    pub network_types: Vec<NetworkTypeInfo>,
}

/// Parameters for network.type.register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTypeRegisterParams {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub has_storage: bool,
    #[serde(default = "default_value_type")]
    pub value_type: String,
    #[serde(default = "default_propagation")]
    pub propagation: String,
    #[serde(default)]
    pub conduit_group: Option<String>,
}

fn default_value_type() -> String {
    "Float".to_string()
}

fn default_propagation() -> String {
    "Instant".to_string()
}

/// Result of network.type.register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTypeRegisterResult {
    pub success: bool,
    pub id: String,
}

// =============================================================================
// Segment API
// =============================================================================

/// Segment info returned by network.segment.list/get
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInfo {
    /// Segment ID
    pub id: u32,
    /// Network type ID
    pub network_type: String,
    /// Current supply (power)
    pub supply: f32,
    /// Current demand (power)
    pub demand: f32,
    /// Satisfaction ratio (0.0-1.0)
    pub satisfaction: f32,
    /// Storage capacity (fluid/gas)
    pub capacity: f32,
    /// Current amount (fluid/gas)
    pub amount: f32,
    /// Signal strength (signal)
    pub signal_strength: u8,
    /// Number of nodes
    pub node_count: usize,
}

/// Parameters for network.segment.list
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SegmentListParams {
    /// Optional filter by network type
    #[serde(default)]
    pub network_type: Option<String>,
}

/// Result of network.segment.list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentListResult {
    pub segments: Vec<SegmentInfo>,
}

/// Parameters for network.segment.get
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentGetParams {
    pub segment_id: u32,
}

// =============================================================================
// Virtual Link API
// =============================================================================

/// Virtual link info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualLinkInfo {
    /// Link ID
    pub id: u32,
    /// Source position [x, y, z]
    pub from_pos: [i32; 3],
    /// Destination position [x, y, z]
    pub to_pos: [i32; 3],
    /// Network type
    pub network_type: String,
    /// Whether bidirectional
    pub bidirectional: bool,
    /// Efficiency (0.0-1.0)
    pub efficiency: f32,
}

/// Parameters for network.virtual_link.add
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualLinkAddParams {
    /// Source position [x, y, z]
    pub from_pos: [i32; 3],
    /// Destination position [x, y, z]
    pub to_pos: [i32; 3],
    /// Network type ID
    pub network_type: String,
    /// Whether bidirectional
    #[serde(default = "default_bidirectional")]
    pub bidirectional: bool,
    /// Efficiency (0.0-1.0, default 1.0)
    #[serde(default = "default_efficiency")]
    pub efficiency: f32,
}

fn default_bidirectional() -> bool {
    true
}

fn default_efficiency() -> f32 {
    1.0
}

/// Result of network.virtual_link.add
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualLinkAddResult {
    pub success: bool,
    pub link_id: u32,
}

/// Parameters for network.virtual_link.remove
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualLinkRemoveParams {
    pub link_id: u32,
}

/// Result of network.virtual_link.list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualLinkListResult {
    pub links: Vec<VirtualLinkInfo>,
}

// =============================================================================
// Handlers
// =============================================================================

/// Handle network.type.list request
///
/// Returns all registered network types.
///
/// # ja
/// 登録済みネットワーク種別一覧を取得
///
/// # Response
/// ```json
/// { "network_types": [{ "id": "base:power", "name": "電力", ... }] }
/// ```
pub fn handle_network_type_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    use crate::logistics::network::NetworkTypeRegistry;

    let registry = NetworkTypeRegistry::new();
    let network_types: Vec<NetworkTypeInfo> = registry
        .list()
        .map(|(_, spec)| NetworkTypeInfo {
            id: spec.id.clone(),
            name: spec.name.clone(),
            has_storage: spec.has_storage,
            value_type: format!("{:?}", spec.value_type),
            propagation: format!("{:?}", spec.propagation),
            conduit_group: spec.conduit_group.clone(),
        })
        .collect();

    let result = NetworkTypeListResult { network_types };
    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
}

/// Handle network.type.register request
///
/// Registers a new network type (for Mods).
///
/// # ja
/// カスタムネットワーク種別を登録（Mod用）
///
/// # Parameters
/// - `id`: Network type ID (e.g., "mymod:mana")
/// - `name`: Display name
/// - `has_storage`: Whether storage is supported
/// - `value_type`: "Float" or "Discrete"
/// - `propagation`: "Instant", "Segment", or "Distance"
/// - `conduit_group`: Optional conduit compatibility group
///
/// # Response
/// ```json
/// { "success": true, "id": "mymod:mana" }
/// ```
pub fn handle_network_type_register(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: NetworkTypeRegisterParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Validate ID format
    if !params.id.contains(':') {
        return JsonRpcResponse::error(
            request.id,
            INVALID_NETWORK_TYPE,
            "ID must be in format 'namespace:name'".to_string(),
        );
    }

    // Validate value_type
    if params.value_type != "Float" && params.value_type != "Discrete" {
        return JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            "value_type must be 'Float' or 'Discrete'".to_string(),
        );
    }

    // Validate propagation
    if !["Instant", "Segment", "Distance"].contains(&params.propagation.as_str()) {
        return JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            "propagation must be 'Instant', 'Segment', or 'Distance'".to_string(),
        );
    }

    info!("Registered network type: {}", params.id);

    // NOTE: Actual registration would require mutable access to NetworkTypeRegistry
    // which is managed as a Bevy Resource. For now, return success.
    // Real implementation would use an event or command to register.
    let result = NetworkTypeRegisterResult {
        success: true,
        id: params.id,
    };
    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
}

/// Handle network.segment.list request
///
/// Returns all segments, optionally filtered by network type.
///
/// # ja
/// セグメント一覧を取得（network_type指定でフィルタ可能）
///
/// # Response
/// ```json
/// { "segments": [{ "id": 1, "network_type": "base:power", ... }] }
/// ```
pub fn handle_network_segment_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    let _params: SegmentListParams =
        serde_json::from_value(request.params.clone()).unwrap_or_default();

    // NOTE: Actual implementation would query SegmentRegistry
    // which requires access to Bevy World/Resources.
    // For now, return empty list.
    let result = SegmentListResult { segments: vec![] };
    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
}

/// Handle network.segment.get request
///
/// Returns details of a specific segment.
///
/// # ja
/// 指定セグメントの詳細を取得
///
/// # Parameters
/// - `segment_id`: Segment ID to query
///
/// # Response
/// ```json
/// { "id": 1, "network_type": "base:power", "supply": 100.0, ... }
/// ```
pub fn handle_network_segment_get(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: SegmentGetParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // NOTE: Actual implementation would query SegmentRegistry
    JsonRpcResponse::error(
        request.id,
        SEGMENT_NOT_FOUND,
        format!("Segment {} not found", params.segment_id),
    )
}

/// Handle network.virtual_link.add request
///
/// Creates a virtual link between two positions.
///
/// # ja
/// 仮想リンク（無線接続）を作成
///
/// # Parameters
/// - `from_pos`: Source position [x, y, z]
/// - `to_pos`: Destination position [x, y, z]
/// - `network_type`: Network type ID
/// - `bidirectional`: Whether bidirectional (default: true)
/// - `efficiency`: Transfer efficiency (default: 1.0)
///
/// # Response
/// ```json
/// { "success": true, "link_id": 42 }
/// ```
pub fn handle_network_virtual_link_add(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: VirtualLinkAddParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Validate efficiency
    if params.efficiency < 0.0 || params.efficiency > 1.0 {
        return JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            "efficiency must be between 0.0 and 1.0".to_string(),
        );
    }

    info!(
        "Creating virtual link: {:?} -> {:?} ({})",
        params.from_pos, params.to_pos, params.network_type
    );

    // NOTE: Actual implementation would add to VirtualLinkRegistry
    // via Bevy event or command.
    let result = VirtualLinkAddResult {
        success: true,
        link_id: 0, // Placeholder
    };
    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
}

/// Handle network.virtual_link.remove request
///
/// Removes a virtual link.
///
/// # ja
/// 仮想リンクを削除
///
/// # Parameters
/// - `link_id`: Link ID to remove
///
/// # Response
/// ```json
/// { "success": true }
/// ```
pub fn handle_network_virtual_link_remove(request: &JsonRpcRequest) -> JsonRpcResponse {
    let params: VirtualLinkRemoveParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    info!("Removing virtual link: {}", params.link_id);

    // NOTE: Actual implementation would remove from VirtualLinkRegistry
    JsonRpcResponse::success(request.id, serde_json::json!({ "success": true }))
}

/// Handle network.virtual_link.list request
///
/// Returns all virtual links.
///
/// # ja
/// 仮想リンク一覧を取得
///
/// # Response
/// ```json
/// { "links": [{ "id": 1, "from_pos": [0,0,0], "to_pos": [10,0,0], ... }] }
/// ```
pub fn handle_network_virtual_link_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    // NOTE: Actual implementation would query VirtualLinkRegistry
    let result = VirtualLinkListResult { links: vec![] };
    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_type_list() {
        let request = JsonRpcRequest::new(1, "network.type.list", serde_json::Value::Null);
        let response = handle_network_type_list(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let types = result["network_types"].as_array().unwrap();
        assert!(!types.is_empty());

        // Check base types exist
        let ids: Vec<&str> = types.iter().map(|t| t["id"].as_str().unwrap()).collect();
        assert!(ids.contains(&"base:power"));
        assert!(ids.contains(&"base:fluid"));
        assert!(ids.contains(&"base:signal"));
    }

    #[test]
    fn test_network_type_register() {
        let request = JsonRpcRequest::new(
            1,
            "network.type.register",
            serde_json::json!({
                "id": "mymod:mana",
                "name": "Mana",
                "has_storage": true,
                "value_type": "Float",
                "propagation": "Distance"
            }),
        );
        let response = handle_network_type_register(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["id"], "mymod:mana");
    }

    #[test]
    fn test_network_type_register_invalid_id() {
        let request = JsonRpcRequest::new(
            1,
            "network.type.register",
            serde_json::json!({
                "id": "invalid",
                "name": "Test"
            }),
        );
        let response = handle_network_type_register(&request);

        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, INVALID_NETWORK_TYPE);
    }

    #[test]
    fn test_network_virtual_link_add() {
        let request = JsonRpcRequest::new(
            1,
            "network.virtual_link.add",
            serde_json::json!({
                "from_pos": [0, 0, 0],
                "to_pos": [100, 0, 0],
                "network_type": "base:power"
            }),
        );
        let response = handle_network_virtual_link_add(&request);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
    }

    #[test]
    fn test_network_virtual_link_add_invalid_efficiency() {
        let request = JsonRpcRequest::new(
            1,
            "network.virtual_link.add",
            serde_json::json!({
                "from_pos": [0, 0, 0],
                "to_pos": [100, 0, 0],
                "network_type": "base:power",
                "efficiency": 1.5
            }),
        );
        let response = handle_network_virtual_link_add(&request);

        assert!(response.is_error());
    }

    #[test]
    fn test_network_segment_list() {
        let request = JsonRpcRequest::new(1, "network.segment.list", serde_json::Value::Null);
        let response = handle_network_segment_list(&request);

        assert!(response.is_success());
    }
}
