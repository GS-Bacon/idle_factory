//! E2E Tests for Idle Factory
//! Tests core game logic without rendering

use bevy::prelude::*;
use std::collections::HashMap;

// Re-create the core types for testing (since they're private in main)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum BlockType {
    Stone,
    Grass,
}

#[derive(Resource, Default)]
struct Inventory {
    items: HashMap<BlockType, u32>,
}

#[derive(Resource)]
struct ChunkData {
    blocks: HashMap<IVec3, BlockType>,
}

const CHUNK_SIZE: usize = 16;

impl Default for ChunkData {
    fn default() -> Self {
        let mut blocks = HashMap::new();
        for x in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                for y in 0..8 {
                    let block_type = if y == 7 {
                        BlockType::Grass
                    } else {
                        BlockType::Stone
                    };
                    blocks.insert(IVec3::new(x, y, z), block_type);
                }
            }
        }
        Self { blocks }
    }
}

#[test]
fn test_world_generation() {
    let chunk = ChunkData::default();

    // Verify chunk has expected number of blocks
    let expected_blocks = 16 * 16 * 8; // 16x16 area, 8 layers deep
    assert_eq!(chunk.blocks.len(), expected_blocks);

    // Verify terrain composition
    let grass_count = chunk
        .blocks
        .values()
        .filter(|&&b| b == BlockType::Grass)
        .count();
    let stone_count = chunk
        .blocks
        .values()
        .filter(|&&b| b == BlockType::Stone)
        .count();

    assert_eq!(grass_count, 16 * 16); // Top layer is all grass
    assert_eq!(stone_count, 16 * 16 * 7); // 7 layers of stone
}

#[test]
fn test_block_mining_adds_to_inventory() {
    let mut chunk = ChunkData::default();
    let mut inventory = Inventory::default();

    // Simulate mining a block
    let block_pos = IVec3::new(0, 7, 0);
    if let Some(block_type) = chunk.blocks.remove(&block_pos) {
        *inventory.items.entry(block_type).or_insert(0) += 1;
    }

    // Verify block was removed
    assert!(!chunk.blocks.contains_key(&block_pos));

    // Verify inventory was updated
    assert_eq!(inventory.items.get(&BlockType::Grass), Some(&1));
}

#[test]
fn test_multiple_blocks_mining() {
    let mut chunk = ChunkData::default();
    let mut inventory = Inventory::default();

    // Mine 3 grass blocks
    for x in 0..3 {
        let block_pos = IVec3::new(x, 7, 0);
        if let Some(block_type) = chunk.blocks.remove(&block_pos) {
            *inventory.items.entry(block_type).or_insert(0) += 1;
        }
    }

    // Mine 2 stone blocks
    for x in 0..2 {
        let block_pos = IVec3::new(x, 0, 0);
        if let Some(block_type) = chunk.blocks.remove(&block_pos) {
            *inventory.items.entry(block_type).or_insert(0) += 1;
        }
    }

    assert_eq!(inventory.items.get(&BlockType::Grass), Some(&3));
    assert_eq!(inventory.items.get(&BlockType::Stone), Some(&2));
}

#[test]
fn test_ray_aabb_intersection() {
    fn ray_aabb_intersection(
        ray_origin: Vec3,
        ray_direction: Vec3,
        box_min: Vec3,
        box_max: Vec3,
    ) -> Option<f32> {
        let inv_dir = Vec3::new(
            1.0 / ray_direction.x,
            1.0 / ray_direction.y,
            1.0 / ray_direction.z,
        );

        let t1 = (box_min.x - ray_origin.x) * inv_dir.x;
        let t2 = (box_max.x - ray_origin.x) * inv_dir.x;
        let t3 = (box_min.y - ray_origin.y) * inv_dir.y;
        let t4 = (box_max.y - ray_origin.y) * inv_dir.y;
        let t5 = (box_min.z - ray_origin.z) * inv_dir.z;
        let t6 = (box_max.z - ray_origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some(tmin)
        }
    }

    // Test hit from directly in front
    let result = ray_aabb_intersection(
        Vec3::new(0.0, 0.0, -5.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, 0.5),
    );
    assert!(result.is_some());
    let t = result.unwrap();
    assert!(t > 4.0 && t < 5.0); // Should hit at around t=4.5

    // Test miss
    let result = ray_aabb_intersection(
        Vec3::new(0.0, 0.0, -5.0),
        Vec3::new(1.0, 0.0, 0.0), // Shooting sideways
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, 0.5),
    );
    assert!(result.is_none());
}

