//! Block breaking system

use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::{
    BlockType, ChunkMesh, ContinuousActionTimer, ConveyorItemVisual, CursorLockState,
    InputStateResources, Inventory, WorldData, BLOCK_SIZE, CHUNK_SIZE, REACH_DISTANCE,
};
use crate::player::GlobalInventory;
use crate::utils::ray_aabb_intersection;

use super::MachineBreakQueries;

#[allow(clippy::too_many_arguments)]
pub fn block_break(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &crate::PlayerCamera)>,
    machines: MachineBreakQueries,
    mut world_data: ResMut<WorldData>,
    mut inventory: ResMut<Inventory>,
    windows: Query<&Window>,
    item_visual_query: Query<Entity, With<ConveyorItemVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cursor_state: ResMut<CursorLockState>,
    input_resources: InputStateResources,
    mut action_timer: ResMut<ContinuousActionTimer>,
    mut global_inventory: ResMut<GlobalInventory>,
) {
    // Only break blocks when cursor is locked and not paused
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Use InputState to check if block actions are allowed (see CLAUDE.md 入力マトリクス)
    let input_state = input_resources.get_state_with(&cursor_state);
    if !input_state.allows_block_actions() {
        return;
    }

    // Support continuous breaking: first click is instant, then timer-gated
    let can_break = mouse_button.just_pressed(MouseButton::Left)
        || (mouse_button.pressed(MouseButton::Left) && action_timer.break_timer.finished());
    if can_break {
        action_timer.break_timer.reset();
    }

    if !cursor_locked || !can_break {
        return;
    }

    // Skip block break if we just locked the cursor (to avoid accidental destruction on resume click)
    if cursor_state.just_locked {
        cursor_state.just_locked = false;
        return;
    }

    let Ok((camera_transform, _camera)) = camera_query.get_single() else {
        return;
    };

    // Calculate ray from camera using its actual transform
    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Track what we hit (world block, conveyor, miner, crusher, or furnace)
    enum HitType {
        WorldBlock(IVec3),
        Conveyor(Entity),
        Miner(Entity),
        Crusher(Entity),
        Furnace(Entity),
    }
    let mut closest_hit: Option<(HitType, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    // Check world blocks using DDA (Digital Differential Analyzer) for precise traversal
    {
        let mut current = IVec3::new(
            ray_origin.x.floor() as i32,
            ray_origin.y.floor() as i32,
            ray_origin.z.floor() as i32,
        );

        let step = IVec3::new(
            if ray_direction.x >= 0.0 { 1 } else { -1 },
            if ray_direction.y >= 0.0 { 1 } else { -1 },
            if ray_direction.z >= 0.0 { 1 } else { -1 },
        );

        let t_delta = Vec3::new(
            if ray_direction.x.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.x).abs() },
            if ray_direction.y.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.y).abs() },
            if ray_direction.z.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.z).abs() },
        );

        let mut t_max = Vec3::new(
            if ray_direction.x >= 0.0 {
                ((current.x + 1) as f32 - ray_origin.x) / ray_direction.x.abs().max(1e-8)
            } else {
                (ray_origin.x - current.x as f32) / ray_direction.x.abs().max(1e-8)
            },
            if ray_direction.y >= 0.0 {
                ((current.y + 1) as f32 - ray_origin.y) / ray_direction.y.abs().max(1e-8)
            } else {
                (ray_origin.y - current.y as f32) / ray_direction.y.abs().max(1e-8)
            },
            if ray_direction.z >= 0.0 {
                ((current.z + 1) as f32 - ray_origin.z) / ray_direction.z.abs().max(1e-8)
            } else {
                (ray_origin.z - current.z as f32) / ray_direction.z.abs().max(1e-8)
            },
        );

        let max_steps = (REACH_DISTANCE * 2.0) as i32;

        for _ in 0..max_steps {
            if world_data.has_block(current) {
                let block_center = Vec3::new(
                    current.x as f32 + 0.5,
                    current.y as f32 + 0.5,
                    current.z as f32 + 0.5,
                );
                if let Some(hit_t) = ray_aabb_intersection(
                    ray_origin,
                    ray_direction,
                    block_center - Vec3::splat(half_size),
                    block_center + Vec3::splat(half_size),
                ) {
                    if hit_t > 0.0 && hit_t < REACH_DISTANCE {
                        let is_closer = closest_hit.as_ref().is_none_or(|h| hit_t < h.1);
                        if is_closer {
                            closest_hit = Some((HitType::WorldBlock(current), hit_t));
                        }
                        break;
                    }
                }
            }

            if t_max.x < t_max.y && t_max.x < t_max.z {
                if t_max.x > REACH_DISTANCE { break; }
                current.x += step.x;
                t_max.x += t_delta.x;
            } else if t_max.y < t_max.z {
                if t_max.y > REACH_DISTANCE { break; }
                current.y += step.y;
                t_max.y += t_delta.y;
            } else {
                if t_max.z > REACH_DISTANCE { break; }
                current.z += step.z;
                t_max.z += t_delta.z;
            }
        }
    }

    // Check conveyors
    for (entity, _conveyor, conveyor_transform) in machines.conveyor.iter() {
        let conveyor_pos = conveyor_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            conveyor_pos - Vec3::new(half_size, 0.15, half_size),
            conveyor_pos + Vec3::new(half_size, 0.15, half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_hit.as_ref().is_none_or(|h| t < h.1);
                if is_closer {
                    closest_hit = Some((HitType::Conveyor(entity), t));
                }
            }
        }
    }

    // Check miners
    for (entity, _miner, miner_transform) in machines.miner.iter() {
        let miner_pos = miner_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            miner_pos - Vec3::splat(half_size),
            miner_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_hit.as_ref().is_none_or(|h| t < h.1);
                if is_closer {
                    closest_hit = Some((HitType::Miner(entity), t));
                }
            }
        }
    }

    // Check crushers
    for (entity, _crusher, crusher_transform) in machines.crusher.iter() {
        let crusher_pos = crusher_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            crusher_pos - Vec3::splat(half_size),
            crusher_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_hit.as_ref().is_none_or(|h| t < h.1);
                if is_closer {
                    closest_hit = Some((HitType::Crusher(entity), t));
                }
            }
        }
    }

    // Check furnaces
    for (entity, _furnace, furnace_transform) in machines.furnace.iter() {
        let furnace_pos = furnace_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            furnace_pos - Vec3::splat(half_size),
            furnace_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_hit.as_ref().is_none_or(|h| t < h.1);
                if is_closer {
                    closest_hit = Some((HitType::Furnace(entity), t));
                }
            }
        }
    }

    // Handle the hit
    if let Some((hit_type, _)) = closest_hit {
        match hit_type {
            HitType::WorldBlock(pos) => {
                if let Some(block_type) = world_data.remove_block(pos) {
                    info!(category = "BLOCK", action = "break", ?pos, ?block_type, "Block broken");
                    inventory.add_item(block_type, 1);

                    let chunk_coord = WorldData::world_to_chunk(pos);

                    let regenerate_chunk = |coord: IVec2,
                                            commands: &mut Commands,
                                            world_data: &mut WorldData,
                                            meshes: &mut Assets<Mesh>,
                                            materials: &mut Assets<StandardMaterial>| {
                        #[allow(unused_variables)]
                        let old_count = if let Some(old_entities) = world_data.chunk_entities.remove(&coord) {
                            let count = old_entities.len();
                            for entity in old_entities {
                                commands.entity(entity).try_despawn_recursive();
                            }
                            count
                        } else {
                            0
                        };

                        if let Some(new_mesh) = world_data.generate_chunk_mesh(coord) {
                            let mesh_handle = meshes.add(new_mesh);
                            let material = materials.add(StandardMaterial {
                                base_color: Color::WHITE,
                                perceptual_roughness: 0.9,
                                ..default()
                            });

                            let entity = commands.spawn((
                                Mesh3d(mesh_handle),
                                MeshMaterial3d(material),
                                Transform::IDENTITY,
                                ChunkMesh { coord },
                            )).id();

                            world_data.chunk_entities.insert(coord, vec![entity]);

                            #[cfg(debug_assertions)]
                            info!("Regenerated chunk {:?}: despawned {} old, spawned new {:?}", coord, old_count, entity);
                        }
                    };

                    regenerate_chunk(chunk_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);

                    let local_pos = WorldData::world_to_local(pos);
                    let neighbor_coords: Vec<IVec2> = [
                        (local_pos.x == 0, IVec2::new(chunk_coord.x - 1, chunk_coord.y)),
                        (local_pos.x == CHUNK_SIZE - 1, IVec2::new(chunk_coord.x + 1, chunk_coord.y)),
                        (local_pos.z == 0, IVec2::new(chunk_coord.x, chunk_coord.y - 1)),
                        (local_pos.z == CHUNK_SIZE - 1, IVec2::new(chunk_coord.x, chunk_coord.y + 1)),
                    ]
                    .iter()
                    .filter(|(is_boundary, _)| *is_boundary)
                    .map(|(_, coord)| *coord)
                    .filter(|coord| world_data.chunks.contains_key(coord))
                    .collect();

                    for neighbor_coord in neighbor_coords {
                        regenerate_chunk(neighbor_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);
                    }
                }
            }
            HitType::Conveyor(entity) => {
                let item_count = if let Ok((_, conveyor, transform)) = machines.conveyor.get(entity) {
                    let pos = transform.translation();
                    let count = conveyor.items.len();
                    for item in &conveyor.items {
                        if let Some(visual_entity) = item.visual_entity {
                            if item_visual_query.get(visual_entity).is_ok() {
                                commands.entity(visual_entity).despawn();
                            }
                        }
                        // Items on conveyor go to GlobalInventory
                        global_inventory.add_item(item.block_type, 1);
                    }
                    info!(category = "MACHINE", action = "break", machine = "conveyor", ?pos, items_returned = count, "Conveyor broken");
                    count
                } else { 0 };
                let _ = item_count;
                commands.entity(entity).despawn_recursive();
                // Machine block goes to GlobalInventory
                global_inventory.add_item(BlockType::ConveyorBlock, 1);
            }
            HitType::Miner(entity) => {
                info!(category = "MACHINE", action = "break", machine = "miner", "Miner broken");
                commands.entity(entity).despawn_recursive();
                // Machine block goes to GlobalInventory
                global_inventory.add_item(BlockType::MinerBlock, 1);
            }
            HitType::Crusher(entity) => {
                if let Ok((_, crusher, _)) = machines.crusher.get(entity) {
                    if let Some(input_type) = crusher.input_type {
                        if crusher.input_count > 0 {
                            // Items inside machine go to GlobalInventory
                            global_inventory.add_item(input_type, crusher.input_count);
                        }
                    }
                    if let Some(output_type) = crusher.output_type {
                        if crusher.output_count > 0 {
                            global_inventory.add_item(output_type, crusher.output_count);
                        }
                    }
                }
                info!(category = "MACHINE", action = "break", machine = "crusher", "Crusher broken");
                commands.entity(entity).despawn_recursive();
                // Machine block goes to GlobalInventory
                global_inventory.add_item(BlockType::CrusherBlock, 1);
            }
            HitType::Furnace(entity) => {
                if let Ok((_, furnace, _)) = machines.furnace.get(entity) {
                    if furnace.fuel > 0 {
                        // Fuel goes to GlobalInventory
                        global_inventory.add_item(BlockType::Coal, furnace.fuel);
                    }
                    if let Some(input_type) = furnace.input_type {
                        if furnace.input_count > 0 {
                            global_inventory.add_item(input_type, furnace.input_count);
                        }
                    }
                    if let Some(output_type) = furnace.output_type {
                        if furnace.output_count > 0 {
                            global_inventory.add_item(output_type, furnace.output_count);
                        }
                    }
                }
                info!(category = "MACHINE", action = "break", machine = "furnace", "Furnace broken");
                commands.entity(entity).despawn_recursive();
                // Machine block goes to GlobalInventory
                global_inventory.add_item(BlockType::FurnaceBlock, 1);
            }
        }
    }
}
