//! Player setup

use crate::components::*;
use crate::BlockType;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use std::collections::HashMap;
use strum::IntoEnumIterator;

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create held item 3D cache
    let cube_mesh = meshes.add(Cuboid::new(0.3, 0.3, 0.3)); // Smaller cube for hand
    let mut block_materials = HashMap::new();
    for block_type in BlockType::iter() {
        let material = materials.add(StandardMaterial {
            base_color: block_type.color(),
            ..default()
        });
        block_materials.insert(block_type, material);
    }
    commands.insert_resource(HeldItem3DCache {
        cube_mesh: cube_mesh.clone(),
        materials: block_materials,
    });

    // Player entity with camera
    commands
        .spawn((
            Player,
            PlayerPhysics::default(),
            Transform::from_xyz(8.0, 12.0, 20.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn((
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
                ))
                .with_children(|camera| {
                    // 3D held item display (bottom-right of view)
                    camera.spawn((
                        HeldItem3D,
                        Mesh3d(cube_mesh),
                        MeshMaterial3d::<StandardMaterial>(Handle::default()),
                        NotShadowCaster,
                        Transform::from_xyz(0.5, -0.4, -0.8)
                            .with_rotation(Quat::from_euler(EulerRot::YXZ, -0.3, 0.2, 0.1))
                            .with_scale(Vec3::splat(1.0)),
                        Visibility::Hidden, // Hidden until item selected
                    ));
                });
        });
}
