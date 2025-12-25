//! 鉱石分布システム
//!
//! 高さに応じた鉱石の配置

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

// Note: constants imported but some may be used in future phases
#[allow(unused_imports)]
use super::constants::*;

/// 鉱石の分布タイプ
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum DistributionType {
    /// 均一分布
    #[default]
    Uniform,
    /// 三角分布（ピークあり）
    Triangular,
    /// ピーク分布（ガウシアン風）
    Peak,
}

/// 鉱石脈のサイズ範囲
#[derive(Debug, Deserialize, Clone)]
pub struct VeinSize {
    pub min: u32,
    pub max: u32,
}

impl Default for VeinSize {
    fn default() -> Self {
        Self { min: 4, max: 8 }
    }
}

/// 鉱石定義
#[derive(Debug, Deserialize, Clone)]
pub struct OreDefinition {
    /// 鉱石ブロックID
    pub id: String,
    /// 最小高さ
    pub min_height: i32,
    /// 最大高さ
    pub max_height: i32,
    /// 分布タイプ
    #[serde(default)]
    pub distribution: DistributionType,
    /// ピーク高さ（Triangular/Peak用）
    #[serde(default)]
    pub peak_height: Option<i32>,
    /// 鉱石脈サイズ
    #[serde(default)]
    pub vein_size: VeinSize,
    /// チャンクあたりの試行回数
    #[serde(default = "default_attempts")]
    pub attempts_per_chunk: u32,
    /// 置き換え対象ブロック
    #[serde(default = "default_replace_blocks")]
    pub replace_blocks: Vec<String>,
}

fn default_attempts() -> u32 {
    10
}

fn default_replace_blocks() -> Vec<String> {
    vec!["stone".to_string(), "deepslate".to_string()]
}

impl OreDefinition {
    /// 指定した高さでの生成確率を計算 (0.0 - 1.0)
    pub fn spawn_probability(&self, height: i32) -> f64 {
        if height < self.min_height || height > self.max_height {
            return 0.0;
        }

        match self.distribution {
            DistributionType::Uniform => 1.0,
            DistributionType::Triangular => {
                let peak = self.peak_height.unwrap_or((self.min_height + self.max_height) / 2);
                let range = if height < peak {
                    (height - self.min_height) as f64 / (peak - self.min_height).max(1) as f64
                } else {
                    (self.max_height - height) as f64 / (self.max_height - peak).max(1) as f64
                };
                range.max(0.0)
            }
            DistributionType::Peak => {
                let peak = self.peak_height.unwrap_or((self.min_height + self.max_height) / 2);
                let half_range = ((self.max_height - self.min_height) / 2).max(1) as f64;
                let distance = (height - peak).abs() as f64 / half_range;
                // ガウシアン風の曲線
                (-distance * distance * 2.0).exp()
            }
        }
    }
}

/// 鉱石レジストリ
#[derive(Resource, Default)]
pub struct OreRegistry {
    pub ores: HashMap<String, OreDefinition>,
    pub ore_ids: Vec<String>,
}

impl OreRegistry {
    /// 指定した高さで生成可能な鉱石を取得
    pub fn get_ores_at_height(&self, height: i32) -> Vec<&OreDefinition> {
        self.ores
            .values()
            .filter(|ore| height >= ore.min_height && height <= ore.max_height)
            .collect()
    }
}

/// 鉱石定義を読み込む
pub fn load_ores(mut registry: ResMut<OreRegistry>) {
    let path = "assets/data/ores/distribution.yaml";

    match fs::read_to_string(path) {
        Ok(content) => {
            match serde_yaml::from_str::<Vec<OreDefinition>>(&content) {
                Ok(defs) => {
                    for def in defs {
                        info!("Loaded ore: {} (Y: {} to {})", def.id, def.min_height, def.max_height);
                        registry.ore_ids.push(def.id.clone());
                        registry.ores.insert(def.id.clone(), def);
                    }
                }
                Err(e) => {
                    warn!("Failed to parse ore YAML: {}. Using defaults.", e);
                    load_default_ores(&mut registry);
                }
            }
        }
        Err(e) => {
            warn!("Failed to read ore file {}: {}. Using defaults.", path, e);
            load_default_ores(&mut registry);
        }
    }

    if registry.ores.is_empty() {
        load_default_ores(&mut registry);
    }
}

