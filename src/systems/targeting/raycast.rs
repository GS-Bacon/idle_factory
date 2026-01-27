//! Target block raycast system

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::utils::dda_raycast;
use crate::world::WorldData;
use crate::{CursorLockState, InteractingMachine, PlayerCamera, TargetBlock, REACH_DISTANCE};

/// Update target block based on player's view direction
pub fn update_target_block(
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    world_data: Res<WorldData>,
    cursor_query: Query<&CursorOptions, With<PrimaryWindow>>,
    mut target: ResMut<TargetBlock>,
    interacting_machine: Res<InteractingMachine>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't update target while UI is open or paused
    if interacting_machine.0.is_some() || cursor_state.paused {
        target.break_target = None;
        target.place_target = None;
        return;
    }

    let cursor_locked = cursor_query
        .single()
        .map(|c| c.grab_mode != CursorGrabMode::None)
        .unwrap_or(false);
    if !cursor_locked {
        target.break_target = None;
        target.place_target = None;
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Use DDA raycast to find the first block
    if let Some(hit) = dda_raycast(ray_origin, ray_direction, REACH_DISTANCE, |pos| {
        world_data.has_block(pos)
    }) {
        target.break_target = Some(hit.position);
        target.place_target = Some(hit.position + hit.normal);
    } else {
        target.break_target = None;
        target.place_target = None;
    }
}
