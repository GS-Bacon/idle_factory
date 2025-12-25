//! ノイズ生成器
//!
//! 地形生成用の複数レイヤーノイズを提供

use noise::{NoiseFn, Perlin};

/// 複数のノイズ生成器をまとめた構造体
pub struct NoiseGenerators {
    // バイオーム決定用
    /// 温度ノイズ (スケール: 0.002)
    pub temperature: Perlin,
    /// 湿度ノイズ (スケール: 0.003)
    pub humidity: Perlin,
    /// 大陸性ノイズ (スケール: 0.001)
    pub continental: Perlin,

    // 地形高さ用
    /// 大規模地形ノイズ (スケール: 0.005)
    pub terrain_large: Perlin,
    /// 中規模地形ノイズ (スケール: 0.02)
    pub terrain_medium: Perlin,
    /// 詳細ノイズ (スケール: 0.08)
    pub terrain_detail: Perlin,

    // 洞窟用
    /// チーズ洞窟ノイズ
    pub cave_cheese: Perlin,
    /// スパゲッティ洞窟ノイズ
    pub cave_spaghetti: Perlin,
    /// ヌードル洞窟ノイズ
    pub cave_noodle: Perlin,
}

impl NoiseGenerators {
    /// 新しいノイズ生成器を作成
    pub fn new(seed: u32) -> Self {
        Self {
            // バイオーム用（異なるシードオフセットで独立したノイズを生成）
            temperature: Perlin::new(seed),
            humidity: Perlin::new(seed.wrapping_add(1000)),
            continental: Perlin::new(seed.wrapping_add(2000)),

            // 地形用
            terrain_large: Perlin::new(seed.wrapping_add(3000)),
            terrain_medium: Perlin::new(seed.wrapping_add(4000)),
            terrain_detail: Perlin::new(seed.wrapping_add(5000)),

            // 洞窟用
            cave_cheese: Perlin::new(seed.wrapping_add(6000)),
            cave_spaghetti: Perlin::new(seed.wrapping_add(7000)),
            cave_noodle: Perlin::new(seed.wrapping_add(8000)),
        }
    }

    /// 温度を取得 (0.0 - 1.0)
    pub fn get_temperature(&self, x: f64, z: f64) -> f64 {
        let scale = 0.002;
        (self.temperature.get([x * scale, z * scale]) + 1.0) / 2.0
    }

    /// 湿度を取得 (0.0 - 1.0)
    pub fn get_humidity(&self, x: f64, z: f64) -> f64 {
        let scale = 0.003;
        (self.humidity.get([x * scale, z * scale]) + 1.0) / 2.0
    }

    /// 大陸性を取得 (0.0 - 1.0)
    /// 0.0 = 海洋, 1.0 = 大陸内部
    pub fn get_continentalness(&self, x: f64, z: f64) -> f64 {
        let scale = 0.001;
        (self.continental.get([x * scale, z * scale]) + 1.0) / 2.0
    }

    /// 地形高さのノイズ値を取得 (-1.0 - 1.0)
    /// 複数オクターブを合成
    pub fn get_terrain_height(&self, x: f64, z: f64) -> f64 {
        // 大規模地形 (大陸・山脈スケール)
        let large = self.terrain_large.get([x * 0.005, z * 0.005]) * 0.5;

        // 中規模地形 (丘・谷スケール)
        let medium = self.terrain_medium.get([x * 0.02, z * 0.02]) * 0.35;

        // 詳細地形 (小さな凹凸)
        let detail = self.terrain_detail.get([x * 0.08, z * 0.08]) * 0.15;

        large + medium + detail
    }

    /// 3D洞窟ノイズを取得 (cheese caves)
    pub fn get_cave_cheese(&self, x: f64, y: f64, z: f64) -> f64 {
        let scale = 0.03;
        self.cave_cheese.get([x * scale, y * scale, z * scale])
    }

    /// 3D洞窟ノイズを取得 (spaghetti caves)
    pub fn get_cave_spaghetti(&self, x: f64, y: f64, z: f64) -> f64 {
        let scale = 0.02;
        // 2つのノイズを組み合わせてトンネル状にする
        let noise1 = self.cave_spaghetti.get([x * scale, y * scale, z * scale]);
        let noise2 = self.cave_spaghetti.get([x * scale + 100.0, y * scale + 100.0, z * scale + 100.0]);
        // 両方が0に近いときに洞窟を形成
        1.0 - (noise1 * noise1 + noise2 * noise2).sqrt()
    }

    /// 3D洞窟ノイズを取得 (noodle caves - 細い通路)
    pub fn get_cave_noodle(&self, x: f64, y: f64, z: f64) -> f64 {
        let scale = 0.015;
        let noise1 = self.cave_noodle.get([x * scale, y * scale, z * scale]);
        let noise2 = self.cave_noodle.get([x * scale + 50.0, y * scale + 50.0, z * scale + 50.0]);
        1.0 - (noise1 * noise1 + noise2 * noise2).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_range() {
        let gen = NoiseGenerators::new(12345);

        // 複数の座標でテスト
        for x in -100..100 {
            for z in -100..100 {
                let temp = gen.get_temperature(x as f64, z as f64);
                assert!((0.0..=1.0).contains(&temp), "Temperature out of range: {}", temp);

                let humid = gen.get_humidity(x as f64, z as f64);
                assert!((0.0..=1.0).contains(&humid), "Humidity out of range: {}", humid);

                let cont = gen.get_continentalness(x as f64, z as f64);
                assert!((0.0..=1.0).contains(&cont), "Continentalness out of range: {}", cont);
            }
        }
    }

    #[test]
    fn test_noise_determinism() {
        let gen1 = NoiseGenerators::new(12345);
        let gen2 = NoiseGenerators::new(12345);

        // 同じシードで同じ値になる
        assert_eq!(
            gen1.get_temperature(100.0, 200.0),
            gen2.get_temperature(100.0, 200.0)
        );
        assert_eq!(
            gen1.get_terrain_height(100.0, 200.0),
            gen2.get_terrain_height(100.0, 200.0)
        );
    }

    #[test]
    fn test_different_seeds_produce_different_noise() {
        let gen1 = NoiseGenerators::new(12345);
        let gen2 = NoiseGenerators::new(54321);

        // 異なるシードで異なる値になる（高確率で）
        assert_ne!(
            gen1.get_temperature(100.0, 200.0),
            gen2.get_temperature(100.0, 200.0)
        );
    }
}
