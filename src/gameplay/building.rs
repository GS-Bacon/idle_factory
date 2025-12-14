use bevy::prelude::*;
use crate::gameplay::grid::{SimulationGrid, MachineInstance, Direction};
use crate::rendering::chunk::{Chunk, CHUNK_SIZE};
use crate::rendering::meshing::MeshDirty;
use crate::core::config::GameConfig;
use crate::gameplay::interaction::PlayerInteractEvent;

#[derive(Resource, Default)]
pub struct BuildTool {
    pub active_block_id: String,
    pub orientation: Direction,
}

pub fn handle_building(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>, // ★ここが抜けていました！追加しました
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut grid: ResMut<SimulationGrid>,
    mut chunk_query: Query<(Entity, &mut Chunk)>,
    mut commands: Commands,
    mut gizmos: Gizmos,
    config: Res<GameConfig>,
    mut interact_events: EventWriter<PlayerInteractEvent>,
    mut build_tool: ResMut<BuildTool>,
) {
    // ツール切り替え
    if keyboard.just_pressed(KeyCode::Digit1) {
        build_tool.active_block_id = "conveyor".to_string();
        info!("Selected: Conveyor");
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        build_tool.active_block_id = "miner".to_string();
        info!("Selected: Miner");
    }

    if build_tool.active_block_id.is_empty() {
        build_tool.active_block_id = "conveyor".to_string();
    }

    let (_camera, cam_transform) = camera_query.single();
    let ray_origin = cam_transform.translation();
    let ray_dir = cam_transform.forward();

    // チャンク取得
    let (chunk_entity, mut chunk) = if let Ok(res) = chunk_query.get_single_mut() {
        res
    } else {
        return;
    };

    let max_dist = 10.0;
    let step = 0.05;
    let mut current_dist = 0.0;
    
    let mut place_pos: Option<IVec3> = None;
    let mut target_machine_pos: Option<IVec3> = None; 
    let mut last_air_pos: IVec3 = ray_origin.floor().as_ivec3();

    while current_dist < max_dist {
        let pos = ray_origin + ray_dir * current_dist;
        let iblock = pos.floor().as_ivec3();

        if iblock.x >= 0 && iblock.x < CHUNK_SIZE as i32 &&
           iblock.y >= 0 && iblock.y < CHUNK_SIZE as i32 &&
           iblock.z >= 0 && iblock.z < CHUNK_SIZE as i32 {
            
            if let Some(id) = chunk.get_block(iblock.x as usize, iblock.y as usize, iblock.z as usize) {
                if id != "air" {
                    place_pos = Some(last_air_pos);
                    target_machine_pos = Some(iblock);
                    break;
                }
            }
        }
        last_air_pos = iblock;
        current_dist += step;
    }

    // ハイライト
    if let Some(target_pos) = target_machine_pos {
        if config.enable_highlight {
            let center = target_pos.as_vec3() + Vec3::splat(0.5);
            gizmos.cuboid(
                Transform::from_translation(center).with_scale(Vec3::splat(1.01)),
                Color::WHITE,
            );
        }
    }

    // --- 左クリック: ブロック設置 ---
    if let Some(pos) = place_pos {
        if mouse.just_pressed(MouseButton::Left) {
            let is_occupied = if let Some(existing) = chunk.get_block(pos.x as usize, pos.y as usize, pos.z as usize) {
                existing != "air"
            } else { true };

            if !is_occupied {
                let cam_forward = cam_transform.forward();
                let flat_forward = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();
                
                let orientation = if flat_forward.x.abs() > flat_forward.z.abs() {
                    if flat_forward.x > 0.0 { Direction::East } else { Direction::West }
                } else {
                    if flat_forward.z > 0.0 { Direction::South } else { Direction::North }
                };

                let id = build_tool.active_block_id.clone();
                info!("🏗️ Placing {} at {:?} Facing {:?}", id, pos, orientation);
                
                grid.machines.insert(pos, MachineInstance {
                    id: id.clone(),
                    orientation, 
                    inventory: Vec::new(),
                    progress: 0.0,
                });
                
                chunk.set_block(pos.x as usize, pos.y as usize, pos.z as usize, &id);
                commands.entity(chunk_entity).insert(MeshDirty);
            }
        }
    }
    
    // --- 右クリック: インタラクション ---
    if let Some(pos) = target_machine_pos {
        if mouse.just_pressed(MouseButton::Right) {
            interact_events.send(PlayerInteractEvent {
                grid_pos: pos,
                mouse_button: MouseButton::Right,
            });
        }
    }
}