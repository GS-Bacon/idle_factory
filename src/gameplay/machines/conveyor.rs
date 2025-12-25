use bevy::prelude::*;
use crate::gameplay::grid::{SimulationGrid, ItemSlot, Machine, ConveyorLane, Direction};
use crate::core::config::GameConfig;
use crate::core::registry::RecipeRegistry;
use crate::gameplay::interaction::PlayerInteractEvent;
use crate::gameplay::machines::assembler;
use serde::{Serialize, Deserialize};

/// コンベアベルト - Factorio風の両側レーンシステム対応
///
/// 各コンベアは左右2つのレーンを持ち、それぞれ独立してアイテムを搬送する。
/// - 正面から合流: 交互に左右レーンに分配
/// - 横から合流（サイドローディング）: 合流方向に応じたレーンに挿入
///   - 左から合流 → 左レーンに挿入
///   - 右から合流 → 右レーンに挿入
#[derive(Component, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Conveyor {
    /// 全アイテムリスト（lane属性で左右を区別）
    pub inventory: Vec<ItemSlot>,
    /// 正面合流時の次レーン（交互に振り分け）
    #[serde(default)]
    pub next_lane_for_front: ConveyorLane,
}

impl Conveyor {
    /// 指定レーンのアイテム数を取得
    pub fn count_items_in_lane(&self, lane: ConveyorLane) -> usize {
        self.inventory.iter().filter(|item| item.lane == lane).count()
    }

    /// 指定レーンのアイテムを進捗順にソートして取得
    pub fn get_items_in_lane(&self, lane: ConveyorLane) -> Vec<&ItemSlot> {
        let mut items: Vec<_> = self.inventory.iter()
            .filter(|item| item.lane == lane)
            .collect();
        items.sort_by(|a, b| b.progress.partial_cmp(&a.progress).unwrap_or(std::cmp::Ordering::Equal));
        items
    }

    /// サイドローディング: 合流方向からレーンを決定
    /// - 左から来た場合 → 左レーン
    /// - 右から来た場合 → 右レーン
    pub fn determine_lane_for_side_load(conveyor_orientation: Direction, from_direction: Direction) -> ConveyorLane {
        // コンベアの向きに対して、どちら側から来たかを判定
        match conveyor_orientation {
            Direction::North => match from_direction {
                Direction::West => ConveyorLane::Left,
                Direction::East => ConveyorLane::Right,
                _ => ConveyorLane::Left, // 正面/背面の場合はデフォルト
            },
            Direction::South => match from_direction {
                Direction::East => ConveyorLane::Left,
                Direction::West => ConveyorLane::Right,
                _ => ConveyorLane::Left,
            },
            Direction::East => match from_direction {
                Direction::North => ConveyorLane::Left,
                Direction::South => ConveyorLane::Right,
                _ => ConveyorLane::Left,
            },
            Direction::West => match from_direction {
                Direction::South => ConveyorLane::Left,
                Direction::North => ConveyorLane::Right,
                _ => ConveyorLane::Left,
            },
        }
    }
}

const CONVEYOR_SPEED: f32 = 1.0;
/// 1レーンあたりの最大アイテム数
const MAX_ITEMS_PER_LANE: usize = 4;

pub fn draw_conveyor_guides(_grid: Res<SimulationGrid>, _gizmos: Gizmos) {
    // Drawing is disabled
}

pub fn handle_conveyor_interaction(
    mut events: EventReader<PlayerInteractEvent>,
    mut grid: ResMut<SimulationGrid>,
    _config: Res<GameConfig>,
) {
    for event in events.read() {
        if event.mouse_button != MouseButton::Right { continue; }

        if let Some(machine) = grid.machines.get_mut(&event.grid_pos) {
            if let Machine::Conveyor(conveyor) = &mut machine.machine_type {
                let item_size = 1.0 / MAX_ITEMS_PER_LANE as f32;

                // 交互にレーンを選択
                let target_lane = conveyor.next_lane_for_front;
                let lane_count = conveyor.count_items_in_lane(target_lane);

                if lane_count < MAX_ITEMS_PER_LANE {
                    let new_progress = 0.1;
                    let has_collision = conveyor.inventory.iter()
                        .filter(|item| item.lane == target_lane)
                        .any(|item| (item.progress - new_progress).abs() < item_size);

                    if !has_collision {
                        debug!("🍎 Conveyor Interaction: Added item to {:?} lane at {:?}", target_lane, event.grid_pos);
                        conveyor.inventory.push(ItemSlot {
                            item_id: "raw_ore".to_string(),
                            count: 1,
                            progress: new_progress,
                            unique_id: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64,
                            from_direction: None,
                            lane: target_lane,
                        });
                        // 次回は反対側のレーン
                        conveyor.next_lane_for_front = target_lane.opposite();
                    } else {
                        debug!("🚫 Conveyor Interaction: Space occupied in {:?} lane.", target_lane);
                    }
                } else {
                    debug!("🚫 Conveyor Interaction: {:?} lane is full.", target_lane);
                }
            }
        }
    }
}