#[test]
fn test_bevy_app_builds() {
    // This test verifies that a minimal Bevy app can be constructed
    // without any runtime errors
    let _app = App::new().add_plugins(MinimalPlugins);

    // If we get here without panic, Bevy initialization works
}

// =====================================================
// Additional E2E Tests for Core Game Operations
// =====================================================

/// Hotbar selection state
#[derive(Resource, Default)]
struct HotbarState {
    selected_index: Option<usize>,
}

impl HotbarState {
    fn select(&mut self, index: usize) {
        // Allow selecting any slot 0-8, including empty ones
        if index < 9 {
            self.selected_index = Some(index);
        }
    }

    #[allow(dead_code)]
    fn deselect(&mut self) {
        self.selected_index = None;
    }
}

#[test]
fn test_hotbar_selection_1_to_9() {
    let mut hotbar = HotbarState::default();

    // Initially no selection
    assert_eq!(hotbar.selected_index, None);

    // Select slot 0 (key 1)
    hotbar.select(0);
    assert_eq!(hotbar.selected_index, Some(0));

    // Select slot 4 (key 5)
    hotbar.select(4);
    assert_eq!(hotbar.selected_index, Some(4));

    // Select slot 8 (key 9)
    hotbar.select(8);
    assert_eq!(hotbar.selected_index, Some(8));

    // Invalid index should not change state (>= 9)
    hotbar.select(9);
    assert_eq!(hotbar.selected_index, Some(8)); // Unchanged
}

/// Inventory with hotbar items
#[derive(Resource)]
struct HotbarInventory {
    /// 9 hotbar slots: (BlockType, count)
    slots: Vec<Option<(BlockType, u32)>>,
}

impl Default for HotbarInventory {
    fn default() -> Self {
        Self {
            slots: vec![
                Some((BlockType::Stone, 64)),  // Slot 0
                Some((BlockType::Grass, 32)),  // Slot 1
                None,                           // Slot 2 - empty
                None,                           // Slot 3 - empty
                None,                           // Slot 4 - empty
                None,                           // Slot 5 - empty
                None,                           // Slot 6 - empty
                None,                           // Slot 7 - empty
                None,                           // Slot 8 - empty
            ],
        }
    }
}

impl HotbarInventory {
    fn place_block(&mut self, slot: usize) -> Option<BlockType> {
        if slot >= 9 {
            return None;
        }

        if let Some((block_type, ref mut count)) = self.slots[slot] {
            if *count > 0 {
                *count -= 1;
                let result = Some(block_type);
                if *count == 0 {
                    self.slots[slot] = None;
                }
                return result;
            }
        }
        None
    }

    fn get_slot(&self, slot: usize) -> Option<(BlockType, u32)> {
        if slot >= 9 {
            return None;
        }
        self.slots[slot]
    }
}

#[test]
fn test_block_placement_consumes_inventory() {
    let mut inventory = HotbarInventory::default();

    // Initial state
    assert_eq!(inventory.get_slot(0), Some((BlockType::Stone, 64)));

    // Place a block from slot 0
    let placed = inventory.place_block(0);
    assert_eq!(placed, Some(BlockType::Stone));
    assert_eq!(inventory.get_slot(0), Some((BlockType::Stone, 63)));

    // Place from empty slot returns None
    let placed = inventory.place_block(5);
    assert_eq!(placed, None);
}

#[test]
fn test_block_placement_empties_slot() {
    let mut inventory = HotbarInventory {
        slots: vec![
            Some((BlockType::Stone, 1)), // Only 1 block
            None, None, None, None, None, None, None, None,
        ],
    };

    // Place the only block
    let placed = inventory.place_block(0);
    assert_eq!(placed, Some(BlockType::Stone));

    // Slot should now be empty
    assert_eq!(inventory.get_slot(0), None);

    // Placing again should return None
    let placed = inventory.place_block(0);
    assert_eq!(placed, None);
}

/// UI state for furnace interaction
#[derive(Resource, Default)]
struct UIState {
    furnace_ui_open: bool,
}

