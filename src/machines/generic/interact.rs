//! Generic machine interaction (open/close UI)

use crate::components::{
    CursorLockState, GenericMachineUI, InteractingMachine, InventoryOpen, Machine, PlayerCamera,
};
use crate::input::{GameAction, InputManager};
use crate::systems::cursor;
use crate::REACH_DISTANCE;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

/// Generic machine interaction (open/close UI)
#[allow(clippy::too_many_arguments)]
pub fn generic_machine_interact(
    input: Res<InputManager>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    machine_query: Query<(Entity, &Transform, &Machine)>,
    mut interacting: ResMut<InteractingMachine>,
    inventory_open: Res<InventoryOpen>,
    mut ui_query: Query<(&GenericMachineUI, &mut Visibility)>,
    mut windows: Query<&mut Window>,
    mut cursor_state: ResMut<CursorLockState>,
) {
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Don't interact if inventory is open
    if inventory_open.0 {
        return;
    }

    let e_pressed = input.just_pressed(GameAction::ToggleInventory);
    let esc_pressed = input.just_pressed(GameAction::Cancel);

    // Close UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        let machine_id = machine_query
            .get(interacting.0.unwrap())
            .map(|(_, _, m)| m.spec.id)
            .unwrap_or("");

        // Hide UI
        for (ui, mut vis) in ui_query.iter_mut() {
            if ui.machine_id == machine_id {
                *vis = Visibility::Hidden;
            }
        }

        interacting.0 = None;

        // Lock cursor (unless ESC)
        if !esc_pressed {
            if let Ok(mut window) = windows.get_single_mut() {
                cursor::lock_cursor(&mut window);
            }
            cursor_state.skip_inventory_toggle = true;
        }
        return;
    }

    // Open UI with right-click when cursor locked
    if !input.just_pressed(GameAction::SecondaryAction) || !cursor_locked {
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_dir = camera_transform.forward().as_vec3();

    // Find closest machine
    let mut closest: Option<(Entity, f32, &'static str)> = None;
    for (entity, transform, machine) in machine_query.iter() {
        let to_machine = transform.translation - ray_origin;
        let dist = to_machine.dot(ray_dir);
        if dist > 0.0 && dist < REACH_DISTANCE {
            let closest_point = ray_origin + ray_dir * dist;
            let diff = (closest_point - transform.translation).length();
            if diff < 0.7 {
                // Within machine hitbox
                if closest.is_none() || dist < closest.unwrap().1 {
                    closest = Some((entity, dist, machine.spec.id));
                }
            }
        }
    }

    if let Some((entity, _, machine_id)) = closest {
        interacting.0 = Some(entity);

        // Show UI
        for (ui, mut vis) in ui_query.iter_mut() {
            if ui.machine_id == machine_id {
                *vis = Visibility::Inherited;
            }
        }

        // Unlock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            cursor::unlock_cursor(&mut window);
        }
    }
}
