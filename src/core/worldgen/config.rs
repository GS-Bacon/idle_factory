//! ワールド生成設定
//!
//! WorldType、ノイズパラメータ、地形設定を定義

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// ワールド生成タイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorldType {
    /// 通常のパーリンノイズ地形
    #[default]
    Normal,
    /// デバッグ用フラットワールド
    Flat,
}

/// ノイズパラメータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseParams {
    /// fBmのオクターブ数
    pub octaves: u32,
    /// 基本周波数（低いほど大きな地形）
    pub frequency: f64,
    /// 各オクターブの寄与度
    pub persistence: f64,
    /// オクターブごとの周波数乗数
    pub lacunarity: f64,
}

impl Default for NoiseParams {
    fn default() -> Self {
        Self {
            octaves: 4,
            frequency: 0.01,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }
}

/// フラットワールド設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatWorldConfig {
    /// フラット地形の表面高さ（Y座標）
    pub surface_height: i32,
    /// レイヤー構成（ブロックID、厚さ）下から順
    pub layers: Vec<(String, u32)>,
}

impl Default for FlatWorldConfig {
    fn default() -> Self {
        Self {
            surface_height: -60,
            layers: vec![
                ("bedrock".to_string(), 1),
                ("stone".to_string(), 2),
                ("dirt".to_string(), 1),
                ("grass".to_string(), 1),
            ],
        }
    }
}

/// 地形高さ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainConfig {
    /// 基準高度（海面レベル）
    pub base_height: i32,
    /// 高低差の最大値
    pub height_variation: i32,
    /// ワールドの最小Y座標
    pub min_y: i32,
    /// ワールドの最大Y座標
    pub max_y: i32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            base_height: 64,
            height_variation: 32,
            min_y: -64,
            max_y: 256,
        }
    }
}

/// ワールド生成設定リソース
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct WorldGenConfig {
    /// ワールドシード
    pub seed: u64,
    /// ワールドタイプ
    pub world_type: WorldType,
    /// ノイズパラメータ
    pub noise_params: NoiseParams,
    /// 地形設定
    pub terrain: TerrainConfig,
    /// フラットワールド設定
    pub flat_config: FlatWorldConfig,
    /// 岩盤の深さ（最下層から）
    pub bedrock_depth: u32,
    /// 土壌の深さ（表面から）
    pub soil_depth: u32,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            world_type: WorldType::Normal,
            noise_params: NoiseParams::default(),
            terrain: TerrainConfig::default(),
            flat_config: FlatWorldConfig::default(),
            bedrock_depth: 5,
            soil_depth: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_type_default() {
        assert_eq!(WorldType::default(), WorldType::Normal);
    }

    #[test]
    fn test_terrain_config_bounds() {
        let config = TerrainConfig::default();
        assert!(config.min_y < config.max_y);
        assert!(config.base_height >= config.min_y);
        assert!(config.base_height <= config.max_y);
    }

    #[test]
    fn test_worldgen_config_serialization() {
        let config = WorldGenConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let restored: WorldGenConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.seed, restored.seed);
        assert_eq!(config.world_type, restored.world_type);
    }
}
