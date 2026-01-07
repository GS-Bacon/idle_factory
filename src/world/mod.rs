//! World and chunk management system
//!
//! ## ItemId Support
//!
//! For ItemId-based APIs, use `get_block_id()`, `set_block_by_id()`.

pub mod biome;

pub use biome::{mining_random, BiomeMap};

use crate::block_type::BlockType;
use crate::constants::*;
use crate::core::ItemId;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
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
    pub blocks: HashMap<IVec3, BlockType>,
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
    pub blocks: Vec<Option<BlockType>>,
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

                    let block_type = if y == GROUND_LEVEL {
                        // Platform area: always stone at ground level
                        if Self::is_platform_area(world_x, world_z) {
                            BlockType::Stone
                        } else if is_ore_patch {
                            // Surface layer: show ore in patches based on biome
                            match biome {
                                1 => BlockType::IronOre,   // Iron biome
                                2 => BlockType::CopperOre, // Copper biome
                                3 => BlockType::Coal,      // Coal biome
                                _ => BlockType::Grass,     // Mixed biome
                            }
                        } else {
                            BlockType::Grass
                        }
                    } else {
                        // Underground: biome-weighted ore distribution
                        let hash = Self::simple_hash(world_x, y, world_z);

                        match biome {
                            1 => {
                                // Iron biome: higher iron, some coal
                                if y <= 5 && hash % 8 == 0 {
                                    BlockType::IronOre // ~12.5% iron
                                } else if y <= 4 && hash % 20 == 1 {
                                    BlockType::Coal // ~5% coal
                                } else {
                                    BlockType::Stone
                                }
                            }
                            2 => {
                                // Copper biome: higher copper, some iron
                                if y <= 5 && hash % 8 == 0 {
                                    BlockType::CopperOre // ~12.5% copper
                                } else if y <= 4 && hash % 25 == 1 {
                                    BlockType::IronOre // ~4% iron
                                } else {
                                    BlockType::Stone
                                }
                            }
                            3 => {
                                // Coal biome: high coal, some iron/copper
                                if y <= 6 && hash % 6 == 0 {
                                    BlockType::Coal // ~16% coal
                                } else if y <= 3 && hash % 30 == 1 {
                                    BlockType::IronOre // ~3% iron
                                } else if y <= 3 && hash % 30 == 2 {
                                    BlockType::CopperOre // ~3% copper
                                } else {
                                    BlockType::Stone
                                }
                            }
                            _ => {
                                // Mixed biome: original distribution
                                if y <= 4 && hash % 20 == 0 {
                                    BlockType::IronOre // 5% iron
                                } else if y <= 3 && hash % 25 == 1 {
                                    BlockType::CopperOre // 4% copper
                                } else if y <= 5 && hash % 15 == 2 {
                                    BlockType::Coal // ~7% coal
                                } else {
                                    BlockType::Stone
                                }
                            }
                        }
                    };
                    let idx = Self::pos_to_index(x, y, z);
                    blocks[idx] = Some(block_type);
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
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<BlockType> {
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

    /// Generate a combined mesh for the entire chunk with face culling using greedy meshing
    /// neighbor_checker: function to check if a block exists at world position (for cross-chunk checks)
    /// lod: Level of detail - affects which blocks are included in the mesh
    pub fn generate_mesh_with_neighbors<F>(
        &self,
        chunk_coord: IVec2,
        neighbor_checker: F,
        lod: ChunkLod,
    ) -> Mesh
    where
        F: Fn(IVec3) -> bool,
    {
        let min_y = lod.min_y();
        // Pre-allocate with estimated capacity (greedy meshing produces fewer quads)
        let estimated_faces = (CHUNK_SIZE * CHUNK_SIZE) as usize;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(estimated_faces * 4);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(estimated_faces * 4);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(estimated_faces * 4);
        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(estimated_faces * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(estimated_faces * 6);

        // Cache chunk world offset
        let chunk_world_x = (chunk_coord.x * CHUNK_SIZE) as f32;
        let chunk_world_z = (chunk_coord.y * CHUNK_SIZE) as f32;

        // Helper to check if neighbor exists
        let has_neighbor = |x: i32, y: i32, z: i32, dx: i32, dy: i32, dz: i32| -> bool {
            let nx = x + dx;
            let ny = y + dy;
            let nz = z + dz;
            if (0..CHUNK_SIZE).contains(&nx)
                && (0..CHUNK_HEIGHT).contains(&ny)
                && (0..CHUNK_SIZE).contains(&nz)
            {
                self.blocks[Self::pos_to_index(nx, ny, nz)].is_some()
            } else if !(0..CHUNK_HEIGHT).contains(&ny) {
                false
            } else {
                let world_pos = IVec3::new(
                    chunk_coord.x * CHUNK_SIZE + nx,
                    ny,
                    chunk_coord.y * CHUNK_SIZE + nz,
                );
                neighbor_checker(world_pos)
            }
        };

        // Face data: (axis, positive, axis1_size, axis2_size)
        // axis: 0=X, 1=Y, 2=Z
        // positive: true for +X/+Y/+Z, false for -X/-Y/-Z
        let face_configs: [(usize, bool); 6] = [
            (1, true),  // +Y (top)
            (1, false), // -Y (bottom)
            (0, true),  // +X (east)
            (0, false), // -X (west)
            (2, true),  // +Z (south)
            (2, false), // -Z (north)
        ];

        // Get axis sizes
        let axis_sizes = [CHUNK_SIZE, CHUNK_HEIGHT, CHUNK_SIZE];

        for (axis, positive) in face_configs {
            let (axis1, axis2) = match axis {
                0 => (1, 2), // X: slice along Y-Z
                1 => (0, 2), // Y: slice along X-Z
                2 => (0, 1), // Z: slice along X-Y
                _ => unreachable!(),
            };

            let d = if positive { 1 } else { -1 };

            // Iterate through slices perpendicular to axis
            for slice in 0..axis_sizes[axis] {
                // Create mask for this slice
                // mask[u][v] = Some(BlockType) if face is visible
                let mut mask: Vec<Vec<Option<BlockType>>> =
                    vec![vec![None; axis_sizes[axis2] as usize]; axis_sizes[axis1] as usize];

                for u in 0..axis_sizes[axis1] {
                    for v in 0..axis_sizes[axis2] {
                        let (x, y, z) = match axis {
                            0 => (slice, u, v),
                            1 => (u, slice, v),
                            2 => (u, v, slice),
                            _ => unreachable!(),
                        };

                        // LOD: Skip blocks below min_y threshold
                        if y < min_y {
                            continue;
                        }

                        if let Some(block_type) = self.get_block(x, y, z) {
                            let (dx, dy, dz) = match axis {
                                0 => (d, 0, 0),
                                1 => (0, d, 0),
                                2 => (0, 0, d),
                                _ => unreachable!(),
                            };
                            if !has_neighbor(x, y, z, dx, dy, dz) {
                                mask[u as usize][v as usize] = Some(block_type);
                            }
                        }
                    }
                }

                // Greedy merge the mask into quads
                let mut processed =
                    vec![vec![false; axis_sizes[axis2] as usize]; axis_sizes[axis1] as usize];

                for u in 0..axis_sizes[axis1] as usize {
                    for v in 0..axis_sizes[axis2] as usize {
                        if processed[u][v] || mask[u][v].is_none() {
                            continue;
                        }

                        let block_type = mask[u][v].unwrap();

                        // Find width (extend in v direction)
                        let mut width = 1;
                        while v + width < axis_sizes[axis2] as usize
                            && !processed[u][v + width]
                            && mask[u][v + width] == Some(block_type)
                        {
                            width += 1;
                        }

                        // Find height (extend in u direction)
                        let mut height = 1;
                        'outer: while u + height < axis_sizes[axis1] as usize {
                            for w in 0..width {
                                if processed[u + height][v + w]
                                    || mask[u + height][v + w] != Some(block_type)
                                {
                                    break 'outer;
                                }
                            }
                            height += 1;
                        }

                        // Mark as processed
                        for du in 0..height {
                            for dv in 0..width {
                                processed[u + du][v + dv] = true;
                            }
                        }

                        // Generate quad
                        let color = block_type.color();
                        let color_arr = [
                            color.to_srgba().red,
                            color.to_srgba().green,
                            color.to_srgba().blue,
                            1.0,
                        ];

                        // Calculate corner positions in u-v space
                        let u0f = u as f32;
                        let v0f = v as f32;
                        let u1f = (u + height) as f32;
                        let v1f = (v + width) as f32;

                        // Generate 4 vertices in CCW order when viewed from outside
                        // The order depends on face direction to ensure correct winding
                        let (verts, normal): ([[f32; 3]; 4], [f32; 3]) = match (axis, positive) {
                            (0, true) => {
                                // +X face: looking from +X toward -X
                                // Y is up, Z is left -> CCW: (y_low,z_low), (y_high,z_low), (y_high,z_high), (y_low,z_high)
                                let x = chunk_world_x + (slice + 1) as f32;
                                (
                                    [
                                        [x, u0f, chunk_world_z + v0f],
                                        [x, u1f, chunk_world_z + v0f],
                                        [x, u1f, chunk_world_z + v1f],
                                        [x, u0f, chunk_world_z + v1f],
                                    ],
                                    [1.0, 0.0, 0.0],
                                )
                            }
                            (0, false) => {
                                // -X face: looking from -X toward +X
                                // Y is up, Z is right -> CCW: (y_low,z_low), (y_low,z_high), (y_high,z_high), (y_high,z_low)
                                let x = chunk_world_x + slice as f32;
                                (
                                    [
                                        [x, u0f, chunk_world_z + v0f],
                                        [x, u0f, chunk_world_z + v1f],
                                        [x, u1f, chunk_world_z + v1f],
                                        [x, u1f, chunk_world_z + v0f],
                                    ],
                                    [-1.0, 0.0, 0.0],
                                )
                            }
                            (1, true) => {
                                // +Y face: looking from +Y down
                                // X is right, Z is up -> CCW: (x_low,z_low), (x_low,z_high), (x_high,z_high), (x_high,z_low)
                                let y = (slice + 1) as f32;
                                (
                                    [
                                        [chunk_world_x + u0f, y, chunk_world_z + v0f],
                                        [chunk_world_x + u0f, y, chunk_world_z + v1f],
                                        [chunk_world_x + u1f, y, chunk_world_z + v1f],
                                        [chunk_world_x + u1f, y, chunk_world_z + v0f],
                                    ],
                                    [0.0, 1.0, 0.0],
                                )
                            }
                            (1, false) => {
                                // -Y face: looking from -Y up
                                // X is left, Z is up -> CCW: (x_low,z_low), (x_high,z_low), (x_high,z_high), (x_low,z_high)
                                let y = slice as f32;
                                (
                                    [
                                        [chunk_world_x + u0f, y, chunk_world_z + v0f],
                                        [chunk_world_x + u1f, y, chunk_world_z + v0f],
                                        [chunk_world_x + u1f, y, chunk_world_z + v1f],
                                        [chunk_world_x + u0f, y, chunk_world_z + v1f],
                                    ],
                                    [0.0, -1.0, 0.0],
                                )
                            }
                            (2, true) => {
                                // +Z face: need +X × +Y = +Z
                                // CCW order: (x_low,y_low), (x_high,y_low), (x_high,y_high), (x_low,y_high)
                                let z = chunk_world_z + (slice + 1) as f32;
                                (
                                    [
                                        [chunk_world_x + u0f, v0f, z],
                                        [chunk_world_x + u1f, v0f, z],
                                        [chunk_world_x + u1f, v1f, z],
                                        [chunk_world_x + u0f, v1f, z],
                                    ],
                                    [0.0, 0.0, 1.0],
                                )
                            }
                            (2, false) => {
                                // -Z face: need +Y × +X = -Z
                                // CCW order: (x_low,y_low), (x_low,y_high), (x_high,y_high), (x_high,y_low)
                                let z = chunk_world_z + slice as f32;
                                (
                                    [
                                        [chunk_world_x + u0f, v0f, z],
                                        [chunk_world_x + u0f, v1f, z],
                                        [chunk_world_x + u1f, v1f, z],
                                        [chunk_world_x + u1f, v0f, z],
                                    ],
                                    [0.0, 0.0, -1.0],
                                )
                            }
                            _ => unreachable!(),
                        };

                        let base_idx = positions.len() as u32;

                        // Push vertices in CCW order
                        for vert in &verts {
                            positions.push(*vert);
                        }

                        for _ in 0..4 {
                            normals.push(normal);
                            colors.push(color_arr);
                        }

                        uvs.push([0.0, 0.0]);
                        uvs.push([width as f32, 0.0]);
                        uvs.push([width as f32, height as f32]);
                        uvs.push([0.0, height as f32]);

                        indices.extend_from_slice(&[
                            base_idx,
                            base_idx + 1,
                            base_idx + 2,
                            base_idx,
                            base_idx + 2,
                            base_idx + 3,
                        ]);
                    }
                }
            }
        }

        tracing::info!(
            "Greedy mesh for chunk {:?}: {} vertices, {} indices",
            chunk_coord,
            positions.len(),
            indices.len()
        );

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
        mesh
    }

    /// Simple mesh generation without neighbor checking (for async tasks)
    /// Uses full LOD (all blocks)
    pub fn generate_mesh(&self, chunk_coord: IVec2) -> Mesh {
        self.generate_mesh_with_neighbors(chunk_coord, |_| false, ChunkLod::Full)
    }

    /// Simple mesh generation with LOD
    pub fn generate_mesh_with_lod(&self, chunk_coord: IVec2, lod: ChunkLod) -> Mesh {
        self.generate_mesh_with_neighbors(chunk_coord, |_| false, lod)
    }
}

