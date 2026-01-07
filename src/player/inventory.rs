//! Player inventory system

use crate::block_type::BlockType;
use crate::constants::{HOTBAR_SLOTS, MAX_STACK_SIZE, NUM_SLOTS};
use bevy::prelude::*;

/// Player inventory with fixed slots
/// Slots 0-8: Hotbar (visible at bottom of screen)
/// Slots 9-35: Main inventory (visible when E is pressed)
#[derive(Resource)]
pub struct Inventory {
    /// 36 slots total: 9 hotbar + 27 main inventory
    pub slots: [Option<(BlockType, u32)>; NUM_SLOTS],
    /// Currently selected hotbar slot index (0-8)
    pub selected_slot: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: [None; NUM_SLOTS],
            selected_slot: 0,
        }
    }
}

impl Inventory {
    /// Create a new inventory with initial items
    pub fn with_initial_items(items: &[(BlockType, u32)]) -> Self {
        let mut inv = Self::default();
        for (block_type, amount) in items {
            inv.add_item(*block_type, *amount);
        }
        inv
    }

    /// Check if a slot index is in the hotbar (0-8)
    #[allow(dead_code)]
    pub fn is_hotbar_slot(slot: usize) -> bool {
        slot < HOTBAR_SLOTS
    }

    /// Check if a slot index is in the main inventory (9-35)
    #[allow(dead_code)]
    pub fn is_main_slot(slot: usize) -> bool {
        (HOTBAR_SLOTS..NUM_SLOTS).contains(&slot)
    }

    /// Get the block type at a given slot index (returns None for empty slots)
    pub fn get_slot(&self, slot: usize) -> Option<BlockType> {
        self.slots.get(slot).and_then(|s| s.map(|(bt, _)| bt))
    }

    /// Get the count at a given slot index
    pub fn get_slot_count(&self, slot: usize) -> u32 {
        self.slots
            .get(slot)
            .and_then(|s| s.map(|(_, c)| c))
            .unwrap_or(0)
    }

    /// Get the currently selected block type (None if empty slot selected)
    pub fn selected_block(&self) -> Option<BlockType> {
        self.get_slot(self.selected_slot)
    }

    /// Add items to inventory, returns the amount that couldn't be added (0 = all added)
    pub fn add_item(&mut self, block_type: BlockType, mut amount: u32) -> u32 {
        // First, try to stack onto existing slots with same block type
        for slot in self.slots.iter_mut() {
            if amount == 0 {
                break;
            }
            if let Some((bt, count)) = slot {
                if *bt == block_type && *count < MAX_STACK_SIZE {
                    let space = MAX_STACK_SIZE - *count;
                    let to_add = amount.min(space);
                    *count += to_add;
                    amount -= to_add;
                }
            }
        }

        // Then, find empty slots for remaining items
        for slot in self.slots.iter_mut() {
            if amount == 0 {
                break;
            }
            if slot.is_none() {
                let to_add = amount.min(MAX_STACK_SIZE);
                *slot = Some((block_type, to_add));
                amount -= to_add;
            }
        }

        amount // Return remaining amount (0 = all added)
    }

    /// Move items from one slot to another. Returns true if any items were moved.
    #[allow(dead_code)]
    pub fn move_items(&mut self, from_slot: usize, to_slot: usize) -> bool {
        if from_slot >= NUM_SLOTS || to_slot >= NUM_SLOTS || from_slot == to_slot {
            return false;
        }

        let from_item = self.slots[from_slot].take();
        let to_item = self.slots[to_slot].take();

        match (from_item, to_item) {
            (None, _) => {
                // Nothing to move
                self.slots[to_slot] = to_item;
                false
            }
            (Some(from), None) => {
                // Move to empty slot
                self.slots[to_slot] = Some(from);
                true
            }
            (Some((from_type, from_count)), Some((to_type, to_count))) => {
                if from_type == to_type {
                    // Same type - try to stack
                    let space = MAX_STACK_SIZE - to_count;
                    let to_transfer = from_count.min(space);
                    if to_transfer > 0 {
                        self.slots[to_slot] = Some((to_type, to_count + to_transfer));
                        let remaining = from_count - to_transfer;
                        if remaining > 0 {
                            self.slots[from_slot] = Some((from_type, remaining));
                        }
                        true
                    } else {
                        // No space - swap
                        self.slots[from_slot] = Some((to_type, to_count));
                        self.slots[to_slot] = Some((from_type, from_count));
                        true
                    }
                } else {
                    // Different types - swap
                    self.slots[from_slot] = Some((to_type, to_count));
                    self.slots[to_slot] = Some((from_type, from_count));
                    true
                }
            }
        }
    }

