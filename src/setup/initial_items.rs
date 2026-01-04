//! Initial items setup
//!
//! NOTE: Initial equipment is now managed by GlobalInventory (see main.rs).
//! Initial world objects (furnace, platform) are now given as tutorial rewards.

use bevy::prelude::*;

/// Setup initial world objects
///
/// Initial equipment (machines) is added to GlobalInventory in main.rs.
/// Furnace and platform are now tutorial rewards, so nothing is spawned here.
pub fn setup_initial_items(
    _commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    // Furnace and platform are now tutorial rewards
    // Nothing to spawn here
}
