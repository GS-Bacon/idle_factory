//! Generic machine systems (Phase C Data-Driven Design)
//!
//! These systems work with the generic `Machine` component,
//! using `MachineSpec` to determine behavior.

use crate::components::Machine;
use crate::game_spec::{find_recipe, MachineType, ProcessType};
use crate::world::biome::{BiomeMap, BiomeType};
use crate::{BlockType, Conveyor};
use bevy::prelude::*;
use std::collections::HashMap;

/// Generic machine tick system - processes all Machine components
pub fn generic_machine_tick(
    time: Res<Time>,
    biome_map: Res<BiomeMap>,
    mut machine_query: Query<&mut Machine>,
    mut conveyor_query: Query<(Entity, &mut Conveyor)>,
) {
    let delta = time.delta_secs();

    // Build conveyor position map for O(1) lookup
    let conveyor_map: HashMap<IVec3, Entity> = conveyor_query
        .iter()
        .map(|(entity, conveyor)| (conveyor.position, entity))
        .collect();

    for mut machine in machine_query.iter_mut() {
        match machine.spec.process_type {
            ProcessType::AutoGenerate => {
                tick_auto_generate(
                    &mut machine,
                    delta,
                    &biome_map,
                    &conveyor_map,
                    &mut conveyor_query,
                );
            }
            ProcessType::Recipe(machine_type) => {
                tick_recipe(
                    &mut machine,
                    delta,
                    machine_type,
                    &conveyor_map,
                    &mut conveyor_query,
                );
            }
            ProcessType::Transfer => {
                // Conveyors are handled separately
            }
        }
    }
}

/// Tick for auto-generating machines (like Miner)
fn tick_auto_generate(
    machine: &mut Machine,
    delta: f32,
    biome_map: &BiomeMap,
    conveyor_map: &HashMap<IVec3, Entity>,
    conveyor_query: &mut Query<(Entity, &mut Conveyor)>,
) {
    let spec = machine.spec;

    // Check if output buffer has space
    let output_slot = machine.slots.outputs.first_mut();
    let can_output = output_slot
        .map(|s| s.count < spec.buffer_size)
        .unwrap_or(false);

    if !can_output {
        return;
    }

    // Progress mining
    machine.progress += delta / spec.process_time;

    if machine.progress >= 1.0 {
        machine.progress = 0.0;
        machine.tick_count = machine.tick_count.wrapping_add(1);

        // Determine what to mine based on biome
        let biome = biome_map.get_biome(machine.position);
        let mined_type = get_biome_output(biome, machine.tick_count);

        // Add to output buffer
        if let Some(output) = machine.slots.outputs.first_mut() {
            if output.item_type.is_none() || output.item_type == Some(mined_type) {
                output.item_type = Some(mined_type);
                output.count += 1;
            }
        }
    }

    // Try to output to conveyor
    try_output_to_conveyor(machine, conveyor_map, conveyor_query);
}

/// Tick for recipe-based machines (Furnace, Crusher, Assembler)
fn tick_recipe(
    machine: &mut Machine,
    delta: f32,
    machine_type: MachineType,
    conveyor_map: &HashMap<IVec3, Entity>,
    conveyor_query: &mut Query<(Entity, &mut Conveyor)>,
) {
    let spec = machine.spec;

    // Get input item
    let input_item = machine.slots.inputs.first().and_then(|s| s.item_type);

    // Find recipe
    let Some(input) = input_item else {
        return;
    };

    let Some(recipe) = find_recipe(machine_type, input) else {
        return;
    };

    // Check fuel requirement
    if spec.requires_fuel && machine.slots.fuel == 0 {
        return;
    }

    // Check if we have enough input
    let input_slot = &machine.slots.inputs[0];
    let required_count = recipe.inputs.first().map(|i| i.count).unwrap_or(1);
    if input_slot.count < required_count {
        return;
    }

    // Check if output has space
    let output_item = recipe.outputs.first().map(|o| o.item);
    let output_count = recipe.outputs.first().map(|o| o.count).unwrap_or(1);

    let output_slot = machine.slots.outputs.first();
    let can_output = output_slot
        .map(|s| {
            s.count + output_count <= spec.buffer_size
                && (s.item_type.is_none() || s.item_type == output_item)
        })
        .unwrap_or(false);

    if !can_output {
        return;
    }

    // Progress processing
    machine.progress += delta / recipe.craft_time;

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
        if let (Some(item), Some(output_slot)) = (output_item, machine.slots.outputs.first_mut()) {
            output_slot.add(item, output_count);
        }
    }

    // Try to output to conveyor
    try_output_to_conveyor(machine, conveyor_map, conveyor_query);
}