/// デフォルト鉱石定義
fn load_default_ores(registry: &mut OreRegistry) {
    let default_ores = vec![
        OreDefinition {
            id: "coal_ore".to_string(),
            min_height: 0,
            max_height: 128,
            distribution: DistributionType::Uniform,
            peak_height: None,
            vein_size: VeinSize { min: 4, max: 16 },
            attempts_per_chunk: 20,
            replace_blocks: default_replace_blocks(),
        },
        OreDefinition {
            id: "iron_ore".to_string(),
            min_height: -16,
            max_height: 72,
            distribution: DistributionType::Triangular,
            peak_height: Some(16),
            vein_size: VeinSize { min: 4, max: 12 },
            attempts_per_chunk: 10,
            replace_blocks: default_replace_blocks(),
        },
        OreDefinition {
            id: "copper_ore".to_string(),
            min_height: -16,
            max_height: 112,
            distribution: DistributionType::Triangular,
            peak_height: Some(48),
            vein_size: VeinSize { min: 4, max: 10 },
            attempts_per_chunk: 16,
            replace_blocks: default_replace_blocks(),
        },
        OreDefinition {
            id: "gold_ore".to_string(),
            min_height: -64,
            max_height: 32,
            distribution: DistributionType::Triangular,
            peak_height: Some(-16),
            vein_size: VeinSize { min: 4, max: 8 },
            attempts_per_chunk: 4,
            replace_blocks: default_replace_blocks(),
        },
    ];

    for ore in default_ores {
        info!("Loaded default ore: {}", ore.id);
        registry.ore_ids.push(ore.id.clone());
        registry.ores.insert(ore.id.clone(), ore);
    }
}

/// 鉱石を配置するかどうか判定
pub fn should_place_ore(
    ore: &OreDefinition,
    x: i32,
    y: i32,
    z: i32,
    seed: u32,
    current_block: &str,
) -> bool {
    // 置き換え対象でなければスキップ
    if !ore.replace_blocks.iter().any(|b| b == current_block) {
        return false;
    }

    // 高さチェック
    if y < ore.min_height || y > ore.max_height {
        return false;
    }

    // 生成確率
    let base_probability = ore.spawn_probability(y);
    if base_probability <= 0.0 {
        return false;
    }

    // ノイズベースの判定
    let ore_noise = Perlin::new(seed.wrapping_add(hash_ore_id(&ore.id)));
    let noise_val = ore_noise.get([
        x as f64 * 0.1,
        y as f64 * 0.1,
        z as f64 * 0.1,
    ]);

    // 確率を調整（0.0-1.0の範囲に正規化）
    let threshold = 1.0 - (base_probability * 0.05); // 基本5%の確率、分布で調整

    noise_val > threshold
}

/// 鉱石IDをハッシュ化（シードオフセット用）
fn hash_ore_id(id: &str) -> u32 {
    let mut hash: u32 = 0;
    for byte in id.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    hash
}

/// ブロックが鉱石かどうかを判定
pub fn is_ore_block(block_id: &str) -> bool {
    block_id.ends_with("_ore")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ore_spawn_probability_uniform() {
        let ore = OreDefinition {
            id: "test_ore".to_string(),
            min_height: 0,
            max_height: 100,
            distribution: DistributionType::Uniform,
            peak_height: None,
            vein_size: VeinSize::default(),
            attempts_per_chunk: 10,
            replace_blocks: vec![],
        };

        // 範囲内は均一
        assert_eq!(ore.spawn_probability(50), 1.0);
        assert_eq!(ore.spawn_probability(0), 1.0);
        assert_eq!(ore.spawn_probability(100), 1.0);

        // 範囲外は0
        assert_eq!(ore.spawn_probability(-1), 0.0);
        assert_eq!(ore.spawn_probability(101), 0.0);
    }

    #[test]
    fn test_ore_spawn_probability_triangular() {
        let ore = OreDefinition {
            id: "test_ore".to_string(),
            min_height: 0,
            max_height: 100,
            distribution: DistributionType::Triangular,
            peak_height: Some(50),
            vein_size: VeinSize::default(),
            attempts_per_chunk: 10,
            replace_blocks: vec![],
        };

        // ピークで最大
        assert!(ore.spawn_probability(50) > ore.spawn_probability(25));
        assert!(ore.spawn_probability(50) > ore.spawn_probability(75));

        // 端で最小
        assert!(ore.spawn_probability(0) < ore.spawn_probability(50));
        assert!(ore.spawn_probability(100) < ore.spawn_probability(50));
    }

    #[test]
    fn test_is_ore_block() {
        assert!(is_ore_block("iron_ore"));
        assert!(is_ore_block("coal_ore"));
        assert!(!is_ore_block("stone"));
        assert!(!is_ore_block("dirt"));
    }

    #[test]
    fn test_hash_ore_id() {
        // 同じIDは同じハッシュ
        assert_eq!(hash_ore_id("iron_ore"), hash_ore_id("iron_ore"));
        // 異なるIDは異なるハッシュ（高確率で）
        assert_ne!(hash_ore_id("iron_ore"), hash_ore_id("coal_ore"));
    }
}
