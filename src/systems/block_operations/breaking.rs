//! Block breaking system with time-based breaking

use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::core::{items, ItemId};
use crate::events::game_events::{BlockBroken, EventSource};
use crate::game_spec::breaking_spec;
use crate::player::PlayerInventory;
use crate::systems::TutorialEvent;
use crate::utils::ray_aabb_intersection;
use crate::world::{DirtyChunks, WorldData};
use crate::{
    BreakingProgress, ConveyorItemVisual, CreativeMode, CursorLockState, InputStateResources,
    TargetBlock, BLOCK_SIZE, PLATFORM_SIZE, REACH_DISTANCE,
};

use super::{BlockBreakEvents, LocalPlayerInventory, MachineBreakQueries};

/// What type of thing we're trying to break
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BreakTarget {
    /// A machine entity with its ItemId
    Machine(Entity, ItemId),
    /// A world block at position
    WorldBlock(IVec3, ItemId),
}

#[allow(clippy::too_many_arguments)]
pub fn block_break(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &crate::PlayerCamera)>,
    machines: MachineBreakQueries,
    mut player_inventory: LocalPlayerInventory,
    windows: Query<&Window>,
    item_visual_query: Query<Entity, With<ConveyorItemVisual>>,
    cursor_state: Res<CursorLockState>,
    input_resources: InputStateResources,
    target_block: Res<TargetBlock>,
    mut world_data: ResMut<WorldData>,
    mut dirty_chunks: ResMut<DirtyChunks>,
    mut breaking_progress: ResMut<BreakingProgress>,
    time: Res<Time>,
    creative_mode: Res<CreativeMode>,
    mut events: BlockBreakEvents,
) {
    // Get player entity before consuming inventory
    let player_entity = player_inventory.entity();
    let Some(mut inventory) = player_inventory.get_mut() else {
        breaking_progress.reset();
        return;
    };
    // Only break blocks when cursor is locked and not paused
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Use InputState to check if block actions are allowed
    let input_state = input_resources.get_state_with(&cursor_state);
    if !input_state.allows_block_actions() || !cursor_locked {
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

    // Get selected tool (now uses ItemId directly)
    let selected_tool = inventory.selected_item_id();
    let tool_multiplier = breaking_spec::get_tool_multiplier(selected_tool);
    let base_time = breaking_spec::get_base_break_time(ItemId::from(block_type));
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

    // Update progress (fast break in creative mode, but not instant)
    if creative_mode.enabled {
        // Fast but not instant - 0.1 seconds per block for smoother control
        breaking_progress.progress += time.delta_secs() / 0.1;
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
            BreakTarget::WorldBlock(pos, broken_block) => {
                // Calculate event source
                let source = player_entity
                    .map(EventSource::Player)
                    .unwrap_or(EventSource::System);
                execute_block_break(
                    pos,
                    broken_block,
                    &mut world_data,
                    &mut dirty_chunks,
                    &mut inventory,
                    &mut events.block_broken,
                    source,
                );
            }
        }
        // Send tutorial event for block breaking
        events.tutorial.send(TutorialEvent::BlockBroken);
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
                closest = Some((BreakTarget::Machine(entity, items::conveyor_block()), t));
            }
        }
    }

    // Check all machines (miner, crusher, furnace)
    for (entity, machine, machine_transform) in machines.machine.iter() {
        let pos = machine_transform.translation();
        let item_id = machine.spec.item_id();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            pos - Vec3::splat(half_size),
            pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE && closest.as_ref().is_none_or(|(_, d)| t < *d) {
                closest = Some((BreakTarget::Machine(entity, item_id), t));
            }
        }
    }

    // Check world block if no machine is closer
    if let Some(break_pos) = target_block.break_target {
        if let Some(item_id) = world_data.get_block(break_pos) {
            // Don't break machines via world data
            if !items::is_machine(item_id) {
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
                    closest = Some((BreakTarget::WorldBlock(break_pos, item_id), dist));
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
    machine_id: ItemId,
    machines: &MachineBreakQueries,
    item_visual_query: &Query<Entity, With<ConveyorItemVisual>>,
    inventory: &mut PlayerInventory,
) {
    if machine_id == items::conveyor_block() {
        if let Ok((_, conveyor, transform)) = machines.conveyor.get(entity) {
            let pos = transform.translation();
            let count = conveyor.items.len();
            for item in &conveyor.items {
                if let Some(visual_entity) = item.visual_entity {
                    if item_visual_query.get(visual_entity).is_ok() {
                        commands.entity(visual_entity).despawn();
                    }
                }
                inventory.add_item_by_id(item.item_id, 1);
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
        inventory.add_item_by_id(items::conveyor_block(), 1);
    } else if machine_id == items::miner_block()
        || machine_id == items::crusher_block()
        || machine_id == items::furnace_block()
    {
        // Return contents from machine slots
        if let Ok((_, machine, _)) = machines.machine.get(entity) {
            // Return fuel
            if machine.slots.fuel > 0 {
                inventory.add_item_by_id(items::coal(), machine.slots.fuel);
            }
            // Return input items
            for input_slot in &machine.slots.inputs {
                if let Some(item_id) = input_slot.item_id {
                    if input_slot.count > 0 {
                        inventory.add_item_by_id(item_id, input_slot.count);
                    }
                }
            }
            // Return output items
            for output_slot in &machine.slots.outputs {
                if let Some(item_id) = output_slot.item_id {
                    if output_slot.count > 0 {
                        inventory.add_item_by_id(item_id, output_slot.count);
                    }
                }
            }
        }
        info!(
            category = "MACHINE",
            action = "break",
            machine = ?machine_id.name(),
            "Machine broken"
        );
        commands.entity(entity).despawn_recursive();
        inventory.add_item_by_id(machine_id, 1);
    }
}

/// Execute world block breaking
fn execute_block_break(
    break_pos: IVec3,
    item_id: ItemId,
    world_data: &mut WorldData,
    dirty_chunks: &mut DirtyChunks,
    inventory: &mut PlayerInventory,
    block_broken_events: &mut crate::events::GuardedEventWriter<BlockBroken>,
    source: EventSource,
) {
    // Remove the block
    world_data.remove_block(break_pos);

    // Add block to inventory
    inventory.add_item_by_id(item_id, 1);

    info!(
        category = "BLOCK",
        action = "break",
        ?break_pos,
        block = ?item_id.name(),
        "Block broken"
    );

    // Mark chunk and neighbors as dirty (mesh will be regenerated by process_dirty_chunks)
    let chunk_coord = WorldData::world_to_chunk(break_pos);
    let local_pos = WorldData::world_to_local(break_pos);
    dirty_chunks.mark_dirty(chunk_coord, local_pos);

    // Send block broken event
    let _ = block_broken_events.send(BlockBroken {
        pos: break_pos,
        block: item_id,
        source,
    });
}
