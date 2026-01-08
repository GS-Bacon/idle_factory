//! Pure inventory calculation logic (Bevy-independent)
//!
//! This module provides inventory operations that don't depend on Bevy.
//! Uses ItemId for type-safe item identification.

use super::ItemId;

/// Maximum stack size for items
pub const MAX_STACK: u32 = 999;

/// An item stack (type + count)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemStack {
    pub item: ItemId,
    pub count: u32,
}

impl ItemStack {
    /// Create a new item stack
    pub fn new(item: ItemId, count: u32) -> Self {
        Self {
            item,
            count: count.min(MAX_STACK),
        }
    }

    /// Check if this stack can merge with another
    pub fn can_merge(&self, other: &ItemStack) -> bool {
        self.item == other.item && self.count < MAX_STACK
    }

    /// Merge as much as possible from another stack, returns remaining count
    pub fn merge_from(&mut self, other: &mut ItemStack) -> u32 {
        if self.item != other.item {
            return other.count;
        }

        let space = MAX_STACK.saturating_sub(self.count);
        let to_add = other.count.min(space);
        self.count += to_add;
        other.count -= to_add;
        other.count
    }
}

/// Try to add an item to a slot array, returns remaining count
pub fn try_add_to_slots(slots: &mut [Option<ItemStack>], item: ItemId, mut count: u32) -> u32 {
    // First, try to stack with existing items
    for slot in slots.iter_mut() {
        if count == 0 {
            break;
        }
        if let Some(stack) = slot {
            if stack.item == item && stack.count < MAX_STACK {
                let space = MAX_STACK - stack.count;
                let to_add = count.min(space);
                stack.count += to_add;
                count -= to_add;
            }
        }
    }

    // Then, use empty slots
    for slot in slots.iter_mut() {
        if count == 0 {
            break;
        }
        if slot.is_none() {
            let to_add = count.min(MAX_STACK);
            *slot = Some(ItemStack::new(item, to_add));
            count -= to_add;
        }
    }

    count
}

/// Remove items from slots, returns actually removed count
pub fn remove_from_slots(slots: &mut [Option<ItemStack>], item: ItemId, mut count: u32) -> u32 {
    let original = count;

    for slot in slots.iter_mut() {
        if count == 0 {
            break;
        }
        if let Some(stack) = slot {
            if stack.item == item {
                let to_remove = count.min(stack.count);
                stack.count -= to_remove;
                count -= to_remove;
                if stack.count == 0 {
                    *slot = None;
                }
            }
        }
    }

    original - count
}

/// Count total items of a type in slots
pub fn count_in_slots(slots: &[Option<ItemStack>], item: ItemId) -> u32 {
    slots
        .iter()
        .filter_map(|s| s.as_ref())
        .filter(|s| s.item == item)
        .map(|s| s.count)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

    #[test]
    fn test_item_stack_merge() {
        let mut stack1 = ItemStack::new(items::iron_ore(), 50);
        let mut stack2 = ItemStack::new(items::iron_ore(), 30);

        let remaining = stack1.merge_from(&mut stack2);
        assert_eq!(remaining, 0);
        assert_eq!(stack1.count, 80);
        assert_eq!(stack2.count, 0);
    }

    #[test]
    fn test_try_add_to_slots() {
        let mut slots: [Option<ItemStack>; 3] = [None, None, None];

        // Add to empty slots
        let remaining = try_add_to_slots(&mut slots, items::iron_ore(), 100);
        assert_eq!(remaining, 0);
        assert_eq!(slots[0], Some(ItemStack::new(items::iron_ore(), 100)));

        // Stack with existing
        let remaining = try_add_to_slots(&mut slots, items::iron_ore(), 50);
        assert_eq!(remaining, 0);
        assert_eq!(slots[0], Some(ItemStack::new(items::iron_ore(), 150)));
    }

    #[test]
    fn test_remove_from_slots() {
        let mut slots: [Option<ItemStack>; 2] = [
            Some(ItemStack::new(items::iron_ore(), 100)),
            Some(ItemStack::new(items::iron_ore(), 50)),
        ];

        let removed = remove_from_slots(&mut slots, items::iron_ore(), 120);
        assert_eq!(removed, 120);
        assert_eq!(slots[0], None);
        assert_eq!(slots[1], Some(ItemStack::new(items::iron_ore(), 30)));
    }

    #[test]
    fn test_count_in_slots() {
        let slots: [Option<ItemStack>; 3] = [
            Some(ItemStack::new(items::iron_ore(), 100)),
            Some(ItemStack::new(items::copper_ore(), 50)),
            Some(ItemStack::new(items::iron_ore(), 25)),
        ];

        assert_eq!(count_in_slots(&slots, items::iron_ore()), 125);
        assert_eq!(count_in_slots(&slots, items::copper_ore()), 50);
        assert_eq!(count_in_slots(&slots, items::coal()), 0);
    }
}