    /// Remove one item from the selected slot, returns the block type if successful
    #[allow(dead_code)] // Used in tests
    pub fn consume_selected(&mut self) -> Option<BlockType> {
        if let Some(Some((block_type, count))) = self.slots.get_mut(self.selected_slot) {
            if *count > 0 {
                let bt = *block_type;
                *count -= 1;
                if *count == 0 {
                    self.slots[self.selected_slot] = None;
                }
                return Some(bt);
            }
        }
        None
    }

    /// Check if we have the selected block type with count > 0
    pub fn has_selected(&self) -> bool {
        self.slots
            .get(self.selected_slot)
            .and_then(|s| s.as_ref())
            .map(|(_, c)| *c > 0)
            .unwrap_or(false)
    }

    /// Get the selected block type if any
    pub fn get_selected_type(&self) -> Option<BlockType> {
        self.slots
            .get(self.selected_slot)
            .and_then(|s| s.as_ref())
            .filter(|(_, c)| *c > 0)
            .map(|(bt, _)| *bt)
    }

    /// Consume a specific block type from inventory (across multiple slots), returns true if successful
    pub fn consume_item(&mut self, block_type: BlockType, mut amount: u32) -> bool {
        // First check if we have enough total
        if self.get_total_count(block_type) < amount {
            return false;
        }

        // Consume from slots
        for slot in self.slots.iter_mut() {
            if amount == 0 {
                break;
            }
            if let Some((bt, count)) = slot {
                if *bt == block_type {
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

    /// Check if inventory has enough of a specific block type
    #[allow(dead_code)] // Reserved for future use
    pub fn has_item(&self, block_type: BlockType, amount: u32) -> bool {
        self.get_total_count(block_type) >= amount
    }

    /// Get count of a specific block type in inventory
    #[allow(dead_code)]
    pub fn get_item_count(&self, block_type: BlockType) -> u32 {
        for (bt, count) in self.slots.iter().flatten() {
            if *bt == block_type {
                return *count;
            }
        }
        0
    }

    /// Get total count of a specific block type across all slots
    pub fn get_total_count(&self, block_type: BlockType) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|(bt, _)| *bt == block_type)
            .map(|(_, count)| count)
            .sum()
    }

    /// Check if inventory is full (all slots occupied)
    #[allow(dead_code)] // Used in tests
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| s.is_some())
    }

    /// Get number of empty slots
    #[allow(dead_code)] // Used in tests
    pub fn empty_slot_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_none()).count()
    }
}

// =============================================================================
// PlayerInventory (Component) - For multiplayer-ready architecture
// =============================================================================

/// Player inventory component (multiplayer-ready)
#[derive(Component, Clone, Debug)]
pub struct PlayerInventory {
    pub slots: [Option<(BlockType, u32)>; NUM_SLOTS],
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
    pub fn with_initial_items(items: &[(BlockType, u32)]) -> Self {
        let mut inv = Self::default();
        for (block_type, amount) in items {
            inv.add_item(*block_type, *amount);
        }
        inv
    }

    pub fn is_hotbar_slot(slot: usize) -> bool {
        slot < HOTBAR_SLOTS
    }

    pub fn is_main_slot(slot: usize) -> bool {
        (HOTBAR_SLOTS..NUM_SLOTS).contains(&slot)
    }

