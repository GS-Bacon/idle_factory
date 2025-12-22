pub mod config;
pub mod input;
pub mod registry;
pub mod debug;
pub mod optimization;
pub mod modding;
pub mod save_system;
pub mod hot_reload;

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(config::ConfigPlugin)
            .add_plugins(input::InputPlugin)
            .add_plugins(registry::RegistryPlugin)
            .add_plugins(debug::DebugPlugin)
            .add_plugins(optimization::OptimizationPlugin)
            .add_plugins(modding::ModdingPlugin)
            .add_plugins(save_system::SaveSystemPlugin)
            .add_plugins(hot_reload::HotReloadPlugin);
    }
}