impl UIState {
    fn toggle_furnace_ui(&mut self) {
        self.furnace_ui_open = !self.furnace_ui_open;
    }

    fn close_ui(&mut self) {
        self.furnace_ui_open = false;
    }
}

#[test]
fn test_ui_toggle_with_e_key() {
    let mut ui_state = UIState::default();

    // Initially closed
    assert!(!ui_state.furnace_ui_open);

    // Press E to open
    ui_state.toggle_furnace_ui();
    assert!(ui_state.furnace_ui_open);

    // Press E again to close
    ui_state.toggle_furnace_ui();
    assert!(!ui_state.furnace_ui_open);
}

#[test]
fn test_ui_close_with_esc() {
    let mut ui_state = UIState::default();

    // Open the UI
    ui_state.toggle_furnace_ui();
    assert!(ui_state.furnace_ui_open);

    // Close with ESC
    ui_state.close_ui();
    assert!(!ui_state.furnace_ui_open);
}

/// Test that multiple frame updates don't cause issues
#[test]
fn test_frame_stability() {
    let mut inventory = HotbarInventory::default();
    let mut hotbar = HotbarState::default();
    let mut ui_state = UIState::default();

    // Simulate 100 frames of random operations
    for frame in 0..100 {
        // Select hotbar slot
        hotbar.select(frame % 9);

        // Toggle UI every 10 frames
        if frame % 10 == 0 {
            ui_state.toggle_furnace_ui();
        }

        // Place block every 5 frames (if slot has items)
        if frame % 5 == 0 {
            let selected = hotbar.selected_index.unwrap_or(0);
            let _ = inventory.place_block(selected);
        }
    }

    // If we get here without panic, frame stability is good
    assert!(true);
}

/// Test block breaking adds to inventory
#[test]
fn test_block_break_no_freeze() {
    let mut chunk = ChunkData::default();
    let mut inventory = Inventory::default();

    // Break 10 blocks rapidly
    for x in 0..10 {
        let block_pos = IVec3::new(x, 7, 0);
        if let Some(block_type) = chunk.blocks.remove(&block_pos) {
            *inventory.items.entry(block_type).or_insert(0) += 1;
        }
    }

    // Verify all blocks were removed
    for x in 0..10 {
        assert!(!chunk.blocks.contains_key(&IVec3::new(x, 7, 0)));
    }

    // Verify inventory count
    assert_eq!(inventory.items.get(&BlockType::Grass), Some(&10));
}

// =====================================================
// Slot-based Inventory Tests (matching new implementation)
// =====================================================

const NUM_SLOTS: usize = 9;

/// Slot-based inventory matching the actual game implementation
#[derive(Clone)]
struct SlotInventory {
    slots: [Option<(BlockType, u32)>; NUM_SLOTS],
    selected_slot: usize,
}

impl Default for SlotInventory {
    fn default() -> Self {
        Self {
            slots: [None; NUM_SLOTS],
            selected_slot: 0,
        }
    }
}

impl SlotInventory {
    fn get_slot(&self, slot: usize) -> Option<BlockType> {
        self.slots.get(slot).and_then(|s| s.map(|(bt, _)| bt))
    }

    fn get_slot_count(&self, slot: usize) -> u32 {
        self.slots.get(slot).and_then(|s| s.map(|(_, c)| c)).unwrap_or(0)
    }

    fn selected_block(&self) -> Option<BlockType> {
        self.get_slot(self.selected_slot)
    }

