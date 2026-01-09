//! Section-based chunk storage for memory optimization
//!
//! Each chunk is divided into 16x16x16 sections. Empty sections (all air)
//! and uniform sections (all same block) are stored efficiently.
//! Mixed sections use palette compression for additional memory savings.

use crate::constants::{CHUNK_SIZE, SECTION_HEIGHT};
use crate::core::ItemId;
use crate::world::palette::PalettedSection;
use bevy::prelude::IVec3;

/// Size of a section in blocks (16x16x16 = 4096)
pub const SECTION_SIZE: usize = (CHUNK_SIZE * SECTION_HEIGHT * CHUNK_SIZE) as usize;

/// A 16x16x16 section of blocks within a chunk
///
/// Uses a three-tier storage optimization:
/// - Empty: Completely air (0 bytes of block data)
/// - Uniform: All same block type (8 bytes)
/// - Paletted: Mixed blocks with palette compression (variable, typically 0.5-4KB)
#[derive(Clone, Debug, Default)]
pub enum ChunkSection {
    /// Completely empty (all air) - 0 bytes of block data
    #[default]
    Empty,
    /// All blocks are the same type - 8 bytes (just the ItemId)
    Uniform(ItemId),
    /// Mixed blocks - palette compressed storage
    Paletted(Box<PalettedSection>),
}

impl ChunkSection {
    /// Create a new empty section
    pub fn new() -> Self {
        ChunkSection::Empty
    }

    /// Create a section from a block array, automatically optimizing to Empty/Uniform if possible
    pub fn from_blocks(blocks: Vec<Option<ItemId>>) -> Self {
        debug_assert_eq!(blocks.len(), SECTION_SIZE);

        // Check if all blocks are the same
        let first = blocks[0];
        let all_same = blocks.iter().all(|&b| b == first);

        if all_same {
            match first {
                None => ChunkSection::Empty,
                Some(id) => ChunkSection::Uniform(id),
            }
        } else {
            // Use palette compression for mixed blocks
            ChunkSection::Paletted(Box::new(PalettedSection::from_blocks(&blocks)))
        }
    }

    /// Get block at local section position (x, y, z in 0..16)
    #[inline]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<ItemId> {
        match self {
            ChunkSection::Empty => None,
            ChunkSection::Uniform(id) => Some(*id),
            ChunkSection::Paletted(paletted) => paletted.get_block(x, y, z),
        }
    }

    /// Set block at local section position, converting to Paletted if necessary
    /// Returns the old block value
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, item_id: Option<ItemId>) -> Option<ItemId> {
        let old = self.get_block(x, y, z);

        // Early return if no change
        if old == item_id {
            return old;
        }

        match self {
            ChunkSection::Empty => {
                if item_id.is_some() {
                    // Convert to Paletted
                    let mut paletted = PalettedSection::new();
                    paletted.set_block(x, y, z, item_id);
                    *self = ChunkSection::Paletted(Box::new(paletted));
                }
            }
            ChunkSection::Uniform(uniform_id) => {
                if item_id != Some(*uniform_id) {
                    // Convert to Paletted with all blocks set to uniform_id
                    let blocks = vec![Some(*uniform_id); SECTION_SIZE];
                    let mut paletted = PalettedSection::from_blocks(&blocks);
                    paletted.set_block(x, y, z, item_id);
                    *self = ChunkSection::Paletted(Box::new(paletted));
                }
            }
            ChunkSection::Paletted(paletted) => {
                paletted.set_block(x, y, z, item_id);
                // Try to optimize back to Empty/Uniform
                self.try_optimize();
            }
        }

        old
    }

    /// Try to convert Paletted section to Empty/Uniform if all blocks are the same
    fn try_optimize(&mut self) {
        if let ChunkSection::Paletted(paletted) = self {
            // Compact the palette first to remove unused entries
            paletted.compact();

            // After compaction, if palette size is 1, all blocks are the same
            if paletted.palette_size() == 1 {
                // Get the single block type
                let block = paletted.get_block(0, 0, 0);
                *self = match block {
                    None => ChunkSection::Empty,
                    Some(id) => ChunkSection::Uniform(id),
                };
            }
        }
    }

    /// Check if this section is empty (all air)
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, ChunkSection::Empty)
    }

    /// Check if this section is uniform (all same block)
    #[inline]
    pub fn is_uniform(&self) -> bool {
        matches!(self, ChunkSection::Empty | ChunkSection::Uniform(_))
    }

    /// Get the uniform block type if this section is uniform
    #[inline]
    pub fn uniform_block(&self) -> Option<ItemId> {
        match self {
            ChunkSection::Empty => None,
            ChunkSection::Uniform(id) => Some(*id),
            ChunkSection::Paletted(_) => None,
        }
    }

    /// Iterate over all non-air blocks in this section
    pub fn iter_blocks(&self) -> impl Iterator<Item = (IVec3, ItemId)> + '_ {
        SectionBlockIterator {
            section: self,
            index: 0,
        }
    }

    /// Get approximate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        match self {
            ChunkSection::Empty => std::mem::size_of::<Self>(),
            ChunkSection::Uniform(_) => std::mem::size_of::<Self>(),
            ChunkSection::Paletted(paletted) => {
                std::mem::size_of::<Self>() + paletted.memory_usage()
            }
        }
    }
}

