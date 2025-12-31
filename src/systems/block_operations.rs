//! Block placement and breaking systems
//!
//! This module contains the core block interaction systems:
//! - block_break: Breaking world blocks and machines
//! - block_place: Placing blocks and machines

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::{
    BlockType, ChunkMesh, Conveyor, ConveyorItemVisual, ConveyorRotationOffset, ConveyorShape,
    ConveyorVisual, CreativeMode, Crusher, CursorLockState, DeliveryPlatform, Direction, Furnace,
    Inventory, Miner, MachineModels, PlayerCamera, WorldData,
    ContinuousActionTimer, InputStateResources, InputStateResourcesWithCursor,
    BLOCK_SIZE, CHUNK_SIZE, CONVEYOR_BELT_HEIGHT, CONVEYOR_BELT_WIDTH, PLATFORM_SIZE, REACH_DISTANCE,
};
use crate::utils::{auto_conveyor_direction, ray_aabb_intersection, ray_aabb_intersection_with_normal, yaw_to_direction};

/// Bundled machine queries for block_break system (reduces parameter count)
#[derive(SystemParam)]
pub struct MachineBreakQueries<'w, 's> {
    pub conveyor: Query<'w, 's, (Entity, &'static Conveyor, &'static GlobalTransform)>,
    pub miner: Query<'w, 's, (Entity, &'static Miner, &'static GlobalTransform)>,
    pub crusher: Query<'w, 's, (Entity, &'static Crusher, &'static GlobalTransform)>,
    pub furnace: Query<'w, 's, (Entity, &'static Furnace, &'static GlobalTransform)>,
}

/// Bundled machine queries for block_place system (reduces parameter count)
#[derive(SystemParam)]
pub struct MachinePlaceQueries<'w, 's> {
    pub conveyor: Query<'w, 's, &'static Conveyor>,
    pub miner: Query<'w, 's, &'static Miner>,
    pub crusher: Query<'w, 's, (&'static Crusher, &'static Transform)>,
    pub furnace: Query<'w, 's, &'static Transform, With<Furnace>>,
}

