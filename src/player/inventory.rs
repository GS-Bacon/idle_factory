//! Player inventory system

use bevy::prelude::*;
use crate::block_type::BlockType;
use crate::constants::{NUM_SLOTS, HOTBAR_SLOTS, MAX_STACK_SIZE};

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
        self.slots.get(slot).and_then(|s| s.map(|(_, c)| c)).unwrap_or(0)
    }

    /// Get the currently selected block type (None if empty slot selected)
    pub fn selected_block(&self) -> Option<BlockType> {
        self.get_slot(self.selected_slot)
    }

    /// Add items to inventory, returns the amount that couldn't be added (0 = all added)
    pub fn add_item(&mut self, block_type: BlockType, mut amount: u32) -> bool {
        // First, try to stack onto existing slots with same block type
        for slot in self.slots.iter_mut() {
            if amount == 0 { break; }
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
            if amount == 0 { break; }
            if slot.is_none() {
                let to_add = amount.min(MAX_STACK_SIZE);
                *slot = Some((block_type, to_add));
                amount -= to_add;
            }
        }

        amount == 0 // Return true if all items were added
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
        self.slots.get(self.selected_slot)
            .and_then(|s| s.as_ref())
            .map(|(_, c)| *c > 0)
            .unwrap_or(false)
    }

    /// Get the selected block type if any
    pub fn get_selected_type(&self) -> Option<BlockType> {
        self.slots.get(self.selected_slot)
            .and_then(|s| s.as_ref())
            .filter(|(_, c)| *c > 0)
            .map(|(bt, _)| *bt)
    }

    /// Consume a specific block type from inventory, returns true if successful
    pub fn consume_item(&mut self, block_type: BlockType, amount: u32) -> bool {
        for slot in self.slots.iter_mut() {
            if let Some((bt, count)) = slot {
                if *bt == block_type && *count >= amount {
                    *count -= amount;
                    if *count == 0 {
                        *slot = None;
                    }
                    return true;
                }
            }
        }
        false
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
}
