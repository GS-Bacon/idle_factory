//! Output to conveyor logic

use crate::components::{ConveyorItem, Machine};
use crate::Conveyor;
use bevy::prelude::*;
use std::collections::HashMap;

/// Try to output items to a connected conveyor (O(1) lookup)
pub(super) fn try_output_to_conveyor(
    machine: &mut Machine,
    conveyor_map: &HashMap<IVec3, Entity>,
    conveyor_query: &mut Query<(Entity, &mut Conveyor)>,
) {
    let output_pos = machine.output_position();

    // O(1) lookup for conveyor at output position
    let Some(&conveyor_entity) = conveyor_map.get(&output_pos) else {
        return;
    };

    // Get the conveyor component
    let Ok((_, mut conveyor)) = conveyor_query.get_mut(conveyor_entity) else {
        return;
    };

    // Check if conveyor can accept items
    if conveyor.items.len() >= crate::constants::CONVEYOR_MAX_ITEMS {
        return;
    }

    // Get item from output slot
    let Some(output_slot) = machine.slots.outputs.first_mut() else {
        return;
    };

    if output_slot.is_empty() {
        return;
    }

    let Some(item_id) = output_slot.item_id else {
        return;
    };

    // Transfer one item
    output_slot.take(1);
    conveyor.items.push(ConveyorItem::new(item_id, 0.0));
}
