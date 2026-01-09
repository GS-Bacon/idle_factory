//! WebSocket server for Mod API
//!
//! Runs in a separate thread with tokio runtime,
//! communicates with Bevy main thread via crossbeam channels.

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};

use super::commands::{TestCommand, TestCommandQueue};
use super::connection::ConnectionManager;
use super::handlers::events::EventSubscriptions;
use super::handlers::inventory::{InventoryStateInfo, SlotInfo};
use super::handlers::player::PlayerStateInfo;
use super::handlers::test::{handle_test_subscribe_event, handle_test_unsubscribe_event};
use super::handlers::ui::{handle_ui_register, handle_ui_set_condition};
use super::handlers::{route_request, GameStateInfo, HandlerContext, TestStateInfo};
use super::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use super::ModManager;

use crate::components::{CommandInputState, CursorLockState, PlayerPhysics, UIState};
use crate::constants::HOTBAR_SLOTS;
use crate::core::items;
use crate::events::{UIConditionChanged, UIRegistration};
use crate::input::{GameAction, TestInputEvent};
use crate::player::{LocalPlayer, PlayerInventory};
use crate::ui::visibility::{UIId, UIVisibilityController};

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
            .init_resource::<EventSubscriptions>()
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

#[allow(clippy::too_many_arguments)]
fn process_server_messages(
    server: Option<ResMut<ModApiServer>>,
    mod_manager: Res<ModManager>,
    mut event_subscriptions: ResMut<EventSubscriptions>,
    mut command_queue: ResMut<TestCommandQueue>,
    cursor_lock: Option<Res<CursorLockState>>,
    time: Res<Time>,
    ui_state: Option<Res<UIState>>,
    command_input_state: Option<Res<CommandInputState>>,
    local_player: Option<Res<LocalPlayer>>,
    visibility_controller: Option<Res<UIVisibilityController>>,
    transforms: Query<&Transform>,
    inventory_query: Query<&PlayerInventory>,
    physics_query: Query<&PlayerPhysics>,
    mut test_input_writer: EventWriter<TestInputEvent>,
    mut ui_condition_writer: EventWriter<UIConditionChanged>,
    mut ui_registration_writer: EventWriter<UIRegistration>,
) {
    let Some(mut server) = server else { return };

    // Build game state info
    let game_state = GameStateInfo {
        paused: cursor_lock.as_ref().is_some_and(|c| c.paused),
        tick: (time.elapsed_secs_f64() * 1000.0) as u64,
        player_count: 1, // Single-player for now
    };

    // Build test state info
    let cursor_state = cursor_lock.as_ref();
    let player_transform = local_player
        .as_ref()
        .and_then(|lp| transforms.get(lp.0).ok());
    // Compute UI state, checking CommandInputState if UIState shows Gameplay
    let computed_ui_state = if command_input_state.as_ref().is_some_and(|c| c.open) {
        "Command".to_string() // Match InputState::Command
    } else {
        ui_state
            .as_ref()
            .map(|s| format!("{:?}", s.current()))
            .unwrap_or_default()
    };

    // Build visible_ui_elements including UIVisibilityController state
    let mut visible_elements = ui_state
        .as_ref()
        .map(|s| s.visible_elements())
        .unwrap_or_default();

    // Add visibility-controlled UI elements from UIVisibilityController
    if let Some(controller) = visibility_controller.as_ref() {
        // Check QuestPanel visibility
        if controller.evaluate(&UIId::QuestPanel) == bevy::prelude::Visibility::Inherited {
            visible_elements.push("QuestPanel".to_string());
        }
        // Check TutorialPanel visibility
        if controller.evaluate(&UIId::TutorialPanel) == bevy::prelude::Visibility::Inherited {
            visible_elements.push("TutorialPanel".to_string());
        }
        // Check Hotbar visibility (replace the static one if needed)
        if controller.evaluate(&UIId::Hotbar) != bevy::prelude::Visibility::Inherited {
            visible_elements.retain(|e| e != "Hotbar");
        }
    }

    let test_state = TestStateInfo {
        ui_state: computed_ui_state,
        player_position: player_transform
            .map(|t| [t.translation.x, t.translation.y, t.translation.z])
            .unwrap_or_default(),
        cursor_locked: cursor_state.is_some_and(|c| !c.paused),
        visible_ui_elements: visible_elements,
        settings_open: ui_state.as_ref().is_some_and(|s| s.settings_open),
        cursor_in_window: cursor_state.is_some_and(|c| c.cursor_in_window),
        cursor_visible: cursor_state.is_some_and(|c| c.cursor_visible),
    };

    // Build inventory state from actual ECS data
    let inventory_state = local_player
        .as_ref()
        .and_then(|lp| inventory_query.get(lp.0).ok())
        .map(build_inventory_state)
        .unwrap_or_default();

    // Build player state from actual ECS data
    let player_state = build_player_state(
        player_transform,
        local_player
            .as_ref()
            .and_then(|lp| physics_query.get(lp.0).ok()),
        local_player
            .as_ref()
            .and_then(|lp| inventory_query.get(lp.0).ok()),
    );

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
                // Clean up event subscriptions for this connection
                event_subscriptions.remove_connection(conn_id);
                tracing::info!("Mod disconnected: {}", conn_id);
            }
            ServerMessage::Request { conn_id, request } => {
                tracing::info!(
                    "Received request from {}: method={}",
                    conn_id,
                    request.method
                );

                // Handle special methods that produce events
                let response = match request.method.as_str() {
                    "test.send_input" => {
                        if let Some(action_str) =
                            request.params.get("action").and_then(|v| v.as_str())
                        {
                            if let Some(action) = parse_game_action(action_str) {
                                test_input_writer.send(TestInputEvent { action });
                            }
                        }
                        // Still route through normal handler for response
                        let ctx = HandlerContext {
                            mod_manager: &mod_manager,
                            game_state: game_state.clone(),
                            test_state: test_state.clone(),
                            inventory_state: inventory_state.clone(),
                            player_state: player_state.clone(),
                        };
                        route_request(&request, &ctx)
                    }
                    "ui.set_condition" => match handle_ui_set_condition(&request) {
                        Ok((response, event)) => {
                            ui_condition_writer.send(event);
                            response
                        }
                        Err(error_response) => error_response,
                    },
                    "ui.register" => match handle_ui_register(&request) {
                        Ok((response, event)) => {
                            ui_registration_writer.send(event);
                            response
                        }
                        Err(error_response) => error_response,
                    },
                    "test.subscribe_event" => {
                        handle_test_subscribe_event(&request, conn_id, &mut event_subscriptions)
                    }
                    "test.unsubscribe_event" => {
                        handle_test_unsubscribe_event(&request, &mut event_subscriptions)
                    }
                    // Command queue methods - these need to be queued for execution
                    "player.teleport" => queue_player_teleport(&request, &mut command_queue),
                    "player.set_selected_slot" => {
                        queue_player_set_slot(&request, &mut command_queue)
                    }
                    "inventory.move_item" => queue_inventory_move(&request, &mut command_queue),
                    "world.place_block" => queue_world_place_block(&request, &mut command_queue),
                    "world.break_block" => queue_world_break_block(&request, &mut command_queue),
                    "test.reset_state" => queue_test_reset_state(&request, &mut command_queue),
                    _ => {
                        // Route to normal handlers
                        let ctx = HandlerContext {
                            mod_manager: &mod_manager,
                            game_state: game_state.clone(),
                            test_state: test_state.clone(),
                            inventory_state: inventory_state.clone(),
                            player_state: player_state.clone(),
                        };
                        route_request(&request, &ctx)
                    }
                };

                tracing::info!("Sending response for conn {}: {:?}", conn_id, response.id);
                match server
                    .tx
                    .send(ClientMessage::Response { conn_id, response })
                {
                    Ok(_) => tracing::info!("Response queued for conn {}", conn_id),
                    Err(e) => tracing::error!("Failed to queue response: {}", e),
                }
            }
        }
    }
}

