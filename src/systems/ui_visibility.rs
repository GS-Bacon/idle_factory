//! UI Visibility control systems
//!
//! This module provides centralized UI visibility management via events.
//! All UI visibility is controlled through the UIVisibilityController resource.

use crate::components::{GenericMachineUI, Machine};
use crate::events::{UIConditionChanged, UIRegistration};
use crate::ui::visibility::{ConditionKey, UIVisibilityController, UIVisibilityTarget};
use crate::{
    CommandInputState, CursorLockState, InputState, InteractingMachine, InventoryOpen,
    TutorialProgress,
};
use bevy::prelude::*;

/// Listen for UIConditionChanged events and update controller state
pub fn on_ui_condition_changed(
    mut events: EventReader<UIConditionChanged>,
    mut controller: ResMut<UIVisibilityController>,
) {
    for event in events.read() {
        match event {
            UIConditionChanged::TutorialCompleted => {
                controller.set_condition(ConditionKey::TutorialCompleted, true);
            }
            UIConditionChanged::TutorialReset => {
                controller.set_condition(ConditionKey::TutorialCompleted, false);
            }
            UIConditionChanged::Custom { name, value } => {
                controller.set_condition(ConditionKey::Custom(name.clone()), *value);
            }
        }
    }
}

/// Sync InputState to controller when any UI state resource changes
pub fn sync_input_state_to_controller(
    inventory_open: Res<InventoryOpen>,
    interacting_machine: Res<InteractingMachine>,
    command_state: Res<CommandInputState>,
    cursor_state: Res<CursorLockState>,
    mut controller: ResMut<UIVisibilityController>,
) {
    // Only run if any resource changed
    if !inventory_open.is_changed()
        && !interacting_machine.is_changed()
        && !command_state.is_changed()
        && !cursor_state.is_changed()
    {
        return;
    }

    let current_state = InputState::current(
        &inventory_open,
        &interacting_machine,
        &command_state,
        &cursor_state,
    );

    // Update all InputState conditions
    let all_states = [
        InputState::Gameplay,
        InputState::Inventory,
        InputState::MachineUI,
        InputState::Command,
        InputState::Paused,
    ];

    for state in all_states {
        controller.set_condition(ConditionKey::InputStateIs(state), current_state == state);
    }
}

/// Initialize tutorial completion state from TutorialProgress
pub fn init_tutorial_state(
    progress: Res<TutorialProgress>,
    mut controller: ResMut<UIVisibilityController>,
) {
    if progress.is_added() || progress.is_changed() {
        controller.set_condition(ConditionKey::TutorialCompleted, progress.completed);
    }
}

/// Machine UI visibility - special handling for multiple machine types
///
/// Unlike other UIs, machine UI depends on which machine is being interacted with.
/// This system shows the correct machine UI based on InteractingMachine.
pub fn update_machine_ui_visibility(
    interacting: Res<InteractingMachine>,
    machine_query: Query<&Machine>,
    mut ui_query: Query<(&GenericMachineUI, &mut Visibility)>,
) {
    // Determine which machine UI should be visible
    let target_machine_id = interacting
        .0
        .and_then(|e| machine_query.get(e).ok())
        .map(|m| m.spec.id);

    for (ui, mut vis) in ui_query.iter_mut() {
        *vis = if target_machine_id == Some(ui.machine_id) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Update all UI visibility based on controller state
///
/// This is the simplified version using UIVisibilityTarget component.
/// Machine UI is handled separately by update_machine_ui_visibility.
pub fn update_all_ui_visibility(
    mut controller: ResMut<UIVisibilityController>,
    mut query: Query<(&UIVisibilityTarget, &mut Visibility)>,
) {
    if !controller.is_dirty() {
        return;
    }

    for (target, mut vis) in query.iter_mut() {
        *vis = controller.evaluate(&target.id);
    }

    controller.clear_dirty();
}

/// Apply UI registration events to the controller
///
/// This system handles UIRegistration events from the Mod API
/// and registers the visibility rules with the controller.
pub fn apply_ui_registration(
    mut events: EventReader<UIRegistration>,
    mut controller: ResMut<UIVisibilityController>,
) {
    for event in events.read() {
        controller.register(event.id.clone(), event.rules.clone());
        tracing::info!("Registered UI visibility rules for {:?}", event.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_condition_keys() {
        // Verify all InputState variants have corresponding ConditionKey
        let states = [
            InputState::Gameplay,
            InputState::Inventory,
            InputState::MachineUI,
            InputState::Command,
            InputState::Paused,
        ];

        for state in states {
            let key = ConditionKey::InputStateIs(state);
            assert!(matches!(key, ConditionKey::InputStateIs(_)));
        }
    }
}
