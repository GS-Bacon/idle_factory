//! Mod management handlers
//!
//! Handlers for mod.list, mod.info, mod.enable, mod.disable

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::modding::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS, MOD_NOT_FOUND};
use crate::modding::ModState;

use super::{HandlerContext, HandlerContextMut};

/// Mod list entry for API response
#[derive(Debug, Serialize, Deserialize)]
pub struct ModListEntry {
    /// Mod ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Version
    pub version: String,
    /// Whether the mod is enabled
    pub enabled: bool,
}

/// Mod info response
#[derive(Debug, Serialize, Deserialize)]
pub struct ModInfoResponse {
    /// Mod ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Version
    pub version: String,
    /// Description
    pub description: String,
    /// Whether the mod is enabled
    pub enabled: bool,
    /// Author
    #[serde(skip_serializing_if = "String::is_empty")]
    pub author: String,
    /// Game version compatibility
    pub game_version: String,
}

/// mod.list handler
///
/// Returns list of all registered mods.
///
/// # Parameters
/// None
///
/// # ja
/// 登録済みMod一覧を取得
///
/// # Returns
/// ```json
/// { "mods": [{ "id": "base", "name": "Base Game", "version": "0.3.78", "enabled": true }] }
/// ```
pub fn handle_mod_list(request: &JsonRpcRequest, ctx: &HandlerContext) -> JsonRpcResponse {
    let mods: Vec<ModListEntry> = ctx
        .mod_manager
        .all()
        .map(|m| ModListEntry {
            id: m.info.id.clone(),
            name: m.info.name.clone(),
            version: m.info.version.clone(),
            enabled: m.state == ModState::Loaded,
        })
        .collect();

    JsonRpcResponse::success(request.id, json!({ "mods": mods }))
}

/// mod.info handler
///
/// Returns detailed information about a specific mod.
///
/// # Parameters
/// - `mod_id` (required): The mod ID to query
///
/// # ja
/// 指定Modの詳細情報を取得
///
/// # Returns
/// ```json
/// { "id": "base", "name": "Base Game", "version": "0.3.78", "description": "...", "enabled": true }
/// ```
///
/// # Errors
/// - INVALID_PARAMS: Missing mod_id parameter
/// - MOD_NOT_FOUND: Mod with given ID not found
pub fn handle_mod_info(request: &JsonRpcRequest, ctx: &HandlerContext) -> JsonRpcResponse {
    // Extract mod_id from params
    let mod_id = match extract_mod_id(&request.params) {
        Ok(id) => id,
        Err(response) => return response.with_id(request.id),
    };

    // Look up the mod
    match ctx.mod_manager.get(&mod_id) {
        Some(loaded_mod) => {
            let info = ModInfoResponse {
                id: loaded_mod.info.id.clone(),
                name: loaded_mod.info.name.clone(),
                version: loaded_mod.info.version.clone(),
                description: loaded_mod.info.description.clone(),
                enabled: loaded_mod.state == ModState::Loaded,
                author: loaded_mod.info.author.clone(),
                game_version: loaded_mod.info.game_version.clone(),
            };
            JsonRpcResponse::success(request.id, serde_json::to_value(info).unwrap())
        }
        None => JsonRpcResponse::error(
            request.id,
            MOD_NOT_FOUND,
            format!("Mod not found: {}", mod_id),
        ),
    }
}

/// mod.enable handler
///
/// Enables a disabled mod.
///
/// # Parameters
/// - `mod_id` (required): The mod ID to enable
///
/// # ja
/// 無効化されたModを有効化
///
/// # Returns
/// ```json
/// { "success": true }
/// ```
///
/// # Errors
/// - INVALID_PARAMS: Missing mod_id parameter
/// - MOD_NOT_FOUND: Mod with given ID not found
pub fn handle_mod_enable(request: &JsonRpcRequest, ctx: &mut HandlerContextMut) -> JsonRpcResponse {
    // Extract mod_id from params
    let mod_id = match extract_mod_id(&request.params) {
        Ok(id) => id,
        Err(response) => return response.with_id(request.id),
    };

    // Check if mod exists
    if ctx.mod_manager.get(&mod_id).is_none() {
        return JsonRpcResponse::error(
            request.id,
            MOD_NOT_FOUND,
            format!("Mod not found: {}", mod_id),
        );
    }

    // Enable the mod
    let success = ctx.mod_manager.enable(&mod_id);
    JsonRpcResponse::success(request.id, json!({ "success": success }))
}

