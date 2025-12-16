use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use infinite_voxel_factory::GamePlugin;
use infinite_voxel_factory::rendering::meshing::ChunkMaterialHandle;
use infinite_voxel_factory::network::NetworkPlugin; // Add this line

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(GamePlugin)
        .add_plugins(NetworkPlugin) // Add this line
        .add_systems(Startup, (setup_lights, setup_shared_material))
        .run();
}

fn setup_lights(mut commands: Commands) {
    // 環境光で全体を照らす（強い日光は不要）
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0, // 全体的に明るく
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