//! Palette-based compression for chunk sections
//!
//! This module provides memory-efficient storage of block data using palette compression.
//! Instead of storing full ItemIds for each block, we store indices into a palette of
//! unique block types. The index bit-width is dynamically adjusted based on palette size.
//!
//! Memory usage comparison (for 4096-block section):
//! - Full ItemId storage: 4096 * 8 bytes = 32KB
//! - Palette with 2 types (1 bit): 512 bytes + palette overhead ≈ 600 bytes
//! - Palette with 16 types (4 bits): 2KB + palette overhead ≈ 2.2KB
//! - Palette with 256 types (8 bits): 4KB + palette overhead ≈ 4.3KB

use crate::constants::{CHUNK_SIZE, SECTION_HEIGHT};
use crate::core::ItemId;

/// Size of a section in blocks
pub const SECTION_SIZE: usize = (CHUNK_SIZE * SECTION_HEIGHT * CHUNK_SIZE) as usize;

/// Maximum palette size before falling back to direct storage
const MAX_PALETTE_SIZE: usize = 256;

/// A packed array storing indices with variable bit width
#[derive(Clone, Debug)]
pub struct PackedArray {
    /// The raw storage (u64 blocks)
    storage: Vec<u64>,
    /// Number of elements in the array
    len: usize,
    /// Bits per entry (1, 2, 4, or 8)
    bits_per_entry: u8,
}

impl PackedArray {
    /// Create a new packed array with the specified bit width
    pub fn new(len: usize, bits_per_entry: u8) -> Self {
        debug_assert!(
            bits_per_entry == 1
                || bits_per_entry == 2
                || bits_per_entry == 4
                || bits_per_entry == 8,
            "bits_per_entry must be 1, 2, 4, or 8"
        );

        let entries_per_u64 = 64 / bits_per_entry as usize;
        let num_u64s = len.div_ceil(entries_per_u64);

        Self {
            storage: vec![0; num_u64s],
            len,
            bits_per_entry,
        }
    }

    /// Get the value at the specified index
    #[inline]
    pub fn get(&self, index: usize) -> u8 {
        debug_assert!(index < self.len);

        let entries_per_u64 = 64 / self.bits_per_entry as usize;
        let u64_index = index / entries_per_u64;
        let bit_offset = (index % entries_per_u64) * self.bits_per_entry as usize;
        let mask = (1u64 << self.bits_per_entry) - 1;

        ((self.storage[u64_index] >> bit_offset) & mask) as u8
    }

    /// Set the value at the specified index
    #[inline]
    pub fn set(&mut self, index: usize, value: u8) {
        debug_assert!(index < self.len);
        debug_assert!((value as u32) < (1 << self.bits_per_entry));

        let entries_per_u64 = 64 / self.bits_per_entry as usize;
        let u64_index = index / entries_per_u64;
        let bit_offset = (index % entries_per_u64) * self.bits_per_entry as usize;
        let mask = (1u64 << self.bits_per_entry) - 1;

        // Clear the old value and set the new one
        self.storage[u64_index] &= !(mask << bit_offset);
        self.storage[u64_index] |= (value as u64) << bit_offset;
    }

    /// Get the bits per entry
    pub fn bits_per_entry(&self) -> u8 {
        self.bits_per_entry
    }

    /// Get the memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.storage.len() * std::mem::size_of::<u64>()
    }

    /// Resize to a new bit width, copying all existing data
    pub fn resize(&self, new_bits: u8) -> Self {
        let mut new_array = PackedArray::new(self.len, new_bits);
        for i in 0..self.len {
            new_array.set(i, self.get(i));
        }
        new_array
    }
}

/// A paletted section storing blocks with palette compression
#[derive(Clone, Debug)]
pub struct PalettedSection {
    /// The palette mapping indices to ItemIds (None = air)
    palette: Vec<Option<ItemId>>,
    /// Packed array of palette indices
    data: PackedArray,
}

impl PalettedSection {
    /// Create a new empty paletted section (all air)
    pub fn new() -> Self {
        Self {
            palette: vec![None],                     // Index 0 = air
            data: PackedArray::new(SECTION_SIZE, 1), // 1 bit for air only
        }
    }

    /// Create a paletted section from a full block array
    pub fn from_blocks(blocks: &[Option<ItemId>]) -> Self {
        debug_assert_eq!(blocks.len(), SECTION_SIZE);

        // Build palette
        let mut palette: Vec<Option<ItemId>> = Vec::new();
        let mut index_map = std::collections::HashMap::new();

        for &block in blocks {
            if let std::collections::hash_map::Entry::Vacant(e) = index_map.entry(block) {
                e.insert(palette.len() as u8);
                palette.push(block);
            }
        }

        // Determine bit width needed
        let bits = Self::bits_for_palette_size(palette.len());

        // Create packed data
        let mut data = PackedArray::new(SECTION_SIZE, bits);
        for (i, &block) in blocks.iter().enumerate() {
            let index = index_map[&block];
            data.set(i, index);
        }

        Self { palette, data }
    }

