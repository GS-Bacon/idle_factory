//! WebSocket server for Mod API
//!
//! Runs in a separate thread with tokio runtime,
//! communicates with Bevy main thread via crossbeam channels.

mod commands;
mod config;
mod message_handler;
mod messages;
mod ui_state;
mod websocket;

use bevy::prelude::*;

// Re-export public types (explicit, no wildcard)
pub use config::{ModApiServer, ModApiServerConfig, TestCommandQueue, UIElementCache};
pub use messages::{ClientMessage, ServerMessage};
pub use websocket::start_websocket_server;

use commands::process_test_command_queue;
use message_handler::{process_server_messages, setup_mod_api_server, update_ui_element_cache};

/// Bevy Plugin for Mod API server
pub struct ModApiServerPlugin;

impl Plugin for ModApiServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModApiServerConfig>()
            .init_resource::<UIElementCache>()
            .init_resource::<TestCommandQueue>()
            .add_systems(Startup, setup_mod_api_server)
            .add_systems(
                Update,
                (
                    update_ui_element_cache,
                    process_server_messages,
                    process_test_command_queue,
                )
                    .chain(),
            );
    }
}