/// mod.disable handler
///
/// Disables an enabled mod.
///
/// # Parameters
/// - `mod_id` (required): The mod ID to disable
///
/// # ja
/// 有効なModを無効化
///
/// # Returns
/// ```json
/// { "success": true }
/// ```
///
/// # Errors
/// - INVALID_PARAMS: Missing mod_id parameter
/// - MOD_NOT_FOUND: Mod with given ID not found
pub fn handle_mod_disable(
    request: &JsonRpcRequest,
    ctx: &mut HandlerContextMut,
) -> JsonRpcResponse {
    // Extract mod_id from params
    let mod_id = match extract_mod_id(&request.params) {
        Ok(id) => id,
        Err(response) => return response.with_id(request.id),
    };

    // Check if mod exists
    if ctx.mod_manager.get(&mod_id).is_none() {
        return JsonRpcResponse::error(
            request.id,
            MOD_NOT_FOUND,
            format!("Mod not found: {}", mod_id),
        );
    }

    // Disable the mod
    let success = ctx.mod_manager.disable(&mod_id);
    JsonRpcResponse::success(request.id, json!({ "success": success }))
}

/// Helper to extract mod_id from params
#[allow(clippy::result_large_err)]
fn extract_mod_id(params: &serde_json::Value) -> Result<String, JsonRpcResponse> {
    // Handle both object format {"mod_id": "..."} and direct value
    let mod_id = params
        .get("mod_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match mod_id {
        Some(id) if !id.is_empty() => Ok(id),
        _ => Err(JsonRpcResponse::error(
            None,
            INVALID_PARAMS,
            "Missing required parameter: mod_id",
        )),
    }
}

/// Extension trait for JsonRpcResponse
trait JsonRpcResponseExt {
    fn with_id(self, id: Option<u64>) -> Self;
}

impl JsonRpcResponseExt for JsonRpcResponse {
    fn with_id(mut self, id: Option<u64>) -> Self {
        self.id = id;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::handlers::game::GameStateInfo;
    use crate::modding::{ModInfo, ModManager};

    fn setup_manager() -> ModManager {
        let mut manager = ModManager::new();

        // Register base mod
        let base_info = ModInfo::new("base", "Base Game", "0.3.78")
            .with_author("Idle Factory Team")
            .with_description("Core game content");
        manager.register(base_info);

        // Set base mod as loaded
        if let Some(m) = manager.get_mut("base") {
            m.state = ModState::Loaded;
        }

        // Register a test mod (disabled)
        let test_info = ModInfo::new("test.mod", "Test Mod", "1.0.0")
            .with_author("Test Author")
            .with_description("A test mod for testing");
        manager.register(test_info);

        // Disable the test mod
        manager.disable("test.mod");

        manager
    }

    fn make_context(manager: &ModManager) -> HandlerContext<'_> {
        use crate::modding::handlers::TestStateInfo;
        HandlerContext {
            mod_manager: manager,
            game_state: GameStateInfo::default(),
            test_state: TestStateInfo::default(),
        }
    }