/// Try to output items to a connected conveyor (O(1) lookup)
fn try_output_to_conveyor(
    machine: &mut Machine,
    conveyor_map: &HashMap<IVec3, Entity>,
    conveyor_query: &mut Query<(Entity, &mut Conveyor)>,
) {
    use crate::components::ConveyorItem;

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

    let Some(item_type) = output_slot.item_type else {
        return;
    };

    // Transfer one item
    output_slot.take(1);
    conveyor.items.push(ConveyorItem {
        block_type: item_type,
        progress: 0.0,
        visual_entity: None,
        lateral_offset: 0.0,
    });
}

/// Get mining output based on biome
fn get_biome_output(biome: BiomeType, tick: u32) -> BlockType {
    use crate::game_spec::biome_mining_spec::*;

    let probabilities = match biome {
        BiomeType::Iron => IRON_BIOME,
        BiomeType::Copper => COPPER_BIOME,
        BiomeType::Coal => COAL_BIOME,
        BiomeType::Stone => STONE_BIOME,
        BiomeType::Mixed => MIXED_BIOME,
        BiomeType::Unmailable => STONE_BIOME,
    };

    // Simple deterministic selection based on tick
    let total: u32 = probabilities.iter().map(|(_, p)| p).sum();
    let roll = tick % total;

    let mut acc = 0;
    for (block_type, prob) in probabilities {
        acc += prob;
        if roll < acc {
            return *block_type;
        }
    }

    BlockType::Stone
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::MachineSlot;
    use crate::game_spec::{CRUSHER, FURNACE, MINER};

    #[test]
    fn test_machine_slot_operations() {
        let mut slot = MachineSlot::empty();
        assert!(slot.is_empty());

        // Add items
        slot.add(BlockType::IronOre, 5);
        assert_eq!(slot.count, 5);
        assert_eq!(slot.item_type, Some(BlockType::IronOre));

        // Can't add different type
        let added = slot.add(BlockType::CopperOre, 3);
        assert_eq!(added, 0);
        assert_eq!(slot.count, 5);

        // Can add same type
        slot.add(BlockType::IronOre, 3);
        assert_eq!(slot.count, 8);

        // Take items
        let taken = slot.take(3);
        assert_eq!(taken, 3);
        assert_eq!(slot.count, 5);

        // Take all
        slot.take(5);
        assert!(slot.is_empty());
    }

    #[test]
    fn test_machine_creation() {
        let machine = Machine::new(
            &MINER,
            IVec3::new(0, 0, 0),
            crate::components::Direction::North,
        );
        assert_eq!(machine.spec.id, "miner");
        assert_eq!(machine.progress, 0.0);
        assert_eq!(machine.slots.outputs.len(), 1);
    }

    #[test]
    fn test_furnace_machine_creation() {
        let machine = Machine::new(
            &FURNACE,
            IVec3::new(0, 0, 0),
            crate::components::Direction::North,
        );
        assert_eq!(machine.spec.id, "furnace");
        assert!(machine.spec.requires_fuel);
        assert_eq!(machine.slots.inputs.len(), 1);
        assert_eq!(machine.slots.outputs.len(), 1);
    }

    #[test]
    fn test_crusher_machine_creation() {
        let machine = Machine::new(
            &CRUSHER,
            IVec3::new(0, 0, 0),
            crate::components::Direction::North,
        );
        assert_eq!(machine.spec.id, "crusher");
        assert!(!machine.spec.requires_fuel);
    }

    #[test]
    fn test_biome_output_deterministic() {
        use crate::world::biome::BiomeType;

        // Same tick should give same output
        let out1 = get_biome_output(BiomeType::Iron, 100);
        let out2 = get_biome_output(BiomeType::Iron, 100);
        assert_eq!(out1, out2);

        // Different biomes can give different outputs
        let iron_out = get_biome_output(BiomeType::Iron, 0);
        let copper_out = get_biome_output(BiomeType::Copper, 0);
        // At tick 0, both should give their primary ore
        assert_eq!(iron_out, BlockType::IronOre);
        assert_eq!(copper_out, BlockType::CopperOre);
    }
}
