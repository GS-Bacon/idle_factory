//! Handlers for game.* JSON-RPC methods
//!
//! Provides:
//! - `game.version`: Returns game and API version info
//! - `game.state`: Returns current game state (paused, tick, player_count)

use crate::modding::protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::json;

/// API version constant
pub const API_VERSION: &str = "1.0.0";

/// Handle `game.version` request
///
/// Returns the game version and API version.
///
/// # ja
/// ゲームバージョンとAPIバージョンを取得
///
/// # Response
/// ```json
/// {
///     "version": "0.3.78",
///     "api_version": "1.0.0"
/// }
/// ```
pub fn handle_game_version(request: &JsonRpcRequest) -> JsonRpcResponse {
    let result = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "api_version": API_VERSION
    });

    JsonRpcResponse::success(request.id, result)
}

/// Game state information for `game.state` response
#[derive(Debug, Clone)]
pub struct GameStateInfo {
    /// Whether the game is paused
    pub paused: bool,
    /// Current game tick (elapsed time in milliseconds)
    pub tick: u64,
    /// Number of players (always 1 for single-player)
    pub player_count: u32,
}

impl Default for GameStateInfo {
    fn default() -> Self {
        Self {
            paused: false,
            tick: 0,
            player_count: 1,
        }
    }
}

/// Handle `game.state` request
///
/// Returns the current game state.
///
/// # ja
/// 現在のゲーム状態を取得
///
/// # Response
/// ```json
/// {
///     "paused": false,
///     "tick": 12345,
///     "player_count": 1
/// }
/// ```
pub fn handle_game_state(request: &JsonRpcRequest, state: &GameStateInfo) -> JsonRpcResponse {
    let result = json!({
        "paused": state.paused,
        "tick": state.tick,
        "player_count": state.player_count
    });

    JsonRpcResponse::success(request.id, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_game_version() {
        let request = JsonRpcRequest::new(1, "game.version", serde_json::Value::Null);
        let response = handle_game_version(&request);

        assert!(response.is_success());
        assert_eq!(response.id, Some(1));

        let result = response.result.unwrap();
        assert_eq!(result["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(result["api_version"], API_VERSION);
    }

    #[test]
    fn test_handle_game_state() {
        let request = JsonRpcRequest::new(2, "game.state", serde_json::Value::Null);
        let state = GameStateInfo {
            paused: true,
            tick: 12345,
            player_count: 1,
        };
        let response = handle_game_state(&request, &state);

        assert!(response.is_success());
        assert_eq!(response.id, Some(2));

        let result = response.result.unwrap();
        assert_eq!(result["paused"], true);
        assert_eq!(result["tick"], 12345);
        assert_eq!(result["player_count"], 1);
    }

    #[test]
    fn test_handle_game_state_default() {
        let request = JsonRpcRequest::new(3, "game.state", serde_json::Value::Null);
        let state = GameStateInfo::default();
        let response = handle_game_state(&request, &state);

        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["paused"], false);
        assert_eq!(result["tick"], 0);
        assert_eq!(result["player_count"], 1);
    }

    #[test]
    fn test_api_version_constant() {
        assert_eq!(API_VERSION, "1.0.0");
    }
}
