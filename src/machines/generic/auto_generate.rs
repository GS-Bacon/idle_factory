//! Auto-generate machines (like Miner)

use crate::components::Machine;
use crate::core::{items, ItemId};
use crate::world::biome::{BiomeMap, BiomeType};
use crate::Conveyor;
use bevy::prelude::*;
use std::collections::HashMap;

use super::output::try_output_to_conveyor;

/// Tick for auto-generating machines (like Miner)
/// Returns Some(output_item_id) when an item is produced
pub(super) fn tick_auto_generate(
    machine: &mut Machine,
    delta: f32,
    biome_map: &BiomeMap,
    conveyor_map: &HashMap<IVec3, Entity>,
    conveyor_query: &mut Query<(Entity, &mut Conveyor)>,
) -> Option<ItemId> {
    let spec = machine.spec;

    // Check if output buffer has space
    let output_slot = machine.slots.outputs.first_mut();
    let can_output = output_slot
        .map(|s| s.count < spec.buffer_size)
        .unwrap_or(false);

    if !can_output {
        return None;
    }

    // Progress mining
    machine.progress += delta / spec.process_time;

    let mut produced = None;
    if machine.progress >= 1.0 {
        machine.progress = 0.0;
        machine.tick_count = machine.tick_count.wrapping_add(1);

        // Determine what to mine based on biome
        let biome = biome_map.get_biome(machine.position);
        let mined_id = get_biome_output(biome, machine.tick_count);

        // Add to output buffer
        if let Some(output) = machine.slots.outputs.first_mut() {
            if output.item_id.is_none() || output.item_id == Some(mined_id) {
                output.item_id = Some(mined_id);
                output.count += 1;
                produced = Some(mined_id);
            }
        }
    }

    // Try to output to conveyor
    try_output_to_conveyor(machine, conveyor_map, conveyor_query);
    produced
}

/// Get mining output based on biome (returns ItemId)
pub fn get_biome_output(biome: BiomeType, tick: u32) -> ItemId {
    use crate::game_spec::biome_mining_spec::*;

    let probabilities: &[(ItemId, u32)] = match biome {
        BiomeType::Iron => &IRON_BIOME,
        BiomeType::Copper => &COPPER_BIOME,
        BiomeType::Coal => &COAL_BIOME,
        BiomeType::Stone => &STONE_BIOME,
        BiomeType::Mixed => &MIXED_BIOME,
        BiomeType::Unmailable => &STONE_BIOME,
    };

    // Simple deterministic selection based on tick
    let total: u32 = probabilities.iter().map(|(_, p)| p).sum();
    let roll = tick % total;

    let mut acc = 0;
    for (item_id, prob) in probabilities {
        acc += prob;
        if roll < acc {
            return *item_id;
        }
    }

    items::stone()
}
