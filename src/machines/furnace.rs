//! Furnace systems: smelting, UI interaction, output to conveyor

use crate::components::{
    CommandInputState, CursorLockState, FurnaceUI, InteractingFurnace, InventoryOpen,
    MachineProgressBar, MachineSlotButton, MachineSlotCount, MachineSlotType, PlayerCamera,
};
use crate::game_spec::{find_recipe, MachineType};
use crate::player::Inventory;
use crate::{BlockType, Conveyor, Furnace, REACH_DISTANCE};
use bevy::prelude::*;

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
    use super::interaction::{
        can_interact, close_machine_ui, get_close_key_pressed, is_cursor_locked, open_machine_ui,
        raycast_closest_machine,
    };

    // Don't process when inventory, command input is open or game is paused
    if !can_interact(&inventory_open, &command_state, &cursor_state) {
        return;
    }

    let (e_pressed, esc_pressed) = get_close_key_pressed(&key_input);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        close_machine_ui::<FurnaceUI>(
            esc_pressed,
            &mut furnace_ui_query,
            &mut windows,
            &mut cursor_state,
        );
        return;
    }

    // Only open furnace UI with right-click when cursor is locked
    if !mouse_button.just_pressed(MouseButton::Right) || !is_cursor_locked(&windows) {
        return;
    }

    // Find closest furnace and open UI
    if let Some(result) = raycast_closest_machine(&camera_query, &furnace_query, REACH_DISTANCE) {
        interacting.0 = Some(result.entity);
        open_machine_ui::<FurnaceUI>(&mut furnace_ui_query, &mut windows);
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
