//! WebSocket server for Mod API
//!
//! Runs in a separate thread with tokio runtime,
//! communicates with Bevy main thread via crossbeam channels.

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};

use super::connection::ConnectionManager;
use super::handlers::{route_request, GameStateInfo, HandlerContext};
use super::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use super::ModManager;

use crate::components::CursorLockState;

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

/// Message from WebSocket server to Bevy main thread
#[derive(Debug)]
pub enum ServerMessage {
    /// New connection established
    Connected { conn_id: u64 },
    /// Connection closed
    Disconnected { conn_id: u64 },
    /// JSON-RPC request received
    Request {
        conn_id: u64,
        request: JsonRpcRequest,
    },
}

/// Message from Bevy main thread to WebSocket server
#[derive(Debug)]
pub enum ClientMessage {
    /// Send response to a specific connection
    Response {
        conn_id: u64,
        response: JsonRpcResponse,
    },
    /// Send notification to a specific connection
    Notify {
        conn_id: u64,
        notification: JsonRpcNotification,
    },
    /// Broadcast notification to all connections
    Broadcast { notification: JsonRpcNotification },
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
}

/// Start the WebSocket server in a separate thread
#[cfg(not(target_arch = "wasm32"))]
pub fn start_websocket_server(
    config: ModApiServerConfig,
) -> (Receiver<ServerMessage>, Sender<ClientMessage>) {
    use futures_util::{SinkExt, StreamExt};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio::sync::RwLock;
    use tokio_tungstenite::accept_async;
    use tokio_tungstenite::tungstenite::Message;

    let (server_tx, server_rx) = crossbeam_channel::unbounded::<ServerMessage>();
    let (client_tx, client_rx) = crossbeam_channel::unbounded::<ClientMessage>();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(async move {
            let addr = format!("{}:{}", config.host, config.port);
            let listener = match TcpListener::bind(&addr).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("Failed to bind WebSocket server to {}: {}", addr, e);
                    return;
                }
            };

            tracing::info!("Mod API WebSocket server listening on ws://{}", addr);

            // Track active connections (conn_id -> sender channel)
            type ConnectionSenders =
                Arc<RwLock<HashMap<u64, tokio::sync::mpsc::UnboundedSender<String>>>>;
            let connections: ConnectionSenders = Arc::new(RwLock::new(HashMap::new()));
            let next_conn_id = Arc::new(std::sync::atomic::AtomicU64::new(0));

            // Spawn task to handle outgoing messages from Bevy
            let connections_clone = connections.clone();
            tokio::spawn(async move {
                while let Ok(msg) = client_rx.recv() {
                    let conns = connections_clone.read().await;
                    match msg {
                        ClientMessage::Response { conn_id, response } => {
                            if let Some(sender) = conns.get(&conn_id) {
                                let json = serde_json::to_string(&response)
                                    .unwrap_or_else(|_| "{}".to_string());
                                let _ = sender.send(json);
                            }
                        }
                        ClientMessage::Notify {
                            conn_id,
                            notification,
                        } => {
                            if let Some(sender) = conns.get(&conn_id) {
                                let json = serde_json::to_string(&notification)
                                    .unwrap_or_else(|_| "{}".to_string());
                                let _ = sender.send(json);
                            }
                        }
                        ClientMessage::Broadcast { notification } => {
                            let json = serde_json::to_string(&notification)
                                .unwrap_or_else(|_| "{}".to_string());
                            for sender in conns.values() {
                                let _ = sender.send(json.clone());
                            }
                        }
                    }
                }
            });

            // Accept connections loop
            loop {
                let (stream, peer_addr) = match listener.accept().await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("Failed to accept connection: {}", e);
                        continue;
                    }
                };

                let ws_stream = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        tracing::warn!("WebSocket handshake failed for {}: {}", peer_addr, e);
                        continue;
                    }
                };

                let conn_id = next_conn_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                tracing::debug!("New WebSocket connection {} from {}", conn_id, peer_addr);

                // Create channel for this connection's outgoing messages
                let (conn_tx, mut conn_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

                // Register connection
                {
                    let mut conns = connections.write().await;
                    conns.insert(conn_id, conn_tx);
                }

                // Notify Bevy of new connection
                let _ = server_tx.send(ServerMessage::Connected { conn_id });

                let server_tx_clone = server_tx.clone();
                let connections_clone = connections.clone();

                // Spawn task for this connection
                tokio::spawn(async move {
                    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                    loop {
                        tokio::select! {
                            // Handle incoming messages from WebSocket
                            msg = ws_receiver.next() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) => {
                                        // Parse JSON-RPC request
                                        match serde_json::from_str::<JsonRpcRequest>(&text) {
                                            Ok(request) => {
                                                let _ = server_tx_clone.send(ServerMessage::Request {
                                                    conn_id,
                                                    request,
                                                });
                                            }
                                            Err(e) => {
                                                tracing::warn!("Invalid JSON-RPC from {}: {}", conn_id, e);
                                                // Send parse error response
                                                let error_response = JsonRpcResponse::error(
                                                    None,
                                                    super::protocol::PARSE_ERROR,
                                                    "Parse error"
                                                );
                                                let json = serde_json::to_string(&error_response)
                                                    .unwrap_or_else(|_| "{}".to_string());
                                                if ws_sender.send(Message::Text(json)).await.is_err() {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    Some(Ok(Message::Close(_))) => {
                                        tracing::debug!("Connection {} closed by client", conn_id);
                                        break;
                                    }
                                    Some(Ok(Message::Ping(data))) => {
                                        if ws_sender.send(Message::Pong(data)).await.is_err() {
                                            break;
                                        }
                                    }
                                    Some(Ok(_)) => {
                                        // Ignore other message types (Binary, Pong, Frame)
                                    }
                                    Some(Err(e)) => {
                                        tracing::warn!("WebSocket error for {}: {}", conn_id, e);
                                        break;
                                    }
                                    None => {
                                        // Stream closed
                                        break;
                                    }
                                }
                            }
                            // Handle outgoing messages to WebSocket
                            msg = conn_rx.recv() => {
                                match msg {
                                    Some(json) => {
                                        if ws_sender.send(Message::Text(json)).await.is_err() {
                                            break;
                                        }
                                    }
                                    None => {
                                        // Channel closed
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // Connection ended, clean up
                    {
                        let mut conns = connections_clone.write().await;
                        conns.remove(&conn_id);
                    }

                    // Notify Bevy of disconnection
                    let _ = server_tx_clone.send(ServerMessage::Disconnected { conn_id });
                    tracing::debug!("Connection {} cleaned up", conn_id);
                });
            }
        });
    });

    (server_rx, client_tx)
}

/// WASM stub (WebSocket server not available)
#[cfg(target_arch = "wasm32")]
pub fn start_websocket_server(
    _config: ModApiServerConfig,
) -> (Receiver<ServerMessage>, Sender<ClientMessage>) {
    let (_server_tx, server_rx) = crossbeam_channel::unbounded();
    let (client_tx, _client_rx) = crossbeam_channel::unbounded();
    tracing::warn!("WebSocket server not available on WASM");
    (server_rx, client_tx)
}

/// Bevy Plugin for Mod API server
pub struct ModApiServerPlugin;

impl Plugin for ModApiServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModApiServerConfig>()
            .add_systems(Startup, setup_mod_api_server)
            .add_systems(Update, process_server_messages);
    }
}

