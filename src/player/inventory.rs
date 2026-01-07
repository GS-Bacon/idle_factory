//! Player inventory system

use crate::block_type::BlockType;
use crate::constants::{HOTBAR_SLOTS, MAX_STACK_SIZE, NUM_SLOTS};
use bevy::prelude::*;

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

    /// Get the currently selected block type (None if empty slot selected)
    pub fn selected_block(&self) -> Option<BlockType> {
        self.get_slot(self.selected_slot)
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

    #[test]
    fn test_inventory_add_item_to_empty() {
        let mut inv = PlayerInventory::default();
        let remaining = inv.add_item(BlockType::Stone, 10);
        assert_eq!(remaining, 0);
        assert_eq!(inv.get_slot(0), Some(BlockType::Stone));
        assert_eq!(inv.get_slot_count(0), 10);
    }

    #[test]
    fn test_inventory_add_item_stacks() {
        let mut inv = PlayerInventory::default();
        inv.add_item(BlockType::Stone, 50);
        inv.add_item(BlockType::Stone, 30);

        // Should stack on first slot
        assert_eq!(inv.get_slot_count(0), 80);
        assert!(inv.get_slot(1).is_none());
    }

    #[test]
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
    fn test_inventory_get_total_count() {
        let mut inv = PlayerInventory::default();
        inv.slots[0] = Some((BlockType::Stone, 50));
        inv.slots[5] = Some((BlockType::Stone, 30));
        inv.slots[10] = Some((BlockType::IronOre, 20));

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
}
