//! Biome system for mining
//!
//! Miners produce resources based on the biome at their location, not the block below.
//! This allows for more interesting resource distribution without visible ore deposits.
//!
//! See game_spec::biome_mining_spec for probability tables.

use crate::game_spec::biome_mining_spec;
use crate::BlockType;
use bevy::prelude::*;

/// Biome types that determine mining output
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BiomeType {
    /// Rich in iron ore (70% iron, 22% stone, 8% coal)
    Iron,
    /// Rich in copper ore (70% copper, 22% stone, 8% iron)
    Copper,
    /// Rich in coal (75% coal, 20% stone, 5% iron)
    Coal,
    /// Mostly stone with some resources (85% stone, 10% coal, 5% iron)
    #[default]
    Stone,
    /// Mixed resources (30% iron, 25% copper, 25% coal, 20% stone)
    Mixed,
    /// Unmailable biomes (ocean, lava, void) - miners don't work here
    Unmailable,
}

impl BiomeType {
    /// Get the probability table for this biome
    pub fn get_probability_table(&self) -> &'static [(BlockType, u32)] {
        match self {
            BiomeType::Iron => biome_mining_spec::IRON_BIOME,
            BiomeType::Copper => biome_mining_spec::COPPER_BIOME,
            BiomeType::Coal => biome_mining_spec::COAL_BIOME,
            BiomeType::Stone => biome_mining_spec::STONE_BIOME,
            BiomeType::Mixed => biome_mining_spec::MIXED_BIOME,
            BiomeType::Unmailable => &[], // No resources
        }
    }

    /// Sample a random resource from this biome's probability table
    pub fn sample_resource(&self, random_value: u32) -> Option<BlockType> {
        let table = self.get_probability_table();
        if table.is_empty() {
            return None;
        }

        // random_value is 0-99
        let mut cumulative = 0u32;
        for (block_type, probability) in table {
            cumulative += probability;
            if random_value < cumulative {
                return Some(*block_type);
            }
        }

        // Fallback to last item if rounding errors
        table.last().map(|(bt, _)| *bt)
    }
}

/// Biome map resource - caches biome lookups
#[derive(Resource, Default)]
pub struct BiomeMap {
    /// Seed for biome generation (set once at world creation)
    pub seed: u64,
}

impl BiomeMap {
    /// Create a new biome map with a seed
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Get the biome at a given world position
    pub fn get_biome(&self, pos: IVec3) -> BiomeType {
        // First, check spawn area guarantees (radius 15 from delivery platform center)
        // Delivery platform is at (20, 8, 10), center at (26, 8, 16)
        let spawn_center = IVec2::new(26, 16);
        let pos_2d = IVec2::new(pos.x, pos.z);
        let distance = (pos_2d - spawn_center).as_vec2().length();

        if distance <= biome_mining_spec::SPAWN_GUARANTEE_RADIUS as f32 {
            // Guaranteed spawn area - divide into sectors for different biomes
            // Use angle from center to determine biome
            let dx = pos.x - spawn_center.x;
            let dz = pos.z - spawn_center.y;

            if distance < 5.0 {
                // Very close to center - mixed biome
                return BiomeType::Mixed;
            }

            // Divide into 3 sectors for iron, copper, coal
            let angle = (dz as f32).atan2(dx as f32);
            let sector =
                ((angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI / 3.0)) as i32 % 3;

            return match sector {
                0 => BiomeType::Iron,
                1 => BiomeType::Copper,
                _ => BiomeType::Coal,
            };
        }

        // Outside spawn area - use procedural generation
        self.procedural_biome(pos)
    }

    /// Generate biome procedurally using hash-based noise
    fn procedural_biome(&self, pos: IVec3) -> BiomeType {
        // Use a simple hash function for deterministic but varied biomes
        let hash = self.hash_position(pos.x, pos.z);

        // Divide hash into biome regions
        match hash % 100 {
            0..=29 => BiomeType::Iron,    // 30%
            30..=54 => BiomeType::Copper, // 25%
            55..=79 => BiomeType::Coal,   // 25%
            80..=94 => BiomeType::Stone,  // 15%
            _ => BiomeType::Mixed,        // 5%
        }
    }

