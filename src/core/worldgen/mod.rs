//! ワールド生成システム
//!
//! パーリンノイズを使用した手続き型地形生成。
//! Normal（通常地形）とFlat（フラット）の2種類をサポート。

mod config;
mod generator;
mod layers;
mod noise;

pub use config::*;
pub use generator::{create_generator, ChunkGenerator, FlatGenerator, NormalGenerator};
pub use layers::BlockLayerResolver;

use bevy::prelude::*;

/// ワールド生成プラグイン
pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenConfig>()
            .add_systems(Startup, initialize_worldgen);
    }
}

/// セーブパラメータからワールド生成を初期化
fn initialize_worldgen(
    mut worldgen_config: ResMut<WorldGenConfig>,
    world_params: Res<crate::core::save_system::WorldGenerationParams>,
) {
    worldgen_config.seed = world_params.seed;
    worldgen_config.world_type = world_params.world_type;
    info!(
        "World generation initialized: seed={}, type={:?}",
        worldgen_config.seed, worldgen_config.world_type
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exports() {
        // 公開されている型がアクセス可能か確認
        let _config = WorldGenConfig::default();
        let _world_type = WorldType::Normal;
        let _gen = create_generator(WorldType::Flat);
    }
}