    pub fn get_slot(&self, slot: usize) -> Option<BlockType> {
        self.slots.get(slot).and_then(|s| s.map(|(bt, _)| bt))
    }

    pub fn get_slot_count(&self, slot: usize) -> u32 {
        self.slots
            .get(slot)
            .and_then(|s| s.map(|(_, c)| c))
            .unwrap_or(0)
    }

    pub fn add_item(&mut self, block_type: BlockType, mut amount: u32) -> u32 {
        for slot in self.slots.iter_mut() {
            if amount == 0 {
                break;
            }
            if let Some((bt, count)) = slot {
                if *bt == block_type && *count < MAX_STACK_SIZE {
                    let space = MAX_STACK_SIZE - *count;
                    let to_add = amount.min(space);
                    *count += to_add;
                    amount -= to_add;
                }
            }
        }
        for slot in self.slots.iter_mut() {
            if amount == 0 {
                break;
            }
            if slot.is_none() {
                let to_add = amount.min(MAX_STACK_SIZE);
                *slot = Some((block_type, to_add));
                amount -= to_add;
            }
        }
        amount
    }

    pub fn get_total_count(&self, block_type: BlockType) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|(bt, _)| *bt == block_type)
            .map(|(_, count)| count)
            .sum()
    }
}

// =============================================================================
// LocalPlayer Resource
// =============================================================================

/// Resource holding the local player's entity
#[derive(Resource)]
pub struct LocalPlayer(pub Entity);

// =============================================================================
// Inventory Sync System
// =============================================================================