/// World data - manages multiple chunks
#[derive(Resource, Default)]
pub struct WorldData {
    /// Loaded chunks indexed by chunk coordinate
    pub chunks: HashMap<IVec2, ChunkData>,
    /// Block entities for each chunk (for despawning)
    pub chunk_entities: HashMap<IVec2, Vec<Entity>>,
    /// Player-modified blocks (persists across chunk unload/reload)
    /// Key: world position, Value: Some(block_type) for placed, None for removed (air)
    pub modified_blocks: HashMap<IVec3, Option<BlockType>>,
}

impl WorldData {
    /// Convert world position to chunk coordinate
    pub fn world_to_chunk(world_pos: IVec3) -> IVec2 {
        IVec2::new(
            world_pos.x.div_euclid(CHUNK_SIZE),
            world_pos.z.div_euclid(CHUNK_SIZE),
        )
    }

    /// Convert world position to local chunk position
    pub fn world_to_local(world_pos: IVec3) -> IVec3 {
        IVec3::new(
            world_pos.x.rem_euclid(CHUNK_SIZE),
            world_pos.y,
            world_pos.z.rem_euclid(CHUNK_SIZE),
        )
    }

    /// Convert chunk coord + local pos to world position
    pub fn local_to_world(chunk_coord: IVec2, local_pos: IVec3) -> IVec3 {
        IVec3::new(
            chunk_coord.x * CHUNK_SIZE + local_pos.x,
            local_pos.y,
            chunk_coord.y * CHUNK_SIZE + local_pos.z,
        )
    }

