//! Command input UI systems
//!
//! Handles the command input text box:
//! - Toggle visibility with T or /
//! - Text input handling
//! - Command execution on Enter
//! - Command suggestions/autocomplete with Tab

use crate::components::*;
use crate::events::SpawnMachineEvent;
use crate::player::{LocalPlayer, PlayerInventory};
use crate::systems::cursor;
use crate::{GlobalInventoryOpen, InteractingMachine};
use bevy::prelude::*;

use super::executor::execute_command;
use super::{
    AssertMachineEvent, DebugEvent, LookEvent, ScreenshotEvent, SetBlockEvent, TeleportEvent,
};

/// Get matching command suggestions for the current input
fn get_suggestions(input: &str) -> Vec<&'static str> {
    if input.is_empty() {
        return Vec::new();
    }
    let input_lower = input.to_lowercase();
    COMMAND_SUGGESTIONS
        .iter()
        .filter(|cmd| cmd.to_lowercase().starts_with(&input_lower))
        .copied()
        .take(5)
        .collect()
}

/// Toggle command input with T or / key
#[allow(clippy::too_many_arguments)]
pub fn command_input_toggle(
    key_input: Res<ButtonInput<KeyCode>>,
    mut command_state: ResMut<CommandInputState>,
    mut ui_query: Query<&mut Visibility, With<CommandInputUI>>,
    mut text_query: Query<&mut Text, With<CommandInputText>>,
    mut windows: Query<&mut Window>,
    interacting_machine: Res<InteractingMachine>,
    inventory_open: Res<InventoryOpen>,
    global_inv_open: Res<GlobalInventoryOpen>,
) {
    // Don't open if other UI is open
    if interacting_machine.0.is_some() || inventory_open.0 || global_inv_open.0 {
        return;
    }

    // T or / to open command input (when not already open)
    if !command_state.open
        && (key_input.just_pressed(KeyCode::KeyT) || key_input.just_pressed(KeyCode::Slash))
    {
        command_state.open = true;
        command_state.text.clear();
        command_state.skip_input_frame = true; // Skip input this frame

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
            cursor::unlock_cursor(&mut window);
        }
    }
}

/// Handle command input text entry
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn command_input_handler(
    key_input: Res<ButtonInput<KeyCode>>,
    mut command_state: ResMut<CommandInputState>,
    mut ui_query: Query<
        (Option<&mut Visibility>, Option<&mut Text>),
        Or<(With<CommandInputUI>, With<CommandInputText>)>,
    >,
    mut windows: Query<&mut Window>,
    mut creative_mode: ResMut<CreativeMode>,
    local_player: Option<Res<LocalPlayer>>,
    mut inventory_query: Query<&mut PlayerInventory>,
    mut save_events: EventWriter<SaveGameEvent>,
    mut load_events: EventWriter<LoadGameEvent>,
    mut tp_events: EventWriter<TeleportEvent>,
    mut look_events: EventWriter<LookEvent>,
    mut setblock_events: EventWriter<SetBlockEvent>,
    mut spawn_machine_events: EventWriter<SpawnMachineEvent>,
    mut debug_events: EventWriter<DebugEvent>,
    mut assert_machine_events: EventWriter<AssertMachineEvent>,
    mut screenshot_events: EventWriter<ScreenshotEvent>,
) {
    if !command_state.open {
        return;
    }

    // ESC to close without executing
    if key_input.just_pressed(KeyCode::Escape) {
        command_state.open = false;
        command_state.text.clear();
        command_state.suggestion_index = 0;

        for (vis, _) in ui_query.iter_mut() {
            if let Some(mut vis) = vis {
                *vis = Visibility::Hidden;
            }
        }

        // Lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            cursor::lock_cursor(&mut window);
        }
        return;
    }

    // Tab to autocomplete from suggestions
    if key_input.just_pressed(KeyCode::Tab) {
        let suggestions = get_suggestions(&command_state.text);
        if !suggestions.is_empty() {
            let idx = command_state.suggestion_index % suggestions.len();
            command_state.text = suggestions[idx].to_string();
            command_state.suggestion_index = (idx + 1) % suggestions.len();
        }
    }

    // Enter to execute command
    if key_input.just_pressed(KeyCode::Enter) {
        let command = command_state.text.clone();
        info!("Command executed: '{}'", command);
        command_state.open = false;
        command_state.text.clear();
        command_state.suggestion_index = 0;

        for (vis, _) in ui_query.iter_mut() {
            if let Some(mut vis) = vis {
                *vis = Visibility::Hidden;
            }
        }

        // Lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            cursor::lock_cursor(&mut window);
        }

        // Execute command (requires player inventory)
        let Some(local_player) = local_player.as_ref() else {
            return;
        };
        let Ok(mut inventory) = inventory_query.get_mut(local_player.0) else {
            return;
        };
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
            &mut debug_events,
            &mut assert_machine_events,
            &mut screenshot_events,
        );
        return;
    }

    // Backspace to delete character
    if key_input.just_pressed(KeyCode::Backspace) {
        command_state.text.pop();
        command_state.suggestion_index = 0;
    }

    // Handle character input (skip if just opened to avoid T/slash being added)
    if command_state.skip_input_frame {
        command_state.skip_input_frame = false;
    } else {
        for key in key_input.get_just_pressed() {
            if *key == KeyCode::Tab {
                continue; // Skip tab, handled above
            }
            if let Some(c) = keycode_to_char(
                *key,
                key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight),
            ) {
                command_state.text.push(c);
                command_state.suggestion_index = 0;
            }
        }
    }

    // Update display text
    for (_, text) in ui_query.iter_mut() {
        if let Some(mut text) = text {
            text.0 = format!("> {}|", command_state.text);
        }
    }
}

/// Update command suggestions display (separate system to reduce parameter count)
#[allow(clippy::type_complexity)]
pub fn update_command_suggestions(
    command_state: Res<CommandInputState>,
    mut suggestions_ui_query: Query<
        &mut Visibility,
        (With<CommandSuggestionsUI>, Without<CommandInputUI>),
    >,
    mut suggestion_text_query: Query<
        (
            &CommandSuggestionText,
            &mut Text,
            &mut TextColor,
            &mut Visibility,
        ),
        (
            Without<CommandInputText>,
            Without<CommandSuggestionsUI>,
            Without<CommandInputUI>,
        ),
    >,
) {
    if !command_state.open {
        // Hide everything when closed
        for mut vis in suggestions_ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        for (_, _, _, mut vis) in suggestion_text_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    let suggestions = get_suggestions(&command_state.text);
    let has_suggestions = !suggestions.is_empty();

    // Show/hide suggestions container
    for mut vis in suggestions_ui_query.iter_mut() {
        *vis = if has_suggestions {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Update suggestion text slots
    for (suggestion_slot, mut text, mut color, mut vis) in suggestion_text_query.iter_mut() {
        let idx = suggestion_slot.0;
        if idx < suggestions.len() {
            text.0 = suggestions[idx].to_string();
            // Highlight selected suggestion
            *color = if idx == command_state.suggestion_index % suggestions.len().max(1) {
                TextColor(Color::srgb(1.0, 1.0, 0.5))
            } else {
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0))
            };
            *vis = Visibility::Visible;
        } else {
            text.0.clear();
            *vis = Visibility::Hidden;
        }
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
