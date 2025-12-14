pub mod conveyor;

use bevy::prelude::*;

// マシン関連のシステムをまとめて登録するプラグイン的関数
pub fn register_machines(app: &mut App) {
    app.add_systems(Update, (
        conveyor::tick_conveyors,
        conveyor::draw_conveyor_guides,
    ));
}