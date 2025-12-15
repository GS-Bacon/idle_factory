use bevy::prelude::*;
use crate::gameplay::grid::{SimulationGrid, ItemSlot, Machine};
use crate::core::config::GameConfig;
use crate::gameplay::interaction::PlayerInteractEvent;

#[derive(Component, Debug, Clone, Default)]
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
                            item_id: "test_item".to_string(),
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
) {
    let dt = time.delta_secs();
    let max_items = config.max_items_per_conveyor.max(1);
    let item_size = 1.0 / max_items as f32;

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
        let mut accepted = false;

        if let Some(target_machine) = grid.machines.get_mut(&to_pos) {
            let is_facing_each_other = if let Machine::Conveyor(_) = &target_machine.machine_type {
                target_machine.orientation == src_dir.opposite()
            } else {
                false
            };

            if !is_facing_each_other {
                if let Machine::Conveyor(target_conveyor) = &mut target_machine.machine_type {
                    if target_conveyor.inventory.len() < max_items {
                        let min_progress = target_conveyor.inventory.iter()
                            .map(|it| it.progress)
                            .fold(1.0f32, |a, b| a.min(b));
                        
                        if target_conveyor.inventory.is_empty() || min_progress > item_size {
                            target_conveyor.inventory.push(ItemSlot {
                                progress: 0.0,
                                ..item
                            });
                            accepted = true;
                        }
                    }
                }
            }
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