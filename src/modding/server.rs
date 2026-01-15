//! WebSocket server for Mod API
//!
//! Runs in a separate thread with tokio runtime,
//! communicates with Bevy main thread via crossbeam channels.

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};

use super::connection::ConnectionManager;
use super::handlers::{
    route_request, GameStateInfo, HandlerContext, InputFlags, SlotInfo, TestStateInfo,
};
use super::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use super::ModManager;

use crate::components::{
    BreakingProgress, CommandInputState, CursorLockState, GlobalInventoryOpen, InputState,
    InteractingMachine, InventoryOpen, TargetBlock, UIContext, UIState,
};
use crate::core::{items, ItemId};
use crate::events::TestEventBuffer;
use crate::input::{GameAction, TestInputEvent};
use crate::modding::handlers::UIElementInfo;
use crate::player::{LocalPlayer, PlayerInventory};
use crate::systems::command::{SetBlockEvent, TeleportEvent};

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
    /// Command queue for test.send_command (processed by separate system)
    pub command_queue: Vec<String>,
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

/// Update the cached UI element states
fn update_ui_element_cache(
    mut cache: ResMut<UIElementCache>,
    registry: Option<Res<crate::game_spec::UIElementRegistry>>,
    query: Query<(
        &crate::game_spec::UIElementTag,
        &Visibility,
        Option<&Interaction>,
    )>,
) {
    let Some(registry) = registry else {
        return;
    };

    cache.elements = crate::systems::ui_visibility::collect_ui_element_states(&registry, &query)
        .into_iter()
        .map(|info| UIElementInfo {
            id: info.id,
            visible: info.visible,
            interactable: info.interactable,
        })
        .collect();
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
        command_queue: Vec::new(),
    });
}

