//! World and chunk management system

use crate::block_type::BlockType;
use crate::constants::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::tasks::Task;
use std::collections::HashMap;

/// Marker for chunk mesh entity (single mesh per chunk)
#[derive(Component)]
pub(crate) struct ChunkMesh {
    pub coord: IVec2,
}

/// Resource to track pending chunk mesh generation tasks
#[derive(Resource, Default)]
pub(crate) struct ChunkMeshTasks {
    /// Tasks generating chunk meshes (coord -> task)
    pub tasks: HashMap<IVec2, Task<ChunkMeshData>>,
}

/// Data for a generated chunk mesh (sent from async task)
pub(crate) struct ChunkMeshData {
    #[allow(dead_code)]
    pub coord: IVec2,
    #[allow(dead_code)]
    pub mesh: Mesh,
    /// Block positions for this chunk (for raycasting/breaking)
    pub blocks: HashMap<IVec3, BlockType>,
}

/// Single chunk data - blocks stored in a flat array for fast access
/// Array index = x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
#[derive(Clone)]
pub(crate) struct ChunkData {
    /// Flat array of blocks. None = air
    pub blocks: Vec<Option<BlockType>>,
    /// HashMap for compatibility with existing code (lazy populated)
    pub blocks_map: HashMap<IVec3, BlockType>,
}

impl ChunkData {
    pub const ARRAY_SIZE: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize;

    /// Convert local position to array index
    #[inline(always)]
    pub fn pos_to_index(x: i32, y: i32, z: i32) -> usize {
        (x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE) as usize
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
        let mut blocks = vec![None; Self::ARRAY_SIZE];
        let mut blocks_map = HashMap::new();

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
                    // Skip ground layer in delivery platform area
                    if y == GROUND_LEVEL && Self::is_platform_area(world_x, world_z) {
                        continue;
                    }

                    let block_type = if y == GROUND_LEVEL {
                        // Surface layer: show ore in patches based on biome
                        if is_ore_patch && !Self::is_platform_area(world_x, world_z) {
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
                                    BlockType::IronOre  // ~12.5% iron
                                } else if y <= 4 && hash % 20 == 1 {
                                    BlockType::Coal     // ~5% coal
                                } else {
                                    BlockType::Stone
                                }
                            }
                            2 => {
                                // Copper biome: higher copper, some iron
                                if y <= 5 && hash % 8 == 0 {
                                    BlockType::CopperOre // ~12.5% copper
                                } else if y <= 4 && hash % 25 == 1 {
                                    BlockType::IronOre   // ~4% iron
                                } else {
                                    BlockType::Stone
                                }
                            }
                            3 => {
                                // Coal biome: high coal, some iron/copper
                                if y <= 6 && hash % 6 == 0 {
                                    BlockType::Coal      // ~16% coal
                                } else if y <= 3 && hash % 30 == 1 {
                                    BlockType::IronOre   // ~3% iron
                                } else if y <= 3 && hash % 30 == 2 {
                                    BlockType::CopperOre // ~3% copper
                                } else {
                                    BlockType::Stone
                                }
                            }
                            _ => {
                                // Mixed biome: original distribution
                                if y <= 4 && hash % 20 == 0 {
                                    BlockType::IronOre   // 5% iron
                                } else if y <= 3 && hash % 25 == 1 {
                                    BlockType::CopperOre // 4% copper
                                } else if y <= 5 && hash % 15 == 2 {
                                    BlockType::Coal      // ~7% coal
                                } else {
                                    BlockType::Stone
                                }
                            }
                        }
                    };
                    let idx = Self::pos_to_index(x, y, z);
                    blocks[idx] = Some(block_type);
                    blocks_map.insert(IVec3::new(x, y, z), block_type);
                }
            }
        }
        Self { blocks, blocks_map }
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
        if !(0..CHUNK_SIZE).contains(&x) || !(0..CHUNK_HEIGHT).contains(&y) || !(0..CHUNK_SIZE).contains(&z) {
            return None;
        }
        self.blocks[Self::pos_to_index(x, y, z)]
    }

    /// Check if a block exists at local position
    #[inline(always)]
    #[allow(dead_code)]
    pub fn has_block_at(&self, local_pos: IVec3) -> bool {
        self.get_block(local_pos.x, local_pos.y, local_pos.z).is_some()
    }

    /// Generate a combined mesh for the entire chunk with face culling
    /// neighbor_checker: function to check if a block exists at world position (for cross-chunk checks)
    pub fn generate_mesh_with_neighbors<F>(&self, chunk_coord: IVec2, neighbor_checker: F) -> Mesh
    where
        F: Fn(IVec3) -> bool,
    {
        // Pre-allocate with estimated capacity (reduces reallocations)
        let estimated_faces = (CHUNK_SIZE * CHUNK_SIZE * 2) as usize; // roughly top + sides
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(estimated_faces * 4);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(estimated_faces * 4);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(estimated_faces * 4);
        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(estimated_faces * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(estimated_faces * 6);

        // Face definitions: (dx, dy, dz, vertices offsets)
        // Vertices ordered so that cross(v1-v0, v2-v0) points in normal direction
        // Triangle indices: 0,1,2 and 0,2,3
        let faces: [(i32, i32, i32, [[f32; 3]; 4]); 6] = [
            // +Y (top): normal = (0,1,0)
            (0, 1, 0, [
                [0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]
            ]),
            // -Y (bottom): normal = (0,-1,0)
            (0, -1, 0, [
                [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]
            ]),
            // +X (east): normal = (1,0,0) - reversed order
            (1, 0, 0, [
                [1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0]
            ]),
            // -X (west): normal = (-1,0,0) - reversed order
            (-1, 0, 0, [
                [0.0, 1.0, 1.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]
            ]),
            // +Z (south): normal = (0,0,1) - reversed order
            (0, 0, 1, [
                [1.0, 1.0, 1.0], [0.0, 1.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0, 1.0]
            ]),
            // -Z (north): normal = (0,0,-1) - reversed order
            (0, 0, -1, [
                [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]
            ]),
        ];

        // Cache chunk world offset
        let chunk_world_x = (chunk_coord.x * CHUNK_SIZE) as f32;
        let chunk_world_z = (chunk_coord.y * CHUNK_SIZE) as f32;

        // Iterate in Y-Z-X order for better cache locality
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block_type = match self.get_block(x, y, z) {
                        Some(bt) => bt,
                        None => continue,
                    };

                    let base_x = chunk_world_x + x as f32;
                    let base_y = y as f32;
                    let base_z = chunk_world_z + z as f32;

                    let color = block_type.color();
                    let color_arr = [color.to_srgba().red, color.to_srgba().green, color.to_srgba().blue, 1.0];

                    for (dx, dy, dz, verts) in &faces {
                        // Fast neighbor check using array
                        let nx = x + dx;
                        let ny = y + dy;
                        let nz = z + dz;

                        // Check if neighbor exists
                        let neighbor_exists = if (0..CHUNK_SIZE).contains(&nx)
                            && (0..CHUNK_HEIGHT).contains(&ny)
                            && (0..CHUNK_SIZE).contains(&nz)
                        {
                            // Within this chunk - use fast array access
                            self.blocks[Self::pos_to_index(nx, ny, nz)].is_some()
                        } else if !(0..CHUNK_HEIGHT).contains(&ny) {
                            // Above or below world bounds - no block
                            false
                        } else {
                            // Cross-chunk boundary - use neighbor checker
                            let world_pos = IVec3::new(
                                chunk_coord.x * CHUNK_SIZE + nx,
                                ny,
                                chunk_coord.y * CHUNK_SIZE + nz,
                            );
                            neighbor_checker(world_pos)
                        };

                        if neighbor_exists {
                            continue; // Skip this face, it's hidden
                        }

                        let base_idx = positions.len() as u32;
                        let normal = [*dx as f32, *dy as f32, *dz as f32];

                        // Add 4 vertices for this face
                        for vert in verts {
                            positions.push([
                                base_x + vert[0],
                                base_y + vert[1],
                                base_z + vert[2],
                            ]);
                            normals.push(normal);
                            uvs.push([0.0, 0.0]);
                            colors.push(color_arr);
                        }

                        // Add 2 triangles (6 indices) for this face
                        // Standard order: vertices are already CCW when viewed from outside
                        indices.extend_from_slice(&[
                            base_idx, base_idx + 1, base_idx + 2,
                            base_idx, base_idx + 2, base_idx + 3,
                        ]);
                    }
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
        mesh
    }

    /// Simple mesh generation without neighbor checking (for async tasks)
    pub fn generate_mesh(&self, chunk_coord: IVec2) -> Mesh {
        self.generate_mesh_with_neighbors(chunk_coord, |_| false)
    }
}

