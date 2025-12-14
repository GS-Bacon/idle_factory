use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

#[derive(Resource, Debug, Deserialize, Clone)]
pub struct GameConfig {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub fov: f32,
    pub mouse_sensitivity: f32,
    #[serde(default = "default_true")] 
    pub enable_highlight: bool,
    // ★追加: コンベアの最大積載数 (デフォルト2)
    #[serde(default = "default_max_items")]
    pub max_items_per_conveyor: usize,
}

fn default_true() -> bool { true }
fn default_max_items() -> usize { 2 }

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            walk_speed: 10.0,
            run_speed: 20.0,
            fov: 75.0,
            mouse_sensitivity: 0.002,
            enable_highlight: true,
            max_items_per_conveyor: 2,
        }
    }
}

pub fn load_config(mut commands: Commands) {
    let path = "assets/config.yaml";
    match fs::read_to_string(path) {
        Ok(content) => {
            let config: GameConfig = serde_yaml::from_str(&content).unwrap_or_default();
            commands.insert_resource(config);
        }
        Err(_) => {
            commands.insert_resource(GameConfig::default());
        }
    }
}