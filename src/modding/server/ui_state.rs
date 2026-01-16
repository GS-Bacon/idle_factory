//! UI state utilities for test API

use bevy::prelude::*;

use crate::components::{
    CommandInputState, CursorLockState, InteractingMachine, InventoryOpen, UIContext, UIState,
};

/// Convert UIContext to string for test API
pub fn ui_context_to_string(ctx: &UIContext) -> String {
    match ctx {
        UIContext::Gameplay => "Gameplay".to_string(),
        UIContext::Inventory => "Inventory".to_string(),
        UIContext::CommandInput => "Command".to_string(),
        UIContext::PauseMenu => "PauseMenu".to_string(),
        UIContext::Settings => "Settings".to_string(),
        UIContext::Machine(_) => "MachineUI".to_string(),
    }
}

/// Apply UI state change for test.set_ui_state API
pub fn apply_ui_state_change(
    state_str: &str,
    ui_state: &mut Option<ResMut<UIState>>,
    inventory_open: &mut Option<ResMut<InventoryOpen>>,
    interacting_machine: &mut Option<ResMut<InteractingMachine>>,
    cursor_lock: &mut Option<ResMut<CursorLockState>>,
    command_state: &mut Option<ResMut<CommandInputState>>,
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
                        cmd: &mut Option<ResMut<CommandInputState>>| {
        inv.0 = false;
        machine.0 = None;
        if let Some(c) = cmd.as_mut() {
            c.open = false;
        }
    };

    match state_str {
        "Gameplay" => {
            ui.clear();
            reset_legacy(inv, machine, command_state);
            cursor.paused = false;
        }
        "Inventory" => {
            ui.clear();
            ui.push(UIContext::Inventory);
            reset_legacy(inv, machine, command_state);
            inv.0 = true;
            cursor.paused = false;
        }
        "MachineUI" => {
            ui.clear();
            // Use a dummy entity for test purposes
            let dummy_entity = Entity::from_raw(999999);
            ui.push(UIContext::Machine(dummy_entity));
            reset_legacy(inv, machine, command_state);
            machine.0 = Some(dummy_entity);
            cursor.paused = false;
        }
        "PauseMenu" => {
            ui.clear();
            ui.push(UIContext::PauseMenu);
            reset_legacy(inv, machine, command_state);
            cursor.paused = true;
        }
        "Command" => {
            ui.clear();
            ui.push(UIContext::CommandInput);
            reset_legacy(inv, machine, command_state);
            if let Some(c) = command_state.as_mut() {
                c.open = true;
            }
            cursor.paused = true; // Unlock cursor when command input is open
        }
        "Settings" => {
            ui.clear();
            ui.push(UIContext::Settings);
            reset_legacy(inv, machine, command_state);
            cursor.paused = true;
        }
        _ => {
            tracing::warn!("Unknown UI state: {}", state_str);
        }
    }

    tracing::info!("UI state changed to: {}", state_str);
}
