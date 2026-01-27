//! Tests for world and chunk systems

#[cfg(test)]
mod tests {
    use crate::constants::*;
    use crate::core::items;
    use crate::world::{ChunkData, WorldData};
    use bevy::mesh::Mesh;
    use bevy::prelude::*;

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
        world.set_block(pos, items::stone());
        assert_eq!(world.get_block(pos), Some(items::stone()));

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
        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(pos)) = positions {
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
