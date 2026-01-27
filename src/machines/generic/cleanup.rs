//! Cleanup and visual feedback systems

use crate::components::{GenericMachineUI, InteractingMachine, Machine};
use crate::systems::cursor;
use bevy::prelude::*;
use bevy::window::{CursorOptions, PrimaryWindow};

/// Cleanup system: clear InteractingMachine if the referenced entity no longer exists
///
/// This handles the case where a machine is despawned while its UI is open.
/// Without this cleanup, the UI would remain in MachineUI state with a dangling entity reference.
pub fn cleanup_invalid_interacting_machine(
    mut interacting: ResMut<InteractingMachine>,
    machine_query: Query<Entity, With<Machine>>,
    mut ui_query: Query<(&GenericMachineUI, &mut Visibility)>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Some(entity) = interacting.0 else {
        return;
    };

    // Check if the entity still exists and is a machine
    if machine_query.get(entity).is_ok() {
        return; // Entity still exists, nothing to cleanup
    }

    // Entity was despawned - clear the interacting machine
    interacting.0 = None;

    // Hide all machine UIs
    for (_ui, mut vis) in ui_query.iter_mut() {
        *vis = Visibility::Hidden;
    }

    // Lock cursor back to gameplay mode
    if let Ok(mut cursor_options) = cursor_query.single_mut() {
        cursor::lock_cursor(&mut cursor_options);
    }
}

/// Visual feedback for machine activity (pulse scale when processing)
pub fn machine_visual_feedback(
    _time: Res<Time>,
    mut machine_query: Query<(&Machine, &mut Transform)>,
) {
    for (machine, mut transform) in machine_query.iter_mut() {
        if machine.progress > 0.0 {
            // Pulse effect while processing
            let pulse = 1.0 + 0.05 * (machine.progress * std::f32::consts::TAU * 2.0).sin();
            transform.scale = Vec3::splat(pulse);
        } else {
            // Reset scale
            transform.scale = Vec3::ONE;
        }
    }
}
