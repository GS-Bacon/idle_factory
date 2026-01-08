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
//! ## Storage
//!
//! This module uses ItemId internally for all item identification.

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
