//! Texture resolver trait for MOD-extensible texture resolution
//!
//! This module provides the extension point for MODs to customize
//! block textures based on context (neighbors, state, randomness, etc.)

use bevy::prelude::*;
use std::collections::HashMap;

use super::atlas::UVRect;
use crate::core::ItemId;

/// Information about neighboring blocks
#[derive(Clone, Debug, Default)]
pub struct NeighborInfo {
    /// Whether each neighbor exists and is the same block type
    pub same_type: [bool; 6], // [+X, -X, +Y, -Y, +Z, -Z]
    /// Neighbor block types (if any)
    pub neighbors: [Option<ItemId>; 6],
}

impl NeighborInfo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set neighbor at direction
    pub fn set(&mut self, direction: usize, item_id: Option<ItemId>, same_type: bool) {
        if direction < 6 {
            self.neighbors[direction] = item_id;
            self.same_type[direction] = same_type;
        }
    }

    /// Get connectivity bitmask (for connected textures)
    /// Bit 0 = +X, Bit 1 = -X, Bit 2 = +Y, etc.
    pub fn connectivity_mask(&self) -> u8 {
        let mut mask = 0u8;
        for (i, &same) in self.same_type.iter().enumerate() {
            if same {
                mask |= 1 << i;
            }
        }
        mask
    }
}

/// Block state information
#[derive(Clone, Debug, Default)]
pub struct BlockState {
    /// State properties (e.g., "facing" -> "north")
    pub properties: HashMap<String, String>,
    /// World position
    pub position: IVec3,
    /// Random seed for this position (for random textures)
    pub random_seed: u32,
}

impl BlockState {
    pub fn new(position: IVec3) -> Self {
        // Generate deterministic random seed from position
        let seed = ((position.x as u32).wrapping_mul(73856093))
            ^ ((position.y as u32).wrapping_mul(19349663))
            ^ ((position.z as u32).wrapping_mul(83492791));
        Self {
            properties: HashMap::new(),
            position,
            random_seed: seed,
        }
    }

    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }

    /// Get a property value
    pub fn get(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }
}

/// Result of texture resolution
#[derive(Clone, Debug)]
pub enum TextureResult {
    /// Use atlas texture with given UV coordinates
    Atlas {
        uv: UVRect,
        /// Optional tint color (for biome coloring)
        tint: Option<Color>,
    },
    /// Replace with a custom 3D model
    Model {
        /// Model asset path
        model_path: String,
        /// Transform to apply
        transform: Transform,
    },
    /// This resolver doesn't handle this block, pass to next
    Pass,
    /// Use a solid color (fallback)
    Color(Color),
}

/// Trait for texture resolution
///
/// MODs can implement this trait to customize texture behavior:
/// - Connected textures (glass, bricks)
/// - Random textures (stone variants)
/// - Animated textures
/// - Custom model replacement
pub trait TextureResolver: Send + Sync + 'static {
    /// Resolve texture for a block face
    ///
    /// # Arguments
    /// * `block_id` - The block's ItemId
    /// * `face` - Which face (0-5: +X, -X, +Y, -Y, +Z, -Z)
    /// * `state` - Block state and position
    /// * `neighbors` - Information about neighboring blocks
    ///
    /// # Returns
    /// `TextureResult` indicating how to render this face
    fn resolve(
        &self,
        block_id: ItemId,
        face: u8,
        state: &BlockState,
        neighbors: &NeighborInfo,
    ) -> TextureResult;

    /// Priority for this resolver (higher = checked first)
    fn priority(&self) -> i32 {
        0
    }

    /// Name for debugging
    fn name(&self) -> &str {
        "unnamed"
    }
}

/// Default texture resolver using the atlas
#[allow(dead_code)]
pub struct DefaultTextureResolver {
    // Reference to texture registry accessed via Res<TextureRegistry>
}

#[allow(dead_code)]
impl DefaultTextureResolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DefaultTextureResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureResolver for DefaultTextureResolver {
    fn resolve(
        &self,
        _block_id: ItemId,
        _face: u8,
        _state: &BlockState,
        _neighbors: &NeighborInfo,
    ) -> TextureResult {
        // Default implementation passes to let the system use the registry
        TextureResult::Pass
    }

    fn priority(&self) -> i32 {
        -1000 // Very low priority, fallback
    }

    fn name(&self) -> &str {
        "default"
    }
}

/// Connected texture resolver for blocks like glass
#[allow(dead_code)]
pub struct ConnectedTextureResolver {
    /// Block IDs that use connected textures
    pub blocks: Vec<ItemId>,
    /// Number of texture variants (typically 47 for full CTM)
    pub variant_count: u32,
}

#[allow(dead_code)]
impl ConnectedTextureResolver {
    pub fn new(blocks: Vec<ItemId>, variant_count: u32) -> Self {
        Self {
            blocks,
            variant_count,
        }
    }

