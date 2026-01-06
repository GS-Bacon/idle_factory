//! Miner systems: mining, UI interaction, output to conveyor

use crate::components::{
    CommandInputState, CursorLockState, InteractingCrusher, InteractingFurnace, InteractingMiner,
    InventoryOpen, MinerBufferButton, MinerBufferCountText, MinerClearButton, MinerUI,
    PlayerCamera,
};
use crate::player::Inventory;
use crate::world::{mining_random, BiomeMap};
use crate::{Conveyor, Crusher, Furnace, Miner, MINE_TIME, REACH_DISTANCE};
use bevy::prelude::*;

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
    mut cursor_state: ResMut<CursorLockState>,
) {
    use super::interaction::{
        can_interact, close_machine_ui, get_close_key_pressed, is_cursor_locked, open_machine_ui,
        raycast_closest_machine,
    };

    // Don't open miner if other UI is open
    if !can_interact(&inventory_open, &command_state, &cursor_state)
        || interacting_furnace.0.is_some()
        || interacting_crusher.0.is_some()
    {
        return;
    }

    let (e_pressed, esc_pressed) = get_close_key_pressed(&key_input);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        close_machine_ui::<MinerUI>(
            esc_pressed,
            &mut miner_ui_query,
            &mut windows,
            &mut cursor_state,
        );
        return;
    }

    // Only open miner UI with right-click when cursor is locked
    if !mouse_button.just_pressed(MouseButton::Right) || !is_cursor_locked(&windows) {
        return;
    }

    // Find closest miner and open UI
    if let Some(result) = raycast_closest_machine(&camera_query, &miner_query, REACH_DISTANCE) {
        interacting.0 = Some(result.entity);
        open_machine_ui::<MinerUI>(&mut miner_ui_query, &mut windows);
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

/// Mining logic - automatically mine based on biome at miner's position
///
/// Miners now produce resources based on the biome they're placed in,
/// not the block below them. This allows infinite mining with varied output.
pub fn miner_mining(time: Res<Time>, mut miner_query: Query<&mut Miner>, biome_map: Res<BiomeMap>) {
    for mut miner in miner_query.iter_mut() {
        // Skip if buffer is full (max 64)
        if let Some((_, count)) = miner.buffer {
            if count >= 64 {
                continue;
            }
        }

        // Get biome at miner's position
        let biome = biome_map.get_biome(miner.position);

        // Check if mining is possible in this biome
        if !biome_map.can_mine(miner.position) {
            miner.progress = 0.0;
            continue;
        }

        // Mine progress
        miner.progress += time.delta_secs() / MINE_TIME;

        if miner.progress >= 1.0 {
            miner.progress = 0.0;
            miner.tick_count = miner.tick_count.wrapping_add(1);

            // Sample resource from biome's probability table
            let random_value = mining_random(miner.position, miner.tick_count, biome_map.seed);
            let Some(resource_type) = biome.sample_resource(random_value) else {
                continue; // Unmailable biome
            };

            // Add to buffer (stack same type, or start new stack)
            if let Some((buf_type, ref mut count)) = miner.buffer {
                if buf_type == resource_type {
                    *count += 1;
                } else {
                    // Different type - only replace if buffer is empty (shouldn't happen normally)
                    // In practice, we should wait for buffer to empty
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

/// Output from miner to conveyor or machine in facing direction only
pub fn miner_output(
    mut miner_query: Query<&mut Miner>,
    mut conveyor_query: Query<&mut Conveyor>,
    mut furnace_query: Query<&mut Furnace>,
    mut crusher_query: Query<&mut Crusher>,
) {
    use super::output::transfer_output;

    for mut miner in miner_query.iter_mut() {
        let Some((block_type, count)) = miner.buffer else {
            continue;
        };
        if count == 0 {
            continue;
        }

        // Use common transfer logic
        let transferred = transfer_output(
            miner.position,
            miner.facing,
            block_type,
            &mut conveyor_query,
            &mut furnace_query,
            &mut crusher_query,
        );

        // Update miner buffer if transferred
        if transferred {
            if let Some((_, ref mut buf_count)) = miner.buffer {
                *buf_count -= 1;
                if *buf_count == 0 {
                    miner.buffer = None;
                }
            }
        }
    }
}
