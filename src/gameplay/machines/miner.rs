use bevy::prelude::*;
use crate::gameplay::grid::{SimulationGrid, ItemSlot, Machine};
use crate::core::config::GameConfig;
use crate::core::worldgen::ores::is_ore_block;
use crate::rendering::chunk::Chunk;
use serde::{Serialize, Deserialize};

const MINING_SPEED: f32 = 1.0; // 1秒に1個

#[derive(Component, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Miner {
    pub progress: f32,
    /// 採掘対象の鉱石ID（自動検出）
    #[serde(default)]
    pub target_ore: Option<String>,
}

/// 鉱石ブロックIDから採掘アイテムIDに変換
fn ore_to_item(ore_id: &str) -> String {
    match ore_id {
        "coal_ore" | "deepslate_coal_ore" => "coal".to_string(),
        "iron_ore" | "deepslate_iron_ore" => "raw_iron".to_string(),
        "copper_ore" | "deepslate_copper_ore" => "raw_copper".to_string(),
        "gold_ore" | "deepslate_gold_ore" => "raw_gold".to_string(),
        _ => "raw_ore".to_string(),
    }
}

pub fn tick_miners(
    mut grid: ResMut<SimulationGrid>,
    time: Res<Time>,
    config: Res<GameConfig>,
    chunks: Query<&Chunk>,
) {
    let dt = time.delta_secs();
    let max_items_on_conveyor = config.max_items_per_conveyor.max(1);
    let item_size = 1.0 / max_items_on_conveyor as f32;

    let mut outputs: Vec<(IVec3, IVec3, ItemSlot)> = Vec::new();

    // Collect positions and orientations to avoid borrowing issues
    let miner_info: Vec<(IVec3, crate::gameplay::grid::Direction, Option<String>)> = grid.machines.iter()
        .filter_map(|(pos, machine)| {
            if let Machine::Miner(miner) = &machine.machine_type {
                Some((*pos, machine.orientation, miner.target_ore.clone()))
            } else {
                None
            }
        })
        .collect();

    for (pos, orientation, _target_ore) in miner_info {
        if let Some(machine) = grid.machines.get_mut(&pos) {
            if let Machine::Miner(miner) = &mut machine.machine_type {
                // 鉱石未検出の場合は検出を試みる
                if miner.target_ore.is_none() {
                    // Minerの下のブロックをチェック
                    let below_pos = pos - IVec3::Y;
                    if let Some(ore_id) = find_ore_at_position(&chunks, below_pos) {
                        miner.target_ore = Some(ore_id);
                    }
                }

                // 採掘対象がない場合はスキップ
                let Some(ore_id) = &miner.target_ore else {
                    continue;
                };

                // 1. 採掘進行
                miner.progress += MINING_SPEED * dt;

                // 2. 完了判定
                if miner.progress >= 1.0 {
                    let target_pos = pos + orientation.to_ivec3();
                    let item_id = ore_to_item(ore_id);

                    let new_item = ItemSlot {
                        item_id,
                        count: 1,
                        progress: 0.0,
                        unique_id: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64,
                        from_direction: Some(orientation),
                        lane: Default::default(),
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
        } else if let Some(miner_machine) = grid.machines.get_mut(&miner_pos) {
            if let Machine::Miner(miner) = &mut miner_machine.machine_type {
                miner.progress = 1.0;
            }
        }
    }
}

/// 指定位置のブロックを取得し、鉱石ブロックならそのIDを返す
fn find_ore_at_position(chunks: &Query<&Chunk>, world_pos: IVec3) -> Option<String> {
    const CHUNK_SIZE: i32 = 32;

    // ワールド座標からチャンク座標を計算
    let chunk_x = world_pos.x.div_euclid(CHUNK_SIZE);
    let chunk_y = world_pos.y.div_euclid(CHUNK_SIZE);
    let chunk_z = world_pos.z.div_euclid(CHUNK_SIZE);

    // ローカル座標を計算
    let local_x = world_pos.x.rem_euclid(CHUNK_SIZE) as usize;
    let local_y = world_pos.y.rem_euclid(CHUNK_SIZE) as usize;
    let local_z = world_pos.z.rem_euclid(CHUNK_SIZE) as usize;

    // チャンクを探す
    for chunk in chunks.iter() {
        if chunk.position == IVec3::new(chunk_x, chunk_y, chunk_z) {
            let idx = (local_y * CHUNK_SIZE as usize * CHUNK_SIZE as usize)
                + (local_z * CHUNK_SIZE as usize)
                + local_x;

            if idx < chunk.blocks.len() {
                let block_id = &chunk.blocks[idx];
                if is_ore_block(block_id) {
                    return Some(block_id.clone());
                }
            }
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ore_to_item() {
        assert_eq!(ore_to_item("iron_ore"), "raw_iron");
        assert_eq!(ore_to_item("deepslate_iron_ore"), "raw_iron");
        assert_eq!(ore_to_item("coal_ore"), "coal");
        assert_eq!(ore_to_item("copper_ore"), "raw_copper");
        assert_eq!(ore_to_item("gold_ore"), "raw_gold");
        assert_eq!(ore_to_item("unknown_ore"), "raw_ore");
    }
}