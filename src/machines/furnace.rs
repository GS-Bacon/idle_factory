//! Furnace systems: smelting, UI interaction, output to conveyor

use crate::components::{
    CommandInputState, CursorLockState, FurnaceUI, InteractingFurnace, InventoryOpen,
    MachineProgressBar, MachineSlotButton, MachineSlotCount, MachineSlotType, PlayerCamera,
};
use crate::game_spec::{find_recipe, MachineType};
use crate::player::Inventory;
use crate::systems::set_ui_open_state;
use crate::utils::ray_aabb_intersection;
use crate::{BlockType, Conveyor, Furnace, BLOCK_SIZE, REACH_DISTANCE};
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

/// Handle furnace right-click interaction (open/close UI)
#[allow(clippy::too_many_arguments)]
pub fn furnace_interact(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    furnace_query: Query<(Entity, &Transform), With<Furnace>>,
    mut interacting: ResMut<InteractingFurnace>,
    mut furnace_ui_query: Query<&mut Visibility, With<FurnaceUI>>,
    mut windows: Query<&mut Window>,
    inventory_open: Res<InventoryOpen>,
    command_state: Res<CommandInputState>,
    mut cursor_state: ResMut<CursorLockState>,
) {
    // Don't process when inventory, command input is open or game is paused (input matrix: Right Click)
    if inventory_open.0 || command_state.open || cursor_state.paused {
        return;
    }

    // ESC or E key to close furnace UI (when open)
    let e_pressed = key_input.just_pressed(KeyCode::KeyE);
    let esc_pressed = key_input.just_pressed(KeyCode::Escape);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        if let Ok(mut vis) = furnace_ui_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        let mut window = windows.single_mut();
        if esc_pressed {
            // ESC: Release pointer lock and show cursor
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(false);
        } else {
            // E key: Keep cursor locked (no browser interference)
            // Set flag to prevent inventory from opening this frame
            cursor_state.skip_inventory_toggle = true;
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Only open furnace UI with right-click
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

    // Find closest furnace intersection
    let mut closest_furnace: Option<(Entity, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    for (entity, furnace_transform) in furnace_query.iter() {
        let furnace_pos = furnace_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            furnace_pos - Vec3::splat(half_size),
            furnace_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_furnace.is_none_or(|f| t < f.1);
                if is_closer {
                    closest_furnace = Some((entity, t));
                }
            }
        }
    }

    // Open furnace UI
    if let Some((entity, _)) = closest_furnace {
        interacting.0 = Some(entity);
        if let Ok(mut vis) = furnace_ui_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
        // Unlock cursor for UI interaction
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        set_ui_open_state(true);
    }
}

/// Handle slot click interactions when furnace UI is open
pub fn furnace_ui_input(
    interacting: Res<InteractingFurnace>,
    mut furnace_query: Query<&mut Furnace>,
    mut inventory: ResMut<Inventory>,
    mut slot_query: Query<
        (
            &Interaction,
            &MachineSlotButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    let Some(furnace_entity) = interacting.0 else {
        return;
    };

    let Ok(mut furnace) = furnace_query.get_mut(furnace_entity) else {
        return;
    };

    for (interaction, slot_button, mut bg_color, mut border_color) in slot_query.iter_mut() {
        let slot_type = slot_button.0;

        match *interaction {
            Interaction::Pressed => {
                match slot_type {
                    MachineSlotType::Fuel => {
                        // Add coal from inventory (max 64)
                        const MAX_FUEL: u32 = 64;
                        if furnace.fuel < MAX_FUEL && inventory.consume_item(BlockType::Coal, 1) {
                            furnace.fuel += 1;
                        }
                    }
                    MachineSlotType::Input => {
                        // Add ore or dust from inventory (prioritize same type, then iron, then copper)
                        let smeltables = [
                            BlockType::IronOre,
                            BlockType::CopperOre,
                            BlockType::IronDust,
                            BlockType::CopperDust,
                        ];
                        for smeltable in smeltables {
                            if furnace.can_add_input(smeltable)
                                && inventory.consume_item(smeltable, 1)
                            {
                                furnace.input_type = Some(smeltable);
                                furnace.input_count += 1;
                                break;
                            }
                        }
                    }
                    MachineSlotType::Output => {
                        // Take output ingot to inventory
                        if furnace.output_count > 0 {
                            if let Some(output_type) = furnace.output_type {
                                furnace.output_count -= 1;
                                inventory.add_item(output_type, 1);
                                if furnace.output_count == 0 {
                                    furnace.output_type = None;
                                }
                            }
                        }
                    }
                }
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                // Brighten background slightly
                let base = match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.25, 0.25, 0.25),
                    MachineSlotType::Input => Color::srgb(0.7, 0.6, 0.5),
                    MachineSlotType::Output => Color::srgb(0.9, 0.9, 0.95),
                };
                *bg_color = BackgroundColor(base);
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.15, 0.15, 0.15),
                    MachineSlotType::Input => Color::srgb(0.6, 0.5, 0.4),
                    MachineSlotType::Output => Color::srgb(0.8, 0.8, 0.85),
                });
            }
        }
    }
}

