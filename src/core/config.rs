use bevy::prelude::*;
use serde::Deserialize;

#[derive(Resource, Deserialize, Clone)]
pub struct GameConfig {
    pub mouse_sensitivity: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub enable_highlight: bool,
    pub max_items_per_conveyor: usize,
    pub max_fps: f64,
    pub enable_ui_blur: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.003,
            walk_speed: 5.0,
            run_speed: 10.0,
            enable_highlight: true,
            max_items_per_conveyor: 4,
            max_fps: 60.0,
            enable_ui_blur: true,
        }
    }
}

// ★追加: プラグイン定義
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        // 本来はファイルから読み込むロジックを入れますが、まずはデフォルト値で初期化
        app.init_resource::<GameConfig>();
    }
}