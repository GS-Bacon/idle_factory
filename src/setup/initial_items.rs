//! Initial items setup

use crate::components::Furnace;
use crate::game_spec::INITIAL_EQUIPMENT;
use crate::player::Inventory;
use crate::BLOCK_SIZE;
use bevy::prelude::*;

/// Give player initial items
pub fn setup_initial_items(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut inventory: ResMut<Inventory>,
) {
    // Give initial equipment from game spec
    for (block_type, count) in INITIAL_EQUIPMENT.iter() {
        inventory.add_item(*block_type, *count);
    }
    inventory.selected_slot = 0; // First slot

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