/// Legacy SectionData for backward compatibility
/// This struct is kept for API compatibility but internally uses PalettedSection
#[derive(Clone, Debug)]
pub struct SectionData {
    /// Internal storage using palette compression
    inner: PalettedSection,
}

impl SectionData {
    /// Create a new section filled with air
    pub fn new() -> Self {
        Self {
            inner: PalettedSection::new(),
        }
    }

    /// Convert local position to array index
    #[inline]
    pub fn pos_to_index(x: i32, y: i32, z: i32) -> usize {
        debug_assert!(
            (0..CHUNK_SIZE).contains(&x)
                && (0..SECTION_HEIGHT).contains(&y)
                && (0..CHUNK_SIZE).contains(&z),
            "SectionData::pos_to_index out of bounds: ({}, {}, {})",
            x,
            y,
            z
        );
        (x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE) as usize
    }

    /// Convert array index to local position
    #[inline]
    pub fn index_to_pos(idx: usize) -> IVec3 {
        let idx = idx as i32;
        let y = idx / (CHUNK_SIZE * CHUNK_SIZE);
        let remaining = idx % (CHUNK_SIZE * CHUNK_SIZE);
        let z = remaining / CHUNK_SIZE;
        let x = remaining % CHUNK_SIZE;
        IVec3::new(x, y, z)
    }

    /// Get block at local position
    #[inline]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<ItemId> {
        if (0..CHUNK_SIZE).contains(&x)
            && (0..SECTION_HEIGHT).contains(&y)
            && (0..CHUNK_SIZE).contains(&z)
        {
            self.inner.get_block(x, y, z)
        } else {
            None
        }
    }
}

impl Default for SectionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over non-air blocks in a section
struct SectionBlockIterator<'a> {
    section: &'a ChunkSection,
    index: usize,
}

impl<'a> Iterator for SectionBlockIterator<'a> {
    type Item = (IVec3, ItemId);

