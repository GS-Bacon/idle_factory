//! Inventory and slot tests

use super::common::*;
use idle_factory::constants::HOTBAR_SLOTS;
use idle_factory::core::items;

#[test]
fn test_hotbar_selection_1_to_9() {
    let mut hotbar = HotbarState::default();

    assert_eq!(hotbar.selected_index, None);

    hotbar.select(0);
    assert_eq!(hotbar.selected_index, Some(0));

    hotbar.select(4);
    assert_eq!(hotbar.selected_index, Some(4));

    hotbar.select(8);
    assert_eq!(hotbar.selected_index, Some(8));

    hotbar.select(9);
    assert_eq!(hotbar.selected_index, Some(8)); // Unchanged
}

#[test]
fn test_block_placement_consumes_inventory() {
    let mut inventory = HotbarInventory::default();

    assert_eq!(inventory.get_slot(0), Some((items::stone(), 64)));

    let placed = inventory.place_block(0);
    assert_eq!(placed, Some(items::stone()));
    assert_eq!(inventory.get_slot(0), Some((items::stone(), 63)));

    let placed = inventory.place_block(5);
    assert_eq!(placed, None);
}

#[test]
fn test_block_placement_empties_slot() {
    let mut inventory = HotbarInventory {
        slots: vec![
            Some((items::stone(), 1)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
    };

    let placed = inventory.place_block(0);
    assert_eq!(placed, Some(items::stone()));
    assert_eq!(inventory.get_slot(0), None);

    let placed = inventory.place_block(0);
    assert_eq!(placed, None);
}

#[test]
fn test_slot_inventory_add_stacks() {
    let mut inv = SlotInventory::default();

    assert!(inv.add_item(items::stone(), 10));
    assert_eq!(inv.get_slot(0), Some(items::stone()));
    assert_eq!(inv.get_slot_count(0), 10);

    assert!(inv.add_item(items::stone(), 5));
    assert_eq!(inv.get_slot_count(0), 15);

    assert!(inv.add_item(items::grass(), 20));
    assert_eq!(inv.get_slot(1), Some(items::grass()));
    assert_eq!(inv.get_slot_count(1), 20);
}

#[test]
fn test_slot_inventory_consume_selected() {
    let mut inv = SlotInventory::default();
    inv.add_item(items::stone(), 3);
    inv.selected_slot = 0;

    assert_eq!(inv.consume_selected(), Some(items::stone()));
    assert_eq!(inv.get_slot_count(0), 2);

    assert_eq!(inv.consume_selected(), Some(items::stone()));
    assert_eq!(inv.get_slot_count(0), 1);

    assert_eq!(inv.consume_selected(), Some(items::stone()));
    assert_eq!(inv.get_slot(0), None);
    assert_eq!(inv.get_slot_count(0), 0);

    assert_eq!(inv.consume_selected(), None);
}

#[test]
fn test_slot_inventory_empty_slot_stays_selected() {
    let mut inv = SlotInventory::default();
    inv.add_item(items::stone(), 1);
    inv.selected_slot = 0;

    inv.consume_selected();

    assert_eq!(inv.selected_slot, 0);
    assert_eq!(inv.get_slot(0), None);
    assert!(!inv.has_selected());

    inv.add_item(items::grass(), 5);
    assert_eq!(inv.get_slot(0), Some(items::grass()));
}

#[test]
fn test_slot_inventory_consume_specific_item() {
    let mut inv = SlotInventory::default();
    inv.add_item(items::stone(), 10);
    inv.add_item(items::grass(), 5);

    assert!(inv.consume_item(items::stone(), 3));
    assert_eq!(inv.get_slot_count(0), 7);

    assert!(inv.consume_item(items::grass(), 5));
    assert_eq!(inv.get_slot(1), None);

    assert!(!inv.consume_item(items::grass(), 1));
}

#[test]
fn test_slot_inventory_full() {
    let mut inv = SlotInventory::default();

    for i in 0..HOTBAR_SLOTS {
        let block = if i % 2 == 0 {
            items::stone()
        } else {
            items::grass()
        };
        inv.slots[i] = Some((block, (i + 1) as u32));
    }

    // All slots full - stacking with existing types still works
    assert!(inv.add_item(items::stone(), 5));
    assert_eq!(inv.get_slot_count(0), 6); // 1 + 5
}

#[test]
fn test_frame_stability() {
    let mut inventory = HotbarInventory::default();
    let mut hotbar = HotbarState::default();
    let mut ui_state = TestUIState::default();

    for frame in 0..100 {
        hotbar.select(frame % 9);

        if frame % 10 == 0 {
            ui_state.toggle_furnace_ui();
        }

        if frame % 5 == 0 {
            let selected = hotbar.selected_index.unwrap_or(0);
            let _ = inventory.place_block(selected);
        }
    }
}