#[allow(clippy::too_many_arguments)]
pub fn block_break(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &PlayerCamera)>,
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
        Conveyor(Entity), // entity only - items handled separately
        Miner(Entity),
        Crusher(Entity),
        Furnace(Entity),
    }
    let mut closest_hit: Option<(HitType, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    // Check world blocks using DDA (Digital Differential Analyzer) for precise traversal
    {
        // Current voxel position
        let mut current = IVec3::new(
            ray_origin.x.floor() as i32,
            ray_origin.y.floor() as i32,
            ray_origin.z.floor() as i32,
        );

        // Direction sign for stepping (+1 or -1 for each axis)
        let step = IVec3::new(
            if ray_direction.x >= 0.0 { 1 } else { -1 },
            if ray_direction.y >= 0.0 { 1 } else { -1 },
            if ray_direction.z >= 0.0 { 1 } else { -1 },
        );

        // How far along the ray we need to travel for one voxel step on each axis
        let t_delta = Vec3::new(
            if ray_direction.x.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.x).abs() },
            if ray_direction.y.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.y).abs() },
            if ray_direction.z.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.z).abs() },
        );

        // Distance to next voxel boundary for each axis
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
                // Calculate hit distance
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
                        break; // Found first block
                    }
                }
            }

            // Step to next voxel
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
                    // No auto-select - keep current slot selected

                    // Regenerate the chunk mesh for the affected chunk (with neighbor awareness)
                    let chunk_coord = WorldData::world_to_chunk(pos);

                    // Helper closure to regenerate a chunk mesh
                    let regenerate_chunk = |coord: IVec2,
                                            commands: &mut Commands,
                                            world_data: &mut WorldData,
                                            meshes: &mut Assets<Mesh>,
                                            materials: &mut Assets<StandardMaterial>| {
                        // First despawn old entities BEFORE generating new mesh
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

                    // Regenerate the main chunk
                    regenerate_chunk(chunk_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);

                    // Check if block is at chunk boundary and regenerate neighbor chunks
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
                // Get conveyor items before despawning
                let item_count = if let Ok((_, conveyor, transform)) = machines.conveyor.get(entity) {
                    let pos = transform.translation();
                    let count = conveyor.items.len();
                    // Despawn all item visuals and return items to inventory
                    for item in &conveyor.items {
                        if let Some(visual_entity) = item.visual_entity {
                            if item_visual_query.get(visual_entity).is_ok() {
                                commands.entity(visual_entity).despawn();
                            }
                        }
                        inventory.add_item(item.block_type, 1);
                    }
                    info!(category = "MACHINE", action = "break", machine = "conveyor", ?pos, items_returned = count, "Conveyor broken");
                    count
                } else { 0 };
                let _ = item_count; // Suppress unused warning
                // Use despawn_recursive to also remove arrow marker child
                commands.entity(entity).despawn_recursive();
                // Return conveyor to inventory
                inventory.add_item(BlockType::ConveyorBlock, 1);
            }
            HitType::Miner(entity) => {
                info!(category = "MACHINE", action = "break", machine = "miner", "Miner broken");
                commands.entity(entity).despawn_recursive();
                // Return miner to inventory
                inventory.add_item(BlockType::MinerBlock, 1);
            }
            HitType::Crusher(entity) => {
                // Return crusher contents to inventory before despawning
                if let Ok((_, crusher, _)) = machines.crusher.get(entity) {
                    if let Some(input_type) = crusher.input_type {
                        if crusher.input_count > 0 {
                            inventory.add_item(input_type, crusher.input_count);
                        }
                    }
                    if let Some(output_type) = crusher.output_type {
                        if crusher.output_count > 0 {
                            inventory.add_item(output_type, crusher.output_count);
                        }
                    }
                }
                info!(category = "MACHINE", action = "break", machine = "crusher", "Crusher broken");
                commands.entity(entity).despawn_recursive();
                // Return crusher to inventory
                inventory.add_item(BlockType::CrusherBlock, 1);
            }
            HitType::Furnace(entity) => {
                // Return furnace contents to inventory before despawning
                if let Ok((_, furnace, _)) = machines.furnace.get(entity) {
                    // Return fuel (coal)
                    if furnace.fuel > 0 {
                        inventory.add_item(BlockType::Coal, furnace.fuel);
                    }
                    // Return input ore
                    if let Some(input_type) = furnace.input_type {
                        if furnace.input_count > 0 {
                            inventory.add_item(input_type, furnace.input_count);
                        }
                    }
                    // Return output ingots
                    if let Some(output_type) = furnace.output_type {
                        if furnace.output_count > 0 {
                            inventory.add_item(output_type, furnace.output_count);
                        }
                    }
                }
                info!(category = "MACHINE", action = "break", machine = "furnace", "Furnace broken");
                commands.entity(entity).despawn_recursive();
                // Return furnace to inventory
                inventory.add_item(BlockType::FurnaceBlock, 1);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn block_place(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &PlayerCamera)>,
    machines: MachinePlaceQueries,
    platform_query: Query<&Transform, With<DeliveryPlatform>>,
    mut world_data: ResMut<WorldData>,
    mut inventory: ResMut<Inventory>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Query<&Window>,
    creative_mode: Res<CreativeMode>,
    input_resources: InputStateResourcesWithCursor,
    mut action_timer: ResMut<ContinuousActionTimer>,
    mut rotation: ResMut<ConveyorRotationOffset>,
    machine_models: Res<MachineModels>,
) {
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Use InputState to check if block actions are allowed (see CLAUDE.md 入力マトリクス)
    let input_state = input_resources.get_state();
    if !input_state.allows_block_actions() || !cursor_locked {
        return;
    }

    // Support continuous placing: first click is instant, then timer-gated
    let can_place = mouse_button.just_pressed(MouseButton::Right)
        || (mouse_button.pressed(MouseButton::Right) && action_timer.place_timer.finished());
    if can_place {
        action_timer.place_timer.reset();
    }

    if !can_place {
        return;
    }

    // Check if we have a selected block type with items
    if !inventory.has_selected() {
        return;
    }
    let selected_type = inventory.selected_block().unwrap();

    let Ok((camera_transform, player_camera)) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();
    let half_size = BLOCK_SIZE / 2.0;

    // Check if looking at a furnace or crusher - if so, don't place (let machine UI handle it)
    for furnace_transform in machines.furnace.iter() {
        let furnace_pos = furnace_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            furnace_pos - Vec3::splat(half_size),
            furnace_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                return; // Looking at furnace, let furnace_interact handle it
            }
        }
    }
    for (_, crusher_transform) in machines.crusher.iter() {
        let crusher_pos = crusher_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            crusher_pos - Vec3::splat(half_size),
            crusher_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                return; // Looking at crusher, let crusher_interact handle it
            }
        }
    }

    // Find closest block intersection with hit normal using DDA
    let mut closest_hit: Option<(IVec3, Vec3, f32)> = None;

    {
        // Current voxel position
        let mut current = IVec3::new(
            ray_origin.x.floor() as i32,
            ray_origin.y.floor() as i32,
            ray_origin.z.floor() as i32,
        );

        // Direction sign for stepping (+1 or -1 for each axis)
        let step = IVec3::new(
            if ray_direction.x >= 0.0 { 1 } else { -1 },
            if ray_direction.y >= 0.0 { 1 } else { -1 },
            if ray_direction.z >= 0.0 { 1 } else { -1 },
        );

        // How far along the ray we need to travel for one voxel step on each axis
        let t_delta = Vec3::new(
            if ray_direction.x.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.x).abs() },
            if ray_direction.y.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.y).abs() },
            if ray_direction.z.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.z).abs() },
        );

        // Distance to next voxel boundary for each axis
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

        // Track which axis we stepped on last (for face normal)
        let mut last_step_axis = 0; // 0=x, 1=y, 2=z
        let max_steps = (REACH_DISTANCE * 2.0) as i32;

        for _ in 0..max_steps {
            if world_data.has_block(current) {
                let block_center = Vec3::new(
                    current.x as f32 + 0.5,
                    current.y as f32 + 0.5,
                    current.z as f32 + 0.5,
                );
                if let Some((hit_t, _normal)) = ray_aabb_intersection_with_normal(
                    ray_origin,
                    ray_direction,
                    block_center - Vec3::splat(half_size),
                    block_center + Vec3::splat(half_size),
                ) {
                    if hit_t > 0.0 && hit_t < REACH_DISTANCE {
                        // Use DDA-calculated normal for more accurate placement
                        let dda_normal = match last_step_axis {
                            0 => Vec3::new(-step.x as f32, 0.0, 0.0),
                            1 => Vec3::new(0.0, -step.y as f32, 0.0),
                            _ => Vec3::new(0.0, 0.0, -step.z as f32),
                        };
                        closest_hit = Some((current, dda_normal, hit_t));
                        break;
                    }
                }
            }

            // Step to next voxel
            if t_max.x < t_max.y && t_max.x < t_max.z {
                if t_max.x > REACH_DISTANCE { break; }
                current.x += step.x;
                t_max.x += t_delta.x;
                last_step_axis = 0;
            } else if t_max.y < t_max.z {
                if t_max.y > REACH_DISTANCE { break; }
                current.y += step.y;
                t_max.y += t_delta.y;
                last_step_axis = 1;
            } else {
                if t_max.z > REACH_DISTANCE { break; }
                current.z += step.z;
                t_max.z += t_delta.z;
                last_step_axis = 2;
            }
        }
    }

    // Also check DeliveryPlatform for raycast hit
    if let Ok(platform_transform) = platform_query.get_single() {
        let platform_center = platform_transform.translation;
        let platform_half_x = (PLATFORM_SIZE as f32 * BLOCK_SIZE) / 2.0;
        let platform_half_y = BLOCK_SIZE * 0.1; // 0.2 height / 2
        let platform_half_z = platform_half_x;

        let platform_min = platform_center - Vec3::new(platform_half_x, platform_half_y, platform_half_z);
        let platform_max = platform_center + Vec3::new(platform_half_x, platform_half_y, platform_half_z);

        if let Some((hit_t, normal)) = ray_aabb_intersection_with_normal(
            ray_origin,
            ray_direction,
            platform_min,
            platform_max,
        ) {
            if hit_t > 0.0 && hit_t < REACH_DISTANCE {
                // Convert hit point to block position for placement
                let hit_point = ray_origin + ray_direction * hit_t;
                let hit_block_pos = IVec3::new(
                    hit_point.x.floor() as i32,
                    hit_point.y.floor() as i32,
                    hit_point.z.floor() as i32,
                );
                let is_closer = closest_hit.is_none_or(|h| hit_t < h.2);
                if is_closer {
                    closest_hit = Some((hit_block_pos, normal, hit_t));
                }
            }
        }
    }

    // Place block on the adjacent face
    if let Some((hit_pos, normal, _)) = closest_hit {
        let place_pos = hit_pos + IVec3::new(
            normal.x.round() as i32,
            normal.y.round() as i32,
            normal.z.round() as i32,
        );

        // Don't place if already occupied (check world data and all machine entities)
        if world_data.has_block(place_pos) {
            return;
        }
        // Check if any conveyor occupies this position
        for conveyor in machines.conveyor.iter() {
            if conveyor.position == place_pos {
                return;
            }
        }
        // Check if any miner occupies this position
        for miner in machines.miner.iter() {
            if miner.position == place_pos {
                return;
            }
        }
        // Check if any crusher occupies this position
        for (crusher, _) in machines.crusher.iter() {
            if crusher.position == place_pos {
                return;
            }
        }
        // Check if any furnace occupies this position
        for furnace_transform in machines.furnace.iter() {
            let furnace_pos = IVec3::new(
                (furnace_transform.translation.x / BLOCK_SIZE).floor() as i32,
                (furnace_transform.translation.y / BLOCK_SIZE).floor() as i32,
                (furnace_transform.translation.z / BLOCK_SIZE).floor() as i32,
            );
            if furnace_pos == place_pos {
                return;
            }
        }

        // Consume from inventory (unless in creative mode)
        if !creative_mode.enabled {
            inventory.consume_selected();
        }

        // Get chunk coord for the placed block
        let chunk_coord = WorldData::world_to_chunk(place_pos);

        // Calculate direction from player yaw for conveyors
        let player_facing = yaw_to_direction(player_camera.yaw);

        // For conveyors, use auto-direction based on adjacent machines
        let facing_direction = if selected_type == BlockType::ConveyorBlock {
            // Collect conveyor positions and directions
            let conveyors: Vec<(IVec3, Direction)> = machines.conveyor
                .iter()
                .map(|c| (c.position, c.direction))
                .collect();

            // Collect machine positions (miners, crushers, furnaces)
            let mut machine_positions: Vec<IVec3> = Vec::new();
            for miner in machines.miner.iter() {
                machine_positions.push(miner.position);
            }
            for (crusher, _) in machines.crusher.iter() {
                machine_positions.push(crusher.position);
            }
            for furnace_transform in machines.furnace.iter() {
                machine_positions.push(IVec3::new(
                    furnace_transform.translation.x.floor() as i32,
                    furnace_transform.translation.y.floor() as i32,
                    furnace_transform.translation.z.floor() as i32,
                ));
            }

            // Apply rotation offset (R key)
            let mut dir = auto_conveyor_direction(place_pos, player_facing, &conveyors, &machine_positions);
            for _ in 0..rotation.offset {
                dir = dir.rotate_cw();
            }
            dir
        } else {
            player_facing
        };

        // Helper closure to regenerate a chunk mesh (same pattern as block_break)
        let regenerate_chunk = |coord: IVec2,
                                commands: &mut Commands,
                                world_data: &mut WorldData,
                                meshes: &mut Assets<Mesh>,
                                materials: &mut Assets<StandardMaterial>| {
            // First despawn old entities BEFORE generating new mesh
            if let Some(old_entities) = world_data.chunk_entities.remove(&coord) {
                for entity in old_entities {
                    commands.entity(entity).try_despawn_recursive();
                }
            }

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
            }
        };

        // Spawn entity based on block type
        match selected_type {
            BlockType::MinerBlock => {
                info!(category = "MACHINE", action = "place", machine = "miner", ?place_pos, "Miner placed");
                // Machines are spawned as separate entities, no need to modify world data
                // (they don't occlude terrain blocks)

                let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                let material = materials.add(StandardMaterial {
                    base_color: selected_type.color(),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(cube_mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE + 0.5,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    )),
                    Miner {
                        position: place_pos,
                        ..default()
                    },
                ));
            }
            BlockType::ConveyorBlock => {
                // Machines are spawned as separate entities, no need to modify world data

                // Check for auto-curve: if there's a conveyor in front pointing a different direction
                let front_pos = place_pos + facing_direction.to_ivec3();
                let mut final_shape = ConveyorShape::Straight;
                let final_direction = facing_direction;

                // Find conveyor at front position
                for conv in machines.conveyor.iter() {
                    if conv.position == front_pos {
                        // There's a conveyor in front
                        let front_dir = conv.direction;

                        // If front conveyor points different direction, we need to curve
                        if front_dir != facing_direction {
                            // Determine curve direction based on front conveyor's direction
                            // We want to output in the same direction as the front conveyor
                            let left_of_facing = facing_direction.left();
                            let right_of_facing = facing_direction.right();

                            if front_dir == left_of_facing {
                                // Front conveyor goes left, so we curve left
                                final_shape = ConveyorShape::CornerLeft;
                                // Keep our input direction, but visually we curve left
                            } else if front_dir == right_of_facing {
                                // Front conveyor goes right, so we curve right
                                final_shape = ConveyorShape::CornerRight;
                            }
                            // If front_dir is opposite, keep straight (odd but valid)
                        }
                        break;
                    }
                }

                info!(category = "MACHINE", action = "place", machine = "conveyor", ?place_pos, ?final_direction, ?final_shape, "Conveyor placed");

                let conveyor_pos = Vec3::new(
                    place_pos.x as f32 * BLOCK_SIZE + 0.5,
                    place_pos.y as f32 * BLOCK_SIZE,
                    place_pos.z as f32 * BLOCK_SIZE + 0.5,
                );

                // Try to use glTF model, fallback to procedural mesh
                if let Some(model_handle) = machine_models.get_conveyor_model(final_shape) {
                    // Spawn with glTF model
                    // Note: GlobalTransform and Visibility are required for rendering
                    commands.spawn((
                        SceneRoot(model_handle),
                        Transform::from_translation(conveyor_pos)
                            .with_rotation(final_direction.to_rotation()),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Conveyor {
                            position: place_pos,
                            direction: final_direction,
                            items: Vec::new(),
                            last_output_index: 0,
                            last_input_source: 0,
                            shape: final_shape,
                        },
                        ConveyorVisual,
                    ));
                } else {
                    // Fallback: procedural mesh
                    let conveyor_mesh = meshes.add(Cuboid::new(
                        BLOCK_SIZE * CONVEYOR_BELT_WIDTH,
                        BLOCK_SIZE * CONVEYOR_BELT_HEIGHT,
                        BLOCK_SIZE
                    ));
                    let material = materials.add(StandardMaterial {
                        base_color: selected_type.color(),
                        ..default()
                    });
                    let arrow_mesh = meshes.add(Cuboid::new(BLOCK_SIZE * 0.12, BLOCK_SIZE * 0.03, BLOCK_SIZE * 0.35));
                    let arrow_material = materials.add(StandardMaterial {
                        base_color: Color::srgb(0.9, 0.9, 0.2),
                        ..default()
                    });
                    let belt_y = place_pos.y as f32 * BLOCK_SIZE + CONVEYOR_BELT_HEIGHT / 2.0;
                    commands.spawn((
                        Mesh3d(conveyor_mesh),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(
                            place_pos.x as f32 * BLOCK_SIZE + 0.5,
                            belt_y,
                            place_pos.z as f32 * BLOCK_SIZE + 0.5,
                        )).with_rotation(final_direction.to_rotation()),
                        Conveyor {
                            position: place_pos,
                            direction: final_direction,
                            items: Vec::new(),
                            last_output_index: 0,
                            last_input_source: 0,
                            shape: final_shape,
                        },
                        ConveyorVisual,
                    )).with_children(|parent| {
                        parent.spawn((
                            Mesh3d(arrow_mesh),
                            MeshMaterial3d(arrow_material),
                            Transform::from_translation(Vec3::new(0.0, CONVEYOR_BELT_HEIGHT / 2.0 + 0.02, -0.25)),
                        ));
                    });
                }
                // Reset rotation offset after placing (so next placement uses auto-direction)
                rotation.offset = 0;
            }
            BlockType::CrusherBlock => {
                info!(category = "MACHINE", action = "place", machine = "crusher", ?place_pos, "Crusher placed");
                // Machines are spawned as separate entities, no need to modify world data
                // (they don't occlude terrain blocks)

                let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                let material = materials.add(StandardMaterial {
                    base_color: selected_type.color(),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(cube_mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE + 0.5,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    )),
                    Crusher {
                        position: place_pos,
                        input_type: None,
                        input_count: 0,
                        output_type: None,
                        output_count: 0,
                        progress: 0.0,
                    },
                ));
            }
            BlockType::FurnaceBlock => {
                info!(category = "MACHINE", action = "place", machine = "furnace", ?place_pos, "Furnace placed");
                // Furnace - similar to crusher, spawns entity with Furnace component
                let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                let material = materials.add(StandardMaterial {
                    base_color: selected_type.color(),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(cube_mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE + 0.5,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    )),
                    Furnace::default(),
                ));
            }
            _ => {
                // Regular block - add to world data and regenerate chunk mesh
                info!(category = "BLOCK", action = "place", ?place_pos, block_type = ?selected_type, "Block placed");
                world_data.set_block(place_pos, selected_type);
                regenerate_chunk(chunk_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);

                // Check if block is at chunk boundary and regenerate neighbor chunks
                let local_pos = WorldData::world_to_local(place_pos);
                let neighbor_offsets: [(i32, i32, bool); 4] = [
                    (-1, 0, local_pos.x == 0),           // West boundary
                    (1, 0, local_pos.x == CHUNK_SIZE - 1), // East boundary
                    (0, -1, local_pos.z == 0),           // North boundary
                    (0, 1, local_pos.z == CHUNK_SIZE - 1), // South boundary
                ];

                for (dx, dz, at_boundary) in neighbor_offsets {
                    if at_boundary {
                        let neighbor_coord = IVec2::new(chunk_coord.x + dx, chunk_coord.y + dz);
                        if world_data.chunks.contains_key(&neighbor_coord) {
                            regenerate_chunk(neighbor_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);
                        }
                    }
                }
            }
        }
    }
}
