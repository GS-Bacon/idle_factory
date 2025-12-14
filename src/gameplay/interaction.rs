use bevy::prelude::*;

// プレイヤーがワールドに対して行ったインタラクションを表すイベント
#[derive(Event)]
pub struct PlayerInteractEvent {
    pub grid_pos: IVec3,      // どこを
    pub mouse_button: MouseButton, // どのボタンで
    // 将来的には「誰が(Entity)」「何を(Item)」持っているか、などもここに含める
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerInteractEvent>();
    }
}