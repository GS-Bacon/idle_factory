use crate::core::config::GameConfig;
use crate::gameplay::grid::{ItemSlot, SimulationGrid};
use crate::gameplay::interaction::PlayerInteractEvent;
use bevy::prelude::*;
const CONVEYOR_SPEED: f32 = 1.0;

// ガイド矢印描画 (変更なし)
pub fn draw_conveyor_guides(grid: Res<SimulationGrid>, mut gizmos: Gizmos) {
    for (pos, machine) in &grid.machines {
        if machine.id == "conveyor" {
            let start = pos.as_vec3() + Vec3::new(0.5, 0.25, 0.5);
            let dir = machine.orientation.to_ivec3().as_vec3();
            let end = start + dir * 0.4;
            gizmos.arrow(start, end, Color::WHITE);
        }
    }
}
pub fn handle_conveyor_interaction(
    mut events: EventReader<PlayerInteractEvent>, // イベントを受け取る
    mut grid: ResMut<SimulationGrid>,
    config: Res<GameConfig>,
) {
    for event in events.read() {
        // 右クリック以外は無視
        if event.mouse_button != MouseButton::Right {
            continue;
        }

        // その場所にマシンがあるか？
        if let Some(machine) = grid.machines.get_mut(&event.grid_pos) {
            // コンベアか？ (ここで種類判定を行うことで分離)
            if machine.id == "conveyor" {
                // --- コンベアへのアイテム投入ロジック ---
                let max_items = config.max_items_per_conveyor.max(1);
                let item_size = 1.0 / max_items as f32;

                if machine.inventory.len() < max_items {
                    let new_progress = 0.1;
                    // 衝突チェック
                    let has_collision = machine
                        .inventory
                        .iter()
                        .any(|item| (item.progress - new_progress).abs() < item_size);

                    if !has_collision {
                        info!(
                            "🍎 Conveyor Interaction: Added item at {:?}",
                            event.grid_pos
                        );
                        machine.inventory.push(ItemSlot {
                            item_id: "test_item".to_string(),
                            count: 1,
                            progress: new_progress,
                            unique_id: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_nanos() as u64,
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
    config: Res<GameConfig>, // 設定読み込み
) {
    let dt = time.delta_secs();

    // アイテムサイズ(間隔)を動的に計算: 2個なら0.5, 3個なら0.33...
    let max_items = config.max_items_per_conveyor.max(1);
    let item_size = 1.0 / max_items as f32;

    // 移動リクエスト: (from_pos, to_pos, item, source_orientation)
    let mut transfers: Vec<(IVec3, IVec3, ItemSlot, crate::gameplay::grid::Direction)> = Vec::new();
    let machine_keys: Vec<IVec3> = grid.machines.keys().cloned().collect();

    for pos in machine_keys {
        if let Some(machine) = grid.machines.get_mut(&pos) {
            if machine.id != "conveyor" {
                continue;
            }
            if machine.inventory.is_empty() {
                continue;
            }

            // 1. ソート (出口に近い順 = progressが大きい順)
            machine.inventory.sort_by(|a, b| {
                b.progress
                    .partial_cmp(&a.progress)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // 2. 内部移動ロジック (プルプル防止版)
            // 先頭のアイテムから順に、「進める限界位置」を計算して移動させる
            for i in 0..machine.inventory.len() {
                // このアイテムが最大どこまで進めるか？
                let limit = if i == 0 {
                    // 先頭は 1.0 まで行ける (搬出待ち)
                    1.0
                } else {
                    // 後続は「前のアイテムの位置 - アイテムサイズ」まで
                    let prev_progress = machine.inventory[i - 1].progress;
                    (prev_progress - item_size).max(0.0)
                };

                let item = &mut machine.inventory[i];

                // 進もうとする距離
                let potential_progress = item.progress + CONVEYOR_SPEED * dt;

                // 限界を超えないようにセット (これでプルプルしない)
                item.progress = potential_progress.min(limit);
            }

            // 3. 搬出判定 (先頭のみ)
            if let Some(first_item) = machine.inventory.first() {
                // 完全に端(1.0)に到達している場合のみ搬出を試みる
                if first_item.progress >= 1.0 {
                    let direction = machine.orientation;
                    let target_pos = pos + direction.to_ivec3();

                    // アイテムをクローンし、ソースの向きをセットして転送リストへ
                    let mut item_to_transfer = first_item.clone();
                    // ★重要: 次のコンベアでのアニメーション用に、今のコンベアの向きを記録
                    item_to_transfer.from_direction = Some(direction);

                    transfers.push((pos, target_pos, item_to_transfer, direction));
                }
            }
        }
    }

    // 4. 搬出実行フェーズ
    for (from_pos, to_pos, item, _src_dir) in transfers {
        let mut accepted = false;

        if let Some(target_machine) = grid.machines.get_mut(&to_pos) {
            // 容量チェック
            if target_machine.inventory.len() < max_items {
                // 最後尾との衝突チェック
                // ターゲット内のアイテムはまだソートされていない可能性があるので注意だが、
                // 基本的に追加は末尾(progress最小)に行われる

                // 入口付近が空いているか？
                // targetにある中で一番後ろ(progressが小さい)のアイテムを探す
                let min_progress = target_machine
                    .inventory
                    .iter()
                    .map(|it| it.progress)
                    .fold(1.0f32, |a, b| a.min(b));

                // 入口(0.0)に入ろうとしたとき、前のアイテムが item_size 以上進んでいればOK
                if target_machine.inventory.is_empty() || min_progress > item_size {
                    target_machine.inventory.push(ItemSlot {
                        item_id: item.item_id,
                        count: 1,
                        progress: 0.0,
                        unique_id: item.unique_id,
                        from_direction: item.from_direction, // 向き情報を継承
                    });
                    accepted = true;
                }
            }
        }

        if accepted {
            // 移動成功：元マシンから削除
            if let Some(from_machine) = grid.machines.get_mut(&from_pos) {
                if !from_machine.inventory.is_empty() {
                    from_machine.inventory.remove(0); // 先頭(ソート済みなので0)を削除
                }
            }
        }
        // 受け入れられなかった場合は、progress 1.0 のまま(次のフレームで再トライ)
    }
}
