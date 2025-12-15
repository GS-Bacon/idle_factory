use bevy::prelude::*;
use crate::gameplay::grid::{ItemSlot, SimulationGrid, Machine};
use crate::core::registry::RecipeRegistry;
use crate::gameplay::interaction::PlayerInteractEvent;
use crate::core::config::GameConfig;

/// A machine that crafts items based on recipes.
#[derive(Component, Debug, Clone, Default)]
pub struct Assembler {
    /// Items waiting to be used for crafting.
    pub input_inventory: Vec<ItemSlot>,
    /// Items that have been crafted and are waiting for pickup.
    pub output_inventory: Vec<ItemSlot>,
    /// The ID of the recipe currently being processed.
    pub active_recipe: Option<String>,
    /// Progress of the current crafting operation, tied to `craft_time`.
    pub crafting_progress: f32,
}

/// System to temporarily assign a recipe on interaction.
/// This will be replaced by a UI later.
pub fn handle_assembler_interaction(
    mut events: EventReader<PlayerInteractEvent>,
    mut grid: ResMut<SimulationGrid>,
) {
    for event in events.read() {
        if event.mouse_button != MouseButton::Right { continue; }
        if let Some(machine) = grid.machines.get_mut(&event.grid_pos) {
            if let Machine::Assembler(assembler) = &mut machine.machine_type {
                if assembler.active_recipe.is_none() {
                    info!("Setting recipe to 'ore_to_ingot' for assembler at {:?}", event.grid_pos);
                    assembler.active_recipe = Some("ore_to_ingot".to_string());
                }
            }
        }
    }
}


pub fn tick_assemblers(
    mut grid: ResMut<SimulationGrid>,
    recipes: Res<RecipeRegistry>,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let max_items_on_conveyor = config.max_items_per_conveyor.max(1);
    let item_size = 1.0 / max_items_on_conveyor as f32;

    let mut ejection_requests = Vec::new();

    // --- Part 1: Crafting Logic ---
    for (pos, machine) in grid.machines.iter_mut() {
        if let Machine::Assembler(assembler) = &mut machine.machine_type {
            
            // --- Crafting ---
            if let Some(recipe_id) = &assembler.active_recipe {
                if let Some(recipe) = recipes.map.get(recipe_id) {
                    if assembler.output_inventory.len() < 10 { // Not full
                        let mut has_inputs = true;
                        for required in &recipe.inputs {
                            let count_in_inventory = assembler.input_inventory.iter()
                                .filter(|slot| slot.item_id == required.item)
                                .map(|slot| slot.count).sum::<u32>();
                            if count_in_inventory < required.count {
                                has_inputs = false;
                                break;
                            }
                        }

                        if has_inputs {
                            assembler.crafting_progress += dt;
                            if assembler.crafting_progress >= recipe.craft_time {
                                // Consume inputs
                                for required in &recipe.inputs {
                                    let mut remaining_to_consume = required.count;
                                    assembler.input_inventory.retain_mut(|slot| {
                                        if slot.item_id == required.item && remaining_to_consume > 0 {
                                            if slot.count > remaining_to_consume {
                                                slot.count -= remaining_to_consume;
                                                remaining_to_consume = 0;
                                                true
                                            } else {
                                                remaining_to_consume -= slot.count;
                                                false
                                            }
                                        } else { true }
                                    });
                                }
                                // Add outputs
                                for produced in &recipe.outputs {
                                    assembler.output_inventory.push(ItemSlot {
                                        item_id: produced.item.clone(),
                                        count: produced.count,
                                        progress: 0.0,
                                        unique_id: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64,
                                        from_direction: None,
                                    });
                                }
                                assembler.crafting_progress = 0.0;
                                info!("Crafted {}!", recipe.name);
                            }
                        } else {
                            assembler.crafting_progress = 0.0;
                        }
                    }
                } else {
                     error!("Assembler has unknown recipe: {}", recipe_id);
                }
            }

            // --- Ejection ---
            if !assembler.output_inventory.is_empty() {
                let output_direction = machine.orientation.opposite();
                let target_pos = *pos + output_direction.to_ivec3();
                ejection_requests.push((*pos, target_pos, output_direction));
            }
        }
    }
    
    // --- Part 2: Ejection Execution ---
    for (assembler_pos, target_pos, output_direction) in ejection_requests {
        let mut accepted = false;

        // Clone the item to be ejected
        let item_to_eject = if let Some(machine) = grid.machines.get(&assembler_pos) {
            if let Machine::Assembler(assembler) = &machine.machine_type {
                assembler.output_inventory.first().cloned()
            } else { None }
        } else { None };

        if let Some(mut item) = item_to_eject {
            if let Some(target_machine) = grid.machines.get_mut(&target_pos) {
                 if let Machine::Conveyor(conveyor) = &mut target_machine.machine_type {
                    if conveyor.inventory.len() < max_items_on_conveyor {
                        let min_progress = conveyor.inventory.iter()
                            .map(|it| it.progress).fold(1.0f32, |a, b| a.min(b));

                        if conveyor.inventory.is_empty() || min_progress > item_size {
                            item.from_direction = Some(output_direction);
                            conveyor.inventory.push(ItemSlot { progress: 0.0, ..item });
                            accepted = true;
                        }
                    }
                 }
            }
        }
        
        if accepted {
            if let Some(machine) = grid.machines.get_mut(&assembler_pos) {
                if let Machine::Assembler(assembler) = &mut machine.machine_type {
                    assembler.output_inventory.remove(0);
                }
            }
        }
    }
}