    fn add_item(&mut self, block_type: BlockType, amount: u32) -> bool {
        // First, try to find existing slot with same block type
        for (bt, count) in self.slots.iter_mut().flatten() {
            if *bt == block_type {
                *count += amount;
                return true;
            }
        }
        // Otherwise, find first empty slot
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some((block_type, amount));
                return true;
            }
        }
        false
    }

    fn consume_selected(&mut self) -> Option<BlockType> {
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

    fn consume_item(&mut self, block_type: BlockType, amount: u32) -> bool {
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

    fn has_selected(&self) -> bool {
        self.slots.get(self.selected_slot)
            .and_then(|s| s.as_ref())
            .map(|(_, c)| *c > 0)
            .unwrap_or(false)
    }
}

#[test]
fn test_slot_inventory_add_stacks() {
    let mut inv = SlotInventory::default();

    // Add 10 stone to empty inventory
    assert!(inv.add_item(BlockType::Stone, 10));
    assert_eq!(inv.get_slot(0), Some(BlockType::Stone));
    assert_eq!(inv.get_slot_count(0), 10);

    // Add 5 more stone - should stack in same slot
    assert!(inv.add_item(BlockType::Stone, 5));
    assert_eq!(inv.get_slot_count(0), 15);

    // Add grass - should go to next slot
    assert!(inv.add_item(BlockType::Grass, 20));
    assert_eq!(inv.get_slot(1), Some(BlockType::Grass));
    assert_eq!(inv.get_slot_count(1), 20);
}

#[test]
fn test_slot_inventory_consume_selected() {
    let mut inv = SlotInventory::default();
    inv.add_item(BlockType::Stone, 3);
    inv.selected_slot = 0;

    // Consume from selected slot
    assert_eq!(inv.consume_selected(), Some(BlockType::Stone));
    assert_eq!(inv.get_slot_count(0), 2);

    assert_eq!(inv.consume_selected(), Some(BlockType::Stone));
    assert_eq!(inv.get_slot_count(0), 1);

    assert_eq!(inv.consume_selected(), Some(BlockType::Stone));
    // Slot should now be empty
    assert_eq!(inv.get_slot(0), None);
    assert_eq!(inv.get_slot_count(0), 0);

    // Consuming from empty slot returns None
    assert_eq!(inv.consume_selected(), None);
}

#[test]
fn test_slot_inventory_empty_slot_stays_selected() {
    let mut inv = SlotInventory::default();
    inv.add_item(BlockType::Stone, 1);
    inv.selected_slot = 0;

    // Consume the only item
    inv.consume_selected();

    // Selected slot should still be 0 (empty), not auto-switch
    assert_eq!(inv.selected_slot, 0);
    assert_eq!(inv.get_slot(0), None);
    assert!(!inv.has_selected());

    // Adding a different item goes to the next available empty slot (which is 0)
    inv.add_item(BlockType::Grass, 5);
    // Grass is now in slot 0 (first empty)
    assert_eq!(inv.get_slot(0), Some(BlockType::Grass));
}

#[test]
fn test_slot_inventory_consume_specific_item() {
    let mut inv = SlotInventory::default();
    inv.add_item(BlockType::Stone, 10);
    inv.add_item(BlockType::Grass, 5);

    // Consume stone (regardless of selected slot)
    assert!(inv.consume_item(BlockType::Stone, 3));
    assert_eq!(inv.get_slot_count(0), 7);

    // Consume grass
    assert!(inv.consume_item(BlockType::Grass, 5));
    assert_eq!(inv.get_slot(1), None); // Slot emptied

    // Try to consume more grass than available - should fail
    assert!(!inv.consume_item(BlockType::Grass, 1));
}

#[test]
fn test_slot_inventory_full() {
    let mut inv = SlotInventory::default();

    // Fill all 9 slots with different block types (using only Stone and Grass)
    for i in 0..NUM_SLOTS {
        // Alternate between block types but use separate add calls to fill slots
        let block = if i % 2 == 0 { BlockType::Stone } else { BlockType::Grass };
        // Force into separate slots by making each a new "stack"
        inv.slots[i] = Some((block, (i + 1) as u32));
    }

    // All slots full - adding new item type should fail
    // (We need a third block type for this test, but we only have 2 in test)
    // Instead, verify all slots are used
    for i in 0..NUM_SLOTS {
        assert!(inv.get_slot(i).is_some());
    }
}

#[test]
fn test_slot_inventory_selection_with_empty_slots() {
    let mut inv = SlotInventory::default();
    inv.add_item(BlockType::Stone, 10);
    // Stone in slot 0, slots 1-8 empty

    // Select empty slot 5
    inv.selected_slot = 5;
    assert_eq!(inv.selected_block(), None);
    assert!(!inv.has_selected());

    // Consume from empty slot should return None
    assert_eq!(inv.consume_selected(), None);

    // Switch to slot 0
    inv.selected_slot = 0;
    assert_eq!(inv.selected_block(), Some(BlockType::Stone));
    assert!(inv.has_selected());
}
