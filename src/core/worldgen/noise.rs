//! ノイズ生成ユーティリティ
//!
//! パーリンノイズを使用した地形高さの計算

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use super::config::{NoiseParams, WorldGenConfig};

/// 地形ノイズ生成器
pub struct TerrainNoise {
    fbm: Fbm<Perlin>,
}

impl TerrainNoise {
    /// 新しい地形ノイズ生成器を作成
    pub fn new(seed: u64, params: &NoiseParams) -> Self {
        let fbm = Fbm::<Perlin>::new(seed as u32)
            .set_octaves(params.octaves as usize)
            .set_frequency(params.frequency)
            .set_persistence(params.persistence)
            .set_lacunarity(params.lacunarity);

        Self { fbm }
    }

    /// 指定座標の地形高さを取得
    ///
    /// # Arguments
    /// * `x` - ワールドX座標
    /// * `z` - ワールドZ座標
    /// * `config` - ワールド生成設定
    ///
    /// # Returns
    /// 地形の表面Y座標
    pub fn get_height(&self, x: i32, z: i32, config: &WorldGenConfig) -> i32 {
        // ノイズ値は[-1, 1]の範囲
        let noise_val = self.fbm.get([x as f64, z as f64]);

        // [0, 1]に正規化
        let normalized = (noise_val + 1.0) / 2.0;

        // 高さに変換
        let height_range = config.terrain.height_variation as f64 * 2.0;
        let height = config.terrain.base_height as f64 + (normalized - 0.5) * height_range;

        height as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_seed_same_result() {
        let params = NoiseParams::default();
        let config = WorldGenConfig::default();

        let noise1 = TerrainNoise::new(12345, &params);
        let noise2 = TerrainNoise::new(12345, &params);

        assert_eq!(
            noise1.get_height(0, 0, &config),
            noise2.get_height(0, 0, &config)
        );
        assert_eq!(
            noise1.get_height(100, 200, &config),
            noise2.get_height(100, 200, &config)
        );
    }

    #[test]
    fn test_different_seed_different_result() {
        let params = NoiseParams::default();
        let config = WorldGenConfig::default();

        let noise1 = TerrainNoise::new(12345, &params);
        let noise2 = TerrainNoise::new(54321, &params);

        // 異なるシードでは異なる結果になる可能性が高い
        // 少なくともいくつかの座標で異なる結果になるはず
        let different_count = (0..10)
            .filter(|&i| noise1.get_height(i, i, &config) != noise2.get_height(i, i, &config))
            .count();
        assert!(different_count > 0, "Different seeds should produce different terrain");
    }

    #[test]
    fn test_height_within_bounds() {
        let params = NoiseParams::default();
        let config = WorldGenConfig::default();
        let noise = TerrainNoise::new(12345, &params);

        // 複数の座標でテスト
        for x in -100..=100 {
            for z in -100..=100 {
                let height = noise.get_height(x, z, &config);
                let min_expected = config.terrain.base_height - config.terrain.height_variation;
                let max_expected = config.terrain.base_height + config.terrain.height_variation;

                assert!(
                    height >= min_expected && height <= max_expected,
                    "Height {} at ({}, {}) out of bounds [{}, {}]",
                    height,
                    x,
                    z,
                    min_expected,
                    max_expected
                );
            }
        }
    }
}
