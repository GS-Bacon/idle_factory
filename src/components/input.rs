//! Input state management

use super::player::CursorLockState;
use super::ui::{CommandInputState, InteractingMachine, InventoryOpen};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// Current input state - used to determine which inputs should be active
/// See CLAUDE.md "入力マトリクス" for the full state table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputState {
    /// Normal gameplay - all inputs active
    Gameplay,
    /// Inventory is open - only inventory interactions active
    Inventory,
    /// Machine UI is open - only machine interactions active
    MachineUI,
    /// Command input is open - only text input active
    Command,
    /// Game is paused (ESC) - only click to resume
    Paused,
}

impl InputState {
    /// Determine current input state from all UI resources
    ///
    /// Priority order (highest to lowest):
    /// 1. Command input (T or / key)
    /// 2. Inventory (E key)
    /// 3. Machine UI (right-click on machine)
    /// 4. Paused (ESC key, only if no other UI is open)
    /// 5. Gameplay (default)
    pub fn current(
        inventory_open: &InventoryOpen,
        interacting_machine: &InteractingMachine,
        command_state: &CommandInputState,
        cursor_state: &CursorLockState,
    ) -> Self {
        // Command input takes highest priority
        if command_state.open {
            InputState::Command
        } else if inventory_open.0 {
            InputState::Inventory
        } else if interacting_machine.0.is_some() {
            InputState::MachineUI
        } else if cursor_state.paused {
            // Only show paused if no other UI is open
            InputState::Paused
        } else {
            InputState::Gameplay
        }
    }

    /// Check if player movement is allowed
    #[allow(dead_code)]
    pub fn allows_movement(self) -> bool {
        matches!(self, InputState::Gameplay)
    }

    /// Check if player camera movement is allowed
    #[allow(dead_code)]
    pub fn allows_camera(self) -> bool {
        matches!(self, InputState::Gameplay)
    }

    /// Check if block operations (break/place) are allowed
    #[allow(dead_code)]
    pub fn allows_block_ops(self) -> bool {
        matches!(self, InputState::Gameplay)
    }

    /// Check if hotbar scrolling is allowed
    #[allow(dead_code)]
    pub fn allows_hotbar_scroll(self) -> bool {
        matches!(self, InputState::Gameplay)
    }

    /// Check if block break/place should be active
    pub fn allows_block_actions(self) -> bool {
        matches!(self, InputState::Gameplay)
    }

    /// Check if hotbar selection (1-9, wheel) should be active
    pub fn allows_hotbar(self) -> bool {
        matches!(self, InputState::Gameplay)
    }
}

/// SystemParam for reading all input state resources
#[derive(SystemParam)]
pub struct InputStateResources<'w> {
    pub inventory_open: Res<'w, InventoryOpen>,
    pub interacting_machine: Res<'w, InteractingMachine>,
    pub command_state: Res<'w, CommandInputState>,
}

impl InputStateResources<'_> {
    /// Get state with external cursor_state (for systems that need ResMut<CursorLockState>)
    pub fn get_state_with(&self, cursor_state: &CursorLockState) -> InputState {
        InputState::current(
            &self.inventory_open,
            &self.interacting_machine,
            &self.command_state,
            cursor_state,
        )
    }
}

/// SystemParam for reading all input state resources including cursor state
#[derive(SystemParam)]
pub struct InputStateResourcesWithCursor<'w> {
    pub inventory_open: Res<'w, InventoryOpen>,
    pub interacting_machine: Res<'w, InteractingMachine>,
    pub command_state: Res<'w, CommandInputState>,
    pub cursor_state: Res<'w, CursorLockState>,
}

impl InputStateResourcesWithCursor<'_> {
    pub fn get_state(&self) -> InputState {
        InputState::current(
            &self.inventory_open,
            &self.interacting_machine,
            &self.command_state,
            &self.cursor_state,
        )
    }
}

// === Events ===

/// Event to trigger a save operation
#[derive(Event)]
pub struct SaveGameEvent {
    /// Filename to save to (without extension)
    pub filename: String,
}

/// Event to trigger a load operation
#[derive(Event)]
pub struct LoadGameEvent {
    /// Filename to load from (without extension)
    pub filename: String,
}
