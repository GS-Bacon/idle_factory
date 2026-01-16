//! Main tick system for generic machines

use crate::components::Machine;
use crate::core::ItemId;
use crate::events::game_events::{MachineCompleted, MachineStarted};
use crate::events::GuardedEventWriter;
use crate::game_spec::ProcessType;
use crate::world::biome::BiomeMap;
use crate::Conveyor;
use bevy::prelude::*;
use std::collections::HashMap;

use super::auto_generate::tick_auto_generate;
use super::recipe::tick_recipe;

/// Generic machine tick system - processes all Machine components
pub fn generic_machine_tick(
    time: Res<Time>,
    biome_map: Res<BiomeMap>,
    mut machine_query: Query<(Entity, &mut Machine)>,
    mut conveyor_query: Query<(Entity, &mut Conveyor)>,
    mut started_events: GuardedEventWriter<MachineStarted>,
    mut completed_events: GuardedEventWriter<MachineCompleted>,
) {
    let delta = time.delta_secs();

    // Build conveyor position map for O(1) lookup
    let conveyor_map: HashMap<IVec3, Entity> = conveyor_query
        .iter()
        .map(|(entity, conveyor)| (conveyor.position, entity))
        .collect();

    // Collect events to send after iteration
    let mut started: Vec<(Entity, Vec<(ItemId, u32)>)> = Vec::new();
    let mut completed: Vec<(Entity, Vec<(ItemId, u32)>)> = Vec::new();

    for (entity, mut machine) in machine_query.iter_mut() {
        match machine.spec.process_type {
            ProcessType::AutoGenerate => {
                let result = tick_auto_generate(
                    &mut machine,
                    delta,
                    &biome_map,
                    &conveyor_map,
                    &mut conveyor_query,
                );
                if let Some(output_id) = result {
                    completed.push((entity, vec![(output_id, 1)]));
                }
            }
            ProcessType::Recipe(machine_type) => {
                let result = tick_recipe(
                    &mut machine,
                    delta,
                    machine_type,
                    &conveyor_map,
                    &mut conveyor_query,
                );
                if let Some((started_inputs, completed_outputs)) = result {
                    if let Some(inputs) = started_inputs {
                        started.push((entity, inputs));
                    }
                    if let Some(outputs) = completed_outputs {
                        completed.push((entity, outputs));
                    }
                }
            }
            ProcessType::Transfer => {
                // Conveyors are handled separately
            }
        }
    }

    // Send events
    for (entity, inputs) in started {
        let _ = started_events.send(MachineStarted { entity, inputs });
    }
    for (entity, outputs) in completed {
        let _ = completed_events.send(MachineCompleted { entity, outputs });
    }
}
