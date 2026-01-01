//! Command input UI systems

use crate::components::*;
use crate::player::Inventory;
use crate::systems::inventory_ui::set_ui_open_state;
use crate::world::WorldData;
use crate::BlockType;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use tracing::info;

/// E2E test command events
#[derive(Event)]
pub struct TeleportEvent {
    pub position: Vec3,
}

#[derive(Event)]
pub struct LookEvent {
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Event)]
pub struct SetBlockEvent {
    pub position: IVec3,
    pub block_type: BlockType,
}

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

/// Event for spawning machines
pub use crate::events::SpawnMachineEvent;

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

/// Execute a command
#[allow(clippy::too_many_arguments)]
fn execute_command(
    command: &str,
    creative_mode: &mut ResMut<CreativeMode>,
    inventory: &mut ResMut<Inventory>,
    save_events: &mut EventWriter<SaveGameEvent>,
    load_events: &mut EventWriter<LoadGameEvent>,
    tp_events: &mut EventWriter<TeleportEvent>,
    look_events: &mut EventWriter<LookEvent>,
    setblock_events: &mut EventWriter<SetBlockEvent>,
    spawn_machine_events: &mut EventWriter<SpawnMachineEvent>,
) {
    info!("execute_command called with: '{}'", command);
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        info!("Command is empty, returning");
        return;
    }

    info!("Command parts: {:?}", parts);
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
            info!("Commands: /creative, /survival, /give <item> [count], /clear, /save [name], /load [name], /tp x y z, /look pitch yaw, /setblock x y z type");
        }
        "/tp" | "tp" => {
            // /tp x y z - Teleport player
            if parts.len() >= 4 {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(12.0);
                let z: f32 = parts[3].parse().unwrap_or(0.0);
                tp_events.send(TeleportEvent { position: Vec3::new(x, y, z) });
                info!("Teleporting to ({}, {}, {})", x, y, z);
            } else {
                info!("Usage: /tp x y z");
            }
        }
        "/look" | "look" => {
            // /look pitch yaw - Set camera direction (in degrees)
            if parts.len() >= 3 {
                let pitch_deg: f32 = parts[1].parse().unwrap_or(0.0);
                let yaw_deg: f32 = parts[2].parse().unwrap_or(0.0);
                let pitch = pitch_deg.to_radians();
                let yaw = yaw_deg.to_radians();
                look_events.send(LookEvent { pitch, yaw });
                info!("Looking at pitch={:.1}° yaw={:.1}°", pitch_deg, yaw_deg);
            } else {
                info!("Usage: /look pitch_deg yaw_deg");
            }
        }
        "/setblock" | "setblock" => {
            // /setblock x y z blocktype - Place a block
            if parts.len() >= 5 {
                let x: i32 = parts[1].parse().unwrap_or(0);
                let y: i32 = parts[2].parse().unwrap_or(0);
                let z: i32 = parts[3].parse().unwrap_or(0);
                let block_name = parts[4].to_lowercase();
                if let Some(block_type) = parse_item_name(&block_name) {
                    setblock_events.send(SetBlockEvent {
                        position: IVec3::new(x, y, z),
                        block_type,
                    });
                    info!("Setting block at ({}, {}, {}) to {}", x, y, z, block_type.name());
                } else {
                    info!("Unknown block type: {}", block_name);
                }
            } else {
                info!("Usage: /setblock x y z blocktype");
            }
        }
        "/spawn" | "spawn" => {
            // /spawn x y z machine [direction] - Spawn a machine entity (E2E testing)
            // direction: 0=North, 1=East, 2=South, 3=West (for conveyors)
            if parts.len() >= 5 {
                let x: i32 = parts[1].parse().unwrap_or(0);
                let y: i32 = parts[2].parse().unwrap_or(0);
                let z: i32 = parts[3].parse().unwrap_or(0);
                let machine_name = parts[4].to_lowercase();
                let direction: Option<u8> = parts.get(5).and_then(|s| s.parse().ok());

                if let Some(machine_type) = parse_item_name(&machine_name) {
                    spawn_machine_events.send(SpawnMachineEvent {
                        position: IVec3::new(x, y, z),
                        machine_type,
                        direction,
                    });
                    info!("Spawning {} at ({}, {}, {})", machine_type.name(), x, y, z);
                } else {
                    info!("Unknown machine type: {}", machine_name);
                }
            } else {
                info!("Usage: /spawn x y z machine [direction]");
            }
        }
        _ => {
            info!("Unknown command: {}", command);
        }
    }
}

/// Handle teleport events
pub fn handle_teleport_event(
    mut events: EventReader<TeleportEvent>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    for event in events.read() {
        info!("TeleportEvent received: {:?}", event.position);
        match player_query.get_single_mut() {
            Ok(mut transform) => {
                transform.translation = event.position;
                info!("Teleported to {:?}", event.position);
            }
            Err(e) => {
                info!("Failed to teleport: {:?}", e);
            }
        }
    }
}

