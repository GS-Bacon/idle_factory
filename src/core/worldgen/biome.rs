//! バイオームシステム
//!
//! 温度・湿度・大陸性に基づくバイオーム選択

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

use super::noise::NoiseGenerators;

/// バイオームの表層ブロック定義
#[derive(Debug, Deserialize, Clone)]
pub struct SurfaceLayer {
    /// ブロックID
    pub block: String,
    /// 深さ（-1 = 無限）
    pub depth: i32,
    /// 条件（オプション）
    #[serde(default)]
    pub condition: Option<SurfaceCondition>,
}

/// 表層ブロックの配置条件
#[derive(Debug, Deserialize, Clone)]
pub struct SurfaceCondition {
    /// この高さ以上で適用
    #[serde(default)]
    pub height_above: Option<i32>,
    /// この高さ以下で適用
    #[serde(default)]
    pub height_below: Option<i32>,
}

/// バイオームの地形設定
#[derive(Debug, Deserialize, Clone)]
pub struct TerrainConfig {
    /// 基準高さ（海抜からのオフセット）
    pub base_height: i32,
    /// 高さの変動幅
    pub height_variation: i32,
    /// 平坦度 (0.0-1.0, 高いほど平坦)
    #[serde(default = "default_flatness")]
    pub flatness: f32,
}

fn default_flatness() -> f32 {
    0.5
}

/// パラメータ範囲
#[derive(Debug, Deserialize, Clone)]
pub struct ParameterRange {
    pub min: f32,
    pub max: f32,
}

impl ParameterRange {
    /// 値が範囲内かチェック
    pub fn contains(&self, value: f32) -> bool {
        value >= self.min && value <= self.max
    }

    /// 範囲の中心からの距離を計算 (0.0 = 中心, 1.0 = 端)
    pub fn distance_from_center(&self, value: f32) -> f32 {
        let center = (self.min + self.max) / 2.0;
        let half_range = (self.max - self.min) / 2.0;
        if half_range == 0.0 {
            0.0
        } else {
            ((value - center).abs() / half_range).min(1.0)
        }
    }
}

/// バイオーム定義（YAML読み込み用）
#[derive(Debug, Deserialize, Clone)]
pub struct BiomeDefinition {
    /// バイオームID
    pub id: String,
    /// 表示名
    pub name: String,
    /// 温度範囲
    pub temperature: ParameterRange,
    /// 湿度範囲
    pub humidity: ParameterRange,
    /// 大陸性範囲
    pub continentalness: ParameterRange,
    /// 地形設定
    pub terrain: TerrainConfig,
    /// 表層ブロック（上から順）
    pub surface_layers: Vec<SurfaceLayer>,
    /// 海面まで水で埋める
    #[serde(default)]
    pub fill_to_sea_level: bool,
    /// 埋めるブロック
    #[serde(default = "default_fill_block")]
    pub fill_block: String,
}

fn default_fill_block() -> String {
    "water".to_string()
}

impl BiomeDefinition {
    /// パラメータがこのバイオームにマッチするかチェック
    pub fn matches(&self, temperature: f32, humidity: f32, continentalness: f32) -> bool {
        self.temperature.contains(temperature)
            && self.humidity.contains(humidity)
            && self.continentalness.contains(continentalness)
    }

    /// パラメータとの適合度を計算 (0.0 = 完全一致, 大きいほど遠い)
    pub fn fitness(&self, temperature: f32, humidity: f32, continentalness: f32) -> f32 {
        let t_dist = self.temperature.distance_from_center(temperature);
        let h_dist = self.humidity.distance_from_center(humidity);
        let c_dist = self.continentalness.distance_from_center(continentalness);

        // ユークリッド距離
        (t_dist * t_dist + h_dist * h_dist + c_dist * c_dist).sqrt()
    }
}

/// バイオームレジストリ
#[derive(Resource, Default)]
pub struct BiomeRegistry {
    pub biomes: HashMap<String, BiomeDefinition>,
    /// IDリスト（検索用）
    pub biome_ids: Vec<String>,
}

impl BiomeRegistry {
    /// パラメータに最も適合するバイオームを取得
    pub fn get_biome(&self, temperature: f32, humidity: f32, continentalness: f32) -> Option<&BiomeDefinition> {
        // まず完全にマッチするものを探す
        for id in &self.biome_ids {
            if let Some(biome) = self.biomes.get(id) {
                if biome.matches(temperature, humidity, continentalness) {
                    return Some(biome);
                }
            }
        }

        // マッチしない場合は最も近いものを返す
        let mut best: Option<(&BiomeDefinition, f32)> = None;
        for biome in self.biomes.values() {
            let fitness = biome.fitness(temperature, humidity, continentalness);
            if best.is_none() || fitness < best.unwrap().1 {
                best = Some((biome, fitness));
            }
        }

        best.map(|(b, _)| b)
    }