#[allow(clippy::too_many_arguments)]
fn process_server_messages(
    server: Option<ResMut<ModApiServer>>,
    mod_manager: Res<ModManager>,
    mut cursor_lock: Option<ResMut<CursorLockState>>,
    time: Res<Time>,
    mut ui_state: Option<ResMut<UIState>>,
    mut inventory_open: Option<ResMut<InventoryOpen>>,
    mut interacting_machine: Option<ResMut<InteractingMachine>>,
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<(&Transform, &PlayerInventory)>,
    mut test_input_writer: EventWriter<TestInputEvent>,
    mut command_state: Option<ResMut<CommandInputState>>,
    mut global_inv_open: Option<ResMut<GlobalInventoryOpen>>,
    target_block: Option<Res<TargetBlock>>,
    breaking_progress: Option<Res<BreakingProgress>>,
    mut test_event_buffer: Option<ResMut<TestEventBuffer>>,
    ui_element_cache: Option<Res<UIElementCache>>,
) {
    let Some(mut server) = server else { return };

    // Build game state info
    let game_state = GameStateInfo {
        paused: cursor_lock.as_ref().is_some_and(|c| c.paused),
        tick: (time.elapsed_secs_f64() * 1000.0) as u64,
        player_count: 1, // Single-player for now
    };

    // Build test state info
    // Check both UIState and legacy resources for compatibility
    let ui_state_str = if let Some(ref s) = ui_state {
        let current = s.current();
        bevy::log::info!(
            "[WS] UIState.current() = {:?}, stack_depth = {}",
            current,
            s.stack_depth()
        );
        // If UIState says Gameplay, check legacy resources
        if current == UIContext::Gameplay {
            if command_state.as_ref().is_some_and(|c| c.open) {
                "Command".to_string()
            } else if global_inv_open.as_ref().is_some_and(|g| g.0) {
                "GlobalInventory".to_string()
            } else {
                ui_context_to_string(&current)
            }
        } else {
            ui_context_to_string(&current)
        }
    } else {
        String::new()
    };

    // Get input state flags
    let input_state = match (
        inventory_open.as_ref(),
        interacting_machine.as_ref(),
        command_state.as_ref(),
        cursor_lock.as_ref(),
    ) {
        (Some(inv), Some(machine), Some(cmd), Some(cursor)) => {
            InputState::current(inv, machine, cmd, cursor)
        }
        _ => InputState::Gameplay,
    };
    let input_flags = InputFlags {
        allows_block_actions: input_state.allows_block_actions(),
        allows_movement: input_state.allows_movement(),
        allows_camera: input_state.allows_camera(),
        allows_hotbar: input_state.allows_hotbar(),
    };

    // Get target block info
    let target_block_pos = target_block
        .as_ref()
        .and_then(|t| t.break_target)
        .map(|pos| [pos.x, pos.y, pos.z]);

    // Get breaking progress
    let breaking_prog = breaking_progress
        .as_ref()
        .map(|b| b.progress)
        .unwrap_or(0.0);

    // Get UI stack info
    let (ui_stack, stack_depth) = if let Some(ref s) = ui_state {
        (s.stack_as_strings(), s.stack_depth())
    } else {
        (vec![], 0)
    };

    // Get player data (position, inventory)
    let (player_position, hotbar, selected_slot) = local_player
        .as_ref()
        .and_then(|lp| player_query.get(lp.0).ok())
        .map(|(transform, inventory)| {
            let pos = [
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ];
            // Build hotbar slots (first 9 slots)
            let hotbar: Vec<SlotInfo> = (0..9)
                .map(|i| {
                    if let Some((item_id, count)) = inventory.slots.get(i).and_then(|s| *s) {
                        SlotInfo {
                            item_id: Some(item_id.name().unwrap_or("base:unknown").to_string()),
                            count,
                        }
                    } else {
                        SlotInfo::default()
                    }
                })
                .collect();
            (pos, hotbar, inventory.selected_slot)
        })
        .unwrap_or_default();

    let test_state = TestStateInfo {
        ui_state: ui_state_str,
        player_position,
        // Use UIState as single source of truth (not CursorLockState.paused)
        // Cursor is locked only when in Gameplay mode
        cursor_locked: ui_state.as_ref().is_some_and(|s| s.is_gameplay()),
        target_block: target_block_pos,
        breaking_progress: breaking_prog,
        input_flags,
        ui_stack,
        stack_depth,
        hotbar,
        selected_slot,
    };

    // Clone events for context (we'll handle clear_events specially)
    let test_events = test_event_buffer
        .as_ref()
        .map(|b| b.events.clone())
        .unwrap_or_default();

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
                tracing::info!(
                    "Received request from {}: method={}",
                    conn_id,
                    request.method
                );

                // Handle test.send_input specially to inject input
                if request.method == "test.send_input" {
                    if let Some(action_str) = request.params.get("action").and_then(|v| v.as_str())
                    {
                        if let Some(action) = parse_game_action(action_str) {
                            test_input_writer.send(TestInputEvent { action });
                        }
                    }
                }

                // Handle test.set_ui_state specially to change UI state
                if request.method == "test.set_ui_state" {
                    if let Some(state_str) = request.params.get("state").and_then(|v| v.as_str()) {
                        apply_ui_state_change(
                            state_str,
                            &mut ui_state,
                            &mut inventory_open,
                            &mut interacting_machine,
                            &mut cursor_lock,
                            &mut command_state,
                            &mut global_inv_open,
                        );
                    }
                }

                // Handle test.send_command to queue command for execution
                if request.method == "test.send_command" {
                    if let Some(cmd) = request.params.get("command").and_then(|v| v.as_str()) {
                        server.command_queue.push(cmd.to_string());
                        tracing::info!("Command queued: {}", cmd);
                    }
                }

                // Handle test.clear_events specially to clear the buffer
                let cleared_count = if request.method == "test.clear_events" {
                    test_event_buffer.as_mut().map(|b| b.clear()).unwrap_or(0)
                } else {
                    0
                };

                // Route to appropriate handler
                // Get cached UI element states
                let ui_elements = ui_element_cache
                    .as_ref()
                    .map(|c| c.elements.clone())
                    .unwrap_or_default();

                let ctx = HandlerContext {
                    mod_manager: &mod_manager,
                    game_state: game_state.clone(),
                    test_state: test_state.clone(),
                    test_events: test_events.clone(),
                    cleared_events_count: cleared_count,
                    ui_elements,
                };
                let response = route_request(&request, &ctx);
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

/// Convert UIContext to string for test API
fn ui_context_to_string(ctx: &UIContext) -> String {
    match ctx {
        UIContext::Gameplay => "Gameplay".to_string(),
        UIContext::Inventory => "Inventory".to_string(),
        UIContext::GlobalInventory => "GlobalInventory".to_string(),
        UIContext::CommandInput => "Command".to_string(),
        UIContext::PauseMenu => "PauseMenu".to_string(),
        UIContext::Settings => "Settings".to_string(),
        UIContext::Machine(_) => "MachineUI".to_string(),
    }
}

/// Apply UI state change for test.set_ui_state API
fn apply_ui_state_change(
    state_str: &str,
    ui_state: &mut Option<ResMut<UIState>>,
    inventory_open: &mut Option<ResMut<InventoryOpen>>,
    interacting_machine: &mut Option<ResMut<InteractingMachine>>,
    cursor_lock: &mut Option<ResMut<CursorLockState>>,
    command_state: &mut Option<ResMut<CommandInputState>>,
    global_inv_open: &mut Option<ResMut<GlobalInventoryOpen>>,
) {
    // Get mutable references to all resources
    let (Some(ui), Some(inv), Some(machine), Some(cursor)) = (
        ui_state.as_mut(),
        inventory_open.as_mut(),
        interacting_machine.as_mut(),
        cursor_lock.as_mut(),
    ) else {
        tracing::warn!("Cannot apply UI state change: missing resources");
        return;
    };

    // Helper to reset legacy resources
    let reset_legacy = |inv: &mut InventoryOpen,
                        machine: &mut InteractingMachine,
                        cmd: &mut Option<ResMut<CommandInputState>>,
                        global: &mut Option<ResMut<GlobalInventoryOpen>>| {
        inv.0 = false;
        machine.0 = None;
        if let Some(c) = cmd.as_mut() {
            c.open = false;
        }
        if let Some(g) = global.as_mut() {
            g.0 = false;
        }
    };

    match state_str {
        "Gameplay" => {
            ui.clear();
            reset_legacy(inv, machine, command_state, global_inv_open);
            cursor.paused = false;
        }
        "Inventory" => {
            ui.clear();
            ui.push(UIContext::Inventory);
            reset_legacy(inv, machine, command_state, global_inv_open);
            inv.0 = true;
            cursor.paused = false;
        }
        "MachineUI" => {
            ui.clear();
            // Use a dummy entity for test purposes
            let dummy_entity = Entity::from_raw(999999);
            ui.push(UIContext::Machine(dummy_entity));
            reset_legacy(inv, machine, command_state, global_inv_open);
            machine.0 = Some(dummy_entity);
            cursor.paused = false;
        }
        "PauseMenu" => {
            ui.clear();
            ui.push(UIContext::PauseMenu);
            reset_legacy(inv, machine, command_state, global_inv_open);
            cursor.paused = true;
        }
        "GlobalInventory" => {
            ui.clear();
            ui.push(UIContext::GlobalInventory);
            reset_legacy(inv, machine, command_state, global_inv_open);
            if let Some(g) = global_inv_open.as_mut() {
                g.0 = true;
            }
            cursor.paused = false;
        }
        "Command" => {
            ui.clear();
            ui.push(UIContext::CommandInput);
            reset_legacy(inv, machine, command_state, global_inv_open);
            if let Some(c) = command_state.as_mut() {
                c.open = true;
            }
            cursor.paused = true; // Unlock cursor when command input is open
        }
        "Settings" => {
            ui.clear();
            ui.push(UIContext::Settings);
            reset_legacy(inv, machine, command_state, global_inv_open);
            cursor.paused = true;
        }
        _ => {
            tracing::warn!("Unknown UI state: {}", state_str);
        }
    }

    tracing::info!("UI state changed to: {}", state_str);
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

/// Process queued commands from test.send_command
/// This runs as a separate system to avoid parameter limit in process_server_messages
fn process_test_command_queue(
    mut server: Option<ResMut<ModApiServer>>,
    mut teleport_writer: EventWriter<TeleportEvent>,
    mut setblock_writer: EventWriter<SetBlockEvent>,
    mut tutorial_shown: Option<ResMut<crate::components::TutorialShown>>,
) {
    let Some(ref mut server) = server else { return };

    for cmd in server.command_queue.drain(..) {
        tracing::info!("Processing command: {}", cmd);
        parse_and_execute_command(
            &cmd,
            &mut teleport_writer,
            &mut setblock_writer,
            &mut tutorial_shown,
        );
    }
}

/// Parse a command string and execute it
fn parse_and_execute_command(
    cmd: &str,
    teleport_writer: &mut EventWriter<TeleportEvent>,
    setblock_writer: &mut EventWriter<SetBlockEvent>,
    tutorial_shown: &mut Option<ResMut<crate::components::TutorialShown>>,
) {
    let cmd = cmd.trim();
    let cmd = cmd.strip_prefix('/').unwrap_or(cmd);
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    if parts.is_empty() {
        tracing::warn!("Empty command");
        return;
    }

    match parts[0] {
        "tp" | "teleport" => {
            // /tp x y z
            if parts.len() < 4 {
                tracing::warn!("tp requires 3 coordinates: /tp x y z");
                return;
            }
            let x: f32 = parts[1].parse().unwrap_or(0.0);
            let y: f32 = parts[2].parse().unwrap_or(0.0);
            let z: f32 = parts[3].parse().unwrap_or(0.0);
            teleport_writer.send(TeleportEvent {
                position: Vec3::new(x, y, z),
            });
            tracing::info!("Teleport to ({}, {}, {})", x, y, z);
        }
        "setblock" => {
            // /setblock x y z item_id
            if parts.len() < 5 {
                tracing::warn!("setblock requires 4 args: /setblock x y z item_id");
                return;
            }
            let x: i32 = parts[1].parse().unwrap_or(0);
            let y: i32 = parts[2].parse().unwrap_or(0);
            let z: i32 = parts[3].parse().unwrap_or(0);
            let item_id_str = parts[4];
            // Try to get ItemId from interner, fall back to stone if not found
            let item_id = items::interner()
                .get(item_id_str)
                .map(ItemId::from_raw)
                .unwrap_or_else(items::stone);
            setblock_writer.send(SetBlockEvent {
                position: IVec3::new(x, y, z),
                block_type: item_id,
            });
            tracing::info!("SetBlock at ({}, {}, {}) = {}", x, y, z, item_id_str);
        }
        "dismiss_tutorial" => {
            // /dismiss_tutorial - Force dismiss tutorial
            if let Some(tutorial) = tutorial_shown.as_mut() {
                tutorial.0 = true;
                tracing::info!("Tutorial dismissed via API");
            } else {
                tracing::warn!("TutorialShown resource not available");
            }
        }
        _ => {
            tracing::warn!("Unknown command: {}", parts[0]);
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
