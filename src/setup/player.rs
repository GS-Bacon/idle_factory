//! Player setup

use crate::components::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;

pub fn setup_player(mut commands: Commands) {
    // Player entity with camera
    commands
        .spawn((
            Player,
            Transform::from_xyz(8.0, 12.0, 20.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Projection::Perspective(PerspectiveProjection {
                    fov: 90.0_f32.to_radians(), // Wider FOV for better responsiveness feel
                    ..default()
                }),
                // Use Reinhard tonemapping (doesn't require tonemapping_luts feature)
                Tonemapping::Reinhard,
                PlayerCamera {
                    pitch: 0.0,
                    yaw: 0.0,
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        });
}
