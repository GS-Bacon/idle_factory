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
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
    });

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.0,
            shadow_depth_bias: 0.05,
            shadow_normal_bias: 0.05,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));
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