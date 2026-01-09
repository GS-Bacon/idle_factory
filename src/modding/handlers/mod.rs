//! JSON-RPC method handlers for Mod API
//!
//! Each module handles a category of methods:
//! - `game`: Game info (version, state)
//! - `mod_handlers`: Mod management (list, info, enable, disable)
//! - `items`: Item registry
//! - `machines`: Machine registry
//! - `recipes`: Recipe registry
//! - `events`: Event subscription
//! - `ui`: UI visibility control
//! - `textures`: Texture system (atlas, resolvers)
//! - `network`: Network segments and virtual links
//! - `inventory`: Player inventory operations (E2E testing)
//! - `world`: World/block operations (E2E testing)
//! - `player`: Player state and actions (E2E testing)

pub mod craft;
pub mod events;
pub mod game;
pub mod inventory;
pub mod items;
pub mod machine;
pub mod machines;
pub mod mod_handlers;
pub mod network;
pub mod player;
pub mod quest;
pub mod recipes;
pub mod test;
pub mod textures;
pub mod ui;
pub mod world;

pub use events::{EventSubscriptions, EventType, Subscription};
pub use game::{GameStateInfo, API_VERSION};
pub use items::{
    handle_item_add, handle_item_list, ItemAddParams, ItemAddResult, ItemInfo, ItemListParams,
    ItemListResult, INVALID_ITEM_ID, ITEM_ALREADY_EXISTS,
};

use super::protocol::{JsonRpcRequest, JsonRpcResponse, METHOD_NOT_FOUND};
use super::ModManager;

/// テスト用ゲーム状態
#[derive(Default, Clone)]
pub struct TestStateInfo {
    /// UI状態 ("Gameplay", "Inventory", etc.)
    pub ui_state: String,
    /// プレイヤー位置 [x, y, z]
    pub player_position: [f32; 3],
    /// カーソルがロックされているか
    pub cursor_locked: bool,
    /// 表示中のUI要素リスト
    pub visible_ui_elements: Vec<String>,
    /// 設定画面が開いているか
    pub settings_open: bool,
    /// カーソルがゲーム画面内にあるか
    pub cursor_in_window: bool,
    /// カーソルが表示されているか（非ロック時）
    pub cursor_visible: bool,
}

