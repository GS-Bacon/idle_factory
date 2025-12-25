pub mod assembler;
pub mod conveyor;
pub mod miner;
pub mod render;
pub mod debug;
pub mod machine_components;
pub mod recipe_system;
pub mod kinetic_machines;
pub mod splitter;

use bevy::prelude::*;
use crate::ui::main_menu::AppState;

// マシン関連のシステムをまとめて登録するプラグイン的関数
pub fn register_machines(app: &mut App) {
    // レシピシステムとKinetic機械プラグインを追加
    app.add_plugins(recipe_system::RecipeSystemPlugin);
    app.add_plugins(kinetic_machines::KineticMachinesPlugin);

    // InGame時のみマシンシステムを実行
    app.add_systems(Update, (
        conveyor::tick_conveyors,
        conveyor::draw_conveyor_guides,
        conveyor::handle_conveyor_interaction,
        miner::tick_miners,
        assembler::tick_assemblers,
        // Note: Assembler interaction is now handled by MachineUiPlugin
        render::update_machine_visuals,
        debug::draw_machine_io_markers,
    ).run_if(in_state(AppState::InGame)));
}