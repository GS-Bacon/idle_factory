use bevy::prelude::*;
use crate::core::debug::DebugSettings;
use crate::gameplay::grid::{SimulationGrid, Machine};

pub fn draw_machine_io_markers(
    settings: Res<DebugSettings>,
    grid: Res<SimulationGrid>,
    mut gizmos: Gizmos,
) {
    if !settings.is_enabled {
        return;
    }

    for (pos, machine) in &grid.machines {
        let machine_center = pos.as_vec3() + 0.5;

        match &machine.machine_type {
            Machine::Miner(_) => {
                // Output: Front
                let output_pos = machine_center + machine.orientation.to_ivec3().as_vec3() * 0.5;
                gizmos.cuboid(
                    Transform::from_translation(output_pos).with_scale(Vec3::splat(0.25)),
                    Color::srgb(1.0, 0.0, 0.0), // Red
                );
            }
            Machine::Assembler(_) => {
                // Input: Front
                let input_pos = machine_center + machine.orientation.to_ivec3().as_vec3() * 0.5;
                gizmos.cuboid(
                    Transform::from_translation(input_pos).with_scale(Vec3::splat(0.25)),
                    Color::srgb(0.0, 0.0, 1.0), // Blue
                );
                // Output: Back
                let output_pos = machine_center + machine.orientation.opposite().to_ivec3().as_vec3() * 0.5;
                 gizmos.cuboid(
                    Transform::from_translation(output_pos).with_scale(Vec3::splat(0.25)),
                    Color::srgb(1.0, 0.0, 0.0), // Red
                );
            }
            Machine::Conveyor(_) => {
                 // Output: Front
                let output_pos = machine_center + machine.orientation.to_ivec3().as_vec3() * 0.4; // Slightly inside
                 gizmos.cuboid(
                    Transform::from_translation(output_pos).with_scale(Vec3::splat(0.15)),
                    Color::srgb(1.0, 0.0, 0.0), // Red
                );
            }
        }
    }
}
