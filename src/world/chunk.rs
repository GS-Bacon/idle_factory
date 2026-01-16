//! Chunk data structures and types
//!
//! Contains ChunkData, ChunkLod, ChunkMesh and related types for chunk management.

use crate::constants::*;
use crate::core::{items, ItemId};
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::tasks::Task;
use std::collections::HashMap;

/// LOD level for chunk meshes
/// Lower values = more detail
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChunkLod {
    /// Full detail - all blocks rendered
    #[default]
    Full = 0,
    /// Medium detail - only top 3 layers (y >= GROUND_LEVEL - 2)
    Medium = 1,
    /// Low detail - only surface layer (y == GROUND_LEVEL)
    Low = 2,
}

impl ChunkLod {
    /// Calculate LOD level based on distance in chunks
    pub fn from_distance(distance: i32) -> Self {
        match distance {
            0..=1 => ChunkLod::Full,
            2..=3 => ChunkLod::Medium,
            _ => ChunkLod::Low,
        }
    }

    /// Get minimum Y level to render for this LOD
    pub fn min_y(&self) -> i32 {
        use crate::constants::GROUND_LEVEL;
        match self {
            ChunkLod::Full => 0,
            ChunkLod::Medium => (GROUND_LEVEL - 2).max(0),
            ChunkLod::Low => GROUND_LEVEL,
        }
    }
}

/// Marker for chunk mesh entity (single mesh per chunk)
#[derive(Component)]
pub struct ChunkMesh {
    pub coord: IVec2,
    pub lod: ChunkLod,
}

/// Data for a generated chunk mesh (sent from async task)
pub struct ChunkMeshData {
    #[allow(dead_code)]
    pub coord: IVec2,
    #[allow(dead_code)]
    pub mesh: Mesh,
    /// Block positions for this chunk (for raycasting/breaking)
    pub blocks: HashMap<IVec3, ItemId>,
}

impl Default for ChunkMeshData {
    fn default() -> Self {
        Self {
            coord: IVec2::ZERO,
            mesh: Mesh::new(PrimitiveTopology::TriangleList, default()),
            blocks: HashMap::new(),
        }
    }
}

/// Pending chunk state (async task)
pub enum PendingChunk {
    Task(Task<ChunkMeshData>),
}

/// Resource to track pending chunk mesh generation
#[derive(Resource, Default)]
pub struct ChunkMeshTasks {
    /// Pending chunk generation (coord -> state)
    pub pending: HashMap<IVec2, PendingChunk>,
}

/// Resource to track chunks that need mesh regeneration due to block changes
/// This enables batched mesh updates instead of immediate per-block regeneration
#[derive(Resource, Default)]
pub struct DirtyChunks {
    /// Set of chunk coordinates that need mesh regeneration
    pub chunks: std::collections::HashSet<IVec2>,
}

impl DirtyChunks {
    /// Mark a chunk and its affected neighbors as needing mesh regeneration
    pub fn mark_dirty(&mut self, chunk_coord: IVec2, local_pos: IVec3) {
        use crate::constants::CHUNK_SIZE;

        // Always mark the changed chunk
        self.chunks.insert(chunk_coord);

        // Mark neighbors if block is at boundary
        if local_pos.x == 0 {
            self.chunks
                .insert(IVec2::new(chunk_coord.x - 1, chunk_coord.y));
        }
        if local_pos.x == CHUNK_SIZE - 1 {
            self.chunks
                .insert(IVec2::new(chunk_coord.x + 1, chunk_coord.y));
        }
        if local_pos.z == 0 {
            self.chunks
                .insert(IVec2::new(chunk_coord.x, chunk_coord.y - 1));
        }
        if local_pos.z == CHUNK_SIZE - 1 {
            self.chunks
                .insert(IVec2::new(chunk_coord.x, chunk_coord.y + 1));
        }
    }

    /// Take all dirty chunks, clearing the set
    pub fn take_all(&mut self) -> std::collections::HashSet<IVec2> {
        std::mem::take(&mut self.chunks)
    }