    /// Get block at world position
    pub fn get_block(&self, world_pos: IVec3) -> Option<BlockType> {
        let chunk_coord = Self::world_to_chunk(world_pos);
        let local_pos = Self::world_to_local(world_pos);
        let chunk = self.chunks.get(&chunk_coord)?;
        chunk.get_block(local_pos.x, local_pos.y, local_pos.z)
    }

    /// Set block at world position
    pub fn set_block(&mut self, world_pos: IVec3, block_type: BlockType) {
        let chunk_coord = Self::world_to_chunk(world_pos);
        let local_pos = Self::world_to_local(world_pos);
        if let Some(chunk) = self.chunks.get_mut(&chunk_coord) {
            // Bounds check for y coordinate
            if local_pos.y < 0 || local_pos.y >= CHUNK_HEIGHT {
                return;
            }
            let idx = ChunkData::pos_to_index(local_pos.x, local_pos.y, local_pos.z);
            chunk.blocks[idx] = Some(block_type);
        }
        // Persist player modification for chunk reload
        self.modified_blocks.insert(world_pos, Some(block_type));
    }

    /// Remove block at world position, returns the removed block type
    #[allow(dead_code)] // Used in tests
    pub fn remove_block(&mut self, world_pos: IVec3) -> Option<BlockType> {
        let chunk_coord = Self::world_to_chunk(world_pos);
        let local_pos = Self::world_to_local(world_pos);
        // Bounds check for y coordinate
        if local_pos.y < 0 || local_pos.y >= CHUNK_HEIGHT {
            return None;
        }
        let chunk = self.chunks.get_mut(&chunk_coord)?;
        let idx = ChunkData::pos_to_index(local_pos.x, local_pos.y, local_pos.z);
        let block = chunk.blocks[idx].take();
        // Persist player modification for chunk reload (None = air/removed)
        self.modified_blocks.insert(world_pos, None);
        block
    }

