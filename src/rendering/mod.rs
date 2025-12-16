use bevy::prelude::*;

pub mod chunk;
pub mod meshing;
pub mod voxel_loader; // ★追加
use chunk::Chunk;
use meshing::MeshDirty;
pub mod models;
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<voxel_loader::VoxelAssets>()
            .add_systems(Startup, (setup_test_chunk, voxel_loader::load_vox_assets))
            .add_systems(Update, meshing::update_chunk_mesh);
    }
}

fn setup_test_chunk(mut commands: Commands) {
    let mut chunk = Chunk::new(IVec3::ZERO);

    // 修正: 床を Y=0 の1層だけにする
    for x in 0..chunk::CHUNK_SIZE {
        for z in 0..chunk::CHUNK_SIZE {
            chunk.set_block(x, 0, z, "dirt");
        }
    }

    commands.spawn((
        chunk,
        MeshDirty, 
        Transform::default(),
        Visibility::default(),
    ));
}