    /// Check if any chunks need regeneration
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

/// Single chunk data - blocks stored in a flat array for fast access
/// Array index = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
#[derive(Clone)]
pub struct ChunkData {
    /// Flat array of blocks. None = air
    pub blocks: Vec<Option<ItemId>>,
}

impl ChunkData {
    pub const ARRAY_SIZE: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize;

    /// Convert local position to array index
    /// Panics if coordinates are out of bounds in debug mode
    #[inline(always)]
    pub fn pos_to_index(x: i32, y: i32, z: i32) -> usize {
        debug_assert!(
            (0..CHUNK_SIZE).contains(&x)
                && (0..CHUNK_HEIGHT).contains(&y)
                && (0..CHUNK_SIZE).contains(&z),
            "pos_to_index out of bounds: ({}, {}, {})",
            x,
            y,
            z
        );
        (x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE) as usize
    }

    /// Safe version of pos_to_index that returns None for out-of-bounds coordinates
    #[inline(always)]
    pub fn pos_to_index_checked(x: i32, y: i32, z: i32) -> Option<usize> {
        if (0..CHUNK_SIZE).contains(&x)
            && (0..CHUNK_HEIGHT).contains(&y)
            && (0..CHUNK_SIZE).contains(&z)
        {
            Some((x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE) as usize)
        } else {
            None
        }
    }

    /// Convert array index to local position
    #[inline(always)]
    #[allow(dead_code)]
    pub fn index_to_pos(idx: usize) -> IVec3 {
        let idx = idx as i32;
        let y = idx / (CHUNK_SIZE * CHUNK_SIZE);
        let remaining = idx % (CHUNK_SIZE * CHUNK_SIZE);
        let z = remaining / CHUNK_SIZE;
        let x = remaining % CHUNK_SIZE;
        IVec3::new(x, y, z)
    }

    /// Check if world position is in the delivery platform area
    #[inline(always)]
    pub fn is_platform_area(world_x: i32, world_z: i32) -> bool {
        // Platform is at (20, 8, 10) with size 12x12
        // Clear the top layer (y=7, which is CHUNK_HEIGHT-1) in the platform area
        const PLATFORM_X_MIN: i32 = 20;
        const PLATFORM_X_MAX: i32 = 31; // 20 + 12 - 1
        const PLATFORM_Z_MIN: i32 = 10;
        const PLATFORM_Z_MAX: i32 = 21; // 10 + 12 - 1

        (PLATFORM_X_MIN..=PLATFORM_X_MAX).contains(&world_x)
            && (PLATFORM_Z_MIN..=PLATFORM_Z_MAX).contains(&world_z)
    }

