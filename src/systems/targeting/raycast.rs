//! Target block raycast system

use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::utils::dda_raycast;
use crate::world::WorldData;
use crate::{CursorLockState, InteractingFurnace, PlayerCamera, TargetBlock, REACH_DISTANCE};

/// Update target block based on player's view direction
pub fn update_target_block(
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    world_data: Res<WorldData>,
    windows: Query<&Window>,
    mut target: ResMut<TargetBlock>,
    interacting_furnace: Res<InteractingFurnace>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't update target while UI is open or paused
    if interacting_furnace.0.is_some() || cursor_state.paused {
        target.break_target = None;
        target.place_target = None;
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;
    if !cursor_locked {
        target.break_target = None;
        target.place_target = None;
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
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
