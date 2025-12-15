// src/lib.rs
pub mod core;
pub mod gameplay;
pub mod rendering;
pub mod ui;
pub mod network;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // 各モジュールのプラグインを登録していく
        app.add_plugins((
            core::CorePlugin,
            rendering::RenderingPlugin,
            gameplay::GameplayPlugin,
            ui::UiPlugin,
        ));
    }
}