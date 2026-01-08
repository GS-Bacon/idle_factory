//! Platform Inventory System
//!
//! A warehouse-style inventory that stores items by type, not by slot.
//! Used for the delivery platform to store machines, materials, and crafted items.
//!
//! ## Architecture
//!
//! - `PlatformInventory` is a Component attached to the DeliveryPlatform entity
//! - `LocalPlatform` resource tracks the platform entity
//! - `LocalPlatformInventory` SystemParam provides convenient access
//!
//! ## Migration Status
//!
//! This module uses ItemId internally.
//! - ItemId-based APIs are primary (e.g., `add_item_by_id`, `get_count_by_id`)
//! - BlockType-based APIs are deprecated and maintained for backward compatibility

use crate::block_type::BlockType;
use crate::core::ItemId;
use bevy::prelude::*;
use std::collections::HashMap;

/// Platform inventory - stores items by type (not slot-based)
/// This is the main storage for all placeable machines and materials.
/// Attached to the DeliveryPlatform entity.
///
/// Uses ItemId internally to support both base game and mod items.
#[derive(Component, Default, Debug, Clone)]
pub struct PlatformInventory {
    /// Items stored: ItemId -> count
    items: HashMap<ItemId, u32>,
}

/// Resource to track the delivery platform entity
#[derive(Resource)]
pub struct LocalPlatform(pub Entity);

impl PlatformInventory {
    /// Create a new empty platform inventory
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    // =========================================================================
    // ItemId-based API (Primary - use these for new code)
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
        *self.items.entry(item_id).or_insert(0) += count;
    }

    /// Remove items from inventory by ItemId. Returns true if successful.
    pub fn remove_item_by_id(&mut self, item_id: ItemId, count: u32) -> bool {
        if let Some(current) = self.items.get_mut(&item_id) {
            if *current >= count {
                *current -= count;
                if *current == 0 {
                    self.items.remove(&item_id);
                }
                return true;
            }
        }
        false
    }

    /// Get count of a specific item by ItemId
    pub fn get_count_by_id(&self, item_id: ItemId) -> u32 {
        self.items.get(&item_id).copied().unwrap_or(0)
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
            .map(|(&id, &count)| (id, count))
            .collect()
    }

    /// Get all item IDs in the inventory
    pub fn all_item_ids(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.items
            .keys()
            .filter(|id| self.items.get(id).copied().unwrap_or(0) > 0)
            .copied()
    }

    /// Get item count (number of different item types)
    pub fn item_type_count(&self) -> usize {
        self.items.iter().filter(|(_, &c)| c > 0).count()
    }

    /// Check if inventory is empty
    pub fn is_empty(&self) -> bool {
        self.items.iter().all(|(_, &c)| c == 0)
    }

    /// Get items HashMap for serialization (ItemId-based)
    pub fn items_by_id(&self) -> &HashMap<ItemId, u32> {
        &self.items
    }

    /// Set items from deserialization (ItemId-based)
    pub fn set_items_by_id(&mut self, items: HashMap<ItemId, u32>) {
        self.items = items;
    }

    // =========================================================================
    // BlockType-based API (Deprecated - for backward compatibility)
    // =========================================================================

    /// Create with initial items
    #[deprecated(since = "0.4.0", note = "Use with_items_by_id instead")]
    pub fn with_items(items: &[(BlockType, u32)]) -> Self {
        let items_by_id: Vec<_> = items
            .iter()
            .map(|(bt, count)| (ItemId::from(*bt), *count))
            .collect();
        Self::with_items_by_id(&items_by_id)
    }

    /// Add items to the inventory
    #[deprecated(since = "0.4.0", note = "Use add_item_by_id instead")]
    pub fn add_item(&mut self, block_type: BlockType, count: u32) {
        self.add_item_by_id(ItemId::from(block_type), count);
    }

    /// Remove items from inventory. Returns true if successful, false if not enough.
    #[deprecated(since = "0.4.0", note = "Use remove_item_by_id instead")]
    pub fn remove_item(&mut self, block_type: BlockType, count: u32) -> bool {
        self.remove_item_by_id(ItemId::from(block_type), count)
    }

    /// Get count of a specific item
    #[deprecated(since = "0.4.0", note = "Use get_count_by_id instead")]
    pub fn get_count(&self, block_type: BlockType) -> u32 {
        self.get_count_by_id(ItemId::from(block_type))
    }

    /// Check if inventory has at least the specified amount
    #[deprecated(since = "0.4.0", note = "Use has_item_by_id instead")]
    pub fn has_item(&self, block_type: BlockType, count: u32) -> bool {
        self.has_item_by_id(ItemId::from(block_type), count)
    }

    /// Try to consume multiple items atomically. Returns true if all items were consumed.
    #[deprecated(since = "0.4.0", note = "Use try_consume_by_id instead")]
    pub fn try_consume(&mut self, items: &[(BlockType, u32)]) -> bool {
        let items_by_id: Vec<_> = items
            .iter()
            .map(|(bt, count)| (ItemId::from(*bt), *count))
            .collect();
        self.try_consume_by_id(&items_by_id)
    }

    /// Get all items as a vec for UI display (sorted by BlockType)
    #[deprecated(since = "0.4.0", note = "Use get_all_items_by_id instead")]
    pub fn get_all_items(&self) -> Vec<(BlockType, u32)> {
        let mut items: Vec<_> = self
            .items
            .iter()
            .filter(|(_, &count)| count > 0)
            .filter_map(|(&id, &count)| BlockType::try_from(id).ok().map(|bt| (bt, count)))
            .collect();
        items.sort_by_key(|(bt, _)| *bt as u32);
        items
    }

    /// Get items HashMap for serialization (BlockType-based, deprecated)
    #[deprecated(since = "0.4.0", note = "Use items_by_id instead")]
    pub fn items(&self) -> HashMap<BlockType, u32> {
        self.items
            .iter()
            .filter_map(|(&id, &count)| BlockType::try_from(id).ok().map(|bt| (bt, count)))
            .collect()
    }

    /// Set items from deserialization (BlockType-based, deprecated)
    #[deprecated(since = "0.4.0", note = "Use set_items_by_id instead")]
    pub fn set_items(&mut self, items: HashMap<BlockType, u32>) {
        self.items = items
            .into_iter()
            .map(|(bt, count)| (ItemId::from(bt), count))
            .collect();
    }
}

