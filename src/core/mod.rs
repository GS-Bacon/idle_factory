pub mod accessibility;
pub mod config;
pub mod debug;
pub mod e2e_test;
pub mod hot_reload;
pub mod input;
pub mod modding;
pub mod optimization;
pub mod profile;
pub mod registry;
pub mod resource_pack;
pub mod save_system;
pub mod sound;
pub mod worldgen;

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(config::ConfigPlugin)
            .add_plugins(input::InputPlugin)
            .add_plugins(registry::RegistryPlugin)
            .add_plugins(debug::DebugPlugin)
            .add_plugins(optimization::OptimizationPlugin)
            .add_plugins(modding::ModdingPlugin)
            .add_plugins(save_system::SaveSystemPlugin)
            .add_plugins(hot_reload::HotReloadPlugin)
            .add_plugins(accessibility::AccessibilityPlugin)
            .add_plugins(sound::SoundPlugin)
            .add_plugins(e2e_test::E2ETestPlugin)
            .add_plugins(resource_pack::ResourcePackPlugin)
            .add_plugins(profile::ProfilePlugin)
            .add_plugins(worldgen::WorldGenPlugin);
    }
}