//! Player setup

use crate::components::*;
use crate::core::items;
use crate::player::{LocalPlayer, PlayerInventory};
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use std::collections::HashMap;

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create held item 3D cache
    let cube_mesh = meshes.add(Cuboid::new(0.3, 0.3, 0.3)); // Smaller cube for hand
    let mut block_materials = HashMap::new();
    for item_id in items::all() {
        let material = materials.add(StandardMaterial {
            base_color: item_id.color(),
            ..default()
        });
        block_materials.insert(item_id, material);
    }
    commands.insert_resource(HeldItem3DCache {
        cube_mesh: cube_mesh.clone(),
        materials: block_materials,
    });

    // Player entity with camera
    let player_entity = commands
        .spawn((
            Player,
            PlayerPhysics::default(),
            PlayerInventory::default(),
            Transform::from_xyz(8.0, 12.0, 20.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Main camera (renders world on layer 0)
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
                    Transform::from_xyz(0.0, 0.7, 0.0), // Eye level higher (2 block player)
                    RenderLayers::layer(0),             // Main world layer
                ))
                .with_children(|camera| {
                    // Overlay camera for held item (renders on layer 1, draws on top)
                    camera.spawn((
                        Camera3d::default(),
                        Camera {
                            order: 1,                            // Render after main camera
                            clear_color: ClearColorConfig::None, // Don't clear (overlay)
                            ..default()
                        },
                        Projection::Perspective(PerspectiveProjection {
                            fov: 90.0_f32.to_radians(),
                            ..default()
                        }),
                        Tonemapping::Reinhard,
                        Transform::default(),
                        RenderLayers::layer(1), // Held item layer
                    ));

                    // 3D held item display (bottom-right of view)
                    camera.spawn((
                        HeldItem3D,
                        Mesh3d(cube_mesh),
                        MeshMaterial3d::<StandardMaterial>(Handle::default()),
                        NotShadowCaster,
                        Transform::from_xyz(0.6, -0.5, -0.8)
                            .with_rotation(Quat::from_euler(EulerRot::YXZ, -0.3, 0.2, 0.1))
                            .with_scale(Vec3::splat(1.0)),
                        Visibility::Hidden,     // Hidden until item selected
                        RenderLayers::layer(1), // Render on overlay layer
                    ));
                });
        })
        .id();
    commands.insert_resource(LocalPlayer(player_entity));
}