    /// 座標からバイオームを取得
    pub fn get_biome_at(&self, generators: &NoiseGenerators, x: f64, z: f64) -> Option<&BiomeDefinition> {
        let temp = generators.get_temperature(x, z) as f32;
        let humid = generators.get_humidity(x, z) as f32;
        let cont = generators.get_continentalness(x, z) as f32;

        self.get_biome(temp, humid, cont)
    }

    /// デフォルトバイオームを取得（フォールバック用）
    pub fn get_default(&self) -> Option<&BiomeDefinition> {
        self.biomes.get("plains")
    }
}

/// バイオーム定義を読み込む
pub fn load_biomes(mut registry: ResMut<BiomeRegistry>) {
    let path = "assets/data/biomes/core.yaml";

    match fs::read_to_string(path) {
        Ok(content) => {
            match serde_yaml::from_str::<Vec<BiomeDefinition>>(&content) {
                Ok(defs) => {
                    for def in defs {
                        info!("Loaded biome: {} ({})", def.name, def.id);
                        registry.biome_ids.push(def.id.clone());
                        registry.biomes.insert(def.id.clone(), def);
                    }
                }
                Err(e) => {
                    warn!("Failed to parse biome YAML: {}. Using defaults.", e);
                    load_default_biomes(&mut registry);
                }
            }
        }
        Err(e) => {
            warn!("Failed to read biome file {}: {}. Using defaults.", path, e);
            load_default_biomes(&mut registry);
        }
    }

    // 必ず少なくとも1つのバイオームがあることを保証
    if registry.biomes.is_empty() {
        load_default_biomes(&mut registry);
    }
}

/// デフォルトバイオームを追加（フォールバック用）
fn load_default_biomes(registry: &mut BiomeRegistry) {
    let plains = BiomeDefinition {
        id: "plains".to_string(),
        name: "Plains".to_string(),
        temperature: ParameterRange { min: 0.0, max: 1.0 },
        humidity: ParameterRange { min: 0.0, max: 1.0 },
        continentalness: ParameterRange { min: 0.0, max: 1.0 },
        terrain: TerrainConfig {
            base_height: 64,
            height_variation: 4,
            flatness: 0.8,
        },
        surface_layers: vec![
            SurfaceLayer { block: "grass".to_string(), depth: 1, condition: None },
            SurfaceLayer { block: "dirt".to_string(), depth: 3, condition: None },
            SurfaceLayer { block: "stone".to_string(), depth: -1, condition: None },
        ],
        fill_to_sea_level: false,
        fill_block: "water".to_string(),
    };

    registry.biome_ids.push(plains.id.clone());
    registry.biomes.insert(plains.id.clone(), plains);
    info!("Loaded default biome: plains");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_range() {
        let range = ParameterRange { min: 0.3, max: 0.7 };

        assert!(range.contains(0.5));
        assert!(range.contains(0.3));
        assert!(range.contains(0.7));
        assert!(!range.contains(0.2));
        assert!(!range.contains(0.8));
    }

    #[test]
    fn test_biome_matching() {
        let biome = BiomeDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            temperature: ParameterRange { min: 0.3, max: 0.7 },
            humidity: ParameterRange { min: 0.3, max: 0.7 },
            continentalness: ParameterRange { min: 0.3, max: 1.0 },
            terrain: TerrainConfig {
                base_height: 64,
                height_variation: 4,
                flatness: 0.5,
            },
            surface_layers: vec![],
            fill_to_sea_level: false,
            fill_block: "water".to_string(),
        };

        // マッチするケース
        assert!(biome.matches(0.5, 0.5, 0.5));

        // マッチしないケース
        assert!(!biome.matches(0.1, 0.5, 0.5)); // 温度が低すぎ
        assert!(!biome.matches(0.5, 0.1, 0.5)); // 湿度が低すぎ
        assert!(!biome.matches(0.5, 0.5, 0.1)); // 大陸性が低すぎ
    }

    #[test]
    fn test_biome_registry() {
        let mut registry = BiomeRegistry::default();
        load_default_biomes(&mut registry);

        // plainsバイオームが存在する
        assert!(registry.biomes.contains_key("plains"));

        // 任意のパラメータでバイオームが取得できる
        let biome = registry.get_biome(0.5, 0.5, 0.5);
        assert!(biome.is_some());
    }
}
