use bevy::prelude::*;
use std::collections::HashMap;
use crate::gameplay::grid::SimulationGrid;

#[derive(Component)]
pub struct VisualItem {
    pub unique_id: u64,
}

pub fn update_visual_items(
    mut commands: Commands,
    grid: Res<SimulationGrid>,
    visuals: Query<(Entity, &VisualItem)>,
    mut transforms: Query<&mut Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut visual_map: HashMap<u64, Entity> = HashMap::new();
    for (entity, visual) in visuals.iter() {
        visual_map.insert(visual.unique_id, entity);
    }

    for (pos, machine) in &grid.machines {
        for item in &machine.inventory {
            // --- 位置計算ロジック ---
            
            // コンベアの中心
            let center_pos = pos.as_vec3() + Vec3::new(0.5, 0.35, 0.5);
            
            // 現在の進行方向ベクトル
            let current_dir = machine.orientation.to_ivec3().as_vec3();
            
            // どこから来たか？ (Noneなら真後ろから来たことにする)
            let from_dir = item.from_direction.unwrap_or(machine.orientation).to_ivec3().as_vec3();
            
            // ★カーブアニメーション
            // 進行度(0.0~1.0)に合わせて位置を決定する
            let final_pos = if from_dir == current_dir {
                // 直線移動: (Entrance) -> (Exit)
                // Entrance = Center - dir*0.5
                // Exit = Center + dir*0.5
                // Lerp(-0.5, 0.5, progress)
                let offset = current_dir * (item.progress - 0.5);
                center_pos + offset
            } else {
                // コーナー移動: (Side Entrance) -> (Center) -> (Exit)
                // 0.0 ~ 0.5: 入口から中心へ
                // 0.5 ~ 1.0: 中心から出口へ
                
                if item.progress < 0.5 {
                    // 前半: 入口から中心へ
                    // 入口位置: 前のマシンが向いていた方向の逆側から入ってくるわけではなく、
                    // 「前のマシンが向いていた方向」に進んで入ってくる。
                    // 例: 前がEast(右)向き、今がSouth(下)向き。
                    // 入ってくるのは、今のブロックのWest(左)面から、East(右)に向かって入ってくる。
                    // つまり、StartPoint = Center - (from_dir * 0.5)
                    let start_point = center_pos - (from_dir * 0.5);
                    let mid_point = center_pos;
                    
                    // progress 0.0->0.5 を 0.0->1.0 にマップ
                    let t = item.progress * 2.0;
                    start_point.lerp(mid_point, t)
                } else {
                    // 後半: 中心から出口へ
                    let mid_point = center_pos;
                    let end_point = center_pos + (current_dir * 0.5);
                    
                    // progress 0.5->1.0 を 0.0->1.0 にマップ
                    let t = (item.progress - 0.5) * 2.0;
                    mid_point.lerp(end_point, t)
                }
            };

            // --- Visual更新 ---
            if let Some(entity) = visual_map.remove(&item.unique_id) {
                if let Ok(mut transform) = transforms.get_mut(entity) {
                    transform.translation = final_pos;
                }
            } else {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.3, 0.3, 0.3))),
                    MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 1.0))),
                    Transform::from_translation(final_pos),
                    VisualItem {
                        unique_id: item.unique_id,
                    },
                ));
            }
        }
    }

    for entity in visual_map.values() {
        commands.entity(*entity).despawn();
    }
}