    /// Get block at local position (x, y, z in 0..16)
    #[inline]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<ItemId> {
        let idx = Self::pos_to_index(x, y, z);
        let palette_idx = self.data.get(idx) as usize;
        self.palette.get(palette_idx).copied().flatten()
    }

    /// Set block at local position, potentially growing the palette
    /// Returns the old block value
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, item_id: Option<ItemId>) -> Option<ItemId> {
        let idx = Self::pos_to_index(x, y, z);
        let old_palette_idx = self.data.get(idx) as usize;
        let old = self.palette.get(old_palette_idx).copied().flatten();

        // Early return if no change
        if old == item_id {
            return old;
        }

        // Find or add to palette
        let new_palette_idx = self.get_or_add_palette_entry(item_id);
        self.data.set(idx, new_palette_idx);

        old
    }

    /// Get or add a palette entry for the given block, resizing if needed
    fn get_or_add_palette_entry(&mut self, item_id: Option<ItemId>) -> u8 {
        // Check if already in palette
        for (i, &entry) in self.palette.iter().enumerate() {
            if entry == item_id {
                return i as u8;
            }
        }

        // Need to add to palette
        let new_idx = self.palette.len();

        // Check if we need to resize
        if new_idx >= MAX_PALETTE_SIZE {
            // TODO: Fall back to direct storage or panic
            panic!("Palette exceeded maximum size of {}", MAX_PALETTE_SIZE);
        }

        let current_max = 1usize << self.data.bits_per_entry();
        if new_idx >= current_max {
            // Resize to larger bit width
            let new_bits = Self::bits_for_palette_size(new_idx + 1);
            self.data = self.data.resize(new_bits);
        }

        self.palette.push(item_id);
        new_idx as u8
    }

    /// Calculate required bits for a given palette size
    fn bits_for_palette_size(size: usize) -> u8 {
        if size <= 2 {
            1
        } else if size <= 4 {
            2
        } else if size <= 16 {
            4
        } else {
            8
        }
    }

    /// Convert local position to array index
    #[inline]
    fn pos_to_index(x: i32, y: i32, z: i32) -> usize {
        debug_assert!(
            (0..CHUNK_SIZE).contains(&x)
                && (0..SECTION_HEIGHT).contains(&y)
                && (0..CHUNK_SIZE).contains(&z)
        );
        (x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE) as usize
    }

    /// Convert array index to local position
    #[inline]
    fn index_to_pos(idx: usize) -> (i32, i32, i32) {
        let idx = idx as i32;
        let y = idx / (CHUNK_SIZE * CHUNK_SIZE);
        let remaining = idx % (CHUNK_SIZE * CHUNK_SIZE);
        let z = remaining / CHUNK_SIZE;
        let x = remaining % CHUNK_SIZE;
        (x, y, z)
    }

    /// Iterate over all non-air blocks
    pub fn iter_blocks(&self) -> impl Iterator<Item = ((i32, i32, i32), ItemId)> + '_ {
        (0..SECTION_SIZE).filter_map(move |idx| {
            let palette_idx = self.data.get(idx) as usize;
            self.palette.get(palette_idx).copied().flatten().map(|id| {
                let (x, y, z) = Self::index_to_pos(idx);
                ((x, y, z), id)
            })
        })
    }

    /// Get approximate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.palette.len() * std::mem::size_of::<Option<ItemId>>()
            + self.data.memory_usage()
    }

    /// Get the number of unique block types (including air)
    pub fn palette_size(&self) -> usize {
        self.palette.len()
    }

    /// Compact the palette by removing unused entries
    pub fn compact(&mut self) {
        // Count usage of each palette entry
        let mut usage = vec![false; self.palette.len()];
        for idx in 0..SECTION_SIZE {
            let palette_idx = self.data.get(idx) as usize;
            usage[palette_idx] = true;
        }

        // Build new palette with only used entries
        let mut new_palette: Vec<Option<ItemId>> = Vec::new();
        let mut old_to_new: Vec<u8> = vec![0; self.palette.len()];

        for (old_idx, &used) in usage.iter().enumerate() {
            if used {
                old_to_new[old_idx] = new_palette.len() as u8;
                new_palette.push(self.palette[old_idx]);
            }
        }

        // Check if compaction is worthwhile
        if new_palette.len() == self.palette.len() {
            return;
        }

        // Determine new bit width
        let new_bits = Self::bits_for_palette_size(new_palette.len());

        // Create new packed data with remapped indices
        let mut new_data = PackedArray::new(SECTION_SIZE, new_bits);
        for idx in 0..SECTION_SIZE {
            let old_palette_idx = self.data.get(idx) as usize;
            let new_palette_idx = old_to_new[old_palette_idx];
            new_data.set(idx, new_palette_idx);
        }

        self.palette = new_palette;
        self.data = new_data;
    }
}

