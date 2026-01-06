//! Crusher systems: processing, UI interaction, output to conveyor

use crate::components::{
    CommandInputState, CrusherProgressBar, CrusherSlotButton, CrusherSlotCount, CrusherUI,
    CursorLockState, InteractingCrusher, InteractingFurnace, InventoryOpen, MachineSlotType,
    PlayerCamera,
};
use crate::game_spec::{find_recipe, MachineType};
use crate::player::Inventory;
use crate::systems::set_ui_open_state;
use crate::utils::ray_aabb_intersection;
use crate::{BlockType, Conveyor, Crusher, Furnace, BLOCK_SIZE, REACH_DISTANCE};
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

/// Crusher processing - converts ore to dust (2x output per recipe)
pub fn crusher_processing(time: Res<Time>, mut crusher_query: Query<&mut Crusher>) {
    for mut crusher in crusher_query.iter_mut() {
        // Need input ore to process
        let Some(input_ore) = crusher.input_type else {
            crusher.progress = 0.0;
            continue;
        };

        if crusher.input_count == 0 {
            crusher.progress = 0.0;
            continue;
        }

        // Get recipe (Single Source of Truth for craft_time and outputs)
        let Some(recipe) = find_recipe(MachineType::Crusher, input_ore) else {
            crusher.progress = 0.0;
            continue;
        };

        // Get output from recipe
        let Some(output) = recipe.outputs.first() else {
            crusher.progress = 0.0;
            continue;
        };
        let output_dust = output.item;
        let output_count = output.count;

        // Check output slot compatibility (same dust type or empty, max 64)
        let output_compatible = match crusher.output_type {
            None => true,
            Some(current) => current == output_dust && crusher.output_count + output_count <= 64,
        };

        if output_compatible {
            // Use recipe's craft_time as Single Source of Truth
            crusher.progress += time.delta_secs() / recipe.craft_time;

            // When progress reaches 1.0, complete crushing
            if crusher.progress >= 1.0 {
                crusher.progress = 0.0;
                crusher.input_count -= 1;
                if crusher.input_count == 0 {
                    crusher.input_type = None;
                }
                crusher.output_type = Some(output_dust);
                crusher.output_count += output_count; // Recipe-defined output count
            }
        } else {
            crusher.progress = 0.0;
        }
    }
}

/// Handle crusher right-click interaction (open/close UI)
#[allow(clippy::too_many_arguments)]
pub fn crusher_interact(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    crusher_query: Query<(Entity, &Transform), With<Crusher>>,
    mut interacting: ResMut<InteractingCrusher>,
    inventory_open: Res<InventoryOpen>,
    interacting_furnace: Res<InteractingFurnace>,
    mut crusher_ui_query: Query<&mut Visibility, With<CrusherUI>>,
    mut windows: Query<&mut Window>,
    command_state: Res<CommandInputState>,
    mut cursor_state: ResMut<CursorLockState>,
) {
    // Don't open crusher if inventory, furnace is open, command input is active, or game is paused (input matrix: Right Click)
    if inventory_open.0
        || interacting_furnace.0.is_some()
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
        if let Ok(mut vis) = crusher_ui_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        let mut window = windows.single_mut();
        if esc_pressed {
            // ESC: Release pointer lock and show cursor
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(false);
        } else {
            // E key: Set flag to prevent inventory from opening this frame
            cursor_state.skip_inventory_toggle = true;
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Only open crusher UI with right-click
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

    // Find closest crusher intersection
    let mut closest_crusher: Option<(Entity, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    for (entity, crusher_transform) in crusher_query.iter() {
        let crusher_pos = crusher_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            crusher_pos - Vec3::splat(half_size),
            crusher_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_crusher.is_none_or(|f| t < f.1);
                if is_closer {
                    closest_crusher = Some((entity, t));
                }
            }
        }
    }

    // Open crusher UI
    if let Some((entity, _)) = closest_crusher {
        interacting.0 = Some(entity);
        if let Ok(mut vis) = crusher_ui_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
        // Unlock cursor for UI interaction
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        set_ui_open_state(true);
    }
}

