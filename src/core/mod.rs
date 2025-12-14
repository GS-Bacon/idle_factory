use bevy::prelude::*;

pub mod registry;
pub mod config;
pub mod input; // 追加

use registry::BlockRegistry;
use config::GameConfig;
use input::KeyBindings; // 追加

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<BlockRegistry>()
            .init_resource::<GameConfig>()
            .init_resource::<KeyBindings>() // 追加
            
            // 読み込みシステムを追加
            .add_systems(PreStartup, (
                config::load_config,
                input::load_keybinds, // 追加
            ))
            .add_systems(Startup, registry::load_block_registry);
    }
}