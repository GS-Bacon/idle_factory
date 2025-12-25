//! 地形生成ロジック
//!
//! バイオームに基づいた地形の生成

use bevy::prelude::*;

use super::biome::SurfaceCondition;
use super::caves;
use super::constants::*;
use super::noise::NoiseGenerators;

/// チャンクの地形を生成
pub fn generate_terrain(generators: &NoiseGenerators, chunk_pos: IVec3) -> Vec<String> {
    let size = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
    let mut blocks = vec!["air".to_string(); size];

    // ワールド座標オフセット
    let world_x_offset = chunk_pos.x * CHUNK_SIZE as i32;
    let world_y_offset = chunk_pos.y * CHUNK_SIZE as i32;
    let world_z_offset = chunk_pos.z * CHUNK_SIZE as i32;

    // まず各X,Z座標のバイオームと地形高さをキャッシュ
    let mut height_cache: Vec<Vec<(i32, BiomeCacheEntry)>> = Vec::with_capacity(CHUNK_SIZE);

    for local_x in 0..CHUNK_SIZE {
        let mut row = Vec::with_capacity(CHUNK_SIZE);
        for local_z in 0..CHUNK_SIZE {
            let world_x = world_x_offset + local_x as i32;
            let world_z = world_z_offset + local_z as i32;

            // バイオームパラメータを取得
            let temp = generators.get_temperature(world_x as f64, world_z as f64) as f32;
            let humid = generators.get_humidity(world_x as f64, world_z as f64) as f32;
            let cont = generators.get_continentalness(world_x as f64, world_z as f64) as f32;

            // バイオームを取得（フォールバック用にデフォルト値を用意）
            let biome_entry = get_biome_cache_entry(temp, humid, cont);

            // 地形高さを計算
            let terrain_height = calculate_terrain_height(generators, world_x, world_z, &biome_entry);

            row.push((terrain_height, biome_entry));
        }
        height_cache.push(row);
    }

    // ブロックを生成
    for local_y in 0..CHUNK_SIZE {
        for (local_z, cache_row) in height_cache.iter().enumerate() {
            for (local_x, (terrain_height, biome_entry)) in cache_row.iter().enumerate() {
                let world_x = world_x_offset + local_x as i32;
                let world_y = world_y_offset + local_y as i32;
                let world_z = world_z_offset + local_z as i32;

                let terrain_height = *terrain_height;

                // ブロックを決定
                let block_id = determine_block(
                    generators,
                    world_x,
                    world_y,
                    world_z,
                    terrain_height,
                    biome_entry,
                );

                let idx = (local_y * CHUNK_SIZE * CHUNK_SIZE) + (local_z * CHUNK_SIZE) + local_x;
                blocks[idx] = block_id;
            }
        }
    }

    blocks
}

/// バイオームキャッシュエントリ（静的データのコピー）
#[derive(Clone)]
pub struct BiomeCacheEntry {
    pub id: String,
    pub base_height: i32,
    pub height_variation: i32,
    pub flatness: f32,
    pub surface_layers: Vec<(String, i32, Option<SurfaceCondition>)>,
    pub fill_to_sea_level: bool,
    pub fill_block: String,
}

/// バイオームパラメータからキャッシュエントリを作成
fn get_biome_cache_entry(temp: f32, humid: f32, cont: f32) -> BiomeCacheEntry {
    // 大陸性が低い = 海洋
    if cont < 0.3 {
        return BiomeCacheEntry {
            id: "ocean".to_string(),
            base_height: 40,
            height_variation: 8,
            flatness: 0.7,
            surface_layers: vec![
                ("gravel".to_string(), 2, None),
                ("sand".to_string(), 3, None),
                ("stone".to_string(), -1, None),
            ],
            fill_to_sea_level: true,
            fill_block: "water".to_string(),
        };
    }

    // 温度・湿度でバイオームを決定
    if temp > 0.7 && humid < 0.3 {
        // 高温・低湿度 = 砂漠
        BiomeCacheEntry {
            id: "desert".to_string(),
            base_height: 64,
            height_variation: 8,
            flatness: 0.6,
            surface_layers: vec![
                ("sand".to_string(), 4, None),
                ("sandstone".to_string(), 8, None),
                ("stone".to_string(), -1, None),
            ],
            fill_to_sea_level: false,
            fill_block: "water".to_string(),
        }
    } else if temp < 0.4 && cont > 0.7 {
        // 低温・高大陸性 = 山岳
        BiomeCacheEntry {
            id: "mountains".to_string(),
            base_height: 96,
            height_variation: 48,
            flatness: 0.2,
            surface_layers: vec![
                ("stone".to_string(), 1, Some(SurfaceCondition { height_above: Some(100), height_below: None })),
                ("grass".to_string(), 1, None),
                ("dirt".to_string(), 2, None),
                ("stone".to_string(), -1, None),
            ],
            fill_to_sea_level: false,
            fill_block: "water".to_string(),
        }
    } else if humid > 0.7 {
        // 高湿度 = 森林
        BiomeCacheEntry {
            id: "forest".to_string(),
            base_height: 68,
            height_variation: 12,
            flatness: 0.5,
            surface_layers: vec![
                ("grass".to_string(), 1, None),
                ("dirt".to_string(), 4, None),
                ("stone".to_string(), -1, None),
            ],
            fill_to_sea_level: false,
            fill_block: "water".to_string(),
        }
    } else if humid > 0.5 && temp > 0.4 && temp < 0.7 {
        // 中温・中湿度 = 湿地
        BiomeCacheEntry {
            id: "swamp".to_string(),
            base_height: 62,
            height_variation: 4,
            flatness: 0.85,
            surface_layers: vec![
                ("grass".to_string(), 1, None),
                ("dirt".to_string(), 3, None),
                ("stone".to_string(), -1, None),
            ],
            fill_to_sea_level: true,
            fill_block: "water".to_string(),
        }
    } else {
        // デフォルト = 平原
        BiomeCacheEntry {
            id: "plains".to_string(),
            base_height: 64,
            height_variation: 4,
            flatness: 0.8,
            surface_layers: vec![
                ("grass".to_string(), 1, None),
                ("dirt".to_string(), 3, None),
                ("stone".to_string(), -1, None),
            ],
            fill_to_sea_level: false,
            fill_block: "water".to_string(),
        }
    }
}

