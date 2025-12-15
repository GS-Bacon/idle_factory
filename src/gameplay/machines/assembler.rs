use bevy::prelude::*;
use crate::gameplay::grid::{ItemSlot, SimulationGrid, Machine};
use crate::core::registry::RecipeRegistry;
use crate::gameplay::interaction::PlayerInteractEvent;

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
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (_pos, machine) in grid.machines.iter_mut() {
        if let Machine::Assembler(assembler) = &mut machine.machine_type {
            
            let recipe_id = if let Some(id) = &assembler.active_recipe {
                id
            } else {
                continue; // No recipe, do nothing
            };

            let recipe = if let Some(r) = recipes.map.get(recipe_id) {
                r
            } else {
                error!("Assembler has unknown recipe: {}", recipe_id);
                continue;
            };

            // Check if output is full
            if assembler.output_inventory.len() >= 10 { //TODO: make configurable
                continue;
            }

            // Check if we have enough input items
            let mut has_inputs = true;
            for required in &recipe.inputs {
                let count_in_inventory = assembler.input_inventory.iter()
                    .filter(|slot| slot.item_id == required.item)
                    .map(|slot| slot.count)
                    .sum::<u32>();
                if count_in_inventory < required.count {
                    has_inputs = false;
                    break;
                }
            }

            if has_inputs {
                // We are able to craft, so increase progress
                assembler.crafting_progress += dt;

                if assembler.crafting_progress >= recipe.craft_time {
                    // Crafting finished!
                    // 1. Consume inputs
                    for required in &recipe.inputs {
                        let mut remaining_to_consume = required.count;
                        assembler.input_inventory.retain_mut(|slot| {
                            if slot.item_id == required.item && remaining_to_consume > 0 {
                                if slot.count > remaining_to_consume {
                                    slot.count -= remaining_to_consume;
                                    remaining_to_consume = 0;
                                    return true; // Keep the slot
                                } else {
                                    remaining_to_consume -= slot.count;
                                    return false; // Remove the slot
                                }
                            }
                            true
                        });
                    }

                    // 2. Add outputs
                    for produced in &recipe.outputs {
                        // TODO: stack with existing items
                        assembler.output_inventory.push(ItemSlot {
                            item_id: produced.item.clone(),
                            count: produced.count,
                            progress: 0.0, // Not relevant in inventory
                            unique_id: 0, // Not relevant
                            from_direction: None,
                        });
                    }

                    // 3. Reset progress
                    assembler.crafting_progress = 0.0;
                    info!("Crafted {}!", recipe.name);
                }
            } else {
                // Not enough resources, reset progress
                assembler.crafting_progress = 0.0;
            }
        }
    }

    // TODO: Add logic to eject items from output_inventory to adjacent conveyors
}