/// World data - manages multiple chunks
#[derive(Resource, Default)]
pub(crate) struct WorldData {
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
    pub fn get_block(&self, world_pos: IVec3) -> Option<&BlockType> {
        let chunk_coord = Self::world_to_chunk(world_pos);
        let local_pos = Self::world_to_local(world_pos);
        self.chunks.get(&chunk_coord)?.blocks_map.get(&local_pos)
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
            chunk.blocks_map.insert(local_pos, block_type);
        }
        // Persist player modification for chunk reload
        self.modified_blocks.insert(world_pos, Some(block_type));
    }

    /// Remove block at world position, returns the removed block type
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
        chunk.blocks_map.remove(&local_pos);
        // Persist player modification for chunk reload (None = air/removed)
        self.modified_blocks.insert(world_pos, None);
        block
    }

    /// Check if block exists at world position
    pub fn has_block(&self, world_pos: IVec3) -> bool {
        self.get_block(world_pos).is_some()
    }

    /// Generate mesh for a chunk with proper neighbor checking across chunk boundaries
    pub fn generate_chunk_mesh(&self, chunk_coord: IVec2) -> Option<Mesh> {
        let chunk_data = self.chunks.get(&chunk_coord)?;
        let mesh = chunk_data.generate_mesh_with_neighbors(chunk_coord, |world_pos| {
            self.has_block(world_pos)
        });
        Some(mesh)
    }
}
