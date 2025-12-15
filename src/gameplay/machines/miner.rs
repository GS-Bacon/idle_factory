use bevy::prelude::*;
use crate::gameplay::grid::{SimulationGrid, ItemSlot, Machine};
use crate::core::config::GameConfig;

const MINING_SPEED: f32 = 1.0; // 1秒に1個

#[derive(Component, Debug, Clone, Default)]
pub struct Miner {
    pub progress: f32,
}

pub fn tick_miners(
    mut grid: ResMut<SimulationGrid>,
    time: Res<Time>,
    config: Res<GameConfig>,
) {
    let dt = time.delta_secs();
    let max_items_on_conveyor = config.max_items_per_conveyor.max(1);
    let item_size = 1.0 / max_items_on_conveyor as f32;

    let mut outputs: Vec<(IVec3, IVec3, ItemSlot)> = Vec::new();
    
    // Collect positions and orientations to avoid borrowing issues
    let miner_info: Vec<(IVec3, crate::gameplay::grid::Direction)> = grid.machines.iter()
        .filter_map(|(pos, machine)| {
            if let Machine::Miner(_) = &machine.machine_type {
                Some((*pos, machine.orientation))
            } else {
                None
            }
        })
        .collect();

    for (pos, orientation) in miner_info {
        if let Some(machine) = grid.machines.get_mut(&pos) {
            if let Machine::Miner(miner) = &mut machine.machine_type {
                // 1. 採掘進行
                miner.progress += MINING_SPEED * dt;

                // 2. 完了判定
                if miner.progress >= 1.0 {
                    let target_pos = pos + orientation.to_ivec3();
                    
                    let new_item = ItemSlot {
                        item_id: "raw_ore".to_string(),
                        count: 1,
                        progress: 0.0,
                        unique_id: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64,
                        from_direction: Some(orientation),
                    };

                    outputs.push((pos, target_pos, new_item));
                }
            }
        }
    }

    // 3. 搬出実行
    for (miner_pos, target_pos, item) in outputs {
        let mut success = false;
        
        if let Some(target_machine) = grid.machines.get_mut(&target_pos) {
            // Check if the target is a conveyor and has space
            if let Machine::Conveyor(conveyor) = &mut target_machine.machine_type {
                if conveyor.inventory.len() < max_items_on_conveyor {
                    let min_progress = conveyor.inventory.iter()
                        .map(|it| it.progress)
                        .fold(1.0f32, |a, b| a.min(b));
                    
                    if conveyor.inventory.is_empty() || min_progress > item_size {
                        conveyor.inventory.push(item);
                        success = true;
                    }
                }
            }
        }

        if success {
            if let Some(miner_machine) = grid.machines.get_mut(&miner_pos) {
                if let Machine::Miner(miner) = &mut miner_machine.machine_type {
                    miner.progress = 0.0;
                }
            }
        } else {
            if let Some(miner_machine) = grid.machines.get_mut(&miner_pos) {
                if let Machine::Miner(miner) = &mut miner_machine.machine_type {
                    miner.progress = 1.0; 
                }
            }
        }
    }
}