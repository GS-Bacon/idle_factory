// src/main.rs
use bevy::prelude::*;
use infinite_voxel_factory::GamePlugin; // これから作るメインプラグイン

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // ドット絵用にニアレストネイバー補間を設定
        .add_plugins(GamePlugin)
        .run();
}