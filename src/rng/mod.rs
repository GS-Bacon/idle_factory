//! Deterministic random number generation for reproducible game runs
//!
//! This module provides a seeded RNG that can be controlled via environment variables
//! for testing and debugging purposes.

use bevy::prelude::*;

/// Default seed for deterministic runs
const DEFAULT_SEED: u64 = 12345;

/// Environment variable name for custom seed
const SEED_ENV_VAR: &str = "GAME_SEED";

/// Game-wide RNG resource using a simple Linear Congruential Generator
/// This is not cryptographically secure but provides deterministic sequences
#[derive(Resource)]
pub struct GameRng {
    state: u64,
    initial_seed: u64,
}

impl GameRng {
    /// Create a new RNG with the specified seed
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed,
            initial_seed: seed,
        }
    }

    /// Create RNG from environment variable or default seed
    pub fn from_env() -> Self {
        let seed = std::env::var(SEED_ENV_VAR)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_SEED);

        info!("GameRng initialized with seed: {}", seed);
        Self::new(seed)
    }

    /// Generate next u64 value using LCG algorithm
    pub fn next_u64(&mut self) -> u64 {
        // LCG parameters (same as glibc)
        const A: u64 = 1103515245;
        const C: u64 = 12345;
        const M: u64 = 1 << 31;

        self.state = (A.wrapping_mul(self.state).wrapping_add(C)) % M;
        self.state
    }

    /// Generate random f32 between 0.0 and 1.0
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() as f32) / ((1u64 << 31) as f32)
    }

    /// Generate random u32 in range [0, max)
    pub fn next_u32(&mut self, max: u32) -> u32 {
        if max == 0 {
            return 0;
        }
        (self.next_u64() % max as u64) as u32
    }

    /// Generate random bool with given probability
    pub fn next_bool(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }

    /// Reset RNG to initial seed for reproducibility
    pub fn reset(&mut self) {
        self.state = self.initial_seed;
    }

    /// Get the initial seed (for debugging)
    pub fn seed(&self) -> u64 {
        self.initial_seed
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Plugin to set up deterministic RNG
pub struct DeterministicRngPlugin;

impl Plugin for DeterministicRngPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameRng>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = GameRng::new(42);
        let mut rng2 = GameRng::new(42);

        // Same seed should produce same sequence
        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_rng_different_seeds() {
        let mut rng1 = GameRng::new(42);
        let mut rng2 = GameRng::new(43);

        // Different seeds should produce different sequences
        let mut same_count = 0;
        for _ in 0..100 {
            if rng1.next_u64() == rng2.next_u64() {
                same_count += 1;
            }
        }
        assert!(
            same_count < 5,
            "Different seeds should produce mostly different values"
        );
    }

    #[test]
    fn test_rng_reset() {
        let mut rng = GameRng::new(42);
        let first_values: Vec<_> = (0..10).map(|_| rng.next_u64()).collect();

        rng.reset();
        let second_values: Vec<_> = (0..10).map(|_| rng.next_u64()).collect();

        assert_eq!(
            first_values, second_values,
            "Reset should produce same sequence"
        );
    }

    #[test]
    fn test_next_f32_range() {
        let mut rng = GameRng::new(12345);
        for _ in 0..1000 {
            let val = rng.next_f32();
            assert!(val >= 0.0 && val < 1.0, "next_f32 should be in [0, 1)");
        }
    }

    #[test]
    fn test_next_u32_range() {
        let mut rng = GameRng::new(12345);
        for max in [1, 10, 100, 1000] {
            for _ in 0..100 {
                let val = rng.next_u32(max);
                assert!(val < max, "next_u32({}) returned {}", max, val);
            }
        }
    }

    #[test]
    fn test_next_bool_distribution() {
        let mut rng = GameRng::new(12345);

        // Test 50% probability
        let mut true_count = 0;
        for _ in 0..1000 {
            if rng.next_bool(0.5) {
                true_count += 1;
            }
        }
        // Should be roughly 50% (with some tolerance)
        assert!(
            true_count > 400 && true_count < 600,
            "50% probability should give ~500 true values, got {}",
            true_count
        );

        // Test extreme probabilities
        let mut rng2 = GameRng::new(12345);
        let all_false = (0..100).all(|_| !rng2.next_bool(0.0));
        assert!(all_false, "0% probability should always be false");

        let mut rng3 = GameRng::new(12345);
        let all_true = (0..100).all(|_| rng3.next_bool(1.0));
        assert!(all_true, "100% probability should always be true");
    }
}
