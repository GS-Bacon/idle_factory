use bevy::prelude::*;

pub mod hud;

// ここで UiPlugin を定義する必要があります
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // HUD（クロスヘア）のシステムを登録
        app.add_systems(Startup, hud::spawn_crosshair);
    }
}