/// Handle crusher slot click interactions
pub fn crusher_ui_input(
    interacting: Res<InteractingCrusher>,
    mut crusher_query: Query<&mut Crusher>,
    mut inventory: ResMut<Inventory>,
    mut slot_query: Query<
        (
            &Interaction,
            &CrusherSlotButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    let Some(crusher_entity) = interacting.0 else {
        return;
    };

    let Ok(mut crusher) = crusher_query.get_mut(crusher_entity) else {
        return;
    };

    for (interaction, slot_button, mut bg_color, mut border_color) in slot_query.iter_mut() {
        let slot_type = slot_button.0;

        match *interaction {
            Interaction::Pressed => {
                match slot_type {
                    MachineSlotType::Fuel => {
                        // Crusher has no fuel slot - do nothing
                    }
                    MachineSlotType::Input => {
                        // Add ore from inventory (prioritize iron, then copper, max 64)
                        const MAX_INPUT: u32 = 64;
                        if crusher.input_count < MAX_INPUT
                            && (crusher.input_type.is_none()
                                || crusher.input_type == Some(BlockType::IronOre))
                            && inventory.consume_item(BlockType::IronOre, 1)
                        {
                            crusher.input_type = Some(BlockType::IronOre);
                            crusher.input_count += 1;
                        } else if crusher.input_count < MAX_INPUT
                            && (crusher.input_type.is_none()
                                || crusher.input_type == Some(BlockType::CopperOre))
                            && inventory.consume_item(BlockType::CopperOre, 1)
                        {
                            crusher.input_type = Some(BlockType::CopperOre);
                            crusher.input_count += 1;
                        }
                    }
                    MachineSlotType::Output => {
                        // Take output ore to inventory
                        if crusher.output_count > 0 {
                            if let Some(output_type) = crusher.output_type {
                                crusher.output_count -= 1;
                                inventory.add_item(output_type, 1);
                                if crusher.output_count == 0 {
                                    crusher.output_type = None;
                                }
                            }
                        }
                    }
                }
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                let base = match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.5, 0.4, 0.35),
                    MachineSlotType::Input => Color::srgb(0.6, 0.5, 0.45),
                    MachineSlotType::Output => Color::srgb(0.7, 0.6, 0.55),
                };
                *bg_color = BackgroundColor(base);
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.5, 0.4, 0.35),
                    MachineSlotType::Input => Color::srgb(0.5, 0.4, 0.35),
                    MachineSlotType::Output => Color::srgb(0.6, 0.5, 0.45),
                });
            }
        }
    }
}

/// Crusher output to conveyor or machine in facing direction only
pub fn crusher_output(
    mut crusher_query: Query<&mut Crusher>,
    mut conveyor_query: Query<&mut Conveyor>,
    mut furnace_query: Query<&mut Furnace>,
) {
    use super::output::{try_transfer_to_conveyor, try_transfer_to_furnace};

    for mut crusher in crusher_query.iter_mut() {
        let Some(output_type) = crusher.output_type else {
            continue;
        };

        if crusher.output_count == 0 {
            continue;
        }

        // Output only in facing direction (front of machine)
        let output_pos = crusher.position + crusher.facing.to_ivec3();

        // Use common transfer logic
        // Priority: Conveyor > Furnace (dust goes to furnace for smelting)
        let transferred = try_transfer_to_conveyor(
            crusher.position,
            output_pos,
            output_type,
            &mut conveyor_query,
        ) || try_transfer_to_furnace(
            crusher.position,
            output_pos,
            output_type,
            &mut furnace_query,
        );

        // Update crusher output if transferred
        if transferred {
            crusher.output_count -= 1;
            if crusher.output_count == 0 {
                crusher.output_type = None;
            }
        }
    }
}

/// Update crusher UI slot counts and progress bar
pub fn update_crusher_ui(
    interacting: Res<InteractingCrusher>,
    crusher_query: Query<&Crusher>,
    mut slot_count_query: Query<(&CrusherSlotCount, &mut Text)>,
    mut progress_bar_query: Query<&mut Node, With<CrusherProgressBar>>,
) {
    let Some(crusher_entity) = interacting.0 else {
        return;
    };

    let Ok(crusher) = crusher_query.get(crusher_entity) else {
        return;
    };

    // Update slot counts
    for (slot_count, mut text) in slot_count_query.iter_mut() {
        **text = match slot_count.0 {
            MachineSlotType::Fuel => String::new(), // Crusher has no fuel
            MachineSlotType::Input => format!("{}", crusher.input_count),
            MachineSlotType::Output => format!("{}", crusher.output_count),
        };
    }

    // Update progress bar
    for mut node in progress_bar_query.iter_mut() {
        node.width = Val::Percent(crusher.progress * 100.0);
    }
}