    /// Generate a chunk at the given chunk coordinate
    pub fn generate(chunk_coord: IVec2) -> Self {
        tracing::debug!("Generating chunk at {:?}", chunk_coord);
        let mut blocks = vec![None; Self::ARRAY_SIZE];
        let mut block_count = 0usize;

        // Generate a 16x16x8 chunk of blocks
        // Bottom layers are stone with ore veins, top layer is grass or ore
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = chunk_coord.x * CHUNK_SIZE + x;
                let world_z = chunk_coord.y * CHUNK_SIZE + z;

                // Get biome for this position
                let biome = Self::get_biome(world_x, world_z);
                let is_ore_patch = Self::is_surface_ore_patch(world_x, world_z);

                // Only generate blocks up to GROUND_LEVEL (y <= 7)
                for y in 0..=GROUND_LEVEL {
                    // Platform area: generate stone at ground level (no skip)
                    // This ensures no "hole" appears under the delivery platform

                    let item_id = if y == GROUND_LEVEL {
                        // Platform area: always stone at ground level
                        if Self::is_platform_area(world_x, world_z) {
                            items::stone()
                        } else if is_ore_patch {
                            // Surface layer: show ore in patches based on biome
                            match biome {
                                1 => items::iron_ore(),   // Iron biome
                                2 => items::copper_ore(), // Copper biome
                                3 => items::coal(),       // Coal biome
                                _ => items::grass(),      // Mixed biome
                            }
                        } else {
                            items::grass()
                        }
                    } else {
                        // Underground: biome-weighted ore distribution
                        let hash = Self::simple_hash(world_x, y, world_z);

                        match biome {
                            1 => {
                                // Iron biome: higher iron, some coal
                                if y <= 5 && hash % 8 == 0 {
                                    items::iron_ore() // ~12.5% iron
                                } else if y <= 4 && hash % 20 == 1 {
                                    items::coal() // ~5% coal
                                } else {
                                    items::stone()
                                }
                            }
                            2 => {
                                // Copper biome: higher copper, some iron
                                if y <= 5 && hash % 8 == 0 {
                                    items::copper_ore() // ~12.5% copper
                                } else if y <= 4 && hash % 25 == 1 {
                                    items::iron_ore() // ~4% iron
                                } else {
                                    items::stone()
                                }
                            }
                            3 => {
                                // Coal biome: high coal, some iron/copper
                                if y <= 6 && hash % 6 == 0 {
                                    items::coal() // ~16% coal
                                } else if y <= 3 && hash % 30 == 1 {
                                    items::iron_ore() // ~3% iron
                                } else if y <= 3 && hash % 30 == 2 {
                                    items::copper_ore() // ~3% copper
                                } else {
                                    items::stone()
                                }
                            }
                            _ => {
                                // Mixed biome: original distribution
                                if y <= 4 && hash % 20 == 0 {
                                    items::iron_ore() // 5% iron
                                } else if y <= 3 && hash % 25 == 1 {
                                    items::copper_ore() // 4% copper
                                } else if y <= 5 && hash % 15 == 2 {
                                    items::coal() // ~7% coal
                                } else {
                                    items::stone()
                                }
                            }
                        }
                    };
                    let idx = Self::pos_to_index(x, y, z);
                    blocks[idx] = Some(item_id);
                    block_count += 1;
                }
            }
        }
        tracing::debug!(
            "Chunk {:?} generated with {} blocks",
            chunk_coord,
            block_count
        );
        Self { blocks }
    }

    /// Simple hash function for deterministic ore generation
    #[inline(always)]
    pub fn simple_hash(x: i32, y: i32, z: i32) -> u32 {
        let mut h = (x as u32).wrapping_mul(374761393);
        h = h.wrapping_add((y as u32).wrapping_mul(668265263));
        h = h.wrapping_add((z as u32).wrapping_mul(2147483647));
        h ^= h >> 13;
        h = h.wrapping_mul(1274126177);
        h ^= h >> 16;
        h
    }

    /// Determine biome type based on world coordinates
    /// Returns: 0=Mixed, 1=Iron, 2=Copper, 3=Coal
    #[inline(always)]
    pub fn get_biome(world_x: i32, world_z: i32) -> u8 {
        // Use larger scale hash for biome regions (32-block regions)
        let region_x = world_x.div_euclid(32);
        let region_z = world_z.div_euclid(32);
        let biome_hash = Self::simple_hash(region_x, 0, region_z);

        // Assign biomes based on hash
        match biome_hash % 10 {
            0..=2 => 1, // 30% Iron biome
            3..=5 => 2, // 30% Copper biome
            6..=7 => 3, // 20% Coal biome
            _ => 0,     // 20% Mixed
        }
    }

    /// Check if position should have surface ore (visible ore patch)
    #[inline(always)]
    pub fn is_surface_ore_patch(world_x: i32, world_z: i32) -> bool {
        // Create ore patches every 8-12 blocks based on hash
        let patch_hash = Self::simple_hash(world_x.div_euclid(4), 100, world_z.div_euclid(4));
        patch_hash.is_multiple_of(8)
    }

    /// Get block at local position (fast array access)
    #[inline(always)]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<ItemId> {
        if !(0..CHUNK_SIZE).contains(&x)
            || !(0..CHUNK_HEIGHT).contains(&y)
            || !(0..CHUNK_SIZE).contains(&z)
        {
            return None;
        }
        self.blocks[Self::pos_to_index(x, y, z)]
    }

    /// Check if a block exists at local position
    #[inline(always)]
    #[allow(dead_code)]
    pub fn has_block_at(&self, local_pos: IVec3) -> bool {
        self.get_block(local_pos.x, local_pos.y, local_pos.z)
            .is_some()
    }
}
