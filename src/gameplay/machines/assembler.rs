use bevy::prelude::*;
use crate::gameplay::grid::{ItemSlot, SimulationGrid, Machine};
use crate::core::registry::RecipeRegistry;
use crate::gameplay::interaction::PlayerInteractEvent;
use crate::core::config::GameConfig;
use serde::{Serialize, Deserialize};

/// A machine that crafts items based on recipes.
#[derive(Component, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

    info!("[Assembler-Debug] tick_assemblers started. dt: {}", dt);

    let mut ejection_requests = Vec::new();

    // --- Part 1: Crafting Logic ---
    for (pos, machine) in grid.machines.iter_mut() {
        if let Machine::Assembler(assembler) = &mut machine.machine_type {
            info!("[Assembler-Debug] Processing assembler at {:?}. Current progress: {}", pos, assembler.crafting_progress);
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
                        info!("[Assembler-Debug] Has inputs: {}. Required: {:?}, In inventory: {:?}", has_inputs, recipe.inputs, assembler.input_inventory);

                        if has_inputs {
                            assembler.crafting_progress += dt;
                            info!("[Assembler-Debug] Progress after dt: {}. Recipe craft_time: {}", assembler.crafting_progress, recipe.craft_time);
                            if assembler.crafting_progress >= recipe.craft_time {
                                info!("[Assembler-Debug] Crafting finished! Consuming inputs...");
                                // Consume inputs
                                for required in &recipe.inputs {
                                    let mut remaining_to_consume = required.count;
                                    assembler.input_inventory.retain_mut(|slot| {
                                        if slot.item_id == required.item && remaining_to_consume > 0 {
                                            let consumed_from_slot = slot.count.min(remaining_to_consume);
                                            slot.count -= consumed_from_slot;
                                            remaining_to_consume -= consumed_from_slot;
                                            info!("[Assembler-Debug] Consumed {} of {} from slot. Remaining to consume: {}", consumed_from_slot, slot.item_id, remaining_to_consume);
                                            slot.count > 0 // Retain if anything left in slot
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
                                info!("Crafted {}! Output inventory: {:?}", recipe.name, assembler.output_inventory);
                            }
                        } else {
                            assembler.crafting_progress = 0.0;
                            info!("[Assembler-Debug] Not enough inputs. Progress reset.");
                        }
                    } else {
                        info!("[Assembler-Debug] Output inventory full.");
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
                info!("[Assembler-Debug] Ejection request for item from {:?} to {:?} in direction {:?}", *pos, target_pos, output_direction);
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
                            info!("[Assembler-Debug] Item ejected to conveyor at {:?}.", target_pos);
                        } else {
                            info!("[Assembler-Debug] Conveyor at {:?} has no space or conflicting item.", target_pos);
                        }
                    } else {
                        info!("[Assembler-Debug] Conveyor at {:?} is full.", target_pos);
                    }
                 } else {
                    info!("[Assembler-Debug] Target at {:?} is not a Conveyor.", target_pos);
                 }
            } else {
                info!("[Assembler-Debug] No machine at target position {:?}.", target_pos);
            }
        }
        
        if accepted {
            if let Some(machine) = grid.machines.get_mut(&assembler_pos) {
                if let Machine::Assembler(assembler) = &mut machine.machine_type {
                    assembler.output_inventory.remove(0);
                    info!("[Assembler-Debug] Item removed from assembler output inventory.");
                }
            }
        } else {
            info!("[Assembler-Debug] Item not ejected from assembler.");
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::grid::{Direction, MachineInstance};
    use crate::gameplay::machines::conveyor::{self, Conveyor};
    use crate::core::registry::{RecipeDefinition, RecipeInput};
    use std::time::Duration;
    use bevy::log::LogPlugin;
    use bevy::MinimalPlugins;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(LogPlugin::default());
        // Removed: app.add_plugins(TimePlugin);
        app.add_systems(Update, (tick_assemblers, conveyor::tick_conveyors));
        app.init_resource::<SimulationGrid>();
        app.init_resource::<RecipeRegistry>();
        app.init_resource::<GameConfig>();
        // Time resource will be initialized by MinimalPlugins
        
        // Setup a recipe
        let mut recipe_registry = app.world_mut().resource_mut::<RecipeRegistry>();
        let ore_to_ingot = RecipeDefinition {
            id: "ore_to_ingot".to_string(),
            name: "Ingot".to_string(),
            inputs: vec![RecipeInput { item: "raw_ore".to_string(), count: 1 }],
            outputs: vec![RecipeInput { item: "ingot".to_string(), count: 1 }],
            craft_time: 0.01, // Reduced craft time for test
        };
        recipe_registry.map.insert("ore_to_ingot".to_string(), ore_to_ingot);
        
        app
    }

    #[test]
    fn test_assembler_full_cycle() {
        let mut app = setup_test_app();

        // --- Setup Grid ---
        let input_conv_pos = IVec3::new(0, 0, 0);
        let assembler_pos = IVec3::new(0, 0, 1);
        let output_conv_pos = IVec3::new(0, 0, 2);

        // Get a mutable reference to the world to setup the grid
        let world = app.world_mut();
        let mut grid = world.resource_mut::<SimulationGrid>();

        // Input Conveyor at (0,0,0) facing South (pushes to (0,0,1))
        let mut input_conveyor = Conveyor::default();
        input_conveyor.inventory.push(ItemSlot {
            item_id: "raw_ore".to_string(),
            count: 1,
            progress: 1.0, // Ready to be ejected
            unique_id: 1,
            from_direction: Some(Direction::North), // Item came from North (relative to conveyor)
        });
        grid.machines.insert(input_conv_pos, MachineInstance {
            id: "conveyor".to_string(),
            orientation: Direction::South, // Conveyor pushes South
            machine_type: Machine::Conveyor(input_conveyor),
            power_node: None,
        });

        // Assembler at (0,0,1) facing North (front is (0,0,0), back is (0,0,2))
        let mut assembler = Assembler::default();
        assembler.active_recipe = Some("ore_to_ingot".to_string());
        grid.machines.insert(assembler_pos, MachineInstance {
            id: "assembler".to_string(),
            orientation: Direction::North, // Assembler faces North
            machine_type: Machine::Assembler(assembler),
            power_node: None,
        });

        // Output Conveyor at (0,0,2) facing North (receives from (0,0,1))
        grid.machines.insert(output_conv_pos, MachineInstance {
            id: "conveyor".to_string(),
            orientation: Direction::North, // Output conveyor faces North
            machine_type: Machine::Conveyor(Conveyor::default()),
            power_node: None,
        });


        // --- 1. Test Item Input ---
        app.update();

        let grid = app.world().resource::<SimulationGrid>();
        let assembler_instance = grid.machines.get(&assembler_pos).unwrap();
        if let Machine::Assembler(asm) = &assembler_instance.machine_type {
             assert_eq!(asm.input_inventory.len(), 1, "Assembler should have received the item");
             assert_eq!(asm.input_inventory[0].item_id, "raw_ore");
        } else {
            panic!("Machine at assembler position is not an assembler");
        }
        let input_conv_instance = grid.machines.get(&input_conv_pos).unwrap();
        if let Machine::Conveyor(conv) = &input_conv_instance.machine_type {
            assert!(conv.inventory.is_empty(), "Input conveyor should be empty after transfer");
        }


        // --- 2. Test Crafting and Ejection ---
        let craft_time = 0.01; // Reduced craft time for test
        let tick_duration = Duration::from_secs_f32(craft_time / 10.0); // Simulate 10 small ticks per craft_time
        let num_ticks = (craft_time / tick_duration.as_secs_f32()).ceil() as u32 + 1; // +1 to ensure it goes past craft_time

        for _ in 0..num_ticks {
            app.world_mut().resource_mut::<Time>().advance_by(tick_duration);
            app.update(); 
        }

        // --- Final State Assertions ---
        let grid = app.world().resource::<SimulationGrid>();
        let assembler_instance = grid.machines.get(&assembler_pos).unwrap();
        if let Machine::Assembler(asm) = &assembler_instance.machine_type {
             assert!(asm.input_inventory.is_empty(), "Assembler input should be empty after crafting and consumption");
             assert!(asm.output_inventory.is_empty(), "Assembler output should be empty after ejection");
        } else {
            panic!("Machine at assembler position is not an assembler");
        }

        let output_conv_instance = grid.machines.get(&output_conv_pos).unwrap();
        if let Machine::Conveyor(conv) = &output_conv_instance.machine_type {
            info!("[Test-Debug] Output Conveyor inventory: {:?}", conv.inventory); // Add this debug log
            assert_eq!(conv.inventory.len(), 1, "Output conveyor should have received the crafted item");
            assert_eq!(conv.inventory[0].item_id, "ingot");
        } else {
            panic!("Machine at output conveyor position is not a conveyor");
        }
    }
}