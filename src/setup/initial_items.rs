//! Initial items setup
//!
//! NOTE: Initial equipment is now managed by GlobalInventory (see main.rs).
//! This file only handles spawning initial world objects (like the starter furnace).

use crate::components::Furnace;
use crate::BLOCK_SIZE;
use bevy::prelude::*;

/// Setup initial world objects
///
/// Initial equipment (machines) is added to GlobalInventory in main.rs.
/// This function spawns the starter furnace near the player spawn point.
pub fn setup_initial_items(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));

    // Spawn a furnace near player spawn point (8, 8, 18)
    let furnace_pos = IVec3::new(10, 8, 18);
    let furnace_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.3, 0.3), // Dark reddish-brown for furnace
        ..default()
    });

    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(furnace_material),
        Transform::from_translation(Vec3::new(
            furnace_pos.x as f32 * BLOCK_SIZE + 0.5,
            furnace_pos.y as f32 * BLOCK_SIZE + 0.5,
            furnace_pos.z as f32 * BLOCK_SIZE + 0.5,
        )),
        Furnace::default(),
    ));
}