    fn next(&mut self) -> Option<Self::Item> {
        match self.section {
            ChunkSection::Empty => None,
            ChunkSection::Uniform(id) => {
                if self.index < SECTION_SIZE {
                    let pos = SectionData::index_to_pos(self.index);
                    self.index += 1;
                    Some((pos, *id))
                } else {
                    None
                }
            }
            ChunkSection::Paletted(paletted) => {
                // Use the internal iterator
                while self.index < SECTION_SIZE {
                    let idx = self.index;
                    self.index += 1;
                    let pos = SectionData::index_to_pos(idx);
                    if let Some(id) = paletted.get_block(pos.x, pos.y, pos.z) {
                        return Some((pos, id));
                    }
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_item_id(n: u32) -> ItemId {
        // Create a mock ItemId for testing using the crate-internal constructor
        crate::core::id::Id::new(n)
    }

    #[test]
    fn test_empty_section() {
        let section = ChunkSection::new();
        assert!(section.is_empty());
        assert!(section.is_uniform());
        assert_eq!(section.get_block(0, 0, 0), None);
        assert_eq!(section.get_block(15, 15, 15), None);
    }

    #[test]
    fn test_uniform_section() {
        let stone = mock_item_id(1);
        let blocks = vec![Some(stone); SECTION_SIZE];
        let section = ChunkSection::from_blocks(blocks);

        assert!(!section.is_empty());
        assert!(section.is_uniform());
        assert_eq!(section.uniform_block(), Some(stone));
        assert_eq!(section.get_block(0, 0, 0), Some(stone));
        assert_eq!(section.get_block(8, 8, 8), Some(stone));
    }

    #[test]
    fn test_mixed_section() {
        let stone = mock_item_id(1);
        let dirt = mock_item_id(2);

        let mut blocks = vec![Some(stone); SECTION_SIZE];
        blocks[0] = Some(dirt);
        let section = ChunkSection::from_blocks(blocks);

        assert!(!section.is_empty());
        assert!(!section.is_uniform());
        assert_eq!(section.get_block(0, 0, 0), Some(dirt));
        assert_eq!(section.get_block(1, 0, 0), Some(stone));
    }

    #[test]
    fn test_set_block_empty_to_paletted() {
        let stone = mock_item_id(1);
        let mut section = ChunkSection::Empty;

        section.set_block(5, 5, 5, Some(stone));

        assert!(!section.is_empty());
        assert!(!section.is_uniform());
        assert_eq!(section.get_block(5, 5, 5), Some(stone));
        assert_eq!(section.get_block(0, 0, 0), None);
    }

    #[test]
    fn test_set_block_uniform_to_paletted() {
        let stone = mock_item_id(1);
        let dirt = mock_item_id(2);
        let mut section = ChunkSection::Uniform(stone);

        section.set_block(0, 0, 0, Some(dirt));

        assert!(!section.is_uniform());
        assert_eq!(section.get_block(0, 0, 0), Some(dirt));
        assert_eq!(section.get_block(1, 0, 0), Some(stone));
    }

    #[test]
    fn test_optimize_to_empty() {
        let stone = mock_item_id(1);
        let mut section = ChunkSection::Empty;

        // Add a block
        section.set_block(0, 0, 0, Some(stone));
        assert!(!section.is_empty());

        // Remove it
        section.set_block(0, 0, 0, None);
        assert!(section.is_empty());
    }

    #[test]
    fn test_optimize_to_uniform() {
        let stone = mock_item_id(1);
        let dirt = mock_item_id(2);

        // Start with uniform stone
        let mut section = ChunkSection::Uniform(stone);

        // Change one block to dirt
        section.set_block(0, 0, 0, Some(dirt));
        assert!(!section.is_uniform());

        // Change it back to stone
        section.set_block(0, 0, 0, Some(stone));
        assert!(section.is_uniform());
        assert_eq!(section.uniform_block(), Some(stone));
    }

    #[test]
    fn test_memory_usage() {
        let section_empty = ChunkSection::Empty;
        let stone = mock_item_id(1);
        let mut section_paletted = ChunkSection::Empty;
        section_paletted.set_block(0, 0, 0, Some(stone));
        section_paletted.set_block(1, 0, 0, Some(mock_item_id(2)));

        // Empty should use much less memory than Paletted
        assert!(section_empty.memory_usage() < section_paletted.memory_usage());

        // Paletted section with few types should use much less than 32KB
        assert!(section_paletted.memory_usage() < 5000);
    }

    #[test]
    fn test_iter_blocks_empty() {
        let section = ChunkSection::Empty;
        assert_eq!(section.iter_blocks().count(), 0);
    }

    #[test]
    fn test_iter_blocks_uniform() {
        let stone = mock_item_id(1);
        let section = ChunkSection::Uniform(stone);
        assert_eq!(section.iter_blocks().count(), SECTION_SIZE);
    }

    #[test]
    fn test_iter_blocks_sparse() {
        let stone = mock_item_id(1);
        let mut section = ChunkSection::Empty;
        section.set_block(0, 0, 0, Some(stone));
        section.set_block(5, 5, 5, Some(stone));
        section.set_block(15, 15, 15, Some(stone));

        let blocks: Vec<_> = section.iter_blocks().collect();
        assert_eq!(blocks.len(), 3);
    }

    #[test]
    fn test_pos_to_index_roundtrip() {
        for x in 0..CHUNK_SIZE {
            for y in 0..SECTION_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    let idx = SectionData::pos_to_index(x, y, z);
                    let pos = SectionData::index_to_pos(idx);
                    assert_eq!(pos, IVec3::new(x, y, z));
                }
            }
        }
    }
}
