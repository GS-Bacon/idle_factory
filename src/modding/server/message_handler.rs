//! Message handling systems

use bevy::prelude::*;

use crate::components::{
    BreakingProgress, CommandInputState, CursorLockState, InputState, InteractingMachine,
    InventoryOpen, TargetBlock, UIContext, UIState,
};
use crate::events::TestEventBuffer;
use crate::input::TestInputEvent;
use crate::modding::connection::ConnectionManager;
use crate::modding::handlers::{
    route_request, GameStateInfo, HandlerContext, InputFlags, SlotInfo, TestStateInfo,
    UIElementInfo,
};
use crate::modding::ModManager;
use crate::player::{LocalPlayer, PlayerInventory};

use super::commands::parse_game_action;
use super::config::{ModApiServer, ModApiServerConfig, UIElementCache};
use super::messages::{ClientMessage, ServerMessage};
use super::ui_state::{apply_ui_state_change, ui_context_to_string};
use super::websocket::start_websocket_server;

/// Update the cached UI element states
pub fn update_ui_element_cache(
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

pub fn setup_mod_api_server(mut commands: Commands, config: Res<ModApiServerConfig>) {
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
pub fn process_server_messages(
    server: Option<ResMut<ModApiServer>>,
    mod_manager: Res<ModManager>,
    mut cursor_lock: Option<ResMut<CursorLockState>>,
    time: Res<Time>,
    mut ui_state: Option<ResMut<UIState>>,
    mut inventory_open: Option<ResMut<InventoryOpen>>,
    mut interacting_machine: Option<ResMut<InteractingMachine>>,
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<(&Transform, &PlayerInventory)>,
    mut test_input_writer: MessageWriter<TestInputEvent>,
    mut command_state: Option<ResMut<CommandInputState>>,
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
        // CAD-style controls: cursor is never locked (always visible)
        cursor_locked: false,
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
                            test_input_writer.write(TestInputEvent { action });
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
