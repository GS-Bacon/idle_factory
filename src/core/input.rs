use bevy::prelude::*;

#[derive(Resource)]
pub struct KeyBindings {
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub jump: KeyCode,
    pub descend: KeyCode,
    pub sprint: KeyCode,
}

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

// ★追加: プラグイン定義
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyBindings>();
    }
}