/// 地形高さを計算
fn calculate_terrain_height(
    generators: &NoiseGenerators,
    world_x: i32,
    world_z: i32,
    biome: &BiomeCacheEntry,
) -> i32 {
    // ノイズ値を取得 (-1.0 ~ 1.0)
    let noise_val = generators.get_terrain_height(world_x as f64, world_z as f64);

    // 平坦度を適用（高いほど変動が少ない）
    let adjusted_noise = noise_val * (1.0 - biome.flatness as f64);

    // 高さを計算
    let height = biome.base_height as f64 + adjusted_noise * biome.height_variation as f64;

    height as i32
}

/// ブロックを決定
fn determine_block(
    generators: &NoiseGenerators,
    world_x: i32,
    world_y: i32,
    world_z: i32,
    terrain_height: i32,
    biome: &BiomeCacheEntry,
) -> String {
    // 岩盤
    if world_y <= BEDROCK_LEVEL {
        return "bedrock".to_string();
    }

    // 空気（地形より上）
    if world_y > terrain_height {
        // 海面より下で水で埋める設定の場合
        if biome.fill_to_sea_level && world_y <= SEA_LEVEL {
            return biome.fill_block.clone();
        }
        return "air".to_string();
    }

    // 洞窟チェック
    if caves::is_cave(generators, world_x, world_y, world_z) {
        // 洞窟内の水/溶岩
        if world_y <= SEA_LEVEL && biome.fill_to_sea_level {
            return biome.fill_block.clone();
        }
        if world_y < -48 {
            return "lava".to_string();
        }
        return "air".to_string();
    }

    // 表層ブロックを決定
    let depth_from_surface = terrain_height - world_y;

    let mut accumulated_depth = 0;
    for (block, layer_depth, condition) in &biome.surface_layers {
        // 条件チェック
        if let Some(cond) = condition {
            if let Some(above) = cond.height_above {
                if world_y < above {
                    continue;
                }
            }
            if let Some(below) = cond.height_below {
                if world_y > below {
                    continue;
                }
            }
        }

        if *layer_depth < 0 {
            // 無限深さ = 残りすべて
            // 深層岩への切り替え
            if world_y < DEEPSLATE_LEVEL {
                return "deepslate".to_string();
            }
            return block.clone();
        }

        accumulated_depth += layer_depth;
        if depth_from_surface < accumulated_depth {
            return block.clone();
        }
    }

    // フォールバック
    if world_y < DEEPSLATE_LEVEL {
        "deepslate".to_string()
    } else {
        "stone".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_terrain_basic() {
        let generators = NoiseGenerators::new(12345);

        // 地表チャンク
        let blocks = generate_terrain(&generators, IVec3::new(0, 2, 0));
        assert!(!blocks.is_empty());
        assert_eq!(blocks.len(), CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
    }

    #[test]
    fn test_terrain_determinism() {
        let gen1 = NoiseGenerators::new(12345);
        let gen2 = NoiseGenerators::new(12345);

        let blocks1 = generate_terrain(&gen1, IVec3::new(5, 2, 5));
        let blocks2 = generate_terrain(&gen2, IVec3::new(5, 2, 5));

        assert_eq!(blocks1, blocks2, "Same seed should produce same terrain");
    }

    #[test]
    fn test_biome_selection() {
        // 海洋（低大陸性）
        let ocean = get_biome_cache_entry(0.5, 0.5, 0.1);
        assert_eq!(ocean.id, "ocean");

        // 砂漠（高温・低湿度）
        let desert = get_biome_cache_entry(0.9, 0.1, 0.5);
        assert_eq!(desert.id, "desert");

        // 平原（デフォルト）
        let plains = get_biome_cache_entry(0.5, 0.4, 0.5);
        assert_eq!(plains.id, "plains");
    }

    #[test]
    fn test_bedrock_generation() {
        let generators = NoiseGenerators::new(12345);

        // 最下層チャンク
        let blocks = generate_terrain(&generators, IVec3::new(0, -8, 0));

        // 最下層には岩盤があるはず
        let bedrock_count = blocks.iter().filter(|b| *b == "bedrock").count();
        assert!(bedrock_count > 0, "Bottom chunk should have bedrock");
    }
}