/// Build InventoryStateInfo from actual PlayerInventory component
fn build_inventory_state(inventory: &PlayerInventory) -> InventoryStateInfo {
    let slots: Vec<SlotInfo> = inventory
        .slots
        .iter()
        .enumerate()
        .map(|(i, slot)| SlotInfo {
            index: i,
            item_id: slot
                .as_ref()
                .and_then(|(id, _)| id.name().map(|s| s.to_string())),
            amount: slot.as_ref().map(|(_, count)| *count).unwrap_or(0),
        })
        .collect();

    let hotbar: Vec<SlotInfo> = slots.iter().take(HOTBAR_SLOTS).cloned().collect();

    InventoryStateInfo {
        slots,
        hotbar,
        selected_hotbar: inventory.selected_slot,
        equipment: vec![], // Equipment slots not implemented yet
    }
}

/// Build PlayerStateInfo from actual ECS data
fn build_player_state(
    transform: Option<&Transform>,
    physics: Option<&PlayerPhysics>,
    inventory: Option<&PlayerInventory>,
) -> PlayerStateInfo {
    let position = transform
        .map(|t| [t.translation.x, t.translation.y, t.translation.z])
        .unwrap_or_default();

    // Extract rotation (yaw, pitch) from transform
    let rotation = transform
        .map(|t| {
            let (_, rotation, _) = t.rotation.to_euler(EulerRot::YXZ);
            // Note: This is a simplification, actual camera rotation may be stored differently
            [rotation.to_degrees(), 0.0]
        })
        .unwrap_or_default();

    let velocity = physics
        .map(|p| [p.velocity.x, p.velocity.y, p.velocity.z])
        .unwrap_or_default();

    let on_ground = physics.map(|p| p.on_ground).unwrap_or(false);

    PlayerStateInfo {
        position,
        rotation,
        velocity,
        on_ground,
        flying: false, // TODO: Track flying state if implemented
        selected_slot: inventory.map(|i| i.selected_slot).unwrap_or(0),
        looking_at: None, // TODO: Implement raycast to get looking_at block
        looking_distance: None,
    }
}

