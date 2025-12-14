use bevy::prelude::*;
// ★修正: Direction はここには不要なので削除しました
use crate::gameplay::grid::{SimulationGrid, ItemSlot};
use crate::core::config::GameConfig;

const MINING_SPEED: f32 = 1.0; // 1秒に1個

pub fn tick_miners(
    mut grid: ResMut<SimulationGrid>,
    time: Res<Time>,
    config: Res<GameConfig>,
) {
    let dt = time.delta_secs();
    let max_items = config.max_items_per_conveyor.max(1);
    let item_size = 1.0 / max_items as f32;

    let machine_keys: Vec<IVec3> = grid.machines.keys().cloned().collect();
    let mut outputs: Vec<(IVec3, IVec3, ItemSlot)> = Vec::new();

    for pos in machine_keys {
        if let Some(machine) = grid.machines.get_mut(&pos) {
            if machine.id != "miner" { continue; }

            // 1. 採掘進行
            machine.progress += MINING_SPEED * dt;

            // 2. 完了判定
            if machine.progress >= 1.0 {
                // 出力先: Minerが向いている方向の隣
                let target_pos = pos + machine.orientation.to_ivec3();
                
                let new_item = ItemSlot {
                    item_id: "raw_ore".to_string(),
                    count: 1,
                    progress: 0.0,
                    unique_id: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64,
                    // ★重要: Minerの向きを「アイテムが来た方向」としてセット
                    from_direction: Some(machine.orientation), 
                };

                outputs.push((pos, target_pos, new_item));
            }
        }
    }

    // 3. 搬出実行
    for (miner_pos, target_pos, item) in outputs {
        let mut success = false;

        if let Some(target_machine) = grid.machines.get_mut(&target_pos) {
            // 容量チェック
            if target_machine.inventory.len() < max_items {
                // コンベアの入口が空いているかチェック
                let min_progress = target_machine.inventory.iter()
                    .map(|it| it.progress)
                    .fold(1.0f32, |a, b| a.min(b));
                
                // アイテムサイズ分の隙間があれば投入
                if target_machine.inventory.is_empty() || min_progress > item_size {
                    target_machine.inventory.push(item);
                    success = true;
                }
            }
        }

        if success {
            if let Some(miner) = grid.machines.get_mut(&miner_pos) {
                miner.progress = 0.0;
            }
        } else {
            // 詰まっている場合、進捗を1.0で維持（待機）
            if let Some(miner) = grid.machines.get_mut(&miner_pos) {
                miner.progress = 1.0; 
            }
        }
    }
}