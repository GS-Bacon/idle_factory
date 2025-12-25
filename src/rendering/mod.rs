use bevy::prelude::*;

pub mod chunk;
pub mod meshing;
pub mod voxel_loader;
pub mod models;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<voxel_loader::VoxelAssets>()
            .add_systems(Startup, voxel_loader::load_vox_assets)
            .add_systems(Update, meshing::update_chunk_mesh);
    }
}

// 注: setup_test_chunkは削除
// チャンク生成はcore::optimization::update_chunks_around_playerで行う