/// Handle look events
pub fn handle_look_event(
    mut events: EventReader<LookEvent>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    for event in events.read() {
        info!("LookEvent received: pitch={:.2} yaw={:.2}", event.pitch, event.yaw);
        match camera_query.get_single_mut() {
            Ok((mut camera_transform, mut camera)) => {
                camera.pitch = event.pitch;
                camera.yaw = event.yaw;
                // Apply rotation immediately to Transform
                camera_transform.rotation = Quat::from_rotation_x(camera.pitch);
                // Also update player yaw
                if let Ok(mut player_transform) = player_query.get_single_mut() {
                    player_transform.rotation = Quat::from_rotation_y(camera.yaw);
                }
                info!("Camera updated: pitch={:.2} yaw={:.2}", event.pitch, event.yaw);
            }
            Err(e) => {
                info!("Failed to get camera: {:?}", e);
            }
        }
    }
}

/// Handle setblock events
pub fn handle_setblock_event(
    mut events: EventReader<SetBlockEvent>,
    mut world_data: ResMut<WorldData>,
) {
    for event in events.read() {
        world_data.set_block(event.position, event.block_type);
        info!("Set block at {:?} to {:?}", event.position, event.block_type);
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

use crate::{
    Conveyor, ConveyorShape, ConveyorVisual, Crusher, Direction, Furnace, Miner,
    MachineModels, BLOCK_SIZE,
};

/// Handle spawn machine events - creates machine entities directly (for E2E testing)
#[allow(clippy::too_many_arguments)]
pub fn handle_spawn_machine_event(
    mut commands: Commands,
    mut events: EventReader<SpawnMachineEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    machine_models: Res<MachineModels>,
) {
    for event in events.read() {
        let pos = event.position;
        let world_pos = Vec3::new(
            pos.x as f32 * BLOCK_SIZE + 0.5,
            pos.y as f32 * BLOCK_SIZE + 0.5,
            pos.z as f32 * BLOCK_SIZE + 0.5,
        );

        match event.machine_type {
            BlockType::ConveyorBlock => {
                // Direction from event or default to North
                let direction = match event.direction.unwrap_or(0) {
                    0 => Direction::North,
                    1 => Direction::East,
                    2 => Direction::South,
                    3 => Direction::West,
                    _ => Direction::North,
                };

                let conveyor_pos = Vec3::new(
                    pos.x as f32 * BLOCK_SIZE + 0.5,
                    pos.y as f32 * BLOCK_SIZE, // Conveyor sits on top of block
                    pos.z as f32 * BLOCK_SIZE + 0.5,
                );

                if let Some(model_handle) = machine_models.get_conveyor_model(ConveyorShape::Straight) {
                    commands.spawn((
                        SceneRoot(model_handle),
                        Transform::from_translation(conveyor_pos)
                            .with_rotation(direction.to_rotation()),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Conveyor {
                            position: pos,
                            direction,
                            items: Vec::new(),
                            last_output_index: 0,
                            last_input_source: 0,
                            shape: ConveyorShape::Straight,
                        },
                        ConveyorVisual,
                    ));
                } else {
                    // Fallback to procedural mesh
                    let mesh = meshes.add(Cuboid::new(BLOCK_SIZE * 0.9, BLOCK_SIZE * 0.15, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: BlockType::ConveyorBlock.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::from_translation(conveyor_pos)
                            .with_rotation(direction.to_rotation()),
                        Conveyor {
                            position: pos,
                            direction,
                            items: Vec::new(),
                            last_output_index: 0,
                            last_input_source: 0,
                            shape: ConveyorShape::Straight,
                        },
                        ConveyorVisual,
                    ));
                }
                info!("Spawned conveyor at {:?} facing {:?}", pos, direction);
            }
            BlockType::MinerBlock => {
                let transform = Transform::from_translation(world_pos);

                if let Some(model) = machine_models.miner.clone() {
                    commands.spawn((
                        SceneRoot(model),
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Miner {
                            position: pos,
                            ..default()
                        },
                    ));
                } else {
                    let mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: BlockType::MinerBlock.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        transform,
                        Miner {
                            position: pos,
                            ..default()
                        },
                    ));
                }
                info!("Spawned miner at {:?}", pos);
            }
            BlockType::FurnaceBlock => {
                let transform = Transform::from_translation(world_pos);

                if let Some(model) = machine_models.furnace.clone() {
                    commands.spawn((
                        SceneRoot(model),
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Furnace::default(),
                    ));
                } else {
                    let mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: BlockType::FurnaceBlock.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        transform,
                        Furnace::default(),
                    ));
                }
                info!("Spawned furnace at {:?}", pos);
            }
            BlockType::CrusherBlock => {
                let transform = Transform::from_translation(world_pos);

                if let Some(model) = machine_models.crusher.clone() {
                    commands.spawn((
                        SceneRoot(model),
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Crusher {
                            position: pos,
                            input_type: None,
                            input_count: 0,
                            output_type: None,
                            output_count: 0,
                            progress: 0.0,
                        },
                    ));
                } else {
                    let mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: BlockType::CrusherBlock.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        transform,
                        Crusher {
                            position: pos,
                            input_type: None,
                            input_count: 0,
                            output_type: None,
                            output_count: 0,
                            progress: 0.0,
                        },
                    ));
                }
                info!("Spawned crusher at {:?}", pos);
            }
            _ => {
                info!("Cannot spawn {:?} as machine", event.machine_type);
            }
        }
    }
}
