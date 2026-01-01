//! Target block raycast system

use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::{
    CursorLockState, InteractingFurnace, PlayerCamera, TargetBlock, WorldData, REACH_DISTANCE,
};

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

    // Use DDA (Digital Differential Analyzer) for precise voxel traversal
    // This ensures we check every voxel the ray passes through, in order

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
        if ray_direction.x.abs() < 1e-8 {
            f32::MAX
        } else {
            (1.0 / ray_direction.x).abs()
        },
        if ray_direction.y.abs() < 1e-8 {
            f32::MAX
        } else {
            (1.0 / ray_direction.y).abs()
        },
        if ray_direction.z.abs() < 1e-8 {
            f32::MAX
        } else {
            (1.0 / ray_direction.z).abs()
        },
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

    // Maximum number of steps (prevent infinite loop)
    let max_steps = (REACH_DISTANCE * 2.0) as i32;

    for _ in 0..max_steps {
        // Check current voxel
        if world_data.has_block(current) {
            target.break_target = Some(current);

            // Calculate place position based on last step axis
            let normal = match last_step_axis {
                0 => IVec3::new(-step.x, 0, 0),
                1 => IVec3::new(0, -step.y, 0),
                _ => IVec3::new(0, 0, -step.z),
            };
            target.place_target = Some(current + normal);
            return;
        }

        // Step to next voxel
        if t_max.x < t_max.y && t_max.x < t_max.z {
            if t_max.x > REACH_DISTANCE {
                break;
            }
            current.x += step.x;
            t_max.x += t_delta.x;
            last_step_axis = 0;
        } else if t_max.y < t_max.z {
            if t_max.y > REACH_DISTANCE {
                break;
            }
            current.y += step.y;
            t_max.y += t_delta.y;
            last_step_axis = 1;
        } else {
            if t_max.z > REACH_DISTANCE {
                break;
            }
            current.z += step.z;
            t_max.z += t_delta.z;
            last_step_axis = 2;
        }
    }

    // No block found
    target.break_target = None;
    target.place_target = None;
}
