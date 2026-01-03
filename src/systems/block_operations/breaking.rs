//! Block breaking system with time-based breaking

use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::game_spec::breaking_spec;
use crate::utils::ray_aabb_intersection;
use crate::world::{ChunkMesh, WorldData};
use crate::{
    BlockType, BreakingProgress, ConveyorItemVisual, CreativeMode, CursorLockState,
    InputStateResources, Inventory, TargetBlock, BLOCK_SIZE, CHUNK_SIZE, PLATFORM_SIZE,
    REACH_DISTANCE,
};

use super::MachineBreakQueries;

/// What type of thing we're trying to break
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BreakTarget {
    /// A machine entity with its block type
    Machine(Entity, BlockType),
    /// A world block at position
    WorldBlock(IVec3, BlockType),
}

#[allow(clippy::too_many_arguments)]
pub fn block_break(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &crate::PlayerCamera)>,
    machines: MachineBreakQueries,
    mut inventory: ResMut<Inventory>,
    windows: Query<&Window>,
    item_visual_query: Query<Entity, With<ConveyorItemVisual>>,
    mut cursor_state: ResMut<CursorLockState>,
    input_resources: InputStateResources,
    target_block: Res<TargetBlock>,
    mut world_data: ResMut<WorldData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut breaking_progress: ResMut<BreakingProgress>,
    time: Res<Time>,
    creative_mode: Res<CreativeMode>,
) {
    // Only break blocks when cursor is locked and not paused
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Use InputState to check if block actions are allowed
    let input_state = input_resources.get_state_with(&cursor_state);
    if !input_state.allows_block_actions() || !cursor_locked {
        breaking_progress.reset();
        return;
    }

    // Skip block break if we just locked the cursor
    if cursor_state.just_locked {
        cursor_state.just_locked = false;
        breaking_progress.reset();
        return;
    }

    // Check if left mouse button is pressed
    let is_pressing = mouse_button.pressed(MouseButton::Left);
    if !is_pressing {
        breaking_progress.reset();
        return;
    }

    let Ok((camera_transform, _camera)) = camera_query.get_single() else {
        breaking_progress.reset();
        return;
    };

    // Calculate ray from camera
    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();
    let half_size = BLOCK_SIZE / 2.0;

    // Find the closest target (machine or world block)
    let platform_transform = machines.platform.get_single().ok();
    let current_target = find_break_target(
        ray_origin,
        ray_direction,
        half_size,
        &machines,
        &target_block,
        &world_data,
        platform_transform,
    );

    let Some(target) = current_target else {
        breaking_progress.reset();
        return;
    };

    // Get the block type for timing calculation
    let block_type = match target {
        BreakTarget::Machine(_, bt) => bt,
        BreakTarget::WorldBlock(_, bt) => bt,
    };

    // Get selected tool
    let selected_tool = inventory.selected_block();
    let tool_multiplier = breaking_spec::get_tool_multiplier(selected_tool);
    let base_time = breaking_spec::get_base_break_time(block_type);
    let total_time = base_time * tool_multiplier;

    // Check if target changed
    let target_changed = match target {
        BreakTarget::Machine(entity, _) => breaking_progress.target_entity != Some(entity),
        BreakTarget::WorldBlock(pos, _) => {
            breaking_progress.target_pos != Some(pos) || breaking_progress.is_machine
        }
    };

    if target_changed {
        // Start new breaking session
        breaking_progress.reset();
        breaking_progress.total_time = total_time;
        match target {
            BreakTarget::Machine(entity, _) => {
                breaking_progress.target_entity = Some(entity);
                breaking_progress.is_machine = true;
            }
            BreakTarget::WorldBlock(pos, _) => {
                breaking_progress.target_pos = Some(pos);
                breaking_progress.is_machine = false;
            }
        }
    }

    // Update progress (instant break in creative mode)
    if creative_mode.enabled {
        breaking_progress.progress = 1.0;
    } else {
        let delta = time.delta_secs();
        breaking_progress.progress += delta / total_time;
    }

    // Check if breaking is complete
    if breaking_progress.progress >= 1.0 {
        // Execute the break
        match target {
            BreakTarget::Machine(entity, machine_type) => {
                execute_machine_break(
                    &mut commands,
                    entity,
                    machine_type,
                    &machines,
                    &item_visual_query,
                    &mut inventory,
                );
            }
            BreakTarget::WorldBlock(pos, block_type) => {
                execute_block_break(
                    &mut commands,
                    pos,
                    block_type,
                    &mut world_data,
                    &mut meshes,
                    &mut materials,
                    &mut inventory,
                );
            }
        }
        breaking_progress.reset();
    }
}