fn setup_mod_api_server(mut commands: Commands, config: Res<ModApiServerConfig>) {
    if !config.enabled {
        tracing::info!("Mod API server disabled");
        return;
    }

    let (rx, tx) = start_websocket_server(config.clone());
    commands.insert_resource(ModApiServer {
        rx,
        tx,
        connections: ConnectionManager::new(),
    });
}

fn process_server_messages(
    server: Option<ResMut<ModApiServer>>,
    mod_manager: Res<ModManager>,
    cursor_lock: Option<Res<CursorLockState>>,
    time: Res<Time>,
) {
    let Some(mut server) = server else { return };

    // Build game state info
    let game_state = GameStateInfo {
        paused: cursor_lock.as_ref().is_some_and(|c| c.paused),
        tick: (time.elapsed_secs_f64() * 1000.0) as u64,
        player_count: 1, // Single-player for now
    };

    // Process received messages (non-blocking)
    while let Ok(msg) = server.rx.try_recv() {
        match msg {
            ServerMessage::Connected { conn_id } => {
                let local_id = server.connections.add_connection();
                // Note: conn_id from server may differ from local ConnectionManager ID
                // For now we log both, in future we may need a mapping
                tracing::info!(
                    "Mod connected: server_id={}, local_id={}",
                    conn_id,
                    local_id
                );
            }
            ServerMessage::Disconnected { conn_id } => {
                // Remove by server conn_id
                // Note: This is a simplified implementation - in production we'd need proper ID mapping
                server.connections.remove_connection(conn_id);
                tracing::info!("Mod disconnected: {}", conn_id);
            }
            ServerMessage::Request { conn_id, request } => {
                tracing::debug!(
                    "Received request from {}: method={}",
                    conn_id,
                    request.method
                );

                // Route to appropriate handler
                let ctx = HandlerContext {
                    mod_manager: &mod_manager,
                    game_state: game_state.clone(),
                };
                let response = route_request(&request, &ctx);
                let _ = server
                    .tx
                    .send(ClientMessage::Response { conn_id, response });
            }
        }
    }
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

    #[test]
    fn test_server_message_variants() {
        // Ensure all variants can be created
        let _connected = ServerMessage::Connected { conn_id: 1 };
        let _disconnected = ServerMessage::Disconnected { conn_id: 1 };
        let _request = ServerMessage::Request {
            conn_id: 1,
            request: JsonRpcRequest::new(1, "test", serde_json::Value::Null),
        };
    }

    #[test]
    fn test_client_message_variants() {
        // Ensure all variants can be created
        let _response = ClientMessage::Response {
            conn_id: 1,
            response: JsonRpcResponse::success(Some(1), serde_json::json!({})),
        };
        let _notify = ClientMessage::Notify {
            conn_id: 1,
            notification: JsonRpcNotification::empty("test"),
        };
        let _broadcast = ClientMessage::Broadcast {
            notification: JsonRpcNotification::empty("test"),
        };
    }
}
