//! Common machine interaction logic
//!
//! This module provides unified functions for machine UI interaction:
//! - Raycast to find closest machine
//! - Open/close UI with cursor handling
//! - Interaction availability checks

use crate::components::{CommandInputState, CursorLockState, InventoryOpen, PlayerCamera};
use crate::systems::set_ui_open_state;
use crate::utils::ray_aabb_intersection;
use crate::BLOCK_SIZE;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

/// Result of a raycast to find the closest machine
pub struct RaycastResult {
    pub entity: Entity,
    pub distance: f32,
}

/// Check if machine interaction is currently allowed
///
/// Returns false if any blocking UI is open (inventory, command input, paused)
pub fn can_interact(
    inventory_open: &InventoryOpen,
    command_state: &CommandInputState,
    cursor_state: &CursorLockState,
) -> bool {
    !inventory_open.0 && !command_state.open && !cursor_state.paused
}

/// Raycast from camera to find the closest machine entity
///
/// Returns the closest machine entity within REACH_DISTANCE, or None if no machine found.
pub fn raycast_closest_machine<T: Component>(
    camera_query: &Query<&GlobalTransform, With<PlayerCamera>>,
    machine_query: &Query<(Entity, &Transform), With<T>>,
    reach_distance: f32,
) -> Option<RaycastResult> {
    let camera_transform = camera_query.get_single().ok()?;
    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    let half_size = BLOCK_SIZE / 2.0;
    let mut closest: Option<RaycastResult> = None;

    for (entity, transform) in machine_query.iter() {
        let pos = transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            pos - Vec3::splat(half_size),
            pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < reach_distance {
                let is_closer = closest.as_ref().is_none_or(|c| t < c.distance);
                if is_closer {
                    closest = Some(RaycastResult {
                        entity,
                        distance: t,
                    });
                }
            }
        }
    }

    closest
}

/// Close machine UI and handle cursor state
///
/// - ESC: Release cursor and show it (exit to menu-like state)
/// - E key: Lock cursor and hide it (return to game)
pub fn close_machine_ui<T: Component>(
    esc_pressed: bool,
    ui_query: &mut Query<&mut Visibility, With<T>>,
    windows: &mut Query<&mut Window>,
    cursor_state: &mut CursorLockState,
) {
    // Hide UI
    if let Ok(mut vis) = ui_query.get_single_mut() {
        *vis = Visibility::Hidden;
    }

    let mut window = windows.single_mut();
    if esc_pressed {
        // ESC: Release pointer lock and show cursor
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    } else {
        // E key: Lock cursor and hide it, prevent inventory toggle
        cursor_state.skip_inventory_toggle = true;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
    set_ui_open_state(false);
}

/// Open machine UI and handle cursor state
///
/// Shows the UI and releases cursor for UI interaction.
pub fn open_machine_ui<T: Component>(
    ui_query: &mut Query<&mut Visibility, With<T>>,
    windows: &mut Query<&mut Window>,
) {
    // Show UI
    if let Ok(mut vis) = ui_query.get_single_mut() {
        *vis = Visibility::Visible;
    }

    // Unlock cursor for UI interaction
    let mut window = windows.single_mut();
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
    set_ui_open_state(true);
}

/// Check if cursor is currently locked (game mode)
pub fn is_cursor_locked(windows: &Query<&mut Window>) -> bool {
    let window = windows.single();
    window.cursor_options.grab_mode != CursorGrabMode::None
}

/// Check if E or ESC was just pressed
pub fn get_close_key_pressed(key_input: &ButtonInput<KeyCode>) -> (bool, bool) {
    (
        key_input.just_pressed(KeyCode::KeyE),
        key_input.just_pressed(KeyCode::Escape),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_interact_all_closed() {
        let inventory = InventoryOpen(false);
        let command = CommandInputState::default();
        let cursor = CursorLockState::default();

        assert!(can_interact(&inventory, &command, &cursor));
    }

    #[test]
    fn test_can_interact_inventory_open() {
        let inventory = InventoryOpen(true);
        let command = CommandInputState::default();
        let cursor = CursorLockState::default();

        assert!(!can_interact(&inventory, &command, &cursor));
    }

    #[test]
    fn test_can_interact_command_open() {
        let inventory = InventoryOpen(false);
        let command = CommandInputState {
            open: true,
            ..Default::default()
        };
        let cursor = CursorLockState::default();

        assert!(!can_interact(&inventory, &command, &cursor));
    }

    #[test]
    fn test_can_interact_paused() {
        let inventory = InventoryOpen(false);
        let command = CommandInputState::default();
        let cursor = CursorLockState {
            paused: true,
            ..Default::default()
        };

        assert!(!can_interact(&inventory, &command, &cursor));
    }
}
