//! Player setup

use crate::components::*;
use crate::core::items;
use crate::player::{LocalPlayer, PlayerInventory};
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::NotShadowCaster;
use bevy::post_process::bloom::Bloom;
use bevy::post_process::dof::{DepthOfField, DepthOfFieldMode};
use bevy::prelude::*;
use bevy::render::view::Hdr;
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
            // Miniature view effect: tilt-shift style with shallow depth of field
            parent
                .spawn((
                    Camera3d::default(),
                    Hdr, // Enable HDR for DepthOfField and Bloom effects
                    Projection::Perspective(PerspectiveProjection {
                        fov: 90.0_f32.to_radians(), // Wider FOV for better responsiveness feel
                        ..default()
                    }),
                    // Use Reinhard tonemapping (doesn't require tonemapping_luts feature)
                    Tonemapping::Reinhard,
                    // Depth of field for miniature/tilt-shift effect
                    // Note: Lower aperture_f_stops = stronger bokeh blur
                    // F/1.8 gives CoC ≈ 0.1px (invisible), F/0.125 gives CoC ≈ 1.4px (visible)
                    DepthOfField {
                        mode: DepthOfFieldMode::Bokeh, // Hexagonal bokeh for cinematic look
                        focal_distance: 10.0,          // Focus slightly closer for tilt-shift
                        aperture_f_stops: 1.0 / 8.0, // 0.125 - unrealistic but needed for visible bokeh
                        max_depth: 50.0,             // Limit blur range
                        ..default()
                    },
                    // Soft bloom for dreamy miniature feel
                    Bloom {
                        intensity: 0.15, // Subtle bloom
                        ..default()
                    },
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