impl Default for PalettedSection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_item_id(n: u32) -> ItemId {
        crate::core::id::Id::new(n)
    }

    #[test]
    fn test_packed_array_1bit() {
        let mut arr = PackedArray::new(64, 1);
        arr.set(0, 1);
        arr.set(1, 0);
        arr.set(63, 1);
        assert_eq!(arr.get(0), 1);
        assert_eq!(arr.get(1), 0);
        assert_eq!(arr.get(63), 1);
    }

    #[test]
    fn test_packed_array_4bit() {
        let mut arr = PackedArray::new(100, 4);
        for i in 0..16 {
            arr.set(i, i as u8);
        }
        for i in 0..16 {
            assert_eq!(arr.get(i), i as u8);
        }
    }

    #[test]
    fn test_packed_array_8bit() {
        let mut arr = PackedArray::new(256, 8);
        for i in 0..256 {
            arr.set(i, i as u8);
        }
        for i in 0..256 {
            assert_eq!(arr.get(i), i as u8);
        }
    }

    #[test]
    fn test_packed_array_resize() {
        let mut arr = PackedArray::new(64, 1);
        arr.set(0, 1);
        arr.set(32, 1);

        let resized = arr.resize(4);
        assert_eq!(resized.get(0), 1);
        assert_eq!(resized.get(32), 1);
        assert_eq!(resized.get(1), 0);
    }

    #[test]
    fn test_paletted_section_empty() {
        let section = PalettedSection::new();
        assert_eq!(section.get_block(0, 0, 0), None);
        assert_eq!(section.get_block(15, 15, 15), None);
        assert_eq!(section.palette_size(), 1);
    }

    #[test]
    fn test_paletted_section_set_get() {
        let mut section = PalettedSection::new();
        let stone = mock_item_id(1);
        let dirt = mock_item_id(2);

        section.set_block(0, 0, 0, Some(stone));
        section.set_block(1, 0, 0, Some(dirt));
        section.set_block(2, 0, 0, Some(stone));

        assert_eq!(section.get_block(0, 0, 0), Some(stone));
        assert_eq!(section.get_block(1, 0, 0), Some(dirt));
        assert_eq!(section.get_block(2, 0, 0), Some(stone));
        assert_eq!(section.get_block(3, 0, 0), None);

        // Should have air, stone, dirt in palette
        assert_eq!(section.palette_size(), 3);
    }

    #[test]
    fn test_paletted_section_from_blocks() {
        let stone = mock_item_id(1);
        let dirt = mock_item_id(2);

        let mut blocks = vec![None; SECTION_SIZE];
        blocks[0] = Some(stone);
        blocks[1] = Some(dirt);
        blocks[100] = Some(stone);

        let section = PalettedSection::from_blocks(&blocks);

        assert_eq!(section.get_block(0, 0, 0), Some(stone));
        assert_eq!(section.get_block(1, 0, 0), Some(dirt));
        assert_eq!(section.get_block(2, 0, 0), None);
        assert_eq!(section.palette_size(), 3);
    }

    #[test]
    fn test_paletted_section_grow_palette() {
        let mut section = PalettedSection::new();

        // Add many different block types
        for i in 1..=20 {
            section.set_block(
                (i % 16) as i32,
                (i / 16) as i32,
                0,
                Some(mock_item_id(i as u32)),
            );
        }

        // All should be retrievable
        for i in 1..=20 {
            let x = (i % 16) as i32;
            let y = (i / 16) as i32;
            assert_eq!(section.get_block(x, y, 0), Some(mock_item_id(i as u32)));
        }
    }

    #[test]
    fn test_paletted_section_iter_blocks() {
        let mut section = PalettedSection::new();
        let stone = mock_item_id(1);

        section.set_block(0, 0, 0, Some(stone));
        section.set_block(5, 5, 5, Some(stone));

        let blocks: Vec<_> = section.iter_blocks().collect();
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_paletted_section_compact() {
        let mut section = PalettedSection::new();
        let stone = mock_item_id(1);
        let dirt = mock_item_id(2);
        let gravel = mock_item_id(3);

        // Add three types
        section.set_block(0, 0, 0, Some(stone));
        section.set_block(1, 0, 0, Some(dirt));
        section.set_block(2, 0, 0, Some(gravel));
        assert_eq!(section.palette_size(), 4);

        // Remove one
        section.set_block(1, 0, 0, None);

        // Compact
        section.compact();

        // Should still work correctly
        assert_eq!(section.get_block(0, 0, 0), Some(stone));
        assert_eq!(section.get_block(1, 0, 0), None);
        assert_eq!(section.get_block(2, 0, 0), Some(gravel));

        // Palette should be smaller (3 instead of 4: air, stone, gravel)
        assert_eq!(section.palette_size(), 3);
    }

    #[test]
    fn test_memory_efficiency() {
        // Empty section
        let empty = PalettedSection::new();
        assert!(empty.memory_usage() < 1000); // Much less than 32KB

        // Section with just 2 types
        let mut two_types = PalettedSection::new();
        let stone = mock_item_id(1);
        for x in 0..16 {
            for z in 0..16 {
                two_types.set_block(x, 0, z, Some(stone));
            }
        }
        assert!(two_types.memory_usage() < 2000); // Should use 1-bit storage
    }
}
