//! UI Navigation Systems
//!
//! Handles UI state transitions, action events, and legacy state synchronization.

use bevy::prelude::*;

use crate::components::{
    CommandInputState, CursorLockState, GlobalInventoryOpen, InteractingMachine, InventoryOpen,
    UIAction, UIContext, UIState,
};

/// Handle UIAction events and update UIState
pub fn ui_action_handler(mut ui_state: ResMut<UIState>, mut action_events: EventReader<UIAction>) {
    for action in action_events.read() {
        match action {
            UIAction::Push(context) => {
                ui_state.push(context.clone());
            }
            UIAction::Pop => {
                ui_state.pop();
            }
            UIAction::Clear => {
                ui_state.clear();
            }
            UIAction::Replace(context) => {
                ui_state.replace(context.clone());
            }
            UIAction::Toggle(context) => {
                if ui_state.is_active(context) {
                    ui_state.pop();
                } else {
                    ui_state.push(context.clone());
                }
            }
        }
    }
}

/// Sync UIState to legacy resources for backwards compatibility
/// This allows gradual migration without breaking existing systems
pub fn sync_legacy_ui_state(
    ui_state: Res<UIState>,
    mut inv_open: ResMut<InventoryOpen>,
    mut global_inv_open: ResMut<GlobalInventoryOpen>,
    mut cmd_state: ResMut<CommandInputState>,
    mut machine_res: ResMut<InteractingMachine>,
    mut cursor_lock: ResMut<CursorLockState>,
) {
    // Only sync if UIState changed
    if !ui_state.is_changed() {
        return;
    }

    let current = ui_state.current();

    // Reset all legacy states
    inv_open.0 = false;
    global_inv_open.0 = false;
    cmd_state.open = false;
    machine_res.0 = None;

    // Set legacy state based on current UIState
    match current {
        UIContext::Gameplay => {
            cursor_lock.paused = false;
        }
        UIContext::Inventory => {
            inv_open.0 = true;
            cursor_lock.paused = true;
        }
        UIContext::GlobalInventory => {
            global_inv_open.0 = true;
            cursor_lock.paused = true;
        }
        UIContext::CommandInput => {
            cmd_state.open = true;
            cursor_lock.paused = true;
        }
        UIContext::PauseMenu => {
            cursor_lock.paused = true;
        }
        UIContext::Settings => {
            cursor_lock.paused = true;
        }
        UIContext::Machine(entity) => {
            cursor_lock.paused = true;
            machine_res.0 = Some(entity);
        }
    }
}

/// Handle ESC key for UI navigation
/// ESC pops current UI or opens pause menu if in gameplay
pub fn ui_escape_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    ui_state: Res<UIState>,
    command_state: Res<CommandInputState>,
    mut action_writer: EventWriter<UIAction>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    // Don't handle ESC if command input is open (it handles its own ESC)
    if command_state.open {
        return;
    }

    if ui_state.is_gameplay() {
        // In gameplay, ESC opens pause menu
        action_writer.send(UIAction::Push(UIContext::PauseMenu));
    } else {
        // In any UI, ESC goes back
        action_writer.send(UIAction::Pop);
    }
}

/// Handle E key for inventory toggle
pub fn ui_inventory_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    ui_state: Res<UIState>,
    mut action_writer: EventWriter<UIAction>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    // Only toggle inventory from gameplay or inventory itself
    match ui_state.current() {
        UIContext::Gameplay => {
            action_writer.send(UIAction::Push(UIContext::Inventory));
        }
        UIContext::Inventory => {
            action_writer.send(UIAction::Pop);
        }
        UIContext::Machine(_) => {
            // Close machine UI and open inventory
            action_writer.send(UIAction::Replace(UIContext::Inventory));
        }
        _ => {} // Ignore E in other contexts
    }
}

/// Handle Tab key for global inventory toggle
pub fn ui_global_inventory_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    ui_state: Res<UIState>,
    command_state: Res<CommandInputState>,
    mut action_writer: EventWriter<UIAction>,
) {
    if !keyboard.just_pressed(KeyCode::Tab) {
        return;
    }

    // Don't handle Tab if command input is open (it handles Tab for autocomplete)
    if command_state.open {
        return;
    }

    match ui_state.current() {
        UIContext::Gameplay => {
            action_writer.send(UIAction::Push(UIContext::GlobalInventory));
        }
        UIContext::GlobalInventory => {
            action_writer.send(UIAction::Pop);
        }
        _ => {} // Ignore Tab in other contexts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_action_push() {
        let mut ui_state = UIState::default();
        ui_state.push(UIContext::Inventory);
        assert_eq!(ui_state.current(), UIContext::Inventory);
    }

    #[test]
    fn test_ui_action_toggle() {
        let mut ui_state = UIState::default();

        // Toggle on
        if !ui_state.is_active(&UIContext::Inventory) {
            ui_state.push(UIContext::Inventory);
        }
        assert_eq!(ui_state.current(), UIContext::Inventory);

        // Toggle off
        if ui_state.is_active(&UIContext::Inventory) {
            ui_state.pop();
        }
        assert!(ui_state.is_gameplay());
    }
}