/// Find the closest break target (machine or world block)
fn find_break_target(
    ray_origin: Vec3,
    ray_direction: Vec3,
    half_size: f32,
    machines: &MachineBreakQueries,
    target_block: &TargetBlock,
    world_data: &WorldData,
    platform_transform: Option<&Transform>,
) -> Option<BreakTarget> {
    let mut closest: Option<(BreakTarget, f32)> = None;

    // Calculate platform intersection distance (blocks break target if closer)
    let platform_hit_distance: Option<f32> = platform_transform.and_then(|pt| {
        let platform_center = pt.translation;
        let platform_half_x = (PLATFORM_SIZE as f32 * BLOCK_SIZE) / 2.0;
        let platform_half_y = BLOCK_SIZE * 0.1;
        let platform_half_z = platform_half_x;
        let platform_min =
            platform_center - Vec3::new(platform_half_x, platform_half_y, platform_half_z);
        let platform_max =
            platform_center + Vec3::new(platform_half_x, platform_half_y, platform_half_z);
        ray_aabb_intersection(ray_origin, ray_direction, platform_min, platform_max)
            .filter(|&t| t > 0.0 && t < REACH_DISTANCE)
    });

    // Check conveyors
    for (entity, _conveyor, conveyor_transform) in machines.conveyor.iter() {
        let pos = conveyor_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            pos - Vec3::new(half_size, 0.15, half_size),
            pos + Vec3::new(half_size, 0.15, half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE && closest.as_ref().is_none_or(|(_, d)| t < *d) {
                closest = Some((BreakTarget::Machine(entity, BlockType::ConveyorBlock), t));
            }
        }
    }

    // Check miners
    for (entity, _miner, miner_transform) in machines.miner.iter() {
        let pos = miner_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            pos - Vec3::splat(half_size),
            pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE && closest.as_ref().is_none_or(|(_, d)| t < *d) {
                closest = Some((BreakTarget::Machine(entity, BlockType::MinerBlock), t));
            }
        }
    }

    // Check crushers
    for (entity, _crusher, crusher_transform) in machines.crusher.iter() {
        let pos = crusher_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            pos - Vec3::splat(half_size),
            pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE && closest.as_ref().is_none_or(|(_, d)| t < *d) {
                closest = Some((BreakTarget::Machine(entity, BlockType::CrusherBlock), t));
            }
        }
    }

    // Check furnaces
    for (entity, _furnace, furnace_transform) in machines.furnace.iter() {
        let pos = furnace_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            pos - Vec3::splat(half_size),
            pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE && closest.as_ref().is_none_or(|(_, d)| t < *d) {
                closest = Some((BreakTarget::Machine(entity, BlockType::FurnaceBlock), t));
            }
        }
    }

    // Check world block if no machine is closer
    if let Some(break_pos) = target_block.break_target {
        if let Some(block_type) = world_data.get_block(break_pos).copied() {
            // Don't break machines via world data
            if !block_type.is_machine() {
                // Calculate distance to world block
                let block_center = Vec3::new(
                    break_pos.x as f32 * BLOCK_SIZE + 0.5,
                    break_pos.y as f32 * BLOCK_SIZE + 0.5,
                    break_pos.z as f32 * BLOCK_SIZE + 0.5,
                );
                let dist = (block_center - ray_origin).length();

                // Don't allow breaking if platform is closer (prevents mining through platform)
                let platform_blocks =
                    platform_hit_distance.is_some_and(|platform_dist| platform_dist < dist);

                if dist < REACH_DISTANCE
                    && !platform_blocks
                    && closest.as_ref().is_none_or(|(_, d)| dist < *d)
                {
                    closest = Some((BreakTarget::WorldBlock(break_pos, block_type), dist));
                }
            }
        }
    }

    closest.map(|(target, _)| target)
}

