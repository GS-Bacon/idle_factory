//! UI state tests

use super::common::*;

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
