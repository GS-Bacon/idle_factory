use bevy::prelude::*;
use infinite_voxel_factory::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(GamePlugin) // ここでプレイヤーやゲームロジックが読み込まれる
        .add_systems(Startup, setup_lights) // ライトだけセットアップ
        .run();
}

fn setup_lights(mut commands: Commands) {
    // 太陽
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));
}