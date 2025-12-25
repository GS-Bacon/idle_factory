//! チャンク生成実装
//!
//! Normal（パーリンノイズ）とFlat（フラット）の2種類のジェネレーター

use bevy::prelude::IVec3;

use super::config::{WorldGenConfig, WorldType};
use super::layers::BlockLayerResolver;
use super::noise::TerrainNoise;
use crate::rendering::chunk::CHUNK_SIZE;

/// チャンク生成トレイト
pub trait ChunkGenerator: Send + Sync {
    /// チャンクのブロックデータを生成
    fn generate(&self, chunk_pos: IVec3, config: &WorldGenConfig) -> Vec<String>;
}

/// 通常地形ジェネレーター（パーリンノイズ使用）
pub struct NormalGenerator;

impl NormalGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NormalGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkGenerator for NormalGenerator {
    fn generate(&self, chunk_pos: IVec3, config: &WorldGenConfig) -> Vec<String> {
        let noise = TerrainNoise::new(config.seed, &config.noise_params);
        let size = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
        let mut blocks = vec!["air".to_string(); size];

        let world_x_offset = chunk_pos.x * CHUNK_SIZE as i32;
        let world_y_offset = chunk_pos.y * CHUNK_SIZE as i32;
        let world_z_offset = chunk_pos.z * CHUNK_SIZE as i32;

        let bedrock_floor = config.terrain.min_y + config.bedrock_depth as i32;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = world_x_offset + x as i32;
                let world_z = world_z_offset + z as i32;

                // この列の地表高さを取得
                let surface_height = noise.get_height(world_x, world_z, config);

                for y in 0..CHUNK_SIZE {
                    let world_y = world_y_offset + y as i32;

                    // ワールド範囲外はスキップ
                    if world_y < config.terrain.min_y || world_y > config.terrain.max_y {
                        continue;
                    }

                    let block_id = BlockLayerResolver::get_block(
                        world_y,
                        surface_height,
                        bedrock_floor,
                        config.soil_depth,
                    );

                    let idx = (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x;
                    blocks[idx] = block_id.to_string();
                }
            }
        }

        blocks
    }
}

/// フラットワールドジェネレーター
pub struct FlatGenerator;

impl ChunkGenerator for FlatGenerator {
    fn generate(&self, chunk_pos: IVec3, config: &WorldGenConfig) -> Vec<String> {
        let size = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
        let mut blocks = vec!["air".to_string(); size];

        let world_y_offset = chunk_pos.y * CHUNK_SIZE as i32;
        let flat_config = &config.flat_config;

        // レイヤー境界を計算
        let mut current_y = config.terrain.min_y;
        let layer_heights: Vec<(i32, i32, &str)> = flat_config
            .layers
            .iter()
            .map(|(block_id, thickness)| {
                let start = current_y;
                let end = current_y + *thickness as i32;
                current_y = end;
                (start, end, block_id.as_str())
            })
            .collect();

        for y in 0..CHUNK_SIZE {
            let world_y = world_y_offset + y as i32;

            // このY座標のブロックを決定
            let block_id = layer_heights
                .iter()
                .find(|(start, end, _)| world_y >= *start && world_y < *end)
                .map(|(_, _, id)| *id)
                .unwrap_or("air");

            // XZ平面全体に同じブロックを配置
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let idx = (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x;
                    blocks[idx] = block_id.to_string();
                }
            }
        }

        blocks
    }
}

/// ワールドタイプに応じたジェネレーターを作成
pub fn create_generator(world_type: WorldType) -> Box<dyn ChunkGenerator> {
    match world_type {
        WorldType::Normal => Box::new(NormalGenerator::new()),
        WorldType::Flat => Box::new(FlatGenerator),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_generator_deterministic() {
        let gen = NormalGenerator::new();
        let config = WorldGenConfig {
            seed: 12345,
            ..Default::default()
        };

        let blocks1 = gen.generate(IVec3::ZERO, &config);
        let blocks2 = gen.generate(IVec3::ZERO, &config);

        assert_eq!(blocks1, blocks2);
    }

    #[test]
    fn test_normal_generator_different_chunks() {
        let gen = NormalGenerator::new();
        let config = WorldGenConfig {
            seed: 12345,
            ..Default::default()
        };

        // 地表を含むY=2のチャンク（base_height=64, CHUNK_SIZE=32なので Y=64〜95）
        let blocks1 = gen.generate(IVec3::new(0, 2, 0), &config);
        let blocks2 = gen.generate(IVec3::new(1, 2, 0), &config);

        // 隣接チャンクは異なるはず（地表の起伏により差が出る）
        assert_ne!(blocks1, blocks2);
    }

    #[test]
    fn test_flat_generator_uniform_layers() {
        let gen = FlatGenerator;
        let config = WorldGenConfig {
            world_type: WorldType::Flat,
            ..Default::default()
        };

        // Y=-64 〜 -32 のチャンク（フラットレイヤーを含む）
        let blocks = gen.generate(IVec3::new(0, -2, 0), &config);

        // 同じY座標のブロックは全て同じはず
        for y in 0..CHUNK_SIZE {
            let first_block = &blocks[y * CHUNK_SIZE * CHUNK_SIZE];
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let idx = (y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x;
                    assert_eq!(
                        &blocks[idx], first_block,
                        "Block at ({}, {}, {}) differs from first block in layer",
                        x, y, z
                    );
                }
            }
        }
    }

    #[test]
    fn test_create_generator() {
        let normal_gen = create_generator(WorldType::Normal);
        let flat_gen = create_generator(WorldType::Flat);

        let config = WorldGenConfig::default();

        // 両方とも正しいサイズのブロック配列を返す
        let normal_blocks = normal_gen.generate(IVec3::ZERO, &config);
        let flat_blocks = flat_gen.generate(IVec3::ZERO, &config);

        assert_eq!(normal_blocks.len(), CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        assert_eq!(flat_blocks.len(), CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
    }
}