/// Smelting logic - convert ore/dust + coal to ingot
/// Uses recipe system for craft time (ore=2.0s, dust=1.5s)
pub fn furnace_smelting(time: Res<Time>, mut furnace_query: Query<&mut Furnace>) {
    for mut furnace in furnace_query.iter_mut() {
        // Need input ore/dust and valid recipe to smelt
        let Some(input_item) = furnace.input_type else {
            furnace.progress = 0.0;
            continue;
        };

        if furnace.fuel == 0 || furnace.input_count == 0 {
            furnace.progress = 0.0;
            continue;
        }

        // Get recipe (uses recipe system as Single Source of Truth)
        let Some(recipe) = find_recipe(MachineType::Furnace, input_item) else {
            furnace.progress = 0.0;
            continue;
        };

        let output_ingot = recipe.outputs.first().map(|o| o.item);

        // Check output slot compatibility
        let output_compatible = match (furnace.output_type, output_ingot) {
            (None, Some(_)) => true,
            (Some(current), Some(new)) => current == new && furnace.output_count < 64,
            _ => false,
        };

        if output_compatible {
            // Use recipe craft_time (ore=2.0s, dust=1.5s)
            furnace.progress += time.delta_secs() / recipe.craft_time;

            // When progress reaches 1.0, complete smelting
            if furnace.progress >= 1.0 {
                furnace.progress = 0.0;
                furnace.fuel -= 1;
                furnace.input_count -= 1;
                if furnace.input_count == 0 {
                    furnace.input_type = None;
                }
                furnace.output_type = output_ingot;
                furnace.output_count += 1;
            }
        } else {
            furnace.progress = 0.0;
        }
    }
}

/// Update furnace UI slot counts and progress bar
pub fn update_furnace_ui(
    interacting: Res<InteractingFurnace>,
    furnace_query: Query<&Furnace>,
    mut slot_count_query: Query<(&MachineSlotCount, &mut Text)>,
    mut progress_bar_query: Query<&mut Node, With<MachineProgressBar>>,
) {
    let Some(furnace_entity) = interacting.0 else {
        return;
    };

    let Ok(furnace) = furnace_query.get(furnace_entity) else {
        return;
    };

    // Update slot counts
    for (slot_count, mut text) in slot_count_query.iter_mut() {
        **text = match slot_count.0 {
            MachineSlotType::Fuel => format!("{}", furnace.fuel),
            MachineSlotType::Input => format!("{}", furnace.input_count),
            MachineSlotType::Output => format!("{}", furnace.output_count),
        };
    }

    // Update progress bar
    for mut node in progress_bar_query.iter_mut() {
        node.width = Val::Percent(furnace.progress * 100.0);
    }
}

/// Furnace output to conveyor in facing direction only
///
/// Note: Furnace outputs ingots which typically go to conveyors.
/// Machine-to-machine transfer is supported but ingots are rarely
/// fed directly to other machines.
pub fn furnace_output(
    mut furnace_query: Query<&mut Furnace>,
    mut conveyor_query: Query<&mut Conveyor>,
) {
    use super::output::try_transfer_to_conveyor;

    for mut furnace in furnace_query.iter_mut() {
        let Some(output_type) = furnace.output_type else {
            continue;
        };

        if furnace.output_count == 0 {
            continue;
        }

        // Output only in facing direction (front of machine)
        let output_pos = furnace.position + furnace.facing.to_ivec3();

        // Use common transfer logic (conveyor only for furnace)
        let transferred = try_transfer_to_conveyor(
            furnace.position,
            output_pos,
            output_type,
            &mut conveyor_query,
        );

        if transferred {
            furnace.output_count -= 1;
            if furnace.output_count == 0 {
                furnace.output_type = None;
            }
        }
    }
}
