pub mod assembler;
pub mod conveyor;
pub mod miner;
pub mod render;
use bevy::prelude::*;

// マシン関連のシステムをまとめて登録するプラグイン的関数
pub fn register_machines(app: &mut App) {
    app.add_systems(Update, (
        conveyor::tick_conveyors,
        conveyor::draw_conveyor_guides,
        conveyor::handle_conveyor_interaction,
        miner::tick_miners,
        assembler::tick_assemblers,
        assembler::handle_assembler_interaction,
        render::update_machine_visuals,
    ));
}