    /// Check if block exists at world position
    pub fn has_block(&self, world_pos: IVec3) -> bool {
        self.get_block(world_pos).is_some()
    }

    // =========================================================================
    // ItemId API
    // =========================================================================

    /// Get block at world position as ItemId
    pub fn get_block_id(&self, world_pos: IVec3) -> Option<ItemId> {
        self.get_block(world_pos).map(|bt| bt.into())
    }

    /// Set block at world position using ItemId
    pub fn set_block_by_id(&mut self, world_pos: IVec3, item_id: ItemId) {
        if let Ok(block_type) = item_id.try_into() {
            self.set_block(world_pos, block_type);
        }
    }

    /// Remove block at world position, returns the removed block as ItemId
    #[allow(dead_code)]
    pub fn remove_block_as_id(&mut self, world_pos: IVec3) -> Option<ItemId> {
        self.remove_block(world_pos).map(|bt| bt.into())
    }

    /// Generate mesh for a chunk with proper neighbor checking across chunk boundaries
    /// Uses full LOD (all blocks)
    pub fn generate_chunk_mesh(&self, chunk_coord: IVec2) -> Option<Mesh> {
        self.generate_chunk_mesh_with_lod(chunk_coord, ChunkLod::Full)
    }

    /// Generate mesh for a chunk with specific LOD level
    pub fn generate_chunk_mesh_with_lod(&self, chunk_coord: IVec2, lod: ChunkLod) -> Option<Mesh> {
        let chunk_data = self.chunks.get(&chunk_coord)?;
        let mesh = chunk_data.generate_mesh_with_neighbors(
            chunk_coord,
            |world_pos| self.has_block(world_pos),
            lod,
        );
        Some(mesh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_data_pos_index_conversion() {
        // Test pos_to_index and index_to_pos are inverses
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_HEIGHT {
                    let idx = ChunkData::pos_to_index(x, y, z);
                    let pos = ChunkData::index_to_pos(idx);
                    assert_eq!(
                        pos,
                        IVec3::new(x, y, z),
                        "Round trip failed for ({}, {}, {})",
                        x,
                        y,
                        z
                    );
                }
            }
        }
    }

