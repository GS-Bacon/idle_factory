//! Global Inventory System
//!
//! All items are stored in a single global inventory instead of per-player slots.
//! This is the Single Source of Truth for item storage.
//!
//! ## Design (from game_spec.rs)
//! - Items delivered to Delivery Platform are added to GlobalInventory
//! - GlobalInventory is accessible from anywhere (E key)
//! - Machine placement consumes from GlobalInventory
//! - Machine demolition returns to GlobalInventory
//! - No storage limit (infinite)
//! - No manual pickup from conveyors

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::BlockType;

/// Global inventory resource - stores all player items in one place
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
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

    /// Create with initial items (from game_spec::INITIAL_EQUIPMENT)
    pub fn with_initial_items(initial: &[(BlockType, u32)]) -> Self {
        let mut inv = Self::new();
        for (block_type, count) in initial {
            inv.add_item(*block_type, *count);
        }
        inv
    }

    /// Add items to inventory
    pub fn add_item(&mut self, block_type: BlockType, count: u32) {
        *self.items.entry(block_type).or_insert(0) += count;
    }

    /// Remove items from inventory, returns true if successful
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

    /// Check if we have at least the specified amount
    pub fn has_amount(&self, block_type: BlockType, count: u32) -> bool {
        self.get_count(block_type) >= count
    }

    /// Try to consume multiple items atomically
    /// Returns true if all items were consumed, false if any item was insufficient
    pub fn try_consume(&mut self, items: &[(BlockType, u32)]) -> bool {
        // First, check if we have enough of everything
        for (block_type, count) in items {
            if !self.has_amount(*block_type, *count) {
                return false;
            }
        }
        // Then consume all
        for (block_type, count) in items {
            self.remove_item(*block_type, *count);
        }
        true
    }

    /// Get all items as a vector (for UI display)
    pub fn get_all_items(&self) -> Vec<(BlockType, u32)> {
        let mut items: Vec<_> = self.items.iter().map(|(k, v)| (*k, *v)).collect();
        // Sort by block type name for consistent display
        items.sort_by_key(|(bt, _)| bt.name());
        items
    }

    /// Get total number of unique item types
    pub fn unique_item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if inventory is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get items iterator
    pub fn iter(&self) -> impl Iterator<Item = (&BlockType, &u32)> {
        self.items.iter()
    }

    /// Clear all items (for save/load and testing)
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);
        assert_eq!(inv.get_count(BlockType::IronOre), 10);
        assert_eq!(inv.get_count(BlockType::Coal), 0);
    }

    #[test]
    fn test_add_stacks() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);
        inv.add_item(BlockType::IronOre, 20);
        assert_eq!(inv.get_count(BlockType::IronOre), 30);
    }

    #[test]
    fn test_remove_item() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);

        assert!(inv.remove_item(BlockType::IronOre, 5));
        assert_eq!(inv.get_count(BlockType::IronOre), 5);

        assert!(!inv.remove_item(BlockType::IronOre, 10)); // Not enough
        assert_eq!(inv.get_count(BlockType::IronOre), 5); // Unchanged

        assert!(inv.remove_item(BlockType::IronOre, 5));
        assert_eq!(inv.get_count(BlockType::IronOre), 0);
    }

    #[test]
    fn test_try_consume() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);
        inv.add_item(BlockType::Coal, 5);

        // Should fail - not enough coal
        let result = inv.try_consume(&[
            (BlockType::IronOre, 5),
            (BlockType::Coal, 10),
        ]);
        assert!(!result);
        assert_eq!(inv.get_count(BlockType::IronOre), 10); // Unchanged
        assert_eq!(inv.get_count(BlockType::Coal), 5); // Unchanged

        // Should succeed
        let result = inv.try_consume(&[
            (BlockType::IronOre, 5),
            (BlockType::Coal, 3),
        ]);
        assert!(result);
        assert_eq!(inv.get_count(BlockType::IronOre), 5);
        assert_eq!(inv.get_count(BlockType::Coal), 2);
    }

    #[test]
    fn test_with_initial_items() {
        use crate::game_spec::INITIAL_EQUIPMENT;
        let inv = GlobalInventory::with_initial_items(INITIAL_EQUIPMENT);

        assert_eq!(inv.get_count(BlockType::MinerBlock), 2);
        assert_eq!(inv.get_count(BlockType::ConveyorBlock), 30);
        assert_eq!(inv.get_count(BlockType::FurnaceBlock), 1);
    }

    #[test]
    fn test_get_all_items() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);
        inv.add_item(BlockType::Coal, 5);

        let items = inv.get_all_items();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_has_amount() {
        let mut inv = GlobalInventory::new();
        inv.add_item(BlockType::IronOre, 10);

        assert!(inv.has_amount(BlockType::IronOre, 5));
        assert!(inv.has_amount(BlockType::IronOre, 10));
        assert!(!inv.has_amount(BlockType::IronOre, 11));
        assert!(!inv.has_amount(BlockType::Coal, 1));
    }
}