    /// Calculate CTM texture index from neighbor connectivity
    /// Uses standard 47-tile CTM pattern
    pub fn calculate_ctm_index(connectivity: u8) -> u32 {
        // This is a simplified version - full CTM needs corner checks too
        // For now, just return based on side connectivity
        match connectivity & 0x0F {
            // Horizontal connectivity only (ignoring Y)
            0b0000 => 0, // Isolated
            0b0001 => 1, // +X connected
            0b0010 => 2, // -X connected
            0b0011 => 3, // X axis connected
            0b0100 => 4, // +Z connected
            0b1000 => 5, // -Z connected
            0b1100 => 6, // Z axis connected
            _ => 0,      // Complex patterns
        }
    }
}

impl TextureResolver for ConnectedTextureResolver {
    fn resolve(
        &self,
        block_id: ItemId,
        _face: u8,
        _state: &BlockState,
        neighbors: &NeighborInfo,
    ) -> TextureResult {
        if !self.blocks.contains(&block_id) {
            return TextureResult::Pass;
        }

        let _index = Self::calculate_ctm_index(neighbors.connectivity_mask());

        // TODO: Return actual CTM UV coordinates
        // For now, pass to default
        TextureResult::Pass
    }

    fn priority(&self) -> i32 {
        100 // High priority for connected textures
    }

    fn name(&self) -> &str {
        "connected"
    }
}

/// Random texture resolver for blocks with variants
#[allow(dead_code)]
pub struct RandomTextureResolver {
    /// Block IDs that use random textures
    pub blocks: HashMap<ItemId, u32>, // ItemId -> variant count
}

#[allow(dead_code)]
impl RandomTextureResolver {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, block_id: ItemId, variant_count: u32) {
        self.blocks.insert(block_id, variant_count);
    }
}

impl Default for RandomTextureResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureResolver for RandomTextureResolver {
    fn resolve(
        &self,
        block_id: ItemId,
        _face: u8,
        state: &BlockState,
        _neighbors: &NeighborInfo,
    ) -> TextureResult {
        if let Some(&variant_count) = self.blocks.get(&block_id) {
            let variant = state.random_seed % variant_count;
            // TODO: Return UV for specific variant
            let _ = variant;
            return TextureResult::Pass;
        }
        TextureResult::Pass
    }

    fn priority(&self) -> i32 {
        50 // Medium priority
    }

    fn name(&self) -> &str {
        "random"
    }
}

/// Registry of texture resolvers
#[allow(dead_code)]
#[derive(Resource, Default)]
pub struct TextureResolverRegistry {
    resolvers: Vec<Box<dyn TextureResolver>>,
}

#[allow(dead_code)]
impl TextureResolverRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register(Box::new(DefaultTextureResolver::new()));
        registry
    }

    /// Register a texture resolver
    pub fn register(&mut self, resolver: Box<dyn TextureResolver>) {
        self.resolvers.push(resolver);
        // Sort by priority (highest first)
        self.resolvers
            .sort_by_key(|r| std::cmp::Reverse(r.priority()));
    }

    /// Resolve texture for a block face
    pub fn resolve(
        &self,
        block_id: ItemId,
        face: u8,
        state: &BlockState,
        neighbors: &NeighborInfo,
    ) -> TextureResult {
        for resolver in &self.resolvers {
            match resolver.resolve(block_id, face, state, neighbors) {
                TextureResult::Pass => continue,
                result => return result,
            }
        }
        // Final fallback
        TextureResult::Color(Color::srgb(1.0, 0.0, 1.0)) // Magenta for missing
    }

    /// Get resolver count
    pub fn count(&self) -> usize {
        self.resolvers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighbor_info_connectivity_mask() {
        let mut info = NeighborInfo::new();
        info.same_type[0] = true; // +X
        info.same_type[2] = true; // +Y

        let mask = info.connectivity_mask();
        assert_eq!(mask, 0b00000101); // Bits 0 and 2 set
    }

    #[test]
    fn test_block_state_random_seed() {
        let state1 = BlockState::new(IVec3::new(0, 0, 0));
        let state2 = BlockState::new(IVec3::new(1, 0, 0));
        let state3 = BlockState::new(IVec3::new(0, 0, 0));

        // Same position = same seed
        assert_eq!(state1.random_seed, state3.random_seed);
        // Different position = different seed
        assert_ne!(state1.random_seed, state2.random_seed);
    }

    #[test]
    fn test_resolver_registry_priority() {
        let mut registry = TextureResolverRegistry::new();

        struct HighPriority;
        impl TextureResolver for HighPriority {
            fn resolve(&self, _: ItemId, _: u8, _: &BlockState, _: &NeighborInfo) -> TextureResult {
                TextureResult::Pass
            }
            fn priority(&self) -> i32 {
                1000
            }
            fn name(&self) -> &str {
                "high"
            }
        }

        registry.register(Box::new(HighPriority));

        // High priority should be first
        assert_eq!(registry.resolvers[0].name(), "high");
    }
}