    #[test]
    fn test_mod_list_empty() {
        let manager = ModManager::new();
        let ctx = make_context(&manager);
        let request = JsonRpcRequest::new(1, "mod.list", serde_json::Value::Null);

        let response = handle_mod_list(&request, &ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let mods = result.get("mods").unwrap().as_array().unwrap();
        assert!(mods.is_empty());
    }

    #[test]
    fn test_mod_list_with_mods() {
        let manager = setup_manager();
        let ctx = make_context(&manager);
        let request = JsonRpcRequest::new(1, "mod.list", serde_json::Value::Null);

        let response = handle_mod_list(&request, &ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let mods = result.get("mods").unwrap().as_array().unwrap();
        assert_eq!(mods.len(), 2);

        // Check base mod
        let base = &mods[0];
        assert_eq!(base.get("id").unwrap().as_str().unwrap(), "base");
        assert_eq!(base.get("name").unwrap().as_str().unwrap(), "Base Game");
        assert_eq!(base.get("version").unwrap().as_str().unwrap(), "0.3.78");
        assert!(base.get("enabled").unwrap().as_bool().unwrap());

        // Check test mod (disabled)
        let test = &mods[1];
        assert_eq!(test.get("id").unwrap().as_str().unwrap(), "test.mod");
        assert!(!test.get("enabled").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_mod_info_success() {
        let manager = setup_manager();
        let ctx = make_context(&manager);
        let request = JsonRpcRequest::new(1, "mod.info", json!({ "mod_id": "base" }));

        let response = handle_mod_info(&request, &ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert_eq!(result.get("id").unwrap().as_str().unwrap(), "base");
        assert_eq!(result.get("name").unwrap().as_str().unwrap(), "Base Game");
        assert_eq!(result.get("version").unwrap().as_str().unwrap(), "0.3.78");
        assert_eq!(
            result.get("description").unwrap().as_str().unwrap(),
            "Core game content"
        );
        assert!(result.get("enabled").unwrap().as_bool().unwrap());
        assert_eq!(
            result.get("author").unwrap().as_str().unwrap(),
            "Idle Factory Team"
        );
    }

    #[test]
    fn test_mod_info_not_found() {
        let manager = setup_manager();
        let ctx = make_context(&manager);
        let request = JsonRpcRequest::new(1, "mod.info", json!({ "mod_id": "nonexistent" }));

        let response = handle_mod_info(&request, &ctx);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, MOD_NOT_FOUND);
        assert!(error.message.contains("nonexistent"));
    }

    #[test]
    fn test_mod_info_missing_param() {
        let manager = setup_manager();
        let ctx = make_context(&manager);
        let request = JsonRpcRequest::new(1, "mod.info", serde_json::Value::Null);

        let response = handle_mod_info(&request, &ctx);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, INVALID_PARAMS);
        assert!(error.message.contains("mod_id"));
    }

    #[test]
    fn test_mod_info_empty_param() {
        let manager = setup_manager();
        let ctx = make_context(&manager);
        let request = JsonRpcRequest::new(1, "mod.info", json!({ "mod_id": "" }));

        let response = handle_mod_info(&request, &ctx);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, INVALID_PARAMS);
    }

    #[test]
    fn test_mod_enable_success() {
        let mut manager = setup_manager();
        let mut ctx = HandlerContextMut {
            mod_manager: &mut manager,
        };
        let request = JsonRpcRequest::new(1, "mod.enable", json!({ "mod_id": "test.mod" }));

        let response = handle_mod_enable(&request, &mut ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert!(result.get("success").unwrap().as_bool().unwrap());

        // Verify mod is no longer disabled
        assert_ne!(
            ctx.mod_manager.get("test.mod").unwrap().state,
            ModState::Disabled
        );
    }

    #[test]
    fn test_mod_enable_not_found() {
        let mut manager = setup_manager();
        let mut ctx = HandlerContextMut {
            mod_manager: &mut manager,
        };
        let request = JsonRpcRequest::new(1, "mod.enable", json!({ "mod_id": "nonexistent" }));

        let response = handle_mod_enable(&request, &mut ctx);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, MOD_NOT_FOUND);
    }

    #[test]
    fn test_mod_enable_missing_param() {
        let mut manager = setup_manager();
        let mut ctx = HandlerContextMut {
            mod_manager: &mut manager,
        };
        let request = JsonRpcRequest::new(1, "mod.enable", serde_json::Value::Null);

        let response = handle_mod_enable(&request, &mut ctx);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, INVALID_PARAMS);
    }

    #[test]
    fn test_mod_disable_success() {
        let mut manager = setup_manager();
        let mut ctx = HandlerContextMut {
            mod_manager: &mut manager,
        };
        let request = JsonRpcRequest::new(1, "mod.disable", json!({ "mod_id": "base" }));

        let response = handle_mod_disable(&request, &mut ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert!(result.get("success").unwrap().as_bool().unwrap());

        // Verify mod is disabled
        assert_eq!(
            ctx.mod_manager.get("base").unwrap().state,
            ModState::Disabled
        );
    }

    #[test]
    fn test_mod_disable_not_found() {
        let mut manager = setup_manager();
        let mut ctx = HandlerContextMut {
            mod_manager: &mut manager,
        };
        let request = JsonRpcRequest::new(1, "mod.disable", json!({ "mod_id": "nonexistent" }));

        let response = handle_mod_disable(&request, &mut ctx);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, MOD_NOT_FOUND);
    }

    #[test]
    fn test_mod_disable_missing_param() {
        let mut manager = setup_manager();
        let mut ctx = HandlerContextMut {
            mod_manager: &mut manager,
        };
        let request = JsonRpcRequest::new(1, "mod.disable", serde_json::Value::Null);

        let response = handle_mod_disable(&request, &mut ctx);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, INVALID_PARAMS);
    }

    #[test]
    fn test_mod_enable_already_enabled() {
        let mut manager = setup_manager();
        // base is already loaded
        let mut ctx = HandlerContextMut {
            mod_manager: &mut manager,
        };
        let request = JsonRpcRequest::new(1, "mod.enable", json!({ "mod_id": "base" }));

        let response = handle_mod_enable(&request, &mut ctx);

        // Should succeed but enable() returns false since it wasn't disabled
        assert!(response.is_success());
        let result = response.result.unwrap();
        // enable() only returns true if state was Disabled
        assert!(!result.get("success").unwrap().as_bool().unwrap());
    }
}
