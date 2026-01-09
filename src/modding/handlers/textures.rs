//! Texture-related JSON-RPC handlers
//!
//! Provides methods for MODs to interact with the texture system:
//! - `texture.list`: List registered textures
//! - `texture.register_resolver`: Register a custom texture resolver
//! - `texture.get_atlas_info`: Get texture atlas information

use serde::{Deserialize, Serialize};

use super::super::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};

/// Error codes for texture operations
pub const TEXTURE_NOT_FOUND: i32 = -32100;
pub const RESOLVER_ALREADY_EXISTS: i32 = -32101;

/// Texture info for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureInfo {
    /// Texture name (e.g., "stone", "grass_top")
    pub name: String,
    /// UV coordinates in atlas [min_x, min_y, max_x, max_y]
    pub uv: [f32; 4],
    /// Whether this texture is from a MOD
    pub is_mod: bool,
}

/// Parameters for texture.list
#[derive(Debug, Deserialize)]
pub struct TextureListParams {
    /// Filter by category (optional)
    #[serde(default)]
    pub category: Option<String>,
}

/// Result for texture.list
#[derive(Debug, Serialize, Deserialize)]
pub struct TextureListResult {
    /// List of textures
    pub textures: Vec<TextureInfo>,
    /// Atlas size [width, height]
    pub atlas_size: [u32; 2],
}

/// Parameters for texture.register_resolver
#[derive(Debug, Deserialize)]
pub struct RegisterResolverParams {
    /// Resolver name (for identification)
    pub name: String,
    /// Priority (higher = checked first)
    pub priority: i32,
    /// Block IDs this resolver handles
    pub block_ids: Vec<String>,
    /// Resolver type
    pub resolver_type: String,
}

/// Result for texture.register_resolver
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResolverResult {
    /// Whether registration succeeded
    pub success: bool,
    /// Resolver ID for later reference
    pub resolver_id: Option<u32>,
}

/// Result for texture.get_atlas_info
#[derive(Debug, Serialize, Deserialize)]
pub struct AtlasInfo {
    /// Atlas dimensions [width, height]
    pub size: [u32; 2],
    /// Tile size (e.g., 16 for 16x16 textures)
    pub tile_size: u32,
    /// Number of textures in atlas
    pub texture_count: usize,
    /// Generation (increments when atlas is rebuilt)
    pub generation: u32,
}

/// Handle texture.list request
///
/// Returns a list of all registered textures with their UV coordinates.
///
/// # ja
/// 登録済みテクスチャ一覧を取得（UV座標付き）
///
/// # Response
/// ```json
/// { "textures": [{ "name": "stone", "uv": [0.0, 0.0, 0.0625, 0.0625], "is_mod": false }], "atlas_size": [256, 256] }
/// ```
pub fn handle_texture_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    // Parse params (optional) - params is Value, not Option<Value>
    let _params: Option<TextureListParams> = if request.params.is_null() {
        None
    } else {
        serde_json::from_value(request.params.clone()).ok()
    };

    // Return placeholder data - real implementation would query TextureRegistry
    let result = TextureListResult {
        textures: vec![
            TextureInfo {
                name: "stone".to_string(),
                uv: [0.0, 0.0, 0.0625, 0.0625],
                is_mod: false,
            },
            TextureInfo {
                name: "grass_top".to_string(),
                uv: [0.0625, 0.0, 0.125, 0.0625],
                is_mod: false,
            },
            TextureInfo {
                name: "grass_side".to_string(),
                uv: [0.125, 0.0, 0.1875, 0.0625],
                is_mod: false,
            },
            TextureInfo {
                name: "dirt".to_string(),
                uv: [0.1875, 0.0, 0.25, 0.0625],
                is_mod: false,
            },
        ],
        atlas_size: [256, 256],
    };

    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
}

/// Handle texture.get_atlas_info request
///
/// Returns information about the texture atlas.
///
/// # ja
/// テクスチャアトラス情報を取得
///
/// # Response
/// ```json
/// { "size": [256, 256], "tile_size": 16, "texture_count": 12, "generation": 1 }
/// ```
pub fn handle_get_atlas_info(request: &JsonRpcRequest) -> JsonRpcResponse {
    // Return placeholder data - real implementation would query TextureRegistry
    let result = AtlasInfo {
        size: [256, 256],
        tile_size: 16,
        texture_count: 12,
        generation: 1,
    };

    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
}

/// Handle texture.register_resolver request
///
/// Registers a custom texture resolver for connected textures, etc.
///
/// # ja
/// カスタムテクスチャリゾルバを登録（接続テクスチャ等用）
///
/// # Response
/// ```json
/// { "success": true, "resolver_id": 1 }
/// ```
pub fn handle_register_resolver(request: &JsonRpcRequest) -> JsonRpcResponse {
    // Parse params - params is Value, not Option<Value>
    let params: RegisterResolverParams = if request.params.is_null() {
        return JsonRpcResponse::error(request.id, INVALID_PARAMS, "Missing params");
    } else {
        match serde_json::from_value(request.params.clone()) {
            Ok(p) => p,
            Err(_) => {
                return JsonRpcResponse::error(request.id, INVALID_PARAMS, "Invalid params");
            }
        }
    };

    // For now, just acknowledge the registration
    // Real implementation would store the resolver config and create a WASM callback
    tracing::info!(
        "Texture resolver registered: {} (priority: {}, type: {})",
        params.name,
        params.priority,
        params.resolver_type
    );

    let result = RegisterResolverResult {
        success: true,
        resolver_id: Some(1), // Placeholder ID
    };

    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_texture_list() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "texture.list".to_string(),
            params: serde_json::Value::Null,
            id: Some(1),
        };

        let response = handle_texture_list(&request);
        assert!(response.error.is_none());

        let result: TextureListResult = serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(!result.textures.is_empty());
        assert_eq!(result.atlas_size, [256, 256]);
    }

    #[test]
    fn test_get_atlas_info() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "texture.get_atlas_info".to_string(),
            params: serde_json::Value::Null,
            id: Some(1),
        };

        let response = handle_get_atlas_info(&request);
        assert!(response.error.is_none());

        let result: AtlasInfo = serde_json::from_value(response.result.unwrap()).unwrap();
        assert_eq!(result.tile_size, 16);
    }

    #[test]
    fn test_register_resolver() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "texture.register_resolver".to_string(),
            params: json!({
                "name": "connected_glass",
                "priority": 10,
                "block_ids": ["base:glass"],
                "resolver_type": "connected"
            }),
            id: Some(1),
        };

        let response = handle_register_resolver(&request);
        assert!(response.error.is_none());

        let result: RegisterResolverResult =
            serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(result.success);
        assert!(result.resolver_id.is_some());
    }

    #[test]
    fn test_register_resolver_missing_params() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "texture.register_resolver".to_string(),
            params: serde_json::Value::Null,
            id: Some(1),
        };

        let response = handle_register_resolver(&request);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, INVALID_PARAMS);
    }
}
