//! Global Inventory System
//!
//! A warehouse-style inventory that stores items by type, not by slot.
//! Used for machines, materials, and crafted items.
//!
//! ## Migration: BlockType -> ItemId
//!
//! This module is being migrated from BlockType to ItemId.
//! - New code should use ItemId-based APIs (e.g., `add_item_by_id`, `get_count_by_id`)
//! - BlockType-based APIs are deprecated and will be removed in a future version
//! - During migration, both APIs coexist

use crate::block_type::BlockType;
use crate::core::ItemId;
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

    // =========================================================================
    // BlockType-based API (Deprecated - use ItemId versions instead)
    // =========================================================================

    /// Create with initial items
    #[deprecated(since = "0.4.0", note = "Use with_items_by_id instead")]
    pub fn with_items(items: &[(BlockType, u32)]) -> Self {
        let mut inv = Self::new();
        for (block_type, count) in items {
            #[allow(deprecated)]
            inv.add_item(*block_type, *count);
        }
        inv
    }

    /// Add items to the inventory
    #[deprecated(since = "0.4.0", note = "Use add_item_by_id instead")]
    pub fn add_item(&mut self, block_type: BlockType, count: u32) {
        *self.items.entry(block_type).or_insert(0) += count;
    }

    /// Remove items from inventory. Returns true if successful, false if not enough.
    #[deprecated(since = "0.4.0", note = "Use remove_item_by_id instead")]
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
    #[deprecated(since = "0.4.0", note = "Use get_count_by_id instead")]
    pub fn get_count(&self, block_type: BlockType) -> u32 {
        self.items.get(&block_type).copied().unwrap_or(0)
    }

    /// Check if inventory has at least the specified amount
    #[deprecated(since = "0.4.0", note = "Use has_item_by_id instead")]
    pub fn has_item(&self, block_type: BlockType, count: u32) -> bool {
        #[allow(deprecated)]
        let result = self.get_count(block_type) >= count;
        result
    }

    /// Try to consume multiple items atomically. Returns true if all items were consumed.
    #[deprecated(since = "0.4.0", note = "Use try_consume_by_id instead")]
    pub fn try_consume(&mut self, items: &[(BlockType, u32)]) -> bool {
        // First check if we have enough of everything
        for (block_type, count) in items {
            #[allow(deprecated)]
            if !self.has_item(*block_type, *count) {
                return false;
            }
        }
        // Then consume all
        for (block_type, count) in items {
            #[allow(deprecated)]
            self.remove_item(*block_type, *count);
        }
        true
    }

    /// Get all items as a vec for UI display (sorted by BlockType)
    #[deprecated(since = "0.4.0", note = "Use get_all_items_by_id instead")]
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

    // =========================================================================
    // ItemId-based API (Preferred - use these for new code)
    // =========================================================================

    /// Create with initial items using ItemId
    pub fn with_items_by_id(items: &[(ItemId, u32)]) -> Self {
        let mut inv = Self::new();
        for (item_id, count) in items {
            inv.add_item_by_id(*item_id, *count);
        }
        inv
    }

    /// Add items to the inventory by ItemId
    pub fn add_item_by_id(&mut self, item_id: ItemId, count: u32) {
        if let Ok(block_type) = BlockType::try_from(item_id) {
            #[allow(deprecated)]
            self.add_item(block_type, count);
        }
        // TODO: support mod items when we have a proper ItemId storage
    }

    /// Remove items from inventory by ItemId. Returns true if successful.
    pub fn remove_item_by_id(&mut self, item_id: ItemId, count: u32) -> bool {
        if let Ok(block_type) = BlockType::try_from(item_id) {
            #[allow(deprecated)]
            return self.remove_item(block_type, count);
        }
        false
    }

    /// Get count of a specific item by ItemId
    pub fn get_count_by_id(&self, item_id: ItemId) -> u32 {
        if let Ok(block_type) = BlockType::try_from(item_id) {
            #[allow(deprecated)]
            return self.get_count(block_type);
        }
        0
    }

    /// Check if inventory has at least the specified amount by ItemId
    pub fn has_item_by_id(&self, item_id: ItemId, count: u32) -> bool {
        self.get_count_by_id(item_id) >= count
    }

    /// Try to consume multiple items atomically by ItemId. Returns true if all items were consumed.
    pub fn try_consume_by_id(&mut self, items: &[(ItemId, u32)]) -> bool {
        // First check if we have enough of everything
        for (item_id, count) in items {
            if !self.has_item_by_id(*item_id, *count) {
                return false;
            }
        }
        // Then consume all
        for (item_id, count) in items {
            self.remove_item_by_id(*item_id, *count);
        }
        true
    }

    /// Get all items as (ItemId, count) pairs for UI display
    pub fn get_all_items_by_id(&self) -> Vec<(ItemId, u32)> {
        self.items
            .iter()
            .filter(|(_, &count)| count > 0)
            .map(|(&bt, &count)| (ItemId::from(bt), count))
            .collect()
    }

    /// Get all item IDs in the inventory
    pub fn all_item_ids(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.items
            .keys()
            .filter(|bt| self.items.get(bt).copied().unwrap_or(0) > 0)
            .map(|&bt| ItemId::from(bt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

    // =========================================================================
    // Legacy BlockType Tests (with #[allow(deprecated)])
    // =========================================================================

    #[test]
    #[allow(deprecated)]
    fn test_add_item() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);
        assert_eq!(inv.get_count(BlockType::IronOre), 10);

        inv.add_item(BlockType::IronOre, 5);
        assert_eq!(inv.get_count(BlockType::IronOre), 15);
    }

    #[test]
    #[allow(deprecated)]
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
    #[allow(deprecated)]
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
    #[allow(deprecated)]
    fn test_with_items() {
        let inv = GlobalInventory::with_items(&[
            (BlockType::MinerBlock, 2),
            (BlockType::ConveyorBlock, 30),
        ]);
        assert_eq!(inv.get_count(BlockType::MinerBlock), 2);
        assert_eq!(inv.get_count(BlockType::ConveyorBlock), 30);
    }

    #[test]
    #[allow(deprecated)]
    fn test_get_all_items() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);
        inv.add_item(BlockType::Coal, 5);
        inv.add_item(BlockType::Stone, 0); // Should not appear

        let items = inv.get_all_items();
        assert_eq!(items.len(), 2);
    }

    // =========================================================================
    // New ItemId Tests
    // =========================================================================

    #[test]
    fn test_add_item_by_id() {
        let mut inv = GlobalInventory::new();
        inv.add_item_by_id(items::iron_ore(), 10);
        assert_eq!(inv.get_count_by_id(items::iron_ore()), 10);

        inv.add_item_by_id(items::iron_ore(), 5);
        assert_eq!(inv.get_count_by_id(items::iron_ore()), 15);
    }

    #[test]
    fn test_remove_item_by_id() {
        let mut inv = GlobalInventory::new();
        inv.add_item_by_id(items::iron_ore(), 10);

        assert!(inv.remove_item_by_id(items::iron_ore(), 5));
        assert_eq!(inv.get_count_by_id(items::iron_ore()), 5);

        assert!(!inv.remove_item_by_id(items::iron_ore(), 10));
        assert_eq!(inv.get_count_by_id(items::iron_ore()), 5);

        assert!(inv.remove_item_by_id(items::iron_ore(), 5));
        assert_eq!(inv.get_count_by_id(items::iron_ore()), 0);
    }

    #[test]
    fn test_has_item_by_id() {
        let mut inv = GlobalInventory::new();
        inv.add_item_by_id(items::iron_ore(), 10);

        assert!(inv.has_item_by_id(items::iron_ore(), 5));
        assert!(inv.has_item_by_id(items::iron_ore(), 10));
        assert!(!inv.has_item_by_id(items::iron_ore(), 11));
        assert!(!inv.has_item_by_id(items::coal(), 1));
    }

    #[test]
    fn test_try_consume_by_id() {
        let mut inv = GlobalInventory::new();
        inv.add_item_by_id(items::iron_ore(), 10);
        inv.add_item_by_id(items::coal(), 5);

        // Fail: not enough coal
        assert!(!inv.try_consume_by_id(&[(items::iron_ore(), 5), (items::coal(), 10)]));
        assert_eq!(inv.get_count_by_id(items::iron_ore()), 10);
        assert_eq!(inv.get_count_by_id(items::coal()), 5);

        // Success
        assert!(inv.try_consume_by_id(&[(items::iron_ore(), 5), (items::coal(), 3)]));
        assert_eq!(inv.get_count_by_id(items::iron_ore()), 5);
        assert_eq!(inv.get_count_by_id(items::coal()), 2);
    }

    #[test]
    fn test_with_items_by_id() {
        let inv = GlobalInventory::with_items_by_id(&[
            (items::miner_block(), 2),
            (items::conveyor_block(), 30),
        ]);
        assert_eq!(inv.get_count_by_id(items::miner_block()), 2);
        assert_eq!(inv.get_count_by_id(items::conveyor_block()), 30);
    }

    #[test]
    fn test_get_all_items_by_id() {
        let mut inv = GlobalInventory::new();
        inv.add_item_by_id(items::iron_ore(), 10);
        inv.add_item_by_id(items::coal(), 5);

        let all_items = inv.get_all_items_by_id();
        assert_eq!(all_items.len(), 2);
        assert!(all_items.contains(&(items::iron_ore(), 10)));
        assert!(all_items.contains(&(items::coal(), 5)));
    }

    #[test]
    fn test_all_item_ids() {
        let mut inv = GlobalInventory::new();
        inv.add_item_by_id(items::iron_ore(), 10);
        inv.add_item_by_id(items::coal(), 5);

        let all_ids: Vec<_> = inv.all_item_ids().collect();
        assert_eq!(all_ids.len(), 2);
        assert!(all_ids.contains(&items::iron_ore()));
        assert!(all_ids.contains(&items::coal()));
    }
}
