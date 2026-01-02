//! Global Inventory System
//!
//! A warehouse-style inventory that stores items by type, not by slot.
//! Used for machines, materials, and crafted items.

use crate::block_type::BlockType;
use bevy::prelude::*;
use std::collections::HashMap;

/// Global inventory - stores items by type (not slot-based)
/// This is the main storage for all placeable machines and materials.
#[derive(Resource, Default, Debug, Clone)]
pub struct GlobalInventory {
    /// Items stored: BlockType -> count
    items: HashMap<BlockType, u32>,
}

impl GlobalInventory {
    /// Create a new empty global inventory
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Create with initial items
    pub fn with_items(items: &[(BlockType, u32)]) -> Self {
        let mut inv = Self::new();
        for (block_type, count) in items {
            inv.add_item(*block_type, *count);
        }
        inv
    }

    /// Add items to the inventory
    pub fn add_item(&mut self, block_type: BlockType, count: u32) {
        *self.items.entry(block_type).or_insert(0) += count;
    }

    /// Remove items from inventory. Returns true if successful, false if not enough.
    pub fn remove_item(&mut self, block_type: BlockType, count: u32) -> bool {
        if let Some(current) = self.items.get_mut(&block_type) {
            if *current >= count {
                *current -= count;
                if *current == 0 {
                    self.items.remove(&block_type);
                }
                return true;
            }
        }
        false
    }

    /// Get count of a specific item
    pub fn get_count(&self, block_type: BlockType) -> u32 {
        self.items.get(&block_type).copied().unwrap_or(0)
    }

    /// Check if inventory has at least the specified amount
    pub fn has_item(&self, block_type: BlockType, count: u32) -> bool {
        self.get_count(block_type) >= count
    }

    /// Try to consume multiple items atomically. Returns true if all items were consumed.
    pub fn try_consume(&mut self, items: &[(BlockType, u32)]) -> bool {
        // First check if we have enough of everything
        for (block_type, count) in items {
            if !self.has_item(*block_type, *count) {
                return false;
            }
        }
        // Then consume all
        for (block_type, count) in items {
            self.remove_item(*block_type, *count);
        }
        true
    }

    /// Get all items as a vec for UI display (sorted by BlockType)
    pub fn get_all_items(&self) -> Vec<(BlockType, u32)> {
        let mut items: Vec<_> = self
            .items
            .iter()
            .filter(|(_, &count)| count > 0)
            .map(|(&bt, &count)| (bt, count))
            .collect();
        items.sort_by_key(|(bt, _)| *bt as u32);
        items
    }

    /// Get item count (number of different item types)
    pub fn item_type_count(&self) -> usize {
        self.items.iter().filter(|(_, &c)| c > 0).count()
    }

    /// Check if inventory is empty
    pub fn is_empty(&self) -> bool {
        self.items.iter().all(|(_, &c)| c == 0)
    }

    /// Get items HashMap for serialization
    pub fn items(&self) -> &HashMap<BlockType, u32> {
        &self.items
    }

    /// Set items from deserialization
    pub fn set_items(&mut self, items: HashMap<BlockType, u32>) {
        self.items = items;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_item() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);
        assert_eq!(inv.get_count(BlockType::IronOre), 10);

        inv.add_item(BlockType::IronOre, 5);
        assert_eq!(inv.get_count(BlockType::IronOre), 15);
    }

    #[test]
    fn test_remove_item() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);

        assert!(inv.remove_item(BlockType::IronOre, 5));
        assert_eq!(inv.get_count(BlockType::IronOre), 5);

        assert!(!inv.remove_item(BlockType::IronOre, 10));
        assert_eq!(inv.get_count(BlockType::IronOre), 5);

        assert!(inv.remove_item(BlockType::IronOre, 5));
        assert_eq!(inv.get_count(BlockType::IronOre), 0);
    }

    #[test]
    fn test_try_consume() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);
        inv.add_item(BlockType::Coal, 5);

        // Fail: not enough coal
        assert!(!inv.try_consume(&[(BlockType::IronOre, 5), (BlockType::Coal, 10)]));
        assert_eq!(inv.get_count(BlockType::IronOre), 10);
        assert_eq!(inv.get_count(BlockType::Coal), 5);

        // Success
        assert!(inv.try_consume(&[(BlockType::IronOre, 5), (BlockType::Coal, 3)]));
        assert_eq!(inv.get_count(BlockType::IronOre), 5);
        assert_eq!(inv.get_count(BlockType::Coal), 2);
    }

    #[test]
    fn test_with_items() {
        let inv = GlobalInventory::with_items(&[
            (BlockType::MinerBlock, 2),
            (BlockType::ConveyorBlock, 30),
        ]);
        assert_eq!(inv.get_count(BlockType::MinerBlock), 2);
        assert_eq!(inv.get_count(BlockType::ConveyorBlock), 30);
    }

    #[test]
    fn test_get_all_items() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);
        inv.add_item(BlockType::Coal, 5);
        inv.add_item(BlockType::Stone, 0); // Should not appear

        let items = inv.get_all_items();
        assert_eq!(items.len(), 2);
    }
}
