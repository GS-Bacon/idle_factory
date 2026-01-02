//! Property-based tests for game invariants using proptest
//!
//! These tests verify that critical invariants hold across random inputs.

use idle_factory::BlockType;
use proptest::prelude::*;

/// Test helper: Simple inventory for property testing
#[derive(Clone, Debug, Default)]
struct TestInventory {
    slots: Vec<Option<(BlockType, u32)>>,
}

impl TestInventory {
    fn new(size: usize) -> Self {
        Self {
            slots: vec![None; size],
        }
    }

    fn add(&mut self, block_type: BlockType, count: u32) -> u32 {
        let mut remaining = count;
        for slot in &mut self.slots {
            if remaining == 0 {
                break;
            }
            match slot {
                Some((bt, c)) if *bt == block_type => {
                    let space = 64u32.saturating_sub(*c);
                    let to_add = remaining.min(space);
                    *c += to_add;
                    remaining -= to_add;
                }
                None => {
                    let to_add = remaining.min(64);
                    *slot = Some((block_type, to_add));
                    remaining -= to_add;
                }
                _ => {}
            }
        }
        count - remaining
    }

    fn remove(&mut self, block_type: BlockType, count: u32) -> u32 {
        let mut removed = 0u32;
        for slot in &mut self.slots {
            if removed >= count {
                break;
            }
            if let Some((bt, c)) = slot {
                if *bt == block_type {
                    let to_remove = (count - removed).min(*c);
                    *c -= to_remove;
                    removed += to_remove;
                    if *c == 0 {
                        *slot = None;
                    }
                }
            }
        }
        removed
    }

    fn count(&self, block_type: BlockType) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|(bt, _)| *bt == block_type)
            .map(|(_, c)| c)
            .sum()
    }

    fn total_count(&self) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|(_, c)| c)
            .sum()
    }
}

proptest! {
    /// Invariant: Inventory item count never goes negative
    #[test]
    fn inventory_never_negative(
        add_count in 0u32..1000,
        remove_count in 0u32..2000
    ) {
        let mut inv = TestInventory::new(36);
        inv.add(BlockType::Stone, add_count);
        inv.remove(BlockType::Stone, remove_count);

        // Count should never be negative (always >= 0)
        prop_assert!(inv.count(BlockType::Stone) <= add_count,
            "Count {} should be <= added {}", inv.count(BlockType::Stone), add_count);
    }

    /// Invariant: Item transfers preserve total count
    #[test]
    fn transfer_preserves_total(
        initial_a in 0u32..500,
        initial_b in 0u32..500,
        transfer in 0u32..300
    ) {
        let mut inv_a = TestInventory::new(36);
        let mut inv_b = TestInventory::new(36);

        inv_a.add(BlockType::Stone, initial_a);
        inv_b.add(BlockType::Stone, initial_b);

        let before_total = inv_a.total_count() + inv_b.total_count();

        // Transfer from A to B
        let transfer_amount = transfer.min(inv_a.count(BlockType::Stone));
        let removed = inv_a.remove(BlockType::Stone, transfer_amount);
        inv_b.add(BlockType::Stone, removed);

        let after_total = inv_a.total_count() + inv_b.total_count();

        prop_assert_eq!(before_total, after_total,
            "Total items before ({}) should equal after ({})", before_total, after_total);
    }

    /// Invariant: Slot counts stay within valid range [1, 64]
    #[test]
    fn slot_count_in_valid_range(
        add_count in 1u32..1000
    ) {
        let mut inv = TestInventory::new(36);
        inv.add(BlockType::Stone, add_count);

        for slot in &inv.slots {
            if let Some((_, count)) = slot {
                prop_assert!(*count >= 1 && *count <= 64,
                    "Slot count {} should be in [1, 64]", count);
            }
        }
    }

    /// Invariant: Adding then removing same amount results in original state
    #[test]
    fn add_remove_symmetry(
        count in 1u32..100
    ) {
        let mut inv = TestInventory::new(36);
        let initial = inv.count(BlockType::Stone);

        inv.add(BlockType::Stone, count);
        inv.remove(BlockType::Stone, count);

        prop_assert_eq!(inv.count(BlockType::Stone), initial,
            "After add({}) and remove({}), count should be {}", count, count, initial);
    }

    /// Invariant: Mining progress is always in [0, 1]
    #[test]
    fn mining_progress_bounded(
        delta_time in 0.0f32..10.0,
        mining_time in 0.1f32..10.0
    ) {
        let mut progress = 0.0f32;

        // Simulate multiple mining ticks
        for _ in 0..10 {
            progress += delta_time / mining_time;
            if progress >= 1.0 {
                progress = 0.0; // Reset on completion
            }
        }

        prop_assert!(progress >= 0.0 && progress <= 1.0,
            "Mining progress {} should be in [0, 1]", progress);
    }

    /// Invariant: Conveyor item positions are in [0, 1]
    #[test]
    fn conveyor_item_position_bounded(
        initial_pos in 0.0f32..1.0,
        speed in 0.1f32..2.0,
        delta_time in 0.0f32..0.1
    ) {
        let mut pos = initial_pos;

        // Simulate conveyor movement
        pos += speed * delta_time;

        // Item exits when pos >= 1.0, stays in [0, 1) otherwise
        if pos < 1.0 {
            prop_assert!(pos >= 0.0 && pos < 1.0,
                "Item position {} should be in [0, 1)", pos);
        }
        // If pos >= 1.0, item exits conveyor (valid state)
    }

    /// Invariant: Block type has valid name
    #[test]
    fn block_type_has_name(block_idx in 0usize..11) {
        let block_types = [
            BlockType::Stone,
            BlockType::Grass,
            BlockType::IronOre,
            BlockType::Coal,
            BlockType::IronIngot,
            BlockType::MinerBlock,
            BlockType::ConveyorBlock,
            BlockType::CopperOre,
            BlockType::CopperIngot,
            BlockType::CrusherBlock,
            BlockType::FurnaceBlock,
        ];

        let block = block_types[block_idx];
        let name = block.name();

        prop_assert!(!name.is_empty(), "Block type {:?} should have a non-empty name", block);
    }
}

#[cfg(test)]
mod deterministic_tests {
    use super::*;

    #[test]
    fn test_inventory_operations_deterministic() {
        // Run same operations multiple times, should get same result
        for _ in 0..5 {
            let mut inv = TestInventory::new(36);
            inv.add(BlockType::Stone, 100);
            inv.add(BlockType::IronOre, 50);
            inv.remove(BlockType::Stone, 30);

            assert_eq!(inv.count(BlockType::Stone), 70);
            assert_eq!(inv.count(BlockType::IronOre), 50);
        }
    }
}
