//! Player inventory system
//!
//! Uses ItemId for all item storage, supporting both base game and mod items.

use crate::block_type::BlockType;
use crate::constants::{HOTBAR_SLOTS, MAX_STACK_SIZE, NUM_SLOTS};
use crate::core::ItemId;
use bevy::prelude::*;

// =============================================================================
// PlayerInventory (Component) - For multiplayer-ready architecture
// =============================================================================

/// Player inventory component (multiplayer-ready)
///
/// Uses ItemId internally to support both base game and mod items.
#[derive(Component, Clone, Debug)]
pub struct PlayerInventory {
    pub slots: [Option<(ItemId, u32)>; NUM_SLOTS],
    pub selected_slot: usize,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self {
            slots: [None; NUM_SLOTS],
            selected_slot: 0,
        }
    }
}

impl PlayerInventory {
    // =========================================================================
    // Slot utilities
    // =========================================================================

    pub fn is_hotbar_slot(slot: usize) -> bool {
        slot < HOTBAR_SLOTS
    }

    pub fn is_main_slot(slot: usize) -> bool {
        (HOTBAR_SLOTS..NUM_SLOTS).contains(&slot)
    }

    /// Check if we have the selected item with count > 0
    pub fn has_selected(&self) -> bool {
        self.slots
            .get(self.selected_slot)
            .and_then(|s| s.as_ref())
            .map(|(_, c)| *c > 0)
            .unwrap_or(false)
    }

    // =========================================================================
    // ItemId-based API
    // =========================================================================

    /// Create inventory with initial items
    pub fn with_initial_items_by_id(items: &[(ItemId, u32)]) -> Self {
        let mut inv = Self::default();
        for (item_id, amount) in items {
            inv.add_item_by_id(*item_id, *amount);
        }
        inv
    }

    /// Get item ID at a specific slot
    pub fn get_slot_item_id(&self, slot: usize) -> Option<ItemId> {
        self.slots.get(slot).and_then(|s| s.map(|(id, _)| id))
    }

    /// Get count at a specific slot
    pub fn get_slot_count(&self, slot: usize) -> u32 {
        self.slots
            .get(slot)
            .and_then(|s| s.map(|(_, c)| c))
            .unwrap_or(0)
    }

    /// Add item by ItemId. Returns the amount that couldn't be added (overflow).
    pub fn add_item_by_id(&mut self, item_id: ItemId, mut amount: u32) -> u32 {
        // First try to stack with existing items
        for slot in self.slots.iter_mut() {
            if amount == 0 {
                break;
            }
            if let Some((id, count)) = slot {
                if *id == item_id && *count < MAX_STACK_SIZE {
                    let space = MAX_STACK_SIZE - *count;
                    let to_add = amount.min(space);
                    *count += to_add;
                    amount -= to_add;
                }
            }
        }
        // Then fill empty slots
        for slot in self.slots.iter_mut() {
            if amount == 0 {
                break;
            }
            if slot.is_none() {
                let to_add = amount.min(MAX_STACK_SIZE);
                *slot = Some((item_id, to_add));
                amount -= to_add;
            }
        }
        amount
    }