    /// Hash function for position-based biome generation
    /// Uses a simple but effective mixing function
    fn hash_position(&self, x: i32, z: i32) -> u64 {
        // Scale down to create larger biome regions (8x8 blocks per biome)
        let bx = x.div_euclid(8) as u64;
        let bz = z.div_euclid(8) as u64;

        // Mix with seed
        let mut h = self.seed;
        h = h.wrapping_add(bx.wrapping_mul(0x9e3779b97f4a7c15));
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h = h.wrapping_add(bz.wrapping_mul(0xc4ceb9fe1a85ec53));
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;

        h
    }

    /// Check if mining is possible at this position
    pub fn can_mine(&self, pos: IVec3) -> bool {
        self.get_biome(pos) != BiomeType::Unmailable
    }
}

/// Simple pseudo-random number generator for mining output
/// Uses the miner's position and a tick counter for variation
pub fn mining_random(pos: IVec3, tick: u32, seed: u64) -> u32 {
    let mut h = seed;
    h = h.wrapping_add((pos.x as u64).wrapping_mul(0x9e3779b97f4a7c15));
    h = h.wrapping_add((pos.y as u64).wrapping_mul(0xff51afd7ed558ccd));
    h = h.wrapping_add((pos.z as u64).wrapping_mul(0xc4ceb9fe1a85ec53));
    h = h.wrapping_add((tick as u64).wrapping_mul(0x1234567890abcdef));
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;

    (h % 100) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_probability_tables_sum_to_100() {
        let biomes = [
            BiomeType::Iron,
            BiomeType::Copper,
            BiomeType::Coal,
            BiomeType::Stone,
            BiomeType::Mixed,
        ];

        for biome in biomes {
            let table = biome.get_probability_table();
            let sum: u32 = table.iter().map(|(_, p)| p).sum();
            assert_eq!(
                sum, 100,
                "Biome {:?} probabilities should sum to 100, got {}",
                biome, sum
            );
        }
    }

    #[test]
    fn test_unmailable_has_no_resources() {
        let biome = BiomeType::Unmailable;
        assert!(biome.get_probability_table().is_empty());
        assert!(biome.sample_resource(50).is_none());
    }

    #[test]
    fn test_spawn_area_has_guaranteed_biomes() {
        let map = BiomeMap::new(12345);

        // Check that different sectors have different biomes
        let iron_found = (0..360).any(|angle| {
            let rad = (angle as f32).to_radians();
            let x = 26 + (10.0 * rad.cos()) as i32;
            let z = 16 + (10.0 * rad.sin()) as i32;
            map.get_biome(IVec3::new(x, 0, z)) == BiomeType::Iron
        });
        let copper_found = (0..360).any(|angle| {
            let rad = (angle as f32).to_radians();
            let x = 26 + (10.0 * rad.cos()) as i32;
            let z = 16 + (10.0 * rad.sin()) as i32;
            map.get_biome(IVec3::new(x, 0, z)) == BiomeType::Copper
        });
        let coal_found = (0..360).any(|angle| {
            let rad = (angle as f32).to_radians();
            let x = 26 + (10.0 * rad.cos()) as i32;
            let z = 16 + (10.0 * rad.sin()) as i32;
            map.get_biome(IVec3::new(x, 0, z)) == BiomeType::Coal
        });

        assert!(iron_found, "Iron biome should be in spawn area");
        assert!(copper_found, "Copper biome should be in spawn area");
        assert!(coal_found, "Coal biome should be in spawn area");
    }

    #[test]
    fn test_biome_sample_returns_valid_resource() {
        let biome = BiomeType::Iron;
        for i in 0..100 {
            let result = biome.sample_resource(i);
            assert!(
                result.is_some(),
                "Iron biome should return resource for value {}",
                i
            );
        }
    }

    #[test]
    fn test_biome_deterministic() {
        let map = BiomeMap::new(12345);
        let pos = IVec3::new(100, 0, 100);

        // Same position should always return same biome
        let biome1 = map.get_biome(pos);
        let biome2 = map.get_biome(pos);
        assert_eq!(biome1, biome2);
    }

    #[test]
    fn test_mining_random_deterministic() {
        let pos = IVec3::new(10, 5, 20);
        let seed = 12345;

        let r1 = mining_random(pos, 100, seed);
        let r2 = mining_random(pos, 100, seed);
        assert_eq!(r1, r2);

        // Different tick should give different value
        let r3 = mining_random(pos, 101, seed);
        assert_ne!(r1, r3);
    }
}
