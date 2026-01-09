//! UI state tests

#[allow(unused_imports)]
use super::common::*;

// ============================================================================
// Basic UI Toggle Tests
// ============================================================================

#[test]
fn test_ui_toggle_with_e_key() {
    let mut ui_state = TestUIState::default();

    assert!(!ui_state.furnace_ui_open);

    ui_state.toggle_furnace_ui();
    assert!(ui_state.furnace_ui_open);

    ui_state.toggle_furnace_ui();
    assert!(!ui_state.furnace_ui_open);
}

#[test]
fn test_ui_close_with_esc() {
    let mut ui_state = TestUIState::default();

    ui_state.toggle_furnace_ui();
    assert!(ui_state.furnace_ui_open);

    ui_state.close_ui();
    assert!(!ui_state.furnace_ui_open);
}

// ============================================================================
// Extended UI State Tests
// ============================================================================

/// All UI modes that can be open (matches InputState in game)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UIMode {
    #[default]
    Gameplay,
    Inventory,
    MachineUI,
    Command,
    Paused,
}

/// Extended UI state for comprehensive testing
#[derive(Default)]
pub struct ExtendedUIState {
    pub mode: UIMode,
    pub cursor_locked: bool,
}

impl ExtendedUIState {
    pub fn new() -> Self {
        Self {
            mode: UIMode::Gameplay,
            cursor_locked: true,
        }
    }

    pub fn toggle_inventory(&mut self) {
        match self.mode {
            UIMode::Gameplay => {
                self.mode = UIMode::Inventory;
                self.cursor_locked = false;
            }
            UIMode::Inventory => {
                self.mode = UIMode::Gameplay;
                self.cursor_locked = true;
            }
            _ => {} // Other UIs take priority
        }
    }

    pub fn press_esc(&mut self) {
        match self.mode {
            UIMode::Gameplay => {
                self.mode = UIMode::Paused;
                self.cursor_locked = false;
            }
            UIMode::Paused => {
                self.mode = UIMode::Gameplay;
                self.cursor_locked = true;
            }
            UIMode::Inventory | UIMode::Command | UIMode::MachineUI => {
                self.mode = UIMode::Gameplay;
                self.cursor_locked = true;
            }
        }
    }

    pub fn open_command(&mut self) {
        if self.mode == UIMode::Gameplay {
            self.mode = UIMode::Command;
            self.cursor_locked = false;
        }
    }

    pub fn open_machine(&mut self) {
        if self.mode == UIMode::Gameplay {
            self.mode = UIMode::MachineUI;
            self.cursor_locked = false;
        }
    }
}

#[test]
fn test_extended_ui_initial_state() {
    let ui = ExtendedUIState::new();
    assert_eq!(ui.mode, UIMode::Gameplay);
    assert!(ui.cursor_locked);
}

#[test]
fn test_extended_ui_inventory_toggle() {
    let mut ui = ExtendedUIState::new();

    // Open inventory
    ui.toggle_inventory();
    assert_eq!(ui.mode, UIMode::Inventory);
    assert!(!ui.cursor_locked);

    // Close inventory
    ui.toggle_inventory();
    assert_eq!(ui.mode, UIMode::Gameplay);
    assert!(ui.cursor_locked);
}

#[test]
fn test_extended_ui_esc_closes_all() {
    let mut ui = ExtendedUIState::new();

    // Test ESC closes inventory
    ui.toggle_inventory();
    ui.press_esc();
    assert_eq!(ui.mode, UIMode::Gameplay);
    assert!(ui.cursor_locked);

    // Test ESC closes command
    ui.open_command();
    ui.press_esc();
    assert_eq!(ui.mode, UIMode::Gameplay);
    assert!(ui.cursor_locked);

    // Test ESC closes machine UI
    ui.open_machine();
    ui.press_esc();
    assert_eq!(ui.mode, UIMode::Gameplay);
    assert!(ui.cursor_locked);
}

#[test]
fn test_extended_ui_pause_menu_toggle() {
    let mut ui = ExtendedUIState::new();

    // Open pause menu
    ui.press_esc();
    assert_eq!(ui.mode, UIMode::Paused);
    assert!(!ui.cursor_locked);

    // Close pause menu
    ui.press_esc();
    assert_eq!(ui.mode, UIMode::Gameplay);
    assert!(ui.cursor_locked);
}

#[test]
fn test_extended_ui_no_stacking() {
    let mut ui = ExtendedUIState::new();

    // Open inventory
    ui.toggle_inventory();
    assert_eq!(ui.mode, UIMode::Inventory);

    // Try to open command while inventory is open - should not change
    ui.open_command();
    assert_eq!(ui.mode, UIMode::Inventory);

    // ESC should close inventory
    ui.press_esc();
    assert_eq!(ui.mode, UIMode::Gameplay);
}

#[test]
fn test_extended_ui_cursor_state_consistency() {
    let mut ui = ExtendedUIState::new();

    // Gameplay = cursor locked
    assert!(ui.cursor_locked);

    // Any UI open = cursor unlocked
    ui.toggle_inventory();
    assert!(!ui.cursor_locked);
    ui.press_esc();

    ui.open_command();
    assert!(!ui.cursor_locked);
    ui.press_esc();

    ui.press_esc(); // Pause menu
    assert!(!ui.cursor_locked);
    ui.press_esc();

    // Back to gameplay
    assert!(ui.cursor_locked);
}
