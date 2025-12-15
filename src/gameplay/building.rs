use bevy::prelude::*;
use crate::gameplay::grid::{SimulationGrid, MachineInstance, Direction, Machine};
use crate::rendering::chunk::{Chunk, CHUNK_SIZE};
use crate::rendering::meshing::MeshDirty;
use crate::core::config::GameConfig;
use crate::gameplay::interaction::PlayerInteractEvent;
use crate::core::registry::BlockRegistry;
use crate::gameplay::machines::{conveyor::Conveyor, miner::Miner, assembler::Assembler};

#[derive(Resource, Default)]
pub struct BuildTool {
    pub active_block_id: String,
    pub orientation: Direction,
}

pub fn handle_building(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut grid: ResMut<SimulationGrid>,
    mut chunk_query: Query<(Entity, &mut Chunk)>,
    mut commands: Commands,
    mut gizmos: Gizmos,
    config: Res<GameConfig>,
    mut interact_events: EventWriter<PlayerInteractEvent>,
    mut build_tool: ResMut<BuildTool>,
    block_registry: Res<BlockRegistry>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        build_tool.active_block_id = "conveyor".to_string();
        info!("Selected: Conveyor");
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        build_tool.active_block_id = "miner".to_string();
        info!("Selected: Miner");
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        build_tool.active_block_id = "assembler".to_string();
        info!("Selected: Assembler");
    }

    if build_tool.active_block_id.is_empty() {
        build_tool.active_block_id = "conveyor".to_string();
    }

    let (_camera, cam_transform) = camera_query.single();
    let ray_origin = cam_transform.translation();
    let ray_dir = cam_transform.forward();

    let (chunk_entity, mut chunk) = if let Ok(res) = chunk_query.get_single_mut() { res } else { return; };

    let max_dist = 10.0;
    let step = 0.05;
    let mut current_dist = 0.0;
    
    let mut target_info: Option<(IVec3, IVec3)> = None;

    'rayloop: while current_dist < max_dist {
        let pos = ray_origin + ray_dir * current_dist;
        let iblock = pos.floor().as_ivec3();

        if iblock.x >= 0 && iblock.x < CHUNK_SIZE as i32 &&
           iblock.y >= 0 && iblock.y < CHUNK_SIZE as i32 &&
           iblock.z >= 0 && iblock.z < CHUNK_SIZE as i32 {
            
            if let Some(id) = chunk.get_block(iblock.x as usize, iblock.y as usize, iblock.z as usize) {
                if id != "air" {
                    if let Some(prop) = block_registry.map.get(id) {
                        let aabb = prop.collision_box; 
                        
                        let local_pos = pos - iblock.as_vec3();

                        if local_pos.x >= aabb[0] && local_pos.x <= aabb[3] &&
                           local_pos.y >= aabb[1] && local_pos.y <= aabb[4] &&
                           local_pos.z >= aabb[2] && local_pos.z <= aabb[5] {
                            
                            let prev_pos = ray_origin + ray_dir * (current_dist - step);
                            let prev_local = prev_pos - iblock.as_vec3();

                            let center = Vec3::new((aabb[0]+aabb[3])/2.0, (aabb[1]+aabb[4])/2.0, (aabb[2]+aabb[5])/2.0);
                            let diff = prev_local - center;
                            let abs_diff = diff.abs();

                            let normal = if abs_diff.x > abs_diff.y && abs_diff.x > abs_diff.z {
                                if diff.x > 0.0 { IVec3::X } else { IVec3::NEG_X }
                            } else if abs_diff.y > abs_diff.x && abs_diff.y > abs_diff.z {
                                if diff.y > 0.0 { IVec3::Y } else { IVec3::NEG_Y }
                            } else {
                                if diff.z > 0.0 { IVec3::Z } else { IVec3::NEG_Z }
                            };

                            let place_pos = iblock + normal;
                            target_info = Some((iblock, place_pos));
                            break 'rayloop;
                        }
                    }
                }
            }
        }
        current_dist += step;
    }

    if let Some((target_pos, _)) = target_info {
        if config.enable_highlight {
            let target_id = chunk.get_block(target_pos.x as usize, target_pos.y as usize, target_pos.z as usize)
                                 .cloned() 
                                 .unwrap_or("air".to_string());
            
            let (size, offset) = if let Some(prop) = block_registry.map.get(&target_id) {
                let box_w = prop.collision_box[3] - prop.collision_box[0];
                let box_h = prop.collision_box[4] - prop.collision_box[1];
                let box_d = prop.collision_box[5] - prop.collision_box[2];
                let center_x = (prop.collision_box[0] + prop.collision_box[3]) / 2.0;
                let center_y = (prop.collision_box[1] + prop.collision_box[4]) / 2.0;
                let center_z = (prop.collision_box[2] + prop.collision_box[5]) / 2.0;
                (Vec3::new(box_w, box_h, box_d), Vec3::new(center_x, center_y, center_z))
            } else {
                (Vec3::ONE, Vec3::splat(0.5))
            };

            let center = target_pos.as_vec3() + offset;
            gizmos.cuboid(
                Transform::from_translation(center).with_scale(size * 1.02),
                Color::WHITE,
            );
        }
    }

    if let Some((_, place_pos)) = target_info {
        if mouse.just_pressed(MouseButton::Left) {
            let is_occupied = if let Some(existing) = chunk.get_block(place_pos.x as usize, place_pos.y as usize, place_pos.z as usize) {
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
                
                let machine_type = match id.as_str() {
                    "conveyor" => Machine::Conveyor(Conveyor::default()),
                    "miner" => Machine::Miner(Miner::default()),
                    "assembler" => Machine::Assembler(Assembler::default()),
                    _ => {
                        error!("Attempted to build unknown machine: {}", id);
                        return;
                    }
                };
                
                info!("🏗️ Placing {} at {:?} Facing {:?}", id, place_pos, orientation);
                
                grid.machines.insert(place_pos, MachineInstance {
                    id: id.clone(),
                    orientation,
                    machine_type,
                });
                
                chunk.set_block(place_pos.x as usize, place_pos.y as usize, place_pos.z as usize, &id);
                commands.entity(chunk_entity).insert(MeshDirty);
            }
        }
    }
    
    if let Some((target_pos, _)) = target_info {
        if mouse.just_pressed(MouseButton::Right) {
            interact_events.send(PlayerInteractEvent {
                grid_pos: target_pos,
                mouse_button: MouseButton::Right,
            });
        }
    }
}