// =============================================================================
// LocalPlatformInventory SystemParam
// =============================================================================

use bevy::ecs::system::SystemParam;

/// Bundled platform inventory access (reduces parameter count)
///
/// Usage:
/// ```ignore
/// fn my_system(mut platform_inv: LocalPlatformInventory) {
///     if let Some(mut inv) = platform_inv.get_mut() {
///         inv.add_item_by_id(items::iron_ore(), 10);
///     }
/// }
/// ```
#[derive(SystemParam)]
pub struct LocalPlatformInventory<'w, 's> {
    local_platform: Option<Res<'w, LocalPlatform>>,
    inventories: Query<'w, 's, &'static mut PlatformInventory>,
}

impl LocalPlatformInventory<'_, '_> {
    /// Get mutable access to the platform's inventory
    pub fn get_mut(&mut self) -> Option<Mut<'_, PlatformInventory>> {
        let local_platform = self.local_platform.as_ref()?;
        self.inventories.get_mut(local_platform.0).ok()
    }

    /// Get read-only access to the platform's inventory
    pub fn get(&self) -> Option<&PlatformInventory> {
        let local_platform = self.local_platform.as_ref()?;
        self.inventories.get(local_platform.0).ok()
    }

    /// Get the platform entity
    pub fn entity(&self) -> Option<Entity> {
        self.local_platform.as_ref().map(|lp| lp.0)
    }

    /// Add item to platform inventory
    ///
    /// Returns the amount that couldn't be added (always 0 for unlimited storage).
    pub fn add_item(&mut self, item_id: ItemId, amount: u32) -> u32 {
        if let Some(mut inventory) = self.get_mut() {
            inventory.add_item_by_id(item_id, amount);
            0
        } else {
            amount // No platform, can't add
        }
    }

    /// Remove item from platform inventory
    ///
    /// Returns true if successful, false if not enough items.
    pub fn remove_item(&mut self, item_id: ItemId, amount: u32) -> bool {
        if let Some(mut inventory) = self.get_mut() {
            inventory.remove_item_by_id(item_id, amount)
        } else {
            false
        }
    }

    /// Check if platform has enough items
    pub fn has_item(&self, item_id: ItemId, amount: u32) -> bool {
        if let Some(inventory) = self.get() {
            inventory.has_item_by_id(item_id, amount)
        } else {
            false
        }
    }

    /// Get count of specific item
    pub fn get_count(&self, item_id: ItemId) -> u32 {
        if let Some(inventory) = self.get() {
            inventory.get_count_by_id(item_id)
        } else {
            0
        }
    }

    /// Get all items
    pub fn get_all_items(&self) -> Vec<(ItemId, u32)> {
        if let Some(inventory) = self.get() {
            inventory.get_all_items_by_id()
        } else {
            Vec::new()
        }
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
        let mut inv = PlatformInventory::new();
        inv.add_item(BlockType::IronOre, 10);
        assert_eq!(inv.get_count(BlockType::IronOre), 10);

        inv.add_item(BlockType::IronOre, 5);
        assert_eq!(inv.get_count(BlockType::IronOre), 15);
    }

    #[test]
    #[allow(deprecated)]
    fn test_remove_item() {
        let mut inv = PlatformInventory::new();
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
        let mut inv = PlatformInventory::new();
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
        let inv = PlatformInventory::with_items(&[
            (BlockType::MinerBlock, 2),
            (BlockType::ConveyorBlock, 30),
        ]);
        assert_eq!(inv.get_count(BlockType::MinerBlock), 2);
        assert_eq!(inv.get_count(BlockType::ConveyorBlock), 30);
    }

    #[test]
    #[allow(deprecated)]
    fn test_get_all_items() {
        let mut inv = PlatformInventory::new();
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
        let mut inv = PlatformInventory::new();
        inv.add_item_by_id(items::iron_ore(), 10);
        assert_eq!(inv.get_count_by_id(items::iron_ore()), 10);

        inv.add_item_by_id(items::iron_ore(), 5);
        assert_eq!(inv.get_count_by_id(items::iron_ore()), 15);
    }

    #[test]
    fn test_remove_item_by_id() {
        let mut inv = PlatformInventory::new();
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
        let mut inv = PlatformInventory::new();
        inv.add_item_by_id(items::iron_ore(), 10);

        assert!(inv.has_item_by_id(items::iron_ore(), 5));
        assert!(inv.has_item_by_id(items::iron_ore(), 10));
        assert!(!inv.has_item_by_id(items::iron_ore(), 11));
        assert!(!inv.has_item_by_id(items::coal(), 1));
    }

    #[test]
    fn test_try_consume_by_id() {
        let mut inv = PlatformInventory::new();
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
        let inv = PlatformInventory::with_items_by_id(&[
            (items::miner_block(), 2),
            (items::conveyor_block(), 30),
        ]);
        assert_eq!(inv.get_count_by_id(items::miner_block()), 2);
        assert_eq!(inv.get_count_by_id(items::conveyor_block()), 30);
    }

    #[test]
    fn test_get_all_items_by_id() {
        let mut inv = PlatformInventory::new();
        inv.add_item_by_id(items::iron_ore(), 10);
        inv.add_item_by_id(items::coal(), 5);

        let all_items = inv.get_all_items_by_id();
        assert_eq!(all_items.len(), 2);
        assert!(all_items.contains(&(items::iron_ore(), 10)));
        assert!(all_items.contains(&(items::coal(), 5)));
    }

    #[test]
    fn test_all_item_ids() {
        let mut inv = PlatformInventory::new();
        inv.add_item_by_id(items::iron_ore(), 10);
        inv.add_item_by_id(items::coal(), 5);

        let all_ids: Vec<_> = inv.all_item_ids().collect();
        assert_eq!(all_ids.len(), 2);
        assert!(all_ids.contains(&items::iron_ore()));
        assert!(all_ids.contains(&items::coal()));
    }
}
