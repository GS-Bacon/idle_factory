pub mod messages;
pub mod client;
pub mod server;

use bevy::prelude::*;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Add networking systems and resources
        info!("NetworkPlugin loaded");
    }
}
