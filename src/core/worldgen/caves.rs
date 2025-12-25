//! 洞窟生成システム
//!
//! 3Dノイズによる自然な洞窟の生成
//! - Cheese caves: 大きな空洞
//! - Spaghetti caves: 細長いトンネル
//! - Noodle caves: 極細トンネル

use super::constants::*;
use super::noise::NoiseGenerators;

/// 洞窟かどうかを判定
pub fn is_cave(generators: &NoiseGenerators, x: i32, y: i32, z: i32) -> bool {
    // 地表付近では洞窟を生成しない（Y > 50）
    if y > 50 {
        return false;
    }

    // 岩盤付近では洞窟を生成しない
    if y <= BEDROCK_LEVEL + 5 {
        return false;
    }

    let fx = x as f64;
    let fy = y as f64;
    let fz = z as f64;

    // Cheese caves（大きな空洞）
    let cheese = generators.get_cave_cheese(fx, fy, fz);
    if cheese > 0.6 {
        return true;
    }

    // Spaghetti caves（細長いトンネル）
    let spaghetti = generators.get_cave_spaghetti(fx, fy, fz);
    if spaghetti > 0.85 {
        return true;
    }

    // Noodle caves（極細トンネル）- 深い場所のみ
    if y < 0 {
        let noodle = generators.get_cave_noodle(fx, fy, fz);
        if noodle > 0.9 {
            return true;
        }
    }

    false
}

/// 洞窟の密度を取得（デバッグ・可視化用）
pub fn get_cave_density(generators: &NoiseGenerators, x: i32, y: i32, z: i32) -> f64 {
    let fx = x as f64;
    let fy = y as f64;
    let fz = z as f64;

    let cheese = generators.get_cave_cheese(fx, fy, fz);
    let spaghetti = generators.get_cave_spaghetti(fx, fy, fz);
    let noodle = if y < 0 {
        generators.get_cave_noodle(fx, fy, fz)
    } else {
        0.0
    };

    cheese.max(spaghetti).max(noodle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_caves_at_surface() {
        let generators = NoiseGenerators::new(12345);

        // 地表付近（Y > 50）では洞窟が生成されない
        let mut cave_count = 0;
        for x in 0..100 {
            for z in 0..100 {
                if is_cave(&generators, x, 60, z) {
                    cave_count += 1;
                }
            }
        }
        assert_eq!(cave_count, 0, "No caves should be at surface level");
    }

    #[test]
    fn test_caves_exist_underground() {
        let generators = NoiseGenerators::new(12345);

        // 地下では洞窟が存在する
        let mut cave_count = 0;
        for x in 0..100 {
            for z in 0..100 {
                if is_cave(&generators, x, 20, z) {
                    cave_count += 1;
                }
            }
        }
        assert!(cave_count > 0, "Some caves should exist underground");
    }

    #[test]
    fn test_no_caves_at_bedrock() {
        let generators = NoiseGenerators::new(12345);

        // 岩盤付近では洞窟が生成されない
        let mut cave_count = 0;
        for x in 0..100 {
            for z in 0..100 {
                if is_cave(&generators, x, BEDROCK_LEVEL + 2, z) {
                    cave_count += 1;
                }
            }
        }
        assert_eq!(cave_count, 0, "No caves should be at bedrock level");
    }

    #[test]
    fn test_cave_determinism() {
        let gen1 = NoiseGenerators::new(12345);
        let gen2 = NoiseGenerators::new(12345);

        // 同じシードで同じ洞窟パターン
        for x in 0..50 {
            for z in 0..50 {
                assert_eq!(
                    is_cave(&gen1, x, 20, z),
                    is_cave(&gen2, x, 20, z),
                    "Cave generation should be deterministic"
                );
            }
        }
    }
}