/// Handler context for accessing game state
pub struct HandlerContext<'a> {
    /// Mod manager
    pub mod_manager: &'a ModManager,
    /// Game state info (paused, tick, player_count)
    pub game_state: GameStateInfo,
    /// Test state info for E2E testing
    pub test_state: TestStateInfo,
    /// Inventory state for E2E testing
    pub inventory_state: inventory::InventoryStateInfo,
    /// Player state for E2E testing
    pub player_state: player::PlayerStateInfo,
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
        "test.send_input" => test::handle_test_send_input(request),
        "test.assert" => test::handle_test_assert(request, &ctx.test_state),
        "test.reset_state" => test::handle_test_reset_state(request),
        // Texture handlers
        "texture.list" => textures::handle_texture_list(request),
        "texture.get_atlas_info" => textures::handle_get_atlas_info(request),
        "texture.register_resolver" => textures::handle_register_resolver(request),
        // Network handlers (N.5: resource network API)
        "network.type.list" => network::handle_network_type_list(request),
        "network.type.register" => network::handle_network_type_register(request),
        "network.segment.list" => network::handle_network_segment_list(request),
        "network.segment.get" => network::handle_network_segment_get(request),
        "network.virtual_link.add" => network::handle_network_virtual_link_add(request),
        "network.virtual_link.remove" => network::handle_network_virtual_link_remove(request),
        "network.virtual_link.list" => network::handle_network_virtual_link_list(request),
        // Inventory handlers (E2E testing)
        "inventory.get_slot" => inventory::handle_inventory_get_slot(request, &ctx.inventory_state),
        "inventory.list" => inventory::handle_inventory_list(request, &ctx.inventory_state),
        "inventory.move_item" => inventory::handle_inventory_move_item(request),
        "inventory.get_hotbar" => {
            inventory::handle_inventory_get_hotbar(request, &ctx.inventory_state)
        }
        // Player handlers (E2E testing)
        "player.get_state" => player::handle_player_get_state(request, &ctx.player_state),
        "player.teleport" => player::handle_player_teleport(request),
        "player.get_looking_at" => player::handle_player_get_looking_at(request, &ctx.player_state),
        "player.set_selected_slot" => player::handle_player_set_selected_slot(request),
        // World handlers (E2E testing)
        "world.get_block" => world::handle_world_get_block(request),
        "world.place_block" => world::handle_world_place_block(request),
        "world.break_block" => world::handle_world_break_block(request),
        "world.raycast" => world::handle_world_raycast(request),
        // Quest handlers (E2E testing)
        "quest.list" => quest::handle_quest_list(request),
        "quest.get" => quest::handle_quest_get(request),
        // Craft handlers (E2E testing)
        "craft.list" => craft::handle_craft_list(request),
        "craft.get" => craft::handle_craft_get(request),
        "craft.can_craft" => craft::handle_craft_can_craft(request, &ctx.inventory_state),
        // Machine slot handlers (E2E testing)
        "machine.get_slots" => machine::handle_machine_get_slots(request),
        "machine.insert_item" => machine::handle_machine_insert_item(request),
        "machine.extract_item" => machine::handle_machine_extract_item(request),
        "machine.get_progress" => machine::handle_machine_get_progress(request),
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
                inventory_state: inventory::InventoryStateInfo::default(),
                player_state: player::PlayerStateInfo::default(),
            };
            mod_handlers::handle_mod_list(request, &read_ctx)
        }
        "mod.info" => {
            let read_ctx = HandlerContext {
                mod_manager: ctx.mod_manager,
                game_state: GameStateInfo::default(),
                test_state: TestStateInfo::default(),
                inventory_state: inventory::InventoryStateInfo::default(),
                player_state: player::PlayerStateInfo::default(),
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

    #[test]
    fn test_route_unknown_method() {
        let manager = ModManager::new();
        let ctx = HandlerContext {
            mod_manager: &manager,
            game_state: GameStateInfo::default(),
            test_state: TestStateInfo::default(),
            inventory_state: inventory::InventoryStateInfo::default(),
            player_state: player::PlayerStateInfo::default(),
        };
        let request = JsonRpcRequest::new(1, "unknown.method", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, METHOD_NOT_FOUND);
    }

    #[test]
    fn test_route_machine_list() {
        let manager = ModManager::new();
        let ctx = HandlerContext {
            mod_manager: &manager,
            game_state: GameStateInfo::default(),
            test_state: TestStateInfo::default(),
            inventory_state: inventory::InventoryStateInfo::default(),
            player_state: player::PlayerStateInfo::default(),
        };
        let request = JsonRpcRequest::new(1, "machine.list", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
    }

    #[test]
    fn test_route_machine_add() {
        let manager = ModManager::new();
        let ctx = HandlerContext {
            mod_manager: &manager,
            game_state: GameStateInfo::default(),
            test_state: TestStateInfo::default(),
            inventory_state: inventory::InventoryStateInfo::default(),
            player_state: player::PlayerStateInfo::default(),
        };
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
        let ctx = HandlerContext {
            mod_manager: &manager,
            game_state: GameStateInfo::default(),
            test_state: TestStateInfo::default(),
            inventory_state: inventory::InventoryStateInfo::default(),
            player_state: player::PlayerStateInfo::default(),
        };
        let request = JsonRpcRequest::new(1, "recipe.list", serde_json::Value::Null);
        let response = route_request(&request, &ctx);

        assert!(response.is_success());
    }

    #[test]
    fn test_route_recipe_add() {
        let manager = ModManager::new();
        let ctx = HandlerContext {
            mod_manager: &manager,
            game_state: GameStateInfo::default(),
            test_state: TestStateInfo::default(),
            inventory_state: inventory::InventoryStateInfo::default(),
            player_state: player::PlayerStateInfo::default(),
        };
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
        let ctx = HandlerContext {
            mod_manager: &manager,
            game_state: GameStateInfo::default(),
            test_state: TestStateInfo::default(),
            inventory_state: inventory::InventoryStateInfo::default(),
            player_state: player::PlayerStateInfo::default(),
        };
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
        let ctx = HandlerContext {
            mod_manager: &manager,
            game_state: GameStateInfo {
                paused: true,
                tick: 12345,
                player_count: 1,
            },
            test_state: TestStateInfo::default(),
            inventory_state: inventory::InventoryStateInfo::default(),
            player_state: player::PlayerStateInfo::default(),
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
        let ctx = HandlerContext {
            mod_manager: &manager,
            game_state: GameStateInfo::default(),
            test_state: TestStateInfo::default(),
            inventory_state: inventory::InventoryStateInfo::default(),
            player_state: player::PlayerStateInfo::default(),
        };
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
        let ctx = HandlerContext {
            mod_manager: &manager,
            game_state: GameStateInfo::default(),
            test_state: TestStateInfo::default(),
            inventory_state: inventory::InventoryStateInfo::default(),
            player_state: player::PlayerStateInfo::default(),
        };
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
}
