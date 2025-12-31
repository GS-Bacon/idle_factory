//! Miner systems: mining, UI interaction, output to conveyor

use crate::components::{
    CommandInputState, CursorLockState, InteractingCrusher, InteractingFurnace, InteractingMiner,
    InventoryOpen, MinerBufferButton, MinerBufferCountText, MinerClearButton, MinerUI,
    PlayerCamera,
};
use crate::player::Inventory;
use crate::world::WorldData;
use crate::{ray_aabb_intersection, set_ui_open_state, BlockType, Conveyor, Miner, BLOCK_SIZE, MINE_TIME, REACH_DISTANCE};
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

/// Handle miner right-click interaction (open/close UI)
#[allow(clippy::too_many_arguments)]
pub fn miner_interact(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    miner_query: Query<(Entity, &Transform), With<Miner>>,
    mut interacting: ResMut<InteractingMiner>,
    inventory_open: Res<InventoryOpen>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    mut miner_ui_query: Query<&mut Visibility, With<MinerUI>>,
    mut windows: Query<&mut Window>,
    command_state: Res<CommandInputState>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't open miner if other UI is open
    if inventory_open.0
        || interacting_furnace.0.is_some()
        || interacting_crusher.0.is_some()
        || command_state.open
        || cursor_state.paused
    {
        return;
    }

    let e_pressed = key_input.just_pressed(KeyCode::KeyE);
    let esc_pressed = key_input.just_pressed(KeyCode::Escape);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        if let Ok(mut vis) = miner_ui_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        let mut window = windows.single_mut();
        if esc_pressed {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(false);
        } else {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Only open miner UI with right-click
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;
    if !cursor_locked {
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Find closest miner intersection
    let mut closest_miner: Option<(Entity, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    for (entity, miner_transform) in miner_query.iter() {
        let miner_pos = miner_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            miner_pos - Vec3::splat(half_size),
            miner_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_miner.is_none_or(|f| t < f.1);
                if is_closer {
                    closest_miner = Some((entity, t));
                }
            }
        }
    }

    // Open miner UI
    if let Some((entity, _)) = closest_miner {
        interacting.0 = Some(entity);
        if let Ok(mut vis) = miner_ui_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
        // Unlock cursor for UI interaction
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        set_ui_open_state(true);
    }
}

/// Handle miner UI button clicks (take buffer, discard buffer)
#[allow(clippy::type_complexity)]
pub fn miner_ui_input(
    interacting: Res<InteractingMiner>,
    mut miner_query: Query<&mut Miner>,
    mut inventory: ResMut<Inventory>,
    mut buffer_btn_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (With<MinerBufferButton>, Changed<Interaction>),
    >,
    mut clear_btn_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            With<MinerClearButton>,
            Without<MinerBufferButton>,
            Changed<Interaction>,
        ),
    >,
) {
    let Some(miner_entity) = interacting.0 else {
        return;
    };

    let Ok(mut miner) = miner_query.get_mut(miner_entity) else {
        return;
    };

    // Buffer button (take all items)
    for (interaction, mut bg_color, mut border_color) in buffer_btn_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Take all items from buffer to inventory
                if let Some((block_type, count)) = miner.buffer.take() {
                    inventory.add_item(block_type, count);
                }
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.5, 0.45));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(Color::srgb(0.5, 0.4, 0.35));
            }
        }
    }

    // Clear button (discard buffer)
    for (interaction, mut bg_color, mut border_color) in clear_btn_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Discard buffer contents
                miner.buffer = None;
                *border_color = BorderColor(Color::srgb(1.0, 0.3, 0.3));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(1.0, 0.5, 0.5));
                *bg_color = BackgroundColor(Color::srgb(0.7, 0.3, 0.3));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.8, 0.3, 0.3, 1.0));
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.2, 0.2));
            }
        }
    }
}

/// Update miner UI buffer count display
pub fn update_miner_ui(
    interacting: Res<InteractingMiner>,
    miner_query: Query<&Miner>,
    mut text_query: Query<&mut Text, With<MinerBufferCountText>>,
) {
    let Some(miner_entity) = interacting.0 else {
        return;
    };

    let Ok(miner) = miner_query.get(miner_entity) else {
        return;
    };

    // Update buffer count text
    for mut text in text_query.iter_mut() {
        if let Some((block_type, count)) = &miner.buffer {
            **text = format!("{}\n{}", block_type.name(), count);
        } else {
            **text = "Empty".to_string();
        }
    }
}