    /// Get total count of an item by ItemId
    pub fn get_total_count_by_id(&self, item_id: ItemId) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|(id, _)| *id == item_id)
            .map(|(_, count)| count)
            .sum()
    }

    /// Get the currently selected item ID (None if empty slot selected)
    pub fn selected_item_id(&self) -> Option<ItemId> {
        self.get_slot_item_id(self.selected_slot)
    }

    /// Get the selected item ID if any (with count > 0)
    pub fn get_selected_item_id(&self) -> Option<ItemId> {
        self.slots
            .get(self.selected_slot)
            .and_then(|s| s.as_ref())
            .filter(|(_, c)| *c > 0)
            .map(|(id, _)| *id)
    }

    /// Consume a specific item from inventory, returns true if successful
    pub fn consume_item_by_id(&mut self, item_id: ItemId, mut amount: u32) -> bool {
        // First check if we have enough total
        if self.get_total_count_by_id(item_id) < amount {
            return false;
        }

        // Consume from slots
        for slot in self.slots.iter_mut() {
            if amount == 0 {
                break;
            }
            if let Some((id, count)) = slot {
                if *id == item_id {
                    let to_consume = amount.min(*count);
                    *count -= to_consume;
                    amount -= to_consume;
                    if *count == 0 {
                        *slot = None;
                    }
                }
            }
        }
        true
    }

    /// Check if inventory has at least the specified amount of an item
    pub fn has_item_by_id(&self, item_id: ItemId, count: u32) -> bool {
        self.get_total_count_by_id(item_id) >= count
    }

    /// Get all items as (ItemId, count) pairs
    pub fn get_all_items_by_id(&self) -> Vec<(ItemId, u32)> {
        self.slots
            .iter()
            .flatten()
            .map(|(id, count)| (*id, *count))
            .collect()
    }

    // =========================================================================
    // BlockType compatibility API (for legacy code during migration)
    // =========================================================================

    /// Create inventory with initial items by BlockType (legacy compatibility)
    #[deprecated(since = "0.4.0", note = "Use with_initial_items_by_id instead")]
    pub fn with_initial_items(items: &[(BlockType, u32)]) -> Self {
        let items_by_id: Vec<(ItemId, u32)> = items
            .iter()
            .map(|(bt, count)| (ItemId::from(*bt), *count))
            .collect();
        Self::with_initial_items_by_id(&items_by_id)
    }

    /// Add item by BlockType (legacy compatibility)
    #[deprecated(since = "0.4.0", note = "Use add_item_by_id instead")]
    pub fn add_item(&mut self, block_type: BlockType, amount: u32) -> u32 {
        self.add_item_by_id(ItemId::from(block_type), amount)
    }

    /// Get total count by BlockType (legacy compatibility)
    #[deprecated(since = "0.4.0", note = "Use get_total_count_by_id instead")]
    pub fn get_total_count(&self, block_type: BlockType) -> u32 {
        self.get_total_count_by_id(ItemId::from(block_type))
    }

    /// Consume item by BlockType (legacy compatibility)
    #[deprecated(since = "0.4.0", note = "Use consume_item_by_id instead")]
    pub fn consume_item(&mut self, block_type: BlockType, amount: u32) -> bool {
        self.consume_item_by_id(ItemId::from(block_type), amount)
    }

    /// Get slot as BlockType (legacy compatibility)
    #[deprecated(since = "0.4.0", note = "Use get_slot_item_id instead")]
    pub fn get_slot(&self, slot: usize) -> Option<BlockType> {
        self.get_slot_item_id(slot)
            .and_then(|id| BlockType::try_from(id).ok())
    }

    /// Get selected as BlockType (legacy compatibility)
    #[deprecated(since = "0.4.0", note = "Use selected_item_id instead")]
    pub fn selected_block(&self) -> Option<BlockType> {
        self.selected_item_id()
            .and_then(|id| BlockType::try_from(id).ok())
    }

    /// Get selected type as BlockType (legacy compatibility)
    #[deprecated(since = "0.4.0", note = "Use get_selected_item_id instead")]
    pub fn get_selected_type(&self) -> Option<BlockType> {
        self.get_selected_item_id()
            .and_then(|id| BlockType::try_from(id).ok())
    }
}

// =============================================================================
// LocalPlayer Resource
// =============================================================================

