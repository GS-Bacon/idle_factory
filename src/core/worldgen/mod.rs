//! ワールド生成システム
//!
//! Minecraft風のバイオーム・地形生成を提供
//! - 複数ノイズによる自然な地形
//! - バイオームシステム（温度・湿度・大陸性）
//! - 洞窟生成
//! - 鉱石分布

pub mod noise;
pub mod biome;
pub mod terrain;
pub mod caves;
pub mod ores;

use bevy::prelude::*;

use self::biome::BiomeRegistry;
use self::noise::NoiseGenerators;
use self::ores::OreRegistry;

/// ワールド生成プラグイン
pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BiomeRegistry>()
            .init_resource::<OreRegistry>()
            .add_systems(Startup, (
                biome::load_biomes,
                ores::load_ores,
            ));
    }
}

/// ワールド生成の定数
pub mod constants {
    /// 海面の高さ (Y座標)
    pub const SEA_LEVEL: i32 = 64;

    /// ワールドの最低高さ
    pub const MIN_HEIGHT: i32 = -256;

    /// ワールドの最高高さ
    pub const MAX_HEIGHT: i32 = 256;

    /// 岩盤の高さ
    pub const BEDROCK_LEVEL: i32 = -256;

    /// 深層岩が始まる高さ
    pub const DEEPSLATE_LEVEL: i32 = 0;

    /// チャンクサイズ
    pub const CHUNK_SIZE: usize = 32;
}

/// チャンクのブロックを生成する（メインエントリポイント）
pub fn generate_chunk_blocks(chunk_pos: IVec3, seed: u32) -> Vec<String> {
    let generators = NoiseGenerators::new(seed);
    terrain::generate_terrain(&generators, chunk_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        const _: () = assert!(constants::MIN_HEIGHT < constants::SEA_LEVEL);
        const _: () = assert!(constants::SEA_LEVEL < constants::MAX_HEIGHT);
        assert_eq!(constants::BEDROCK_LEVEL, constants::MIN_HEIGHT);
    }
}
