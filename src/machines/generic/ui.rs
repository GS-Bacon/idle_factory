//! Generic machine UI systems

use crate::components::{
    GenericMachineProgressBar, GenericMachineSlotButton, GenericMachineSlotCount,
    InteractingMachine, Machine, MachineSlot,
};
use crate::core::items;
use crate::player::{LocalPlayer, PlayerInventory};
use bevy::prelude::*;

/// Update generic machine UI slot counts and progress bar
pub fn update_generic_machine_ui(
    interacting: Res<InteractingMachine>,
    machine_query: Query<&Machine>,
    mut slot_count_query: Query<(&GenericMachineSlotCount, &mut Text)>,
    mut progress_bar_query: Query<&mut Node, With<GenericMachineProgressBar>>,
) {
    let Some(entity) = interacting.0 else {
        return;
    };

    let Ok(machine) = machine_query.get(entity) else {
        return;
    };

    // Update slot counts
    for (slot_count, mut text) in slot_count_query.iter_mut() {
        let display = if slot_count.is_fuel {
            format_count(machine.slots.fuel)
        } else if slot_count.is_input {
            machine
                .slots
                .inputs
                .get(slot_count.slot_id as usize)
                .map(format_slot)
                .unwrap_or_default()
        } else {
            machine
                .slots
                .outputs
                .get(slot_count.slot_id as usize)
                .map(format_slot)
                .unwrap_or_default()
        };
        **text = display;
    }

    // Update progress bar
    for mut node in progress_bar_query.iter_mut() {
        node.width = Val::Percent(machine.progress * 100.0);
    }
}

/// Format slot count for display
fn format_slot(slot: &MachineSlot) -> String {
    if slot.is_empty() {
        String::new()
    } else {
        let short_name = slot.get_item_id().map(|id| id.short_name()).unwrap_or("");
        if slot.count > 1 {
            format!("{}{}", short_name, slot.count)
        } else {
            short_name.to_string()
        }
    }
}

fn format_count(count: u32) -> String {
    if count == 0 {
        String::new()
    } else {
        count.to_string()
    }
}

/// Handle generic machine UI input (slot clicks)
#[allow(clippy::too_many_arguments)]
pub fn generic_machine_ui_input(
    interacting: Res<InteractingMachine>,
    mut machine_query: Query<&mut Machine>,
    local_player: Option<Res<LocalPlayer>>,
    mut inventory_query: Query<&mut PlayerInventory>,
    mut slot_btn_query: Query<
        (
            &Interaction,
            &GenericMachineSlotButton,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
) {
    let Some(entity) = interacting.0 else {
        return;
    };

    let Ok(mut machine) = machine_query.get_mut(entity) else {
        return;
    };

    let Some(local_player) = local_player else {
        return;
    };

    let Ok(mut inventory) = inventory_query.get_mut(local_player.0) else {
        return;
    };

    for (interaction, slot_btn, mut bg_color) in slot_btn_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Take from output / put to input
                if slot_btn.is_input {
                    // Try to put selected item into input slot
                    if let Some(selected_id) = inventory.selected_item_id() {
                        // Valid item check - must have a name registered
                        if selected_id.name().is_some() {
                            if let Some(input_slot) =
                                machine.slots.inputs.get_mut(slot_btn.slot_id as usize)
                            {
                                if (input_slot.item_id.is_none()
                                    || input_slot.item_id == Some(selected_id))
                                    && inventory.consume_item_by_id(selected_id, 1)
                                {
                                    input_slot.add_id(selected_id, 1);
                                }
                            }
                        }
                    }
                } else if slot_btn.is_fuel {
                    // Put coal into fuel slot
                    let coal_id = items::coal();
                    if let Some(selected_id) = inventory.selected_item_id() {
                        if selected_id == coal_id && inventory.consume_item_by_id(coal_id, 1) {
                            machine.slots.fuel += 1;
                        }
                    }
                } else {
                    // Take from output slot
                    if let Some(output_slot) =
                        machine.slots.outputs.get_mut(slot_btn.slot_id as usize)
                    {
                        if let Some(item_id) = output_slot.item_id {
                            let taken = output_slot.take(1);
                            if taken > 0 {
                                inventory.add_item_by_id(item_id, taken);
                            }
                        }
                    }
                }
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.5));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
        }
    }
}
