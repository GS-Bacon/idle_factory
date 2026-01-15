//! JSON-RPC method handlers for Mod API
//!
//! Each module handles a category of methods:
//! - `game`: Game info (version, state)
//! - `mod_handlers`: Mod management (list, info, enable, disable)
//! - `items`: Item registry
//! - `machines`: Machine registry
//! - `recipes`: Recipe registry
//! - `events`: Event subscription

pub mod events;
pub mod game;
pub mod items;
pub mod machines;
pub mod mod_handlers;
pub mod recipes;
pub mod test;

pub use events::{EventSubscriptions, EventType, Subscription};
pub use game::{GameStateInfo, API_VERSION};
pub use items::{
    handle_item_add, handle_item_list, ItemAddParams, ItemAddResult, ItemInfo, ItemListParams,
    ItemListResult, INVALID_ITEM_ID, ITEM_ALREADY_EXISTS,
};
pub use test::{
    handle_test_clear_events, handle_test_get_events, handle_test_get_input_state,
    handle_test_get_state, handle_test_get_ui_elements, UIElementInfo,
};

use super::protocol::{JsonRpcRequest, JsonRpcResponse, METHOD_NOT_FOUND};
use super::ModManager;

/// インベントリスロット情報
#[derive(Default, Clone)]
pub struct SlotInfo {
    /// アイテムID (例: "base:stone")
    pub item_id: Option<String>,
    /// 個数
    pub count: u32,
}

/// テスト用ゲーム状態
#[derive(Default, Clone)]
pub struct TestStateInfo {
    /// UI状態 ("Gameplay", "Inventory", etc.)
    pub ui_state: String,
    /// プレイヤー位置 [x, y, z]
    pub player_position: [f32; 3],
    /// カーソルがロックされているか
    pub cursor_locked: bool,
    /// ターゲットブロック位置 (破壊対象)
    pub target_block: Option<[i32; 3]>,
    /// 破壊進行度 (0.0-1.0)
    pub breaking_progress: f32,
    /// 入力許可フラグ
    pub input_flags: InputFlags,
    /// UIスタック (底から順)
    pub ui_stack: Vec<String>,
    /// UIスタックの深さ
    pub stack_depth: usize,
    /// ホットバースロット (0-8)
    pub hotbar: Vec<SlotInfo>,
    /// 選択中のホットバースロット
    pub selected_slot: usize,
}

/// 入力許可フラグ
#[derive(Default, Clone)]
pub struct InputFlags {
    pub allows_block_actions: bool,
    pub allows_movement: bool,
    pub allows_camera: bool,
    pub allows_hotbar: bool,
}

/// Handler context for accessing game state
pub struct HandlerContext<'a> {
    /// Mod manager
    pub mod_manager: &'a ModManager,
    /// Game state info (paused, tick, player_count)
    pub game_state: GameStateInfo,
    /// Test state info for E2E testing
    pub test_state: TestStateInfo,
    /// Test events buffer (for E2E testing)
    pub test_events: Vec<crate::events::TestEvent>,
    /// Cleared events count (set when test.clear_events is called)
    pub cleared_events_count: usize,
    /// UI elements info (for test.get_ui_elements)
    pub ui_elements: Vec<UIElementInfo>,
}

/// Mutable handler context for modifying game state
pub struct HandlerContextMut<'a> {
    /// Mod manager
    pub mod_manager: &'a mut ModManager,
}

/// Route a JSON-RPC request to the appropriate handler
pub fn route_request(request: &JsonRpcRequest, ctx: &HandlerContext) -> JsonRpcResponse {
    match request.method.as_str() {
        // Game handlers
        "game.version" => game::handle_game_version(request),
        "game.state" => game::handle_game_state(request, &ctx.game_state),
        // Mod handlers
        "mod.list" => mod_handlers::handle_mod_list(request, ctx),
        "mod.info" => mod_handlers::handle_mod_info(request, ctx),
        // Item handlers (read-only, no context needed)
        "item.list" => items::handle_item_list(request),
        "item.add" => items::handle_item_add(request),
        // Machine handlers (read-only, no context needed)
        "machine.list" => machines::handle_machine_list(request),
        "machine.add" => machines::handle_machine_add(request),
        // Recipe handlers (read-only, no context needed)
        "recipe.list" => recipes::handle_recipe_list(request),
        "recipe.add" => recipes::handle_recipe_add(request),
        // Test handlers (for E2E testing)
        "test.get_state" => test::handle_test_get_state(request, &ctx.test_state),
        "test.get_input_state" => {
            test::handle_test_get_input_state(request, &ctx.test_state.input_flags)
        }
        "test.get_events" => test::handle_test_get_events(request, &ctx.test_events),
        "test.clear_events" => test::handle_test_clear_events(request, ctx.cleared_events_count),
        "test.send_input" => test::handle_test_send_input(request),
        "test.send_command" => test::handle_test_send_command(request),
        "test.set_ui_state" => test::handle_test_set_ui_state(request),
        "test.assert" => test::handle_test_assert(request, &ctx.test_state),
        "test.get_ui_elements" => test::handle_test_get_ui_elements(request, &ctx.ui_elements),
        // Enable/disable require mutation, handled separately
        _ => JsonRpcResponse::error(
            request.id,
            METHOD_NOT_FOUND,
            format!("Method not found: {}", request.method),
        ),
    }
}