/// Mining logic - automatically mine blocks below the miner
pub fn miner_mining(time: Res<Time>, mut miner_query: Query<&mut Miner>, world_data: Res<WorldData>) {
    for mut miner in miner_query.iter_mut() {
        // Skip if buffer is full
        if let Some((_, count)) = miner.buffer {
            if count >= 64 {
                continue;
            }
        }

        // Find block below miner to determine resource type
        let below_pos = miner.position + IVec3::new(0, -1, 0);
        let Some(&block_type) = world_data.get_block(below_pos) else {
            miner.progress = 0.0;
            continue;
        };

        // Only mine resource blocks (not grass/stone)
        let resource_type = match block_type {
            BlockType::IronOre => BlockType::IronOre,
            BlockType::Coal => BlockType::Coal,
            BlockType::CopperOre => BlockType::CopperOre,
            BlockType::Stone => BlockType::Stone,
            _ => {
                // Can't mine this block type, skip
                miner.progress = 0.0;
                continue;
            }
        };

        // Mine progress
        miner.progress += time.delta_secs() / MINE_TIME;

        if miner.progress >= 1.0 {
            miner.progress = 0.0;

            // Generate resource infinitely (don't remove block)
            // Add to buffer
            if let Some((buf_type, ref mut count)) = miner.buffer {
                if buf_type == resource_type {
                    *count += 1;
                }
            } else {
                miner.buffer = Some((resource_type, 1));
            }
        }
    }
}

/// Visual feedback for miner activity (pulse scale when mining)
pub fn miner_visual_feedback(time: Res<Time>, mut miner_query: Query<(&Miner, &mut Transform)>) {
    for (miner, mut transform) in miner_query.iter_mut() {
        // If progress > 0, the miner is actively mining
        if miner.progress > 0.0 {
            // Pulse effect: scale between 0.95 and 1.05 based on progress
            let pulse = 1.0 + 0.05 * (miner.progress * std::f32::consts::TAU * 2.0).sin();
            transform.scale = Vec3::splat(pulse);
        } else if miner.buffer.is_some() {
            // Buffer full but not mining: slight glow/scale up
            let pulse = 1.0 + 0.02 * (time.elapsed_secs() * 3.0).sin();
            transform.scale = Vec3::splat(pulse);
        } else {
            // Idle: reset scale
            transform.scale = Vec3::ONE;
        }
    }
}

/// Output from miner to adjacent conveyor
pub fn miner_output(
    mut miner_query: Query<&mut Miner>,
    mut conveyor_query: Query<&mut Conveyor>,
) {
    for mut miner in miner_query.iter_mut() {
        let Some((block_type, count)) = miner.buffer else {
            continue;
        };
        if count == 0 {
            continue;
        }

        // Check for adjacent conveyors (horizontal + above)
        let adjacent_positions = [
            miner.position + IVec3::new(1, 0, 0),  // east
            miner.position + IVec3::new(-1, 0, 0), // west
            miner.position + IVec3::new(0, 0, 1),  // south
            miner.position + IVec3::new(0, 0, -1), // north
            miner.position + IVec3::new(0, 1, 0),  // above
        ];

        'outer: for mut conveyor in conveyor_query.iter_mut() {
            for pos in &adjacent_positions {
                if conveyor.position == *pos {
                    // Use get_join_progress to determine if item can join and at what progress
                    // For above conveyor, allow joining at entry (0.0) for any direction
                    let join_progress = if *pos == miner.position + IVec3::new(0, 1, 0) {
                        Some(0.0) // Above: always join at entry
                    } else {
                        conveyor.get_join_progress(miner.position)
                    };

                    if let Some(progress) = join_progress {
                        if conveyor.can_accept_item(progress) {
                            conveyor.add_item(block_type, progress);
                            if let Some((_, ref mut buf_count)) = miner.buffer {
                                *buf_count -= 1;
                                if *buf_count == 0 {
                                    miner.buffer = None;
                                }
                            }
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
}
