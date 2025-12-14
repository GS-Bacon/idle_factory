use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

#[derive(Resource, Debug, Deserialize, Clone)]
pub struct KeyBindings {
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub jump: KeyCode,
    pub descend: KeyCode,
    pub sprint: KeyCode,
}

// デフォルト設定 (読み込み失敗時用)
impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW,
            backward: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            jump: KeyCode::Space,
            descend: KeyCode::ShiftLeft,
            sprint: KeyCode::ControlLeft,
        }
    }
}

pub fn load_keybinds(mut commands: Commands) {
    let path = "assets/keybinds.yaml";
    match fs::read_to_string(path) {
        Ok(content) => {
            // YAMLの文字列をKeyCodeに変換して読み込む
            match serde_yaml::from_str::<KeyBindings>(&content) {
                Ok(bindings) => {
                    info!("Loaded KeyBindings: {:?}", bindings);
                    commands.insert_resource(bindings);
                },
                Err(e) => {
                    error!("Failed to parse keybinds.yaml: {}", e);
                    commands.insert_resource(KeyBindings::default());
                }
            }
        }
        Err(e) => {
            warn!("keybinds.yaml not found ({}), using default.", e);
            commands.insert_resource(KeyBindings::default());
        }
    }
}