// =============================================================================
// Command Queue Helper Functions
// =============================================================================

use super::protocol::INVALID_PARAMS;

/// Queue player teleport command
fn queue_player_teleport(
    request: &JsonRpcRequest,
    queue: &mut TestCommandQueue,
) -> JsonRpcResponse {
    #[derive(serde::Deserialize)]
    struct Params {
        x: f32,
        y: f32,
        z: f32,
        yaw: Option<f32>,
        pitch: Option<f32>,
    }

    let params: Params = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    let request_id = queue.next_request_id();
    queue.queue(TestCommand::PlayerTeleport {
        position: Vec3::new(params.x, params.y, params.z),
        rotation: params.yaw.zip(params.pitch),
        request_id,
    });

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "request_id": request_id,
            "position": [params.x, params.y, params.z],
        }),
    )
}

/// Queue player set selected slot command
fn queue_player_set_slot(
    request: &JsonRpcRequest,
    queue: &mut TestCommandQueue,
) -> JsonRpcResponse {
    #[derive(serde::Deserialize)]
    struct Params {
        slot: usize,
    }

    let params: Params = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    if params.slot > 8 {
        return JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            format!("Slot must be 0-8, got {}", params.slot),
        );
    }

    let request_id = queue.next_request_id();
    queue.queue(TestCommand::PlayerSetSlot {
        slot: params.slot,
        request_id,
    });

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "request_id": request_id,
            "slot": params.slot,
        }),
    )
}

/// Queue inventory move command
fn queue_inventory_move(request: &JsonRpcRequest, queue: &mut TestCommandQueue) -> JsonRpcResponse {
    #[derive(serde::Deserialize)]
    struct Params {
        from: usize,
        to: usize,
        amount: Option<u32>,
    }

    let params: Params = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    let request_id = queue.next_request_id();
    queue.queue(TestCommand::InventoryMove {
        from: params.from,
        to: params.to,
        amount: params.amount,
        request_id,
    });

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "request_id": request_id,
            "from": params.from,
            "to": params.to,
        }),
    )
}

