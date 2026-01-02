//! Block breaking system

use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::{
    BlockType, ContinuousActionTimer, ConveyorItemVisual, CursorLockState,
    InputStateResources, Inventory, BLOCK_SIZE, REACH_DISTANCE,
};
use crate::utils::ray_aabb_intersection;

use super::MachineBreakQueries;

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

    // Track what we hit (conveyor, miner, crusher, or furnace) - no world block breaking
    enum HitType {
        Conveyor(Entity),
        Miner(Entity),
        Crusher(Entity),
        Furnace(Entity),
    }
    let mut closest_hit: Option<(HitType, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

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
                        // Items on conveyor go to Inventory
                        inventory.add_item(item.block_type, 1);
                    }
                    info!(category = "MACHINE", action = "break", machine = "conveyor", ?pos, items_returned = count, "Conveyor broken");
                    count
                } else { 0 };
                let _ = item_count;
                commands.entity(entity).despawn_recursive();
                // Machine block goes to Inventory
                inventory.add_item(BlockType::ConveyorBlock, 1);
            }
            HitType::Miner(entity) => {
                info!(category = "MACHINE", action = "break", machine = "miner", "Miner broken");
                commands.entity(entity).despawn_recursive();
                // Machine block goes to Inventory
                inventory.add_item(BlockType::MinerBlock, 1);
            }
            HitType::Crusher(entity) => {
                if let Ok((_, crusher, _)) = machines.crusher.get(entity) {
                    if let Some(input_type) = crusher.input_type {
                        if crusher.input_count > 0 {
                            // Items inside machine go to Inventory
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
                // Machine block goes to Inventory
                inventory.add_item(BlockType::CrusherBlock, 1);
            }
            HitType::Furnace(entity) => {
                if let Ok((_, furnace, _)) = machines.furnace.get(entity) {
                    if furnace.fuel > 0 {
                        // Fuel goes to Inventory
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
                info!(category = "MACHINE", action = "break", machine = "furnace", "Furnace broken");
                commands.entity(entity).despawn_recursive();
                // Machine block goes to Inventory
                inventory.add_item(BlockType::FurnaceBlock, 1);
            }
        }
    }
}
