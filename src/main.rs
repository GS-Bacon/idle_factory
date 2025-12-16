use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::window::{WindowPlugin, PresentMode};
use bevy::winit::WinitSettings;
use infinite_voxel_factory::GamePlugin;
use infinite_voxel_factory::rendering::meshing::ChunkMaterialHandle;
use infinite_voxel_factory::network::NetworkPlugin;
use infinite_voxel_factory::core::config::GameConfig;
use std::time::Duration;

fn main() {
    let config = GameConfig::default();

    App::new()
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }))
        .insert_resource(WinitSettings {
            focused_mode: bevy::winit::UpdateMode::reactive_low_power(Duration::from_secs_f64(1.0 / config.max_fps)),
            unfocused_mode: bevy::winit::UpdateMode::reactive_low_power(Duration::from_secs(1)),
        })
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(GamePlugin)
        .add_plugins(NetworkPlugin)
        .add_systems(Startup, (setup_lights, setup_shared_material))
        .run();
}

fn setup_lights(mut commands: Commands) {
    // 環境光で全体を照らす（強い日光は不要）
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 2000.0, // さらに明るく
    });
}

fn setup_shared_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.8,
        reflectance: 0.2,
        ..default()
    });
    commands.insert_resource(ChunkMaterialHandle(handle));
}