/// Execute machine breaking
fn execute_machine_break(
    commands: &mut Commands,
    entity: Entity,
    machine_type: BlockType,
    machines: &MachineBreakQueries,
    item_visual_query: &Query<Entity, With<ConveyorItemVisual>>,
    inventory: &mut Inventory,
) {
    match machine_type {
        BlockType::ConveyorBlock => {
            if let Ok((_, conveyor, transform)) = machines.conveyor.get(entity) {
                let pos = transform.translation();
                let count = conveyor.items.len();
                for item in &conveyor.items {
                    if let Some(visual_entity) = item.visual_entity {
                        if item_visual_query.get(visual_entity).is_ok() {
                            commands.entity(visual_entity).despawn();
                        }
                    }
                    inventory.add_item(item.block_type, 1);
                }
                info!(
                    category = "MACHINE",
                    action = "break",
                    machine = "conveyor",
                    ?pos,
                    items_returned = count,
                    "Conveyor broken"
                );
            }
            commands.entity(entity).despawn_recursive();
            inventory.add_item(BlockType::ConveyorBlock, 1);
        }
        BlockType::MinerBlock => {
            info!(
                category = "MACHINE",
                action = "break",
                machine = "miner",
                "Miner broken"
            );
            commands.entity(entity).despawn_recursive();
            inventory.add_item(BlockType::MinerBlock, 1);
        }
        BlockType::CrusherBlock => {
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
            info!(
                category = "MACHINE",
                action = "break",
                machine = "crusher",
                "Crusher broken"
            );
            commands.entity(entity).despawn_recursive();
            inventory.add_item(BlockType::CrusherBlock, 1);
        }
        BlockType::FurnaceBlock => {
            if let Ok((_, furnace, _)) = machines.furnace.get(entity) {
                if furnace.fuel > 0 {
                    inventory.add_item(BlockType::Coal, furnace.fuel);
                }
                if let Some(input_type) = furnace.input_type {
                    if furnace.input_count > 0 {
                        inventory.add_item(input_type, furnace.input_count);
                    }
                }
                if let Some(output_type) = furnace.output_type {
                    if furnace.output_count > 0 {
                        inventory.add_item(output_type, furnace.output_count);
                    }
                }
            }
            info!(
                category = "MACHINE",
                action = "break",
                machine = "furnace",
                "Furnace broken"
            );
            commands.entity(entity).despawn_recursive();
            inventory.add_item(BlockType::FurnaceBlock, 1);
        }
        _ => {}
    }
}

/// Execute world block breaking
fn execute_block_break(
    commands: &mut Commands,
    break_pos: IVec3,
    block_type: BlockType,
    world_data: &mut WorldData,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    inventory: &mut Inventory,
) {
    // Remove the block
    world_data.remove_block(break_pos);

    // Add block to inventory
    inventory.add_item(block_type, 1);

    info!(
        category = "BLOCK",
        action = "break",
        ?break_pos,
        ?block_type,
        "Block broken"
    );

    // Regenerate chunk mesh
    let chunk_coord = WorldData::world_to_chunk(break_pos);
    regenerate_chunk(chunk_coord, commands, world_data, meshes, materials);

    // Also regenerate neighbor chunks if at boundary
    let local_pos = WorldData::world_to_local(break_pos);
    for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let at_boundary = (dx == -1 && local_pos.x == 0)
            || (dx == 1 && local_pos.x == CHUNK_SIZE - 1)
            || (dz == -1 && local_pos.z == 0)
            || (dz == 1 && local_pos.z == CHUNK_SIZE - 1);

        if at_boundary {
            let neighbor_coord = IVec2::new(chunk_coord.x + dx, chunk_coord.y + dz);
            if world_data.chunks.contains_key(&neighbor_coord) {
                regenerate_chunk(neighbor_coord, commands, world_data, meshes, materials);
            }
        }
    }
}

/// Regenerate a chunk's mesh
fn regenerate_chunk(
    coord: IVec2,
    commands: &mut Commands,
    world_data: &mut WorldData,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
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

        let entity = commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material),
                Transform::IDENTITY,
                ChunkMesh { coord },
            ))
            .id();

        world_data.chunk_entities.insert(coord, vec![entity]);
    }
}
