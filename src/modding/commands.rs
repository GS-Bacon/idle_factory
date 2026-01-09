//! Test command queue for E2E testing
//!
//! This module provides a command queue system that allows WebSocket API
//! handlers to queue commands for execution in the main game loop.
//!
//! Commands are queued by handlers and processed by the `process_test_commands`
//! system which runs in Bevy's Update schedule.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::{
    CommandInputState, CursorLockState, InteractingMachine, InventoryOpen, UIState,
};
use crate::core::ItemId;
use crate::player::{LocalPlayer, PlayerInventory};

/// Unique ID for tracking command results
pub type RequestId = u64;

/// A command to be executed by the game
#[derive(Debug, Clone)]
pub enum TestCommand {
    /// Move items between inventory slots
    InventoryMove {
        from: usize,
        to: usize,
        amount: Option<u32>,
        request_id: RequestId,
    },
    /// Teleport player to a position
    PlayerTeleport {
        position: Vec3,
        rotation: Option<(f32, f32)>,
        request_id: RequestId,
    },
    /// Set player's selected hotbar slot
    PlayerSetSlot { slot: usize, request_id: RequestId },
    /// Place a block in the world
    WorldPlaceBlock {
        pos: IVec3,
        item_id: ItemId,
        facing: Option<u8>,
        request_id: RequestId,
    },
    /// Break a block in the world
    WorldBreakBlock { pos: IVec3, request_id: RequestId },
    /// Reset game state to Gameplay mode (close all UIs)
    ResetState { request_id: RequestId },
}

/// Result of a command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub data: serde_json::Value,
}

impl CommandResult {
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
        }
    }

    pub fn failure(message: &str) -> Self {
        Self {
            success: false,
            data: serde_json::json!({ "error": message }),
        }
    }
}

/// Queue of test commands to be executed
#[derive(Resource, Default)]
pub struct TestCommandQueue {
    /// Commands waiting to be executed
    pub commands: Vec<TestCommand>,
    /// Results of executed commands (keyed by request_id)
    pub results: HashMap<RequestId, CommandResult>,
    /// Next request ID
    next_id: RequestId,
}

impl TestCommandQueue {
    /// Generate a new unique request ID
    pub fn next_request_id(&mut self) -> RequestId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Queue a command and return its request ID
    pub fn queue(&mut self, command: TestCommand) -> RequestId {
        let request_id = match &command {
            TestCommand::InventoryMove { request_id, .. } => *request_id,
            TestCommand::PlayerTeleport { request_id, .. } => *request_id,
            TestCommand::PlayerSetSlot { request_id, .. } => *request_id,
            TestCommand::WorldPlaceBlock { request_id, .. } => *request_id,
            TestCommand::WorldBreakBlock { request_id, .. } => *request_id,
            TestCommand::ResetState { request_id, .. } => *request_id,
        };
        self.commands.push(command);
        request_id
    }

    /// Get result for a request ID (consumes the result)
    pub fn take_result(&mut self, request_id: RequestId) -> Option<CommandResult> {
        self.results.remove(&request_id)
    }
}

