//! WebSocket server implementation

use crossbeam_channel::{Receiver, Sender};

use super::config::ModApiServerConfig;
use super::messages::{ClientMessage, ServerMessage};

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

    use crate::modding::protocol::{JsonRpcRequest, JsonRpcResponse, PARSE_ERROR};

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
            // Note: crossbeam recv() is blocking, so we use try_recv() with async sleep
            let connections_clone = connections.clone();
            tokio::spawn(async move {
                loop {
                    // Non-blocking poll for messages from Bevy
                    match client_rx.try_recv() {
                        Ok(msg) => {
                            let conns = connections_clone.read().await;
                            match msg {
                                ClientMessage::Response { conn_id, response } => {
                                    if let Some(sender) = conns.get(&conn_id) {
                                        let json = serde_json::to_string(&response)
                                            .unwrap_or_else(|_| "{}".to_string());
                                        tracing::info!(
                                            "Sending WS response to conn {}: {}",
                                            conn_id,
                                            &json[..json.len().min(100)]
                                        );
                                        let _ = sender.send(json);
                                    } else {
                                        tracing::warn!(
                                            "No connection found for conn_id {}",
                                            conn_id
                                        );
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
                        Err(crossbeam_channel::TryRecvError::Empty) => {
                            // No messages, yield and try again
                            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                        }
                        Err(crossbeam_channel::TryRecvError::Disconnected) => {
                            // Channel closed, exit loop
                            break;
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
                                        tracing::info!("WS received text from {}: {}", conn_id, &text[..text.len().min(100)]);
                                        // Parse JSON-RPC request
                                        match serde_json::from_str::<JsonRpcRequest>(&text) {
                                            Ok(request) => {
                                                tracing::info!("Parsed request: method={}", request.method);
                                                let _ = server_tx_clone.send(ServerMessage::Request {
                                                    conn_id,
                                                    request,
                                                });
                                            }
                                            Err(e) => {
                                                tracing::warn!("Invalid JSON-RPC from {}: {} (text: {})", conn_id, e, &text[..text.len().min(50)]);
                                                // Send parse error response
                                                let error_response = JsonRpcResponse::error(
                                                    None,
                                                    PARSE_ERROR,
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
