pub mod config;
pub mod input;
pub mod registry;
pub mod debug;

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(config::ConfigPlugin)
            .add_plugins(input::InputPlugin)
            .add_plugins(registry::RegistryPlugin) // ★ここ修正: 関数ではなくPluginを追加
            .add_plugins(debug::DebugPlugin);
    }
}