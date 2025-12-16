use bevy::prelude::*;
use crate::gameplay::grid::{SimulationGrid, MachineInstance, Direction, Machine};
use crate::rendering::chunk::{Chunk, CHUNK_SIZE};
use crate::rendering::meshing::MeshDirty;
use crate::core::config::GameConfig;
use crate::core::registry::BlockRegistry;
use crate::gameplay::machines::{conveyor::Conveyor, miner::Miner, assembler::Assembler};
use crate::gameplay::commands::GameMode;

#[derive(Resource, Default)]
pub struct BuildTool {
    pub active_block_id: String,
    pub orientation: Direction,
}

/// ホログラムプレビューマーカー
#[derive(Component)]
pub struct PlacementHologram;

/// ホログラムの状態を管理するリソース
#[derive(Resource, Default)]
pub struct HologramState {
    pub current_entity: Option<Entity>,
    pub last_position: Option<IVec3>,
}

#[derive(Event)]
pub struct MachinePlacedEvent {
    pub pos: IVec3,
    pub machine_id: String,
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
    mut build_tool: ResMut<BuildTool>,
    block_registry: Res<BlockRegistry>,
    mut machine_placed_events: EventWriter<MachinePlacedEvent>,
    game_mode: Res<GameMode>,
    mut hologram_state: ResMut<HologramState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ホットバーで選択されたアイテムを取得
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

    if let Some((target_pos, place_pos)) = target_info {
        // 左クリック: ブロック破壊（Shift押下時またはクリエイティブモード）
        // 右クリック: ブロック設置
        let is_breaking = mouse.just_pressed(MouseButton::Left) && (
            keyboard.pressed(KeyCode::ShiftLeft) ||
            keyboard.pressed(KeyCode::ShiftRight) ||
            *game_mode == GameMode::Creative
        );
        let is_placing = mouse.just_pressed(MouseButton::Right);

        if is_breaking {
                // ターゲットブロックを破壊
                if target_pos.x >= 0 && target_pos.x < CHUNK_SIZE as i32 &&
                   target_pos.y >= 0 && target_pos.y < CHUNK_SIZE as i32 &&
                   target_pos.z >= 0 && target_pos.z < CHUNK_SIZE as i32 {

                    let target_block = chunk.get_block(target_pos.x as usize, target_pos.y as usize, target_pos.z as usize);
                    if let Some(block_id) = target_block {
                        if block_id != "air" {
                            info!("⛏️ Breaking block '{}' at {:?}", block_id, target_pos);

                            // グリッドから機械を削除
                            grid.machines.remove(&target_pos);

                            // チャンクからブロックを削除
                            chunk.set_block(target_pos.x as usize, target_pos.y as usize, target_pos.z as usize, "air");
                            commands.entity(chunk_entity).insert(MeshDirty);
                        }
                    }
                }
        }

        if is_placing {
            // ブロック設置
            let is_occupied = if let Some(existing) = chunk.get_block(place_pos.x as usize, place_pos.y as usize, place_pos.z as usize) {
                existing != "air"
            } else { true };

            if !is_occupied {
                let cam_forward = cam_transform.forward();
                let flat_forward = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();
                let player_facing_direction = if flat_forward.x.abs() > flat_forward.z.abs() {
                    if flat_forward.x > 0.0 { Direction::East } else { Direction::West }
                } else {
                    if flat_forward.z > 0.0 { Direction::South } else { Direction::North }
                };

                let id = build_tool.active_block_id.clone();

                // 機械ごとに設置方向を決定
                let machine_orientation = match id.as_str() {
                    // コンベア: プレイヤーの視線向きに流れる
                    "conveyor" => player_facing_direction,
                    // その他の機械: プレイヤーの視線と逆向き（出力がプレイヤー向き）
                    _ => player_facing_direction.opposite(),
                };

                let machine_type = match id.as_str() {
                    "conveyor" => Machine::Conveyor(Conveyor::default()),
                    "miner" => Machine::Miner(Miner::default()),
                    "assembler" => Machine::Assembler(Assembler::default()),
                    _ => {
                        error!("Attempted to build unknown machine: {}", id);
                        return;
                    }
                };
                
                info!("🏗️ Placing {} at {:?} Facing {:?}", id, place_pos, machine_orientation);
                
                grid.machines.insert(place_pos, MachineInstance {
                    id: id.clone(),
                    orientation: machine_orientation, // Use the new calculated orientation
                    machine_type,
                    power_node: None, // Initialize the power node placeholder
                });
                
                chunk.set_block(place_pos.x as usize, place_pos.y as usize, place_pos.z as usize, &id);
                commands.entity(chunk_entity).insert(MeshDirty);

                // Emit MachinePlacedEvent
                machine_placed_events.send(MachinePlacedEvent {
                    pos: place_pos,
                    machine_id: id,
                });
            }
        }
    }

    // ホログラム表示の更新
    update_hologram(
        target_info.map(|(_, place_pos)| place_pos),
        &mut hologram_state,
        &mut commands,
        &mut meshes,
        &mut materials,
        &build_tool,
        &chunk,
    );
}

/// ホログラムプレビューを更新
fn update_hologram(
    place_pos: Option<IVec3>,
    hologram_state: &mut HologramState,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    _build_tool: &BuildTool,
    chunk: &Chunk,
) {
    // 位置が変わったか、または位置がなくなったらホログラムを更新
    let needs_update = hologram_state.last_position != place_pos;

    if !needs_update {
        return;
    }

    // 既存のホログラムを削除
    if let Some(entity) = hologram_state.current_entity {
        commands.entity(entity).despawn_recursive();
        hologram_state.current_entity = None;
    }

    hologram_state.last_position = place_pos;

    // 新しいホログラムを生成
    if let Some(pos) = place_pos {
        // 設置可能かチェック
        let is_occupied = if let Some(existing) = chunk.get_block(pos.x as usize, pos.y as usize, pos.z as usize) {
            existing != "air"
        } else {
            true
        };

        // ホログラムの色（設置可能: 緑、不可能: 赤）
        let hologram_color = if is_occupied {
            Color::srgba(1.0, 0.2, 0.2, 0.5) // 赤（設置不可）
        } else {
            Color::srgba(0.2, 1.0, 0.2, 0.5) // 緑（設置可能）
        };

        // シンプルなキューブメッシュを生成
        let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let material = materials.add(StandardMaterial {
            base_color: hologram_color,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        let entity = commands.spawn((
            PlacementHologram,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(pos.as_vec3() + Vec3::splat(0.5)),
        )).id();

        hologram_state.current_entity = Some(entity);
    }
}