/// Route a JSON-RPC request that requires mutable access
pub fn route_request_mut(request: &JsonRpcRequest, ctx: &mut HandlerContextMut) -> JsonRpcResponse {
    match request.method.as_str() {
        // Mod handlers (read-only can also be called here)
        "mod.list" => {
            let read_ctx = HandlerContext {
                mod_manager: ctx.mod_manager,
                game_state: GameStateInfo::default(),
                test_state: TestStateInfo::default(),
                test_events: vec![],
                cleared_events_count: 0,
                ui_elements: vec![],
            };
            mod_handlers::handle_mod_list(request, &read_ctx)
        }
        "mod.info" => {
            let read_ctx = HandlerContext {
                mod_manager: ctx.mod_manager,
                game_state: GameStateInfo::default(),
                test_state: TestStateInfo::default(),
                test_events: vec![],
                cleared_events_count: 0,
                ui_elements: vec![],
            };
            mod_handlers::handle_mod_info(request, &read_ctx)
        }
        // Mod handlers (write)
        "mod.enable" => mod_handlers::handle_mod_enable(request, ctx),
        "mod.disable" => mod_handlers::handle_mod_disable(request, ctx),
        // Item handlers (read-only, no context needed)
        "item.list" => items::handle_item_list(request),
        "item.add" => items::handle_item_add(request),
        // Machine handlers (read-only, no context needed)
        "machine.list" => machines::handle_machine_list(request),
        "machine.add" => machines::handle_machine_add(request),
        // Recipe handlers (read-only, no context needed)
        "recipe.list" => recipes::handle_recipe_list(request),
        "recipe.add" => recipes::handle_recipe_add(request),
        _ => JsonRpcResponse::error(
            request.id,
            METHOD_NOT_FOUND,
            format!("Method not found: {}", request.method),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(manager: &ModManager) -> HandlerContext<'_> {
        HandlerContext {
            mod_manager: manager,
            game_state: GameStateInfo::default(),
            test_state: TestStateInfo::default(),
            test_events: vec![],
            cleared_events_count: 0,
            ui_elements: vec![],
        }
    }

    #[test]
    fn test_route_unknown_method() {
        let manager = ModManager::new();
        let ctx = make_ctx(&manager);
        let request = JsonRpcRequest::new(1, "unknown.method", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, METHOD_NOT_FOUND);
    }

    #[test]
    fn test_route_machine_list() {
        let manager = ModManager::new();
        let ctx = make_ctx(&manager);
        let request = JsonRpcRequest::new(1, "machine.list", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
    }

    #[test]
    fn test_route_machine_add() {
        let manager = ModManager::new();
        let ctx = make_ctx(&manager);
        let request = JsonRpcRequest::new(
            1,
            "machine.add",
            serde_json::json!({
                "id": "test:machine",
                "name": "Test Machine"
            }),
        );
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
    }

    #[test]
    fn test_route_recipe_list() {
        let manager = ModManager::new();
        let ctx = make_ctx(&manager);
        let request = JsonRpcRequest::new(1, "recipe.list", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
    }

    #[test]
    fn test_route_recipe_add() {
        let manager = ModManager::new();
        let ctx = make_ctx(&manager);
        let request = JsonRpcRequest::new(
            1,
            "recipe.add",
            serde_json::json!({
                "id": "mymod:test_recipe",
                "inputs": ["base:iron_ore"],
                "outputs": ["base:iron_ingot"],
                "time": 1.0
            }),
        );
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
    }

    #[test]
    fn test_route_game_version() {
        let manager = ModManager::new();
        let ctx = make_ctx(&manager);
        let request = JsonRpcRequest::new(1, "game.version", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert!(result.get("version").is_some());
        assert!(result.get("api_version").is_some());
        assert_eq!(result["api_version"], API_VERSION);
    }

    #[test]
    fn test_route_game_state() {
        let manager = ModManager::new();
        let mut ctx = make_ctx(&manager);
        ctx.game_state = GameStateInfo {
            paused: true,
            tick: 12345,
            player_count: 1,
        };
        let request = JsonRpcRequest::new(1, "game.state", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert_eq!(result["paused"], true);
        assert_eq!(result["tick"], 12345);
        assert_eq!(result["player_count"], 1);
    }

    #[test]
    fn test_route_item_list() {
        let manager = ModManager::new();
        let ctx = make_ctx(&manager);
        let request = JsonRpcRequest::new(1, "item.list", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let items = result["items"].as_array().unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_route_item_add() {
        let manager = ModManager::new();
        let ctx = make_ctx(&manager);
        let request = JsonRpcRequest::new(
            1,
            "item.add",
            serde_json::json!({
                "id": "test:item",
                "name": "Test Item"
            }),
        );
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["id"], "test:item");
    }

    #[test]
    fn test_route_test_get_input_state() {
        let manager = ModManager::new();
        let mut ctx = make_ctx(&manager);
        ctx.test_state.input_flags = InputFlags {
            allows_block_actions: true,
            allows_movement: false,
            allows_camera: true,
            allows_hotbar: false,
        };
        let request = JsonRpcRequest::new(1, "test.get_input_state", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert_eq!(result["allows_block_actions"], true);
        assert_eq!(result["allows_movement"], false);
    }

    #[test]
    fn test_route_test_get_events() {
        let manager = ModManager::new();
        let mut ctx = make_ctx(&manager);
        ctx.test_events = vec![crate::events::TestEvent {
            event_type: "BlockBroken".to_string(),
            position: Some([1, 2, 3]),
            item_id: Some("stone".to_string()),
        }];
        let request = JsonRpcRequest::new(1, "test.get_events", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["type"], "BlockBroken");
    }

    #[test]
    fn test_route_test_clear_events() {
        let manager = ModManager::new();
        let mut ctx = make_ctx(&manager);
        ctx.cleared_events_count = 5;
        let request = JsonRpcRequest::new(1, "test.clear_events", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert_eq!(result["cleared"], 5);
    }
}