/// System to process test commands
#[allow(clippy::too_many_arguments)]
pub fn process_test_commands(
    mut queue: ResMut<TestCommandQueue>,
    local_player: Option<Res<LocalPlayer>>,
    mut ui_state: ResMut<UIState>,
    mut command_input_state: Option<ResMut<CommandInputState>>,
    mut cursor_lock_state: Option<ResMut<CursorLockState>>,
    mut inventory_open: Option<ResMut<InventoryOpen>>,
    mut interacting_machine: Option<ResMut<InteractingMachine>>,
    mut inventory_query: Query<&mut PlayerInventory>,
    mut transform_query: Query<&mut Transform>,
) {
    // Collect commands first to avoid borrow conflict with results
    let commands: Vec<_> = queue.commands.drain(..).collect();

    if commands.is_empty() {
        return;
    }

    // Process all queued commands
    for command in commands {
        let result = match command {
            // ResetState doesn't need LocalPlayer
            TestCommand::ResetState { request_id } => {
                // Clear all UI state and return to gameplay
                ui_state.clear();
                ui_state.settings_open = false;
                // Also close command input if open
                if let Some(ref mut cmd_state) = command_input_state {
                    cmd_state.open = false;
                    cmd_state.text.clear();
                }
                // Reset cursor lock state (unpause)
                if let Some(ref mut cursor_state) = cursor_lock_state {
                    cursor_state.paused = false;
                }
                // Close inventory
                if let Some(ref mut inv_open) = inventory_open {
                    inv_open.0 = false;
                }
                // Close machine UI
                if let Some(ref mut machine) = interacting_machine {
                    machine.0 = None;
                }
                let result = CommandResult::success(serde_json::json!({
                    "reset": true,
                    "ui_state": "Gameplay"
                }));
                (request_id, result)
            }

            TestCommand::InventoryMove {
                from,
                to,
                amount,
                request_id,
            } => {
                let Some(ref lp) = local_player else {
                    queue
                        .results
                        .insert(request_id, CommandResult::failure("LocalPlayer not found"));
                    continue;
                };
                let result = if let Ok(mut inventory) = inventory_query.get_mut(lp.0) {
                    // Get item from source slot
                    if let Some((item_id, count)) = inventory.slots[from].take() {
                        let move_amount = amount.unwrap_or(count).min(count);
                        let remaining = count - move_amount;

                        // Put remaining back in source
                        if remaining > 0 {
                            inventory.slots[from] = Some((item_id, remaining));
                        }

                        // Add to destination
                        if let Some((dest_id, dest_count)) = inventory.slots[to].take() {
                            if dest_id == item_id {
                                // Stack with existing
                                let new_count = dest_count + move_amount;
                                inventory.slots[to] = Some((item_id, new_count));
                            } else {
                                // Swap items
                                inventory.slots[to] = Some((item_id, move_amount));
                                inventory.slots[from] = Some((dest_id, dest_count));
                            }
                        } else {
                            // Empty destination
                            inventory.slots[to] = Some((item_id, move_amount));
                        }

                        CommandResult::success(serde_json::json!({
                            "moved": move_amount,
                            "from": from,
                            "to": to
                        }))
                    } else {
                        CommandResult::failure("Source slot is empty")
                    }
                } else {
                    CommandResult::failure("Player inventory not found")
                };
                (request_id, result)
            }

            TestCommand::PlayerTeleport {
                position,
                rotation,
                request_id,
            } => {
                let Some(ref lp) = local_player else {
                    queue
                        .results
                        .insert(request_id, CommandResult::failure("LocalPlayer not found"));
                    continue;
                };
                let result = if let Ok(mut transform) = transform_query.get_mut(lp.0) {
                    transform.translation = position;
                    if let Some((yaw, pitch)) = rotation {
                        transform.rotation = Quat::from_euler(
                            EulerRot::YXZ,
                            yaw.to_radians(),
                            pitch.to_radians(),
                            0.0,
                        );
                    }
                    CommandResult::success(serde_json::json!({
                        "position": [position.x, position.y, position.z],
                        "rotation": rotation
                    }))
                } else {
                    CommandResult::failure("Player transform not found")
                };
                (request_id, result)
            }

            TestCommand::PlayerSetSlot { slot, request_id } => {
                let Some(ref lp) = local_player else {
                    queue
                        .results
                        .insert(request_id, CommandResult::failure("LocalPlayer not found"));
                    continue;
                };
                let result = if slot > 8 {
                    CommandResult::failure("Slot must be 0-8")
                } else if let Ok(mut inventory) = inventory_query.get_mut(lp.0) {
                    inventory.selected_slot = slot;
                    CommandResult::success(serde_json::json!({ "slot": slot }))
                } else {
                    CommandResult::failure("Player inventory not found")
                };
                (request_id, result)
            }

            TestCommand::WorldPlaceBlock {
                pos: _,
                item_id: _,
                facing: _,
                request_id,
            } => {
                // TODO: Implement block placement via WorldData
                // This requires access to WorldData and proper block placement logic
                let result = CommandResult::failure("Block placement not yet implemented");
                (request_id, result)
            }

            TestCommand::WorldBreakBlock { pos: _, request_id } => {
                // TODO: Implement block breaking via WorldData
                // This requires access to WorldData and proper block breaking logic
                let result = CommandResult::failure("Block breaking not yet implemented");
                (request_id, result)
            }
        };

        queue.results.insert(result.0, result.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_queue_next_id() {
        let mut queue = TestCommandQueue::default();
        assert_eq!(queue.next_request_id(), 0);
        assert_eq!(queue.next_request_id(), 1);
        assert_eq!(queue.next_request_id(), 2);
    }

    #[test]
    fn test_command_result_success() {
        let result = CommandResult::success(serde_json::json!({ "test": 123 }));
        assert!(result.success);
        assert_eq!(result.data["test"], 123);
    }

    #[test]
    fn test_command_result_failure() {
        let result = CommandResult::failure("Something went wrong");
        assert!(!result.success);
        assert_eq!(result.data["error"], "Something went wrong");
    }

    #[test]
    fn test_queue_command() {
        let mut queue = TestCommandQueue::default();
        let request_id = queue.next_request_id();
        let cmd = TestCommand::PlayerSetSlot {
            slot: 3,
            request_id,
        };
        queue.queue(cmd);
        assert_eq!(queue.commands.len(), 1);
    }

    #[test]
    fn test_take_result() {
        let mut queue = TestCommandQueue::default();
        let request_id = 42;
        queue.results.insert(
            request_id,
            CommandResult::success(serde_json::json!({ "ok": true })),
        );

        let result = queue.take_result(request_id);
        assert!(result.is_some());
        assert!(result.unwrap().success);

        // Taking again should return None
        assert!(queue.take_result(request_id).is_none());
    }
}
