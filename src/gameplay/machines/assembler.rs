use bevy::prelude::*;
use crate::gameplay::grid::{ItemSlot, SimulationGrid, Machine};
use crate::core::registry::RecipeRegistry;
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

// Note: Assembler interaction is now handled by MachineUiPlugin in src/ui/machine_ui.rs
// The UI allows users to select recipes from available options.

/// 入力アイテムに適合するレシピを検索
fn find_matching_recipe(input_inventory: &[ItemSlot], recipes: &RecipeRegistry) -> Option<String> {
    use std::collections::HashMap;

    // 入力アイテムのIDと数量を集計
    let mut item_counts = HashMap::new();
    for slot in input_inventory {
        *item_counts.entry(&slot.item_id).or_insert(0) += slot.count;
    }

    // 全レシピをチェック
    for (recipe_id, recipe) in &recipes.map {
        let mut matches = true;
        for required in &recipe.inputs {
            if item_counts.get(&required.item).copied().unwrap_or(0) < required.count {
                matches = false;
                break;
            }
        }
        if matches {
            return Some(recipe_id.clone());
        }
    }
    None
}

/// 特定のアイテムが任意のレシピの入力に使えるかチェック
pub fn can_accept_item(item_id: &str, recipes: &RecipeRegistry) -> bool {
    for recipe in recipes.map.values() {
        for input in &recipe.inputs {
            if input.item == item_id {
                return true;
            }
        }
    }
    false
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
            // 自動レシピ検索: active_recipeが未設定の場合、入力アイテムから適合するレシピを検索
            if assembler.active_recipe.is_none() && !assembler.input_inventory.is_empty() {
                if let Some(matched_recipe_id) = find_matching_recipe(&assembler.input_inventory, &recipes) {
                    assembler.active_recipe = Some(matched_recipe_id.clone());
                }
            }

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
                                            let consumed_from_slot = slot.count.min(remaining_to_consume);
                                            slot.count -= consumed_from_slot;
                                            remaining_to_consume -= consumed_from_slot;
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
                                        unique_id: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0),
                                        from_direction: None,
                                        lane: Default::default(),
                                    });
                                }
                                assembler.crafting_progress = 0.0;
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::grid::{Direction, MachineInstance};
    use crate::gameplay::machines::conveyor::{self, Conveyor};
    use crate::core::registry::{RecipeDefinition, RecipeInput};
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
            lane: Default::default(),
        });
        grid.machines.insert(input_conv_pos, MachineInstance {
            id: "conveyor".to_string(),
            orientation: Direction::South, // Conveyor pushes South
            machine_type: Machine::Conveyor(input_conveyor),
            power_node: None,
        });

        // Assembler at (0,0,1) facing North (front is (0,0,0), back is (0,0,2))
        let assembler = Assembler {
            active_recipe: Some("ore_to_ingot".to_string()),
            ..Default::default()
        };
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
        // Directly set crafting progress to bypass timing issues in tests
        // Then run updates to complete the crafting
        {
            let mut grid = app.world_mut().resource_mut::<SimulationGrid>();
            if let Some(machine) = grid.machines.get_mut(&assembler_pos) {
                if let Machine::Assembler(assembler) = &mut machine.machine_type {
                    // Set progress just above craft_time to trigger completion
                    assembler.crafting_progress = 0.015; // > 0.01 craft_time
                }
            }
        }

        // Run updates to process crafting completion and ejection
        for _ in 0..5 {
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