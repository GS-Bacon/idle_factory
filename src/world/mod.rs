//! World and chunk management system
//!
//! ## ItemId Support
//!
//! For ItemId-based APIs, use `get_block_id()`, `set_block_by_id()`.

pub mod biome;
mod chunk;
mod mesh_gen;
#[cfg(test)]
mod tests;

// Explicit re-exports from biome
pub use biome::{mining_random, BiomeMap};

// Explicit re-exports from chunk
pub use chunk::{
    ChunkData, ChunkLod, ChunkMesh, ChunkMeshData, ChunkMeshTasks, DirtyChunks, PendingChunk,
};

use crate::constants::*;
use crate::core::ItemId;
use bevy::prelude::*;
use std::collections::HashMap;

/// World data - manages multiple chunks
#[derive(Resource, Default)]
pub struct WorldData {
    /// Loaded chunks indexed by chunk coordinate
    pub chunks: HashMap<IVec2, ChunkData>,
    /// Block entities for each chunk (for despawning)
    pub chunk_entities: HashMap<IVec2, Vec<Entity>>,
    /// Player-modified blocks (persists across chunk unload/reload)
    /// Key: world position, Value: Some(item_id) for placed, None for removed (air)
    pub modified_blocks: HashMap<IVec3, Option<ItemId>>,
}

impl WorldData {
    /// Log block operation to file (logs/block_ops.log)
    /// Does not print to console to avoid noise
    fn log_block_op(op: &str, status: &str, world_pos: IVec3, item: Option<ItemId>) {
        use std::io::Write;
        use std::sync::Mutex;
        use std::sync::OnceLock;

        static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

        let file = LOG_FILE.get_or_init(|| {
            let _ = std::fs::create_dir_all("logs");
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("logs/block_ops.log")
                .expect("Failed to open block_ops.log");
            Mutex::new(file)
        });

        if let Ok(mut f) = file.lock() {
            let item_name = item.and_then(|id| id.name()).unwrap_or("-");
            let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
            let _ = writeln!(
                f,
                "{} {} {} pos=({},{},{}) item={}",
                timestamp, op, status, world_pos.x, world_pos.y, world_pos.z, item_name
            );
        }
    }

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
    pub fn get_block(&self, world_pos: IVec3) -> Option<ItemId> {
        let chunk_coord = Self::world_to_chunk(world_pos);
        let local_pos = Self::world_to_local(world_pos);
        let chunk = self.chunks.get(&chunk_coord)?;
        chunk.get_block(local_pos.x, local_pos.y, local_pos.z)
    }

    /// Set block at world position
    pub fn set_block(&mut self, world_pos: IVec3, item_id: ItemId) {
        let chunk_coord = Self::world_to_chunk(world_pos);
        let local_pos = Self::world_to_local(world_pos);
        if let Some(chunk) = self.chunks.get_mut(&chunk_coord) {
            // Bounds check for y coordinate
            if local_pos.y < 0 || local_pos.y >= CHUNK_HEIGHT {
                Self::log_block_op("set_block", "Y_OUT_OF_BOUNDS", world_pos, Some(item_id));
                return;
            }
            let idx = ChunkData::pos_to_index(local_pos.x, local_pos.y, local_pos.z);
            chunk.blocks[idx] = Some(item_id);
            Self::log_block_op("set_block", "SUCCESS", world_pos, Some(item_id));
        } else {
            Self::log_block_op("set_block", "CHUNK_NOT_LOADED", world_pos, Some(item_id));
        }
        // Persist player modification for chunk reload
        self.modified_blocks.insert(world_pos, Some(item_id));
    }

    /// Remove block at world position, returns the removed block ItemId
    #[allow(dead_code)] // Used in tests
    pub fn remove_block(&mut self, world_pos: IVec3) -> Option<ItemId> {
        let chunk_coord = Self::world_to_chunk(world_pos);
        let local_pos = Self::world_to_local(world_pos);
        // Bounds check for y coordinate
        if local_pos.y < 0 || local_pos.y >= CHUNK_HEIGHT {
            Self::log_block_op("remove_block", "Y_OUT_OF_BOUNDS", world_pos, None);
            return None;
        }
        let chunk = self.chunks.get_mut(&chunk_coord)?;
        let idx = ChunkData::pos_to_index(local_pos.x, local_pos.y, local_pos.z);
        let block = chunk.blocks[idx].take();
        let status = if block.is_some() {
            "SUCCESS"
        } else {
            "NO_BLOCK"
        };
        Self::log_block_op("remove_block", status, world_pos, block);
        // Persist player modification for chunk reload (None = air/removed)
        self.modified_blocks.insert(world_pos, None);
        block
    }

    /// Check if block exists at world position
    pub fn has_block(&self, world_pos: IVec3) -> bool {
        self.get_block(world_pos).is_some()
    }

    // =========================================================================
    // ItemId API (now primary)
    // =========================================================================

    /// Get block at world position as ItemId (alias for get_block)
    pub fn get_block_id(&self, world_pos: IVec3) -> Option<ItemId> {
        self.get_block(world_pos)
    }

    /// Set block at world position using ItemId (alias for set_block)
    pub fn set_block_by_id(&mut self, world_pos: IVec3, item_id: ItemId) {
        self.set_block(world_pos, item_id);
    }

    /// Remove block at world position, returns the removed block as ItemId (alias)
    #[allow(dead_code)]
    pub fn remove_block_as_id(&mut self, world_pos: IVec3) -> Option<ItemId> {
        self.remove_block(world_pos)
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
