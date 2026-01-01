//! Command input UI systems
//!
//! Handles the command input text box:
//! - Toggle visibility with T or /
//! - Text input handling
//! - Command execution on Enter

use crate::components::*;
use crate::player::Inventory;
use crate::systems::inventory_ui::set_ui_open_state;
use crate::events::SpawnMachineEvent;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use super::executor::execute_command;
use super::{TeleportEvent, LookEvent, SetBlockEvent, DebugConveyorEvent};

/// Toggle command input with T or / key
#[allow(clippy::too_many_arguments)]
pub fn command_input_toggle(
    key_input: Res<ButtonInput<KeyCode>>,
    mut command_state: ResMut<CommandInputState>,
    mut ui_query: Query<&mut Visibility, With<CommandInputUI>>,
    mut text_query: Query<&mut Text, With<CommandInputText>>,
    mut windows: Query<&mut Window>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    creative_inv_open: Res<InventoryOpen>,
) {
    // Don't open if other UI is open
    if interacting_furnace.0.is_some() || interacting_crusher.0.is_some() || creative_inv_open.0 {
        return;
    }

    // T or / to open command input (when not already open)
    if !command_state.open
        && (key_input.just_pressed(KeyCode::KeyT) || key_input.just_pressed(KeyCode::Slash))
    {
        command_state.open = true;
        command_state.text.clear();
        command_state.skip_input_frame = true;  // Skip input this frame

        // Start with / if opened with slash key
        if key_input.just_pressed(KeyCode::Slash) {
            command_state.text.push('/');
        }

        // Show UI
        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Visible;
        }

        // Reset text display
        for mut text in text_query.iter_mut() {
            text.0 = format!("> {}|", command_state.text);
        }

        // Unlock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(true);
        }
    }
}

/// Handle command input text entry
#[allow(clippy::too_many_arguments)]
pub fn command_input_handler(
    key_input: Res<ButtonInput<KeyCode>>,
    mut command_state: ResMut<CommandInputState>,
    mut ui_query: Query<&mut Visibility, With<CommandInputUI>>,
    mut text_query: Query<&mut Text, With<CommandInputText>>,
    mut windows: Query<&mut Window>,
    mut creative_mode: ResMut<CreativeMode>,
    mut inventory: ResMut<Inventory>,
    mut save_events: EventWriter<SaveGameEvent>,
    mut load_events: EventWriter<LoadGameEvent>,
    mut tp_events: EventWriter<TeleportEvent>,
    mut look_events: EventWriter<LookEvent>,
    mut setblock_events: EventWriter<SetBlockEvent>,
    mut spawn_machine_events: EventWriter<SpawnMachineEvent>,
    mut debug_conveyor_events: EventWriter<DebugConveyorEvent>,
) {
    if !command_state.open {
        return;
    }

    // ESC to close without executing
    if key_input.just_pressed(KeyCode::Escape) {
        command_state.open = false;
        command_state.text.clear();

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Enter to execute command
    if key_input.just_pressed(KeyCode::Enter) {
        let command = command_state.text.clone();
        println!(">>> ENTER pressed, command: '{}'", command);
        command_state.open = false;
        command_state.text.clear();

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }

        // Execute command
        execute_command(
            &command,
            &mut creative_mode,
            &mut inventory,
            &mut save_events,
            &mut load_events,
            &mut tp_events,
            &mut look_events,
            &mut setblock_events,
            &mut spawn_machine_events,
            &mut debug_conveyor_events,
        );
        return;
    }

    // Backspace to delete character
    if key_input.just_pressed(KeyCode::Backspace) {
        command_state.text.pop();
    }

    // Handle character input (skip if just opened to avoid T/slash being added)
    if command_state.skip_input_frame {
        command_state.skip_input_frame = false;
    } else {
        for key in key_input.get_just_pressed() {
            if let Some(c) = keycode_to_char(*key, key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight)) {
                command_state.text.push(c);
            }
        }
    }

    // Update display text
    for mut text in text_query.iter_mut() {
        text.0 = format!("> {}|", command_state.text);
    }
}

/// Convert key code to character
fn keycode_to_char(key_code: KeyCode, shift: bool) -> Option<char> {
    match key_code {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some(if shift { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift { '(' } else { '9' }),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Equal => Some(if shift { '+' } else { '=' }),
        KeyCode::Slash => Some(if shift { '?' } else { '/' }),
        _ => None,
    }
}
