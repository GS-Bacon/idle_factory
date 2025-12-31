//! Lighting setup

use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use std::f32::consts::PI;

pub fn setup_lighting(mut commands: Commands) {
    // Directional light with high-quality shadows
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            first_cascade_far_bound: 10.0,
            maximum_distance: 100.0,
            ..default()
        }
        .build(),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });
}