/// Queue world place block command
fn queue_world_place_block(
    request: &JsonRpcRequest,
    queue: &mut TestCommandQueue,
) -> JsonRpcResponse {
    #[derive(serde::Deserialize)]
    struct Params {
        x: i32,
        y: i32,
        z: i32,
        item_id: String,
        facing: Option<u8>,
    }

    let params: Params = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Convert item_id string to ItemId
    let item_id = match items::by_name(&params.item_id) {
        Some(id) => id,
        None => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Unknown item_id: {}", params.item_id),
            );
        }
    };

    let request_id = queue.next_request_id();
    queue.queue(TestCommand::WorldPlaceBlock {
        pos: IVec3::new(params.x, params.y, params.z),
        item_id,
        facing: params.facing,
        request_id,
    });

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "request_id": request_id,
            "position": [params.x, params.y, params.z],
            "item_id": params.item_id,
        }),
    )
}

/// Queue world break block command
fn queue_world_break_block(
    request: &JsonRpcRequest,
    queue: &mut TestCommandQueue,
) -> JsonRpcResponse {
    #[derive(serde::Deserialize)]
    struct Params {
        x: i32,
        y: i32,
        z: i32,
    }

    let params: Params = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    let request_id = queue.next_request_id();
    queue.queue(TestCommand::WorldBreakBlock {
        pos: IVec3::new(params.x, params.y, params.z),
        request_id,
    });

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "request_id": request_id,
            "position": [params.x, params.y, params.z],
        }),
    )
}

/// Queue a test.reset_state command
fn queue_test_reset_state(
    request: &JsonRpcRequest,
    queue: &mut TestCommandQueue,
) -> JsonRpcResponse {
    let request_id = queue.next_request_id();
    queue.queue(TestCommand::ResetState { request_id });

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "queued": true,
            "request_id": request_id,
            "note": "State reset queued - will close all UIs and return to Gameplay"
        }),
    )
}

/// Parse GameAction from string
fn parse_game_action(s: &str) -> Option<GameAction> {
    match s {
        "MoveForward" => Some(GameAction::MoveForward),
        "MoveBackward" => Some(GameAction::MoveBackward),
        "MoveLeft" => Some(GameAction::MoveLeft),
        "MoveRight" => Some(GameAction::MoveRight),
        "Jump" => Some(GameAction::Jump),
        "Descend" => Some(GameAction::Descend),
        "LookUp" => Some(GameAction::LookUp),
        "LookDown" => Some(GameAction::LookDown),
        "LookLeft" => Some(GameAction::LookLeft),
        "LookRight" => Some(GameAction::LookRight),
        "ToggleInventory" => Some(GameAction::ToggleInventory),
        "TogglePause" => Some(GameAction::TogglePause),
        "ToggleGlobalInventory" => Some(GameAction::ToggleGlobalInventory),
        "ToggleQuest" => Some(GameAction::ToggleQuest),
        "OpenCommand" => Some(GameAction::OpenCommand),
        "CloseUI" => Some(GameAction::CloseUI),
        "Confirm" => Some(GameAction::Confirm),
        "Cancel" => Some(GameAction::Cancel),
        "Hotbar1" => Some(GameAction::Hotbar1),
        "Hotbar2" => Some(GameAction::Hotbar2),
        "Hotbar3" => Some(GameAction::Hotbar3),
        "Hotbar4" => Some(GameAction::Hotbar4),
        "Hotbar5" => Some(GameAction::Hotbar5),
        "Hotbar6" => Some(GameAction::Hotbar6),
        "Hotbar7" => Some(GameAction::Hotbar7),
        "Hotbar8" => Some(GameAction::Hotbar8),
        "Hotbar9" => Some(GameAction::Hotbar9),
        "PrimaryAction" => Some(GameAction::PrimaryAction),
        "SecondaryAction" => Some(GameAction::SecondaryAction),
        "RotateBlock" => Some(GameAction::RotateBlock),
        "ModifierShift" => Some(GameAction::ModifierShift),
        "ToggleDebug" => Some(GameAction::ToggleDebug),
        "DeleteChar" => Some(GameAction::DeleteChar),
        _ => None,
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
