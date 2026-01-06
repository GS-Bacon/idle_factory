//! Crusher systems: processing, UI interaction, output to conveyor

use crate::components::{
    CommandInputState, CrusherProgressBar, CrusherSlotButton, CrusherSlotCount, CrusherUI,
    CursorLockState, InteractingCrusher, InteractingFurnace, InventoryOpen, MachineSlotType,
    PlayerCamera,
};
use crate::game_spec::{find_recipe, MachineType};
use crate::player::Inventory;
use crate::{BlockType, Conveyor, Crusher, Furnace, REACH_DISTANCE};
use bevy::prelude::*;

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
    use super::interaction::{
        can_interact, close_machine_ui, get_close_key_pressed, is_cursor_locked, open_machine_ui,
        raycast_closest_machine,
    };

    // Don't open crusher if other UI is open
    if !can_interact(&inventory_open, &command_state, &cursor_state)
        || interacting_furnace.0.is_some()
    {
        return;
    }

    let (e_pressed, esc_pressed) = get_close_key_pressed(&key_input);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        close_machine_ui::<CrusherUI>(
            esc_pressed,
            &mut crusher_ui_query,
            &mut windows,
            &mut cursor_state,
        );
        return;
    }

    // Only open crusher UI with right-click when cursor is locked
    if !mouse_button.just_pressed(MouseButton::Right) || !is_cursor_locked(&windows) {
        return;
    }

    // Find closest crusher and open UI
    if let Some(result) = raycast_closest_machine(&camera_query, &crusher_query, REACH_DISTANCE) {
        interacting.0 = Some(result.entity);
        open_machine_ui::<CrusherUI>(&mut crusher_ui_query, &mut windows);
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