    #[test]
    fn test_chunk_data_generate_has_blocks() {
        let chunk = ChunkData::generate(IVec2::ZERO);

        // Ground level should have blocks
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Skip platform area
                if !ChunkData::is_platform_area(x, z) {
                    assert!(
                        chunk.get_block(x, GROUND_LEVEL, z).is_some(),
                        "Expected block at ground level ({}, {}, {})",
                        x,
                        GROUND_LEVEL,
                        z
                    );
                }
            }
        }

        // Above ground level should be empty
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                assert!(
                    chunk.get_block(x, GROUND_LEVEL + 1, z).is_none(),
                    "Expected no block above ground at ({}, {}, {})",
                    x,
                    GROUND_LEVEL + 1,
                    z
                );
            }
        }
    }

    #[test]
    fn test_chunk_data_biome_deterministic() {
        // Same coordinates should always produce same biome
        let biome1 = ChunkData::get_biome(100, 200);
        let biome2 = ChunkData::get_biome(100, 200);
        assert_eq!(biome1, biome2);

        // Biome should be 0-3
        assert!(biome1 <= 3);
    }

    #[test]
    fn test_chunk_data_platform_area() {
        // Platform is at (20, 10) to (31, 21)
        assert!(ChunkData::is_platform_area(20, 10));
        assert!(ChunkData::is_platform_area(31, 21));
        assert!(ChunkData::is_platform_area(25, 15));

        // Outside platform
        assert!(!ChunkData::is_platform_area(19, 10));
        assert!(!ChunkData::is_platform_area(20, 9));
        assert!(!ChunkData::is_platform_area(32, 21));
        assert!(!ChunkData::is_platform_area(31, 22));
    }

    #[test]
    fn test_world_data_coordinate_conversion() {
        // Test world_to_chunk
        assert_eq!(WorldData::world_to_chunk(IVec3::new(0, 0, 0)), IVec2::ZERO);
        assert_eq!(
            WorldData::world_to_chunk(IVec3::new(15, 0, 15)),
            IVec2::ZERO
        );
        assert_eq!(
            WorldData::world_to_chunk(IVec3::new(16, 0, 0)),
            IVec2::new(1, 0)
        );
        assert_eq!(
            WorldData::world_to_chunk(IVec3::new(-1, 0, -1)),
            IVec2::new(-1, -1)
        );

        // Test world_to_local
        assert_eq!(
            WorldData::world_to_local(IVec3::new(0, 5, 0)),
            IVec3::new(0, 5, 0)
        );
        assert_eq!(
            WorldData::world_to_local(IVec3::new(17, 3, 18)),
            IVec3::new(1, 3, 2)
        );
        assert_eq!(
            WorldData::world_to_local(IVec3::new(-1, 2, -1)),
            IVec3::new(15, 2, 15)
        );

        // Test local_to_world
        assert_eq!(
            WorldData::local_to_world(IVec2::ZERO, IVec3::new(5, 3, 7)),
            IVec3::new(5, 3, 7)
        );
        assert_eq!(
            WorldData::local_to_world(IVec2::new(1, 2), IVec3::new(3, 4, 5)),
            IVec3::new(19, 4, 37)
        );
    }

    #[test]
    fn test_world_data_block_operations() {
        let mut world = WorldData::default();

        // Generate a chunk
        let chunk_coord = IVec2::ZERO;
        world
            .chunks
            .insert(chunk_coord, ChunkData::generate(chunk_coord));

        // Get existing block
        let pos = IVec3::new(5, GROUND_LEVEL, 5);
        assert!(world.get_block(pos).is_some());

        // Remove block
        let removed = world.remove_block(pos);
        assert!(removed.is_some());
        assert!(world.get_block(pos).is_none());

        // Set block
        world.set_block(pos, BlockType::Stone);
        assert_eq!(world.get_block(pos), Some(BlockType::Stone));

        // Verify modification is tracked
        assert!(world.modified_blocks.contains_key(&pos));
    }

    #[test]
    fn test_world_data_cross_chunk_query() {
        let mut world = WorldData::default();

        // Generate two adjacent chunks
        world
            .chunks
            .insert(IVec2::new(0, 0), ChunkData::generate(IVec2::new(0, 0)));
        world
            .chunks
            .insert(IVec2::new(1, 0), ChunkData::generate(IVec2::new(1, 0)));

        // Query block in first chunk
        assert!(world.has_block(IVec3::new(0, GROUND_LEVEL, 0)));

        // Query block in second chunk
        assert!(world.has_block(IVec3::new(16, GROUND_LEVEL, 0)));

        // Query non-existent chunk should return false
        assert!(!world.has_block(IVec3::new(100, GROUND_LEVEL, 100)));
    }

    #[test]
    fn test_chunk_mesh_generation() {
        let chunk = ChunkData::generate(IVec2::ZERO);
        let mesh = chunk.generate_mesh(IVec2::ZERO);

        // Mesh should have positions
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION);
        assert!(positions.is_some());

        // Mesh should have non-zero vertices (ground blocks exist)
        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(pos)) = positions {
            assert!(!pos.is_empty(), "Mesh should have vertices");
        }
    }

    // =========================================================================
    // ItemId API tests
    // =========================================================================

    #[test]
    fn test_world_data_item_id_api() {
        use crate::core::items;

        let mut world = WorldData::default();
        let chunk_coord = IVec2::ZERO;
        world
            .chunks
            .insert(chunk_coord, ChunkData::generate(chunk_coord));

        let pos = IVec3::new(5, GROUND_LEVEL, 5);

        // Get block as ItemId
        let block_id = world.get_block_id(pos);
        assert!(block_id.is_some());

        // Set block by ItemId
        world.set_block_by_id(pos, items::stone());
        let new_block = world.get_block_id(pos);
        assert!(new_block.is_some());
        assert_eq!(new_block.unwrap().name(), Some("base:stone"));

        // Remove and get as ItemId
        let removed = world.remove_block_as_id(pos);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name(), Some("base:stone"));
    }
}
