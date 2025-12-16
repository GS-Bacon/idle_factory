use bevy::prelude::*;
use crate::gameplay::grid::{SimulationGrid, ItemSlot, Machine};
use crate::core::config::GameConfig;
use crate::core::registry::RecipeRegistry;
use crate::gameplay::interaction::PlayerInteractEvent;
use crate::gameplay::machines::assembler;
use serde::{Serialize, Deserialize};

#[derive(Component, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Conveyor {
    pub inventory: Vec<ItemSlot>,
}

const CONVEYOR_SPEED: f32 = 1.0;

pub fn draw_conveyor_guides(_grid: Res<SimulationGrid>, _gizmos: Gizmos) {
    // Drawing is disabled
}

pub fn handle_conveyor_interaction(
    mut events: EventReader<PlayerInteractEvent>,
    mut grid: ResMut<SimulationGrid>,
    config: Res<GameConfig>,
) {
    for event in events.read() {
        if event.mouse_button != MouseButton::Right { continue; }

        if let Some(machine) = grid.machines.get_mut(&event.grid_pos) {
            if let Machine::Conveyor(conveyor) = &mut machine.machine_type {
                let max_items = config.max_items_per_conveyor.max(1);
                let item_size = 1.0 / max_items as f32;

                if conveyor.inventory.len() < max_items {
                    let new_progress = 0.1;
                    let has_collision = conveyor.inventory.iter().any(|item| {
                        (item.progress - new_progress).abs() < item_size
                    });

                    if !has_collision {
                        info!("🍎 Conveyor Interaction: Added item at {:?}", event.grid_pos);
                        conveyor.inventory.push(ItemSlot {
                            item_id: "raw_ore".to_string(), // Changed to raw_ore for testing
                            count: 1,
                            progress: new_progress,
                            unique_id: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64,
                            from_direction: None,
                        });
                    } else {
                        info!("🚫 Conveyor Interaction: Space occupied.");
                    }
                } else {
                    info!("🚫 Conveyor Interaction: Full.");
                }
            }
        }
    }
}

pub fn tick_conveyors(
    mut grid: ResMut<SimulationGrid>,
    time: Res<Time>,
    config: Res<GameConfig>,
    recipes: Res<RecipeRegistry>,
) {
    let dt = time.delta_secs();
    let max_items_on_conveyor = config.max_items_per_conveyor.max(1);
    let item_size = 1.0 / max_items_on_conveyor as f32;

    let mut transfers = Vec::new();
    let machine_keys: Vec<IVec3> = grid.machines.keys().cloned().collect();

    for pos in &machine_keys {
        if let Some(machine) = grid.machines.get_mut(pos) {
            if let Machine::Conveyor(conveyor) = &mut machine.machine_type {
                if conveyor.inventory.is_empty() { continue; }

                conveyor.inventory.sort_by(|a, b| b.progress.partial_cmp(&a.progress).unwrap_or(std::cmp::Ordering::Equal));

                for i in 0..conveyor.inventory.len() {
                    let limit = if i == 0 { 1.0 } else {
                        (conveyor.inventory[i - 1].progress - item_size).max(0.0)
                    };
                    let item = &mut conveyor.inventory[i];
                    item.progress = (item.progress + CONVEYOR_SPEED * dt).min(limit);
                }

                if let Some(first_item) = conveyor.inventory.first() {
                    if first_item.progress >= 1.0 {
                        info!("[Test-Debug] Conveyor at {:?} has item ready for transfer.", pos);
                        let mut item_to_transfer = first_item.clone();
                        item_to_transfer.from_direction = Some(machine.orientation);
                        transfers.push((*pos, item_to_transfer, machine.orientation));
                    }
                }
            }
        }
    }

    for (from_pos, item, src_dir) in transfers {
        let to_pos = from_pos + src_dir.to_ivec3();
        info!("[Test-Debug] Attempting transfer from {:?} to {:?}.", from_pos, to_pos);
        let mut accepted = false;

        if let Some(target_machine) = grid.machines.get_mut(&to_pos) {
            match &mut target_machine.machine_type {
                Machine::Conveyor(target_conveyor) => {
                    info!("[Test-Debug] Target at {:?} is a Conveyor.", to_pos);
                    let is_facing_each_other = target_machine.orientation == src_dir.opposite();
                    if !is_facing_each_other {
                        if target_conveyor.inventory.len() < max_items_on_conveyor {
                            let min_progress = target_conveyor.inventory.iter()
                                .map(|it| it.progress)
                                .fold(1.0f32, |a, b| a.min(b));
                            
                            if target_conveyor.inventory.is_empty() || min_progress > item_size {
                                target_conveyor.inventory.push(ItemSlot { progress: 0.0, ..item });
                                accepted = true;
                            }
                        }
                    }
                }
                Machine::Assembler(target_assembler) => {
                    info!("[Test-Debug] Target at {:?} is an Assembler. Assembler orientation: {:?}, Conveyor src_dir: {:?}", to_pos, target_machine.orientation, src_dir);
                    // Assembler accepts from its front, which is opposite to its orientation
                    if target_machine.orientation.opposite() == src_dir {
                        info!("[Test-Debug] Orientations match for front input. Checking inventory.");
                        // レシピチェック: アイテムが任意のレシピに使えるかチェック
                        if assembler::can_accept_item(&item.item_id, &recipes) {
                            if target_assembler.input_inventory.len() < 10 { // TODO: make configurable
                                target_assembler.input_inventory.push(ItemSlot { progress: 0.0, ..item });
                                accepted = true;
                                info!("[Test-Debug] Item accepted by assembler.");
                            } else {
                                info!("[Test-Debug] Assembler input inventory full.");
                            }
                        } else {
                            info!("[Test-Debug] Item '{}' cannot be used in any recipe. Rejected.", item.item_id);
                        }
                    } else {
                        info!("[Test-Debug] Orientations do not match for front input.");
                    }
                }
                Machine::Miner(_) => {
                    // Do nothing, can't push into a miner
                }
            }
        } else {
            info!("[Test-Debug] No machine found at target position {:?}", to_pos);
        }

        if accepted {
            if let Some(from_machine) = grid.machines.get_mut(&from_pos) {
                if let Machine::Conveyor(from_conveyor) = &mut from_machine.machine_type {
                    if !from_conveyor.inventory.is_empty() {
                        from_conveyor.inventory.remove(0);
                    }
                }
            }
        }
    }
}