/// コンベアのティック処理（両側レーン対応）
///
/// 各レーンは独立して処理され、アイテムは自分のレーン内でのみ衝突判定を行う。
pub fn tick_conveyors(
    mut grid: ResMut<SimulationGrid>,
    time: Res<Time>,
    _config: Res<GameConfig>,
    recipes: Res<RecipeRegistry>,
) {
    let dt = time.delta_secs();
    let item_size = 1.0 / MAX_ITEMS_PER_LANE as f32;

    let mut transfers: Vec<(IVec3, ItemSlot, Direction)> = Vec::new();
    let machine_keys: Vec<IVec3> = grid.machines.keys().cloned().collect();

    for pos in &machine_keys {
        if let Some(machine) = grid.machines.get_mut(pos) {
            if let Machine::Conveyor(conveyor) = &mut machine.machine_type {
                if conveyor.inventory.is_empty() { continue; }

                // 各レーンを独立して処理
                for lane in [ConveyorLane::Left, ConveyorLane::Right] {
                    // このレーンのアイテムを進捗順にソート
                    let mut lane_indices: Vec<usize> = conveyor.inventory.iter()
                        .enumerate()
                        .filter(|(_, item)| item.lane == lane)
                        .map(|(i, _)| i)
                        .collect();
                    lane_indices.sort_by(|&a, &b| {
                        conveyor.inventory[b].progress
                            .partial_cmp(&conveyor.inventory[a].progress)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    // レーン内のアイテムを順に進める
                    for (rank, &idx) in lane_indices.iter().enumerate() {
                        let limit = if rank == 0 {
                            1.0
                        } else {
                            let prev_idx = lane_indices[rank - 1];
                            (conveyor.inventory[prev_idx].progress - item_size).max(0.0)
                        };
                        conveyor.inventory[idx].progress =
                            (conveyor.inventory[idx].progress + CONVEYOR_SPEED * dt).min(limit);
                    }

                    // このレーンの先頭アイテムが転送可能かチェック
                    if let Some(&first_idx) = lane_indices.first() {
                        if conveyor.inventory[first_idx].progress >= 1.0 {
                            let mut item_to_transfer = conveyor.inventory[first_idx].clone();
                            item_to_transfer.from_direction = Some(machine.orientation);
                            transfers.push((*pos, item_to_transfer, machine.orientation));
                        }
                    }
                }
            }
        }
    }

    // 転送処理
    for (from_pos, item, src_dir) in transfers {
        let to_pos = from_pos + src_dir.to_ivec3();
        let mut accepted = false;

        if let Some(target_machine) = grid.machines.get_mut(&to_pos) {
            match &mut target_machine.machine_type {
                Machine::Conveyor(target_conveyor) => {
                    let is_facing_each_other = target_machine.orientation == src_dir.opposite();
                    if !is_facing_each_other {
                        // 横から来た場合はサイドローディング、正面/背面からは交互レーン
                        let target_lane = if src_dir == target_machine.orientation.opposite() {
                            // 正面から来た場合: 交互に振り分け
                            let lane = target_conveyor.next_lane_for_front;
                            target_conveyor.next_lane_for_front = lane.opposite();
                            lane
                        } else {
                            // サイドローディング: 来た方向に応じたレーン
                            Conveyor::determine_lane_for_side_load(target_machine.orientation, src_dir)
                        };

                        let lane_count = target_conveyor.count_items_in_lane(target_lane);
                        if lane_count < MAX_ITEMS_PER_LANE {
                            let min_progress = target_conveyor.inventory.iter()
                                .filter(|it| it.lane == target_lane)
                                .map(|it| it.progress)
                                .fold(1.0f32, |a, b| a.min(b));

                            if lane_count == 0 || min_progress > item_size {
                                target_conveyor.inventory.push(ItemSlot {
                                    progress: 0.0,
                                    lane: target_lane,
                                    ..item
                                });
                                accepted = true;
                            }
                        }
                    }
                }
                Machine::Assembler(target_assembler) => {
                    // Assembler accepts from its front
                    if target_machine.orientation.opposite() == src_dir
                        && assembler::can_accept_item(&item.item_id, &recipes)
                        && target_assembler.input_inventory.len() < 10
                    {
                        target_assembler.input_inventory.push(ItemSlot { progress: 0.0, ..item });
                        accepted = true;
                    }
                }
                Machine::Miner(_) => {
                    // Can't push into a miner
                }
            }
        }

        if accepted {
            if let Some(from_machine) = grid.machines.get_mut(&from_pos) {
                if let Machine::Conveyor(from_conveyor) = &mut from_machine.machine_type {
                    // 転送したアイテムを削除（unique_idで特定）
                    from_conveyor.inventory.retain(|it| it.unique_id != item.unique_id);
                }
            }
        }
    }
}