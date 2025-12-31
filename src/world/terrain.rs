//! World terrain data management

use crate::block_type::BlockType;
use crate::constants::*;
use super::chunk::ChunkData;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_chunk() {
        assert_eq!(WorldData::world_to_chunk(IVec3::new(0, 0, 0)), IVec2::new(0, 0));
        assert_eq!(WorldData::world_to_chunk(IVec3::new(15, 0, 15)), IVec2::new(0, 0));
        assert_eq!(WorldData::world_to_chunk(IVec3::new(16, 0, 0)), IVec2::new(1, 0));
        assert_eq!(WorldData::world_to_chunk(IVec3::new(-1, 0, -1)), IVec2::new(-1, -1));
    }

    #[test]
    fn test_world_to_local() {
        assert_eq!(WorldData::world_to_local(IVec3::new(0, 5, 0)), IVec3::new(0, 5, 0));
        assert_eq!(WorldData::world_to_local(IVec3::new(17, 5, 18)), IVec3::new(1, 5, 2));
        assert_eq!(WorldData::world_to_local(IVec3::new(-1, 5, -1)), IVec3::new(15, 5, 15));
    }

    #[test]
    fn test_local_to_world() {
        assert_eq!(
            WorldData::local_to_world(IVec2::new(1, 2), IVec3::new(5, 3, 7)),
            IVec3::new(21, 3, 39)
        );
    }
}
