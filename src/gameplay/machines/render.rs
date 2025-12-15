use bevy::prelude::*;
use std::collections::HashMap;
use crate::gameplay::grid::SimulationGrid;

#[derive(Component)]
pub struct VisualMachine {
    pub grid_pos: IVec3,
}

pub fn update_machine_visuals(
    mut commands: Commands,
    grid: Res<SimulationGrid>,
    visuals: Query<(Entity, &VisualMachine)>,
    asset_server: Res<AssetServer>,
) {
    let mut visual_map: HashMap<IVec3, Entity> = HashMap::new();
    for (entity, visual) in visuals.iter() {
        visual_map.insert(visual.grid_pos, entity);
    }

    for (pos, machine) in &grid.machines {
        if visual_map.remove(pos).is_some() {
            continue;
        }

        let model_handle = match machine.id.as_str() {
            "miner" => asset_server.load("models/miner.glb#Scene0"),
            "conveyor" => asset_server.load("models/conveyor.glb#Scene0"),
            _ => continue,
        };

        let translation = pos.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
        
        // ★修正: EastとWestの値を入れ替えました
        // モデルのデフォルトが South(+Z) 向きであることを前提とした回転計算
        let rotation = match machine.orientation {
            // North (-Z) : 反対側へ180度
            crate::gameplay::grid::Direction::North => Quat::from_rotation_y(std::f32::consts::PI), 
            
            // South (+Z) : そのまま (0度)
            crate::gameplay::grid::Direction::South => Quat::from_rotation_y(0.0),
            
            // East (+X) : +Z から +90度 (左回り)
            // 修正前: -PI/2 -> 修正後: +PI/2
            crate::gameplay::grid::Direction::East  => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            
            // West (-X) : +Z から -90度 (右回り)
            // 修正前: +PI/2 -> 修正後: -PI/2
            crate::gameplay::grid::Direction::West  => Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
        };

        commands.spawn((
            SceneRoot(model_handle),
            Transform::from_translation(translation).with_rotation(rotation),
            VisualMachine { grid_pos: *pos },
        ));
    }

    for entity in visual_map.values() {
        commands.entity(*entity).despawn_recursive();
    }
}