/// Resource holding the local player's entity
#[derive(Resource)]
pub struct LocalPlayer(pub Entity);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

    // =========================================================================
    // Legacy BlockType Tests (with #[allow(deprecated)])
    // =========================================================================

    #[test]
    #[allow(deprecated)]
    fn test_inventory_add_item_to_empty() {
        let mut inv = PlayerInventory::default();
        let remaining = inv.add_item(BlockType::Stone, 10);
        assert_eq!(remaining, 0);
        assert_eq!(inv.get_slot(0), Some(BlockType::Stone));
        assert_eq!(inv.get_slot_count(0), 10);
    }

    #[test]
    #[allow(deprecated)]
    fn test_inventory_add_item_stacks() {
        let mut inv = PlayerInventory::default();
        inv.add_item(BlockType::Stone, 50);
        inv.add_item(BlockType::Stone, 30);

        // Should stack on first slot
        assert_eq!(inv.get_slot_count(0), 80);
        assert!(inv.get_slot(1).is_none());
    }

    #[test]
    #[allow(deprecated)]
    fn test_inventory_add_item_overflow_to_new_slot() {
        let mut inv = PlayerInventory::default();
        inv.add_item(BlockType::Stone, MAX_STACK_SIZE - 10);
        inv.add_item(BlockType::Stone, 50);

        // First slot should be maxed
        assert_eq!(inv.get_slot_count(0), MAX_STACK_SIZE);
        // Remaining should go to second slot
        assert_eq!(inv.get_slot_count(1), 40);
    }

    #[test]
    #[allow(deprecated)]
    fn test_inventory_different_block_types() {
        let mut inv = PlayerInventory::default();
        inv.add_item(BlockType::Stone, 10);
        inv.add_item(BlockType::IronOre, 20);

        assert_eq!(inv.get_slot(0), Some(BlockType::Stone));
        assert_eq!(inv.get_slot(1), Some(BlockType::IronOre));
        assert_eq!(inv.get_slot_count(0), 10);
        assert_eq!(inv.get_slot_count(1), 20);
    }

    #[test]
    #[allow(deprecated)]
    fn test_inventory_consume_item() {
        let mut inv = PlayerInventory::default();
        inv.add_item(BlockType::Stone, 10);
        inv.add_item(BlockType::IronOre, 5);

        assert!(inv.consume_item(BlockType::Stone, 5));
        assert_eq!(inv.get_slot_count(0), 5);

        assert!(!inv.consume_item(BlockType::Stone, 10)); // Not enough
        assert_eq!(inv.get_slot_count(0), 5); // Unchanged
    }

    #[test]
    #[allow(deprecated)]
    fn test_inventory_selected_block() {
        let mut inv = PlayerInventory::default();
        inv.add_item(BlockType::Stone, 10);
        inv.add_item(BlockType::IronOre, 5);

        inv.selected_slot = 0;
        assert_eq!(inv.selected_block(), Some(BlockType::Stone));

        inv.selected_slot = 1;
        assert_eq!(inv.selected_block(), Some(BlockType::IronOre));

        inv.selected_slot = 5; // Empty slot
        assert_eq!(inv.selected_block(), None);
    }

    #[test]
    #[allow(deprecated)]
    fn test_inventory_get_total_count() {
        let mut inv = PlayerInventory::default();
        inv.slots[0] = Some((items::stone(), 50));
        inv.slots[5] = Some((items::stone(), 30));
        inv.slots[10] = Some((items::iron_ore(), 20));

        assert_eq!(inv.get_total_count(BlockType::Stone), 80);
        assert_eq!(inv.get_total_count(BlockType::IronOre), 20);
        assert_eq!(inv.get_total_count(BlockType::Coal), 0);
    }

    #[test]
    fn test_inventory_hotbar_main_slots() {
        assert!(PlayerInventory::is_hotbar_slot(0));
        assert!(PlayerInventory::is_hotbar_slot(8));
        assert!(!PlayerInventory::is_hotbar_slot(9));

        assert!(!PlayerInventory::is_main_slot(0));
        assert!(PlayerInventory::is_main_slot(9));
        assert!(PlayerInventory::is_main_slot(35));
    }

    // =========================================================================
    // New ItemId Tests
    // =========================================================================

    #[test]
    fn test_inventory_add_item_by_id() {
        let mut inv = PlayerInventory::default();
        let remaining = inv.add_item_by_id(items::stone(), 10);
        assert_eq!(remaining, 0);
        assert_eq!(inv.get_slot_item_id(0), Some(items::stone()));
        assert_eq!(inv.get_slot_count(0), 10);
    }

    #[test]
    fn test_inventory_add_item_by_id_stacks() {
        let mut inv = PlayerInventory::default();
        inv.add_item_by_id(items::stone(), 50);
        inv.add_item_by_id(items::stone(), 30);

        // Should stack on first slot
        assert_eq!(inv.get_slot_count(0), 80);
        assert!(inv.get_slot_item_id(1).is_none());
    }

    #[test]
    fn test_inventory_get_total_count_by_id() {
        let mut inv = PlayerInventory::default();
        inv.slots[0] = Some((items::stone(), 50));
        inv.slots[5] = Some((items::stone(), 30));
        inv.slots[10] = Some((items::iron_ore(), 20));

        assert_eq!(inv.get_total_count_by_id(items::stone()), 80);
        assert_eq!(inv.get_total_count_by_id(items::iron_ore()), 20);
        assert_eq!(inv.get_total_count_by_id(items::coal()), 0);
    }

    #[test]
    fn test_inventory_consume_item_by_id() {
        let mut inv = PlayerInventory::default();
        inv.add_item_by_id(items::stone(), 10);
        inv.add_item_by_id(items::iron_ore(), 5);

        assert!(inv.consume_item_by_id(items::stone(), 5));
        assert_eq!(inv.get_slot_count(0), 5);

        assert!(!inv.consume_item_by_id(items::stone(), 10)); // Not enough
        assert_eq!(inv.get_slot_count(0), 5); // Unchanged
    }

    #[test]
    fn test_inventory_selected_item_id() {
        let mut inv = PlayerInventory::default();
        inv.add_item_by_id(items::stone(), 10);
        inv.add_item_by_id(items::iron_ore(), 5);

        inv.selected_slot = 0;
        assert_eq!(inv.selected_item_id(), Some(items::stone()));

        inv.selected_slot = 1;
        assert_eq!(inv.selected_item_id(), Some(items::iron_ore()));

        inv.selected_slot = 5; // Empty slot
        assert_eq!(inv.selected_item_id(), None);
    }

    #[test]
    fn test_inventory_has_item_by_id() {
        let mut inv = PlayerInventory::default();
        inv.add_item_by_id(items::stone(), 50);

        assert!(inv.has_item_by_id(items::stone(), 30));
        assert!(inv.has_item_by_id(items::stone(), 50));
        assert!(!inv.has_item_by_id(items::stone(), 51));
        assert!(!inv.has_item_by_id(items::iron_ore(), 1));
    }

    #[test]
    fn test_inventory_get_all_items_by_id() {
        let mut inv = PlayerInventory::default();
        inv.add_item_by_id(items::stone(), 10);
        inv.add_item_by_id(items::iron_ore(), 20);

        let all_items = inv.get_all_items_by_id();
        assert_eq!(all_items.len(), 2);
        assert!(all_items.contains(&(items::stone(), 10)));
        assert!(all_items.contains(&(items::iron_ore(), 20)));
    }

    #[test]
    fn test_inventory_with_initial_items_by_id() {
        let inv = PlayerInventory::with_initial_items_by_id(&[
            (items::stone(), 10),
            (items::iron_ore(), 20),
        ]);

        assert_eq!(inv.get_total_count_by_id(items::stone()), 10);
        assert_eq!(inv.get_total_count_by_id(items::iron_ore()), 20);
    }
}