/// Sync Inventory (Resource) with PlayerInventory (Component)
pub fn sync_inventory_system(
    mut legacy_inv: ResMut<Inventory>,
    local_player: Option<Res<LocalPlayer>>,
    mut inventory_query: Query<&mut PlayerInventory>,
) {
    let Some(local_player) = local_player else {
        return;
    };
    let Ok(mut component_inv) = inventory_query.get_mut(local_player.0) else {
        return;
    };
    if legacy_inv.is_changed() {
        component_inv.slots = legacy_inv.slots;
        component_inv.selected_slot = legacy_inv.selected_slot;
    } else if component_inv.is_changed() {
        legacy_inv.slots = component_inv.slots;
        legacy_inv.selected_slot = component_inv.selected_slot;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_add_item_to_empty() {
        let mut inv = Inventory::default();
        let remaining = inv.add_item(BlockType::Stone, 10);
        assert_eq!(remaining, 0);
        assert_eq!(inv.get_slot(0), Some(BlockType::Stone));
        assert_eq!(inv.get_slot_count(0), 10);
    }

    #[test]
    fn test_inventory_add_item_stacks() {
        let mut inv = Inventory::default();
        inv.add_item(BlockType::Stone, 50);
        inv.add_item(BlockType::Stone, 30);

        // Should stack on first slot
        assert_eq!(inv.get_slot_count(0), 80);
        assert!(inv.get_slot(1).is_none());
    }

    #[test]
    fn test_inventory_add_item_overflow_to_new_slot() {
        let mut inv = Inventory::default();
        inv.add_item(BlockType::Stone, MAX_STACK_SIZE - 10);
        inv.add_item(BlockType::Stone, 50);

        // First slot should be maxed
        assert_eq!(inv.get_slot_count(0), MAX_STACK_SIZE);
        // Remaining should go to second slot
        assert_eq!(inv.get_slot_count(1), 40);
    }

    #[test]
    fn test_inventory_different_block_types() {
        let mut inv = Inventory::default();
        inv.add_item(BlockType::Stone, 10);
        inv.add_item(BlockType::IronOre, 20);

        assert_eq!(inv.get_slot(0), Some(BlockType::Stone));
        assert_eq!(inv.get_slot(1), Some(BlockType::IronOre));
        assert_eq!(inv.get_slot_count(0), 10);
        assert_eq!(inv.get_slot_count(1), 20);
    }

    #[test]
    fn test_inventory_consume_selected() {
        let mut inv = Inventory::default();
        inv.add_item(BlockType::Stone, 5);
        inv.selected_slot = 0;

        let consumed = inv.consume_selected();
        assert_eq!(consumed, Some(BlockType::Stone));
        assert_eq!(inv.get_slot_count(0), 4);

        // Consume until empty
        for _ in 0..4 {
            inv.consume_selected();
        }
        assert!(inv.get_slot(0).is_none());
        assert_eq!(inv.consume_selected(), None);
    }

    #[test]
    fn test_inventory_consume_item() {
        let mut inv = Inventory::default();
        inv.add_item(BlockType::Stone, 10);
        inv.add_item(BlockType::IronOre, 5);

        assert!(inv.consume_item(BlockType::Stone, 5));
        assert_eq!(inv.get_slot_count(0), 5);

        assert!(!inv.consume_item(BlockType::Stone, 10)); // Not enough
        assert_eq!(inv.get_slot_count(0), 5); // Unchanged
    }

    #[test]
    fn test_inventory_move_items_swap() {
        let mut inv = Inventory::default();
        inv.slots[0] = Some((BlockType::Stone, 10));
        inv.slots[1] = Some((BlockType::IronOre, 20));

        assert!(inv.move_items(0, 1));
        assert_eq!(inv.get_slot(0), Some(BlockType::IronOre));
        assert_eq!(inv.get_slot(1), Some(BlockType::Stone));
    }

    #[test]
    fn test_inventory_move_items_to_empty() {
        let mut inv = Inventory::default();
        inv.slots[0] = Some((BlockType::Stone, 10));

        assert!(inv.move_items(0, 5));
        assert!(inv.get_slot(0).is_none());
        assert_eq!(inv.get_slot(5), Some(BlockType::Stone));
    }

    #[test]
    fn test_inventory_move_items_stack_same_type() {
        let mut inv = Inventory::default();
        inv.slots[0] = Some((BlockType::Stone, 30));
        inv.slots[1] = Some((BlockType::Stone, 40));

        assert!(inv.move_items(0, 1));
        // Should stack: 40 + 30 = 70, within MAX_STACK_SIZE
        assert_eq!(inv.get_slot_count(1), 70);
        assert!(inv.get_slot(0).is_none());
    }

    #[test]
    fn test_inventory_selected_block() {
        let mut inv = Inventory::default();
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
    fn test_inventory_is_full() {
        let mut inv = Inventory::default();
        assert!(!inv.is_full());

        // Fill all slots
        for i in 0..NUM_SLOTS {
            inv.slots[i] = Some((BlockType::Stone, 1));
        }
        assert!(inv.is_full());
    }

    #[test]
    fn test_inventory_empty_slot_count() {
        let mut inv = Inventory::default();
        assert_eq!(inv.empty_slot_count(), NUM_SLOTS);

        inv.add_item(BlockType::Stone, 10);
        assert_eq!(inv.empty_slot_count(), NUM_SLOTS - 1);
    }

    #[test]
    fn test_inventory_get_total_count() {
        let mut inv = Inventory::default();
        inv.slots[0] = Some((BlockType::Stone, 50));
        inv.slots[5] = Some((BlockType::Stone, 30));
        inv.slots[10] = Some((BlockType::IronOre, 20));

        assert_eq!(inv.get_total_count(BlockType::Stone), 80);
        assert_eq!(inv.get_total_count(BlockType::IronOre), 20);
        assert_eq!(inv.get_total_count(BlockType::Coal), 0);
    }

    #[test]
    fn test_inventory_hotbar_main_slots() {
        assert!(Inventory::is_hotbar_slot(0));
        assert!(Inventory::is_hotbar_slot(8));
        assert!(!Inventory::is_hotbar_slot(9));

        assert!(!Inventory::is_main_slot(0));
        assert!(Inventory::is_main_slot(9));
        assert!(Inventory::is_main_slot(35));
    }
}
