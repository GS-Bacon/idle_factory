//! Server configuration and resources

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};

use super::messages::{ClientMessage, ServerMessage};
use crate::modding::connection::ConnectionManager;
use crate::modding::handlers::UIElementInfo;

/// Cached UI element states for test API
#[derive(Resource, Default)]
pub struct UIElementCache {
    pub elements: Vec<UIElementInfo>,
}

/// Queue for test commands (processed by separate system to avoid param limit)
#[derive(Resource, Default)]
pub struct TestCommandQueue {
    pub commands: Vec<String>,
}

/// Server configuration
#[derive(Resource, Clone)]
pub struct ModApiServerConfig {
    /// Whether the server is enabled
    pub enabled: bool,
    /// Host address to bind
    pub host: String,
    /// Port number
    pub port: u16,
}

impl Default for ModApiServerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 9877,
        }
    }
}

/// Server resource for Bevy
#[derive(Resource)]
pub struct ModApiServer {
    /// Receive messages from WebSocket server
    pub rx: Receiver<ServerMessage>,
    /// Send messages to WebSocket server
    pub tx: Sender<ClientMessage>,
    /// Connection manager
    pub connections: ConnectionManager,
    /// Command queue for test.send_command (processed by separate system)
    pub command_queue: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_api_server_config_default() {
        let config = ModApiServerConfig::default();

        assert!(config.enabled);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 9877);
    }
}
