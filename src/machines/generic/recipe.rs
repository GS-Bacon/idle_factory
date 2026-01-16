//! Recipe-based machine processing (Furnace, Crusher, Assembler)

use crate::components::Machine;
use crate::core::ItemId;
use crate::game_spec::{find_recipe, MachineType};
use crate::Conveyor;
use bevy::prelude::*;
use std::collections::HashMap;

use super::output::try_output_to_conveyor;

/// Event result from tick_recipe: (started_inputs, completed_outputs)
pub(super) type RecipeEventResult =
    Option<(Option<Vec<(ItemId, u32)>>, Option<Vec<(ItemId, u32)>>)>;

/// Tick for recipe-based machines (Furnace, Crusher, Assembler)
/// Returns Some((started_inputs, completed_outputs)) for event emission
/// - started_inputs: Some when processing started (inputs consumed)
/// - completed_outputs: Some when processing completed (outputs produced)
pub(super) fn tick_recipe(
    machine: &mut Machine,
    delta: f32,
    machine_type: MachineType,
    conveyor_map: &HashMap<IVec3, Entity>,
    conveyor_query: &mut Query<(Entity, &mut Conveyor)>,
) -> RecipeEventResult {
    let spec = machine.spec;

    // Get input item
    let input_item_id = machine.slots.inputs.first().and_then(|s| s.item_id);

    // Find recipe
    let input_id = input_item_id?;
    let recipe = find_recipe(machine_type, input_id)?;

    // Check fuel requirement
    if spec.requires_fuel && machine.slots.fuel == 0 {
        return None;
    }

    // Check if we have enough input
    let input_slot = &machine.slots.inputs[0];
    let required_count = recipe.inputs.first().map(|i| i.count).unwrap_or(1);
    if input_slot.count < required_count {
        return None;
    }

    // Check if output has space
    let output_item_id: Option<ItemId> = recipe.outputs.first().map(|o| o.item);
    let output_count = recipe.outputs.first().map(|o| o.count).unwrap_or(1);

    let output_slot = machine.slots.outputs.first();
    let can_output = output_slot
        .map(|s| {
            s.count + output_count <= spec.buffer_size
                && (s.item_id.is_none() || s.item_id == output_item_id)
        })
        .unwrap_or(false);

    if !can_output {
        return None;
    }

    // Track if we just started processing
    let was_idle = machine.progress == 0.0;

    // Progress processing
    machine.progress += delta / recipe.craft_time;

    // Determine started event (only when transitioning from idle to processing)
    let started_inputs = if was_idle && machine.progress > 0.0 && machine.progress < 1.0 {
        Some(vec![(input_id, required_count)])
    } else {
        None
    };

    let mut completed_outputs = None;
    if machine.progress >= 1.0 {
        machine.progress = 0.0;

        // Consume input
        if let Some(input_slot) = machine.slots.inputs.first_mut() {
            input_slot.take(required_count);
        }

        // Consume fuel if required
        if spec.requires_fuel {
            if let Some(fuel_req) = &recipe.fuel {
                machine.slots.fuel = machine.slots.fuel.saturating_sub(fuel_req.amount);
            }
        }

        // Produce output
        if let (Some(item_id), Some(output_slot)) =
            (output_item_id, machine.slots.outputs.first_mut())
        {
            output_slot.add_id(item_id, output_count);
            completed_outputs = Some(vec![(item_id, output_count)]);
        }
    }

    // Try to output to conveyor
    try_output_to_conveyor(machine, conveyor_map, conveyor_query);

    // Return event info if anything happened
    if started_inputs.is_some() || completed_outputs.is_some() {
        Some((started_inputs, completed_outputs))
    } else {
        None
    }
}
