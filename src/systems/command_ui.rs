//! Command input UI systems

use crate::components::*;
use crate::player::Inventory;
use crate::systems::inventory_ui::set_ui_open_state;
use crate::BlockType;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use tracing::info;

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
        execute_command(&command, &mut creative_mode, &mut inventory, &mut save_events, &mut load_events);
        return;
    }

    // Backspace to delete character
    if key_input.just_pressed(KeyCode::Backspace) {
        command_state.text.pop();
    }

    // Handle character input
    for key in key_input.get_just_pressed() {
        if let Some(c) = keycode_to_char(*key, key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight)) {
            command_state.text.push(c);
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

/// Execute a command
fn execute_command(
    command: &str,
    creative_mode: &mut ResMut<CreativeMode>,
    inventory: &mut ResMut<Inventory>,
    save_events: &mut EventWriter<SaveGameEvent>,
    load_events: &mut EventWriter<LoadGameEvent>,
) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "/creative" | "creative" => {
            creative_mode.enabled = true;
            // Give all items when entering creative mode
            let all_items = [
                BlockType::Stone,
                BlockType::Grass,
                BlockType::IronOre,
                BlockType::Coal,
                BlockType::IronIngot,
                BlockType::CopperOre,
                BlockType::CopperIngot,
                BlockType::MinerBlock,
                BlockType::ConveyorBlock,
                BlockType::CrusherBlock,
            ];
            for (i, block_type) in all_items.iter().take(9).enumerate() {
                inventory.slots[i] = Some((*block_type, 64));
            }
            info!("Creative mode enabled");
        }
        "/survival" | "survival" => {
            creative_mode.enabled = false;
            info!("Survival mode enabled");
        }
        "/give" | "give" => {
            // /give <item> [count]
            if parts.len() >= 2 {
                let item_name = parts[1].to_lowercase();
                let count: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(64);

                if let Some(block_type) = parse_item_name(&item_name) {
                    inventory.add_item(block_type, count);
                    info!("Gave {} x{}", block_type.name(), count);
                }
            }
        }
        "/clear" | "clear" => {
            // Clear inventory
            for slot in inventory.slots.iter_mut() {
                *slot = None;
            }
            info!("Inventory cleared");
        }
        "/save" | "save" => {
            // /save [filename]
            let filename = parts.get(1).unwrap_or(&"quicksave").to_string();
            save_events.send(SaveGameEvent { filename });
        }
        "/load" | "load" => {
            // /load [filename]
            let filename = parts.get(1).unwrap_or(&"quicksave").to_string();
            load_events.send(LoadGameEvent { filename });
        }
        "/help" | "help" => {
            info!("Commands: /creative, /survival, /give <item> [count], /clear, /save [name], /load [name]");
        }
        _ => {
            info!("Unknown command: {}", command);
        }
    }
}

/// Parse item name to BlockType
fn parse_item_name(name: &str) -> Option<BlockType> {
    match name {
        "stone" => Some(BlockType::Stone),
        "grass" => Some(BlockType::Grass),
        "ironore" | "iron_ore" => Some(BlockType::IronOre),
        "copperore" | "copper_ore" => Some(BlockType::CopperOre),
        "coal" => Some(BlockType::Coal),
        "ironingot" | "iron_ingot" | "iron" => Some(BlockType::IronIngot),
        "copperingot" | "copper_ingot" | "copper" => Some(BlockType::CopperIngot),
        "miner" => Some(BlockType::MinerBlock),
        "conveyor" => Some(BlockType::ConveyorBlock),
        "crusher" => Some(BlockType::CrusherBlock),
        "furnace" => Some(BlockType::FurnaceBlock),
        _ => None,
    }
}
