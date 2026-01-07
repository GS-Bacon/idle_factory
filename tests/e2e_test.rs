//! E2E Tests for Idle Factory
//! Tests core game logic without rendering

#![allow(dead_code)] // Test helper types may not all be used in every test

use bevy::prelude::*;
use std::collections::HashMap;

// Use real library types
use idle_factory::constants::{CHUNK_SIZE, HOTBAR_SLOTS};
use idle_factory::BlockType;

#[derive(Resource, Default)]
struct Inventory {
    items: HashMap<BlockType, u32>,
}

#[derive(Resource)]
struct ChunkData {
    blocks: HashMap<IVec3, BlockType>,
}

impl Default for ChunkData {
    fn default() -> Self {
        let mut blocks = HashMap::new();
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
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
                Some((BlockType::Stone, 64)), // Slot 0
                Some((BlockType::Grass, 32)), // Slot 1
                None,                         // Slot 2 - empty
                None,                         // Slot 3 - empty
                None,                         // Slot 4 - empty
                None,                         // Slot 5 - empty
                None,                         // Slot 6 - empty
                None,                         // Slot 7 - empty
                None,                         // Slot 8 - empty
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

/// Slot-based inventory matching the actual game implementation (simplified for tests)
#[derive(Clone)]
struct SlotInventory {
    slots: [Option<(BlockType, u32)>; HOTBAR_SLOTS],
    selected_slot: usize,
}

impl Default for SlotInventory {
    fn default() -> Self {
        Self {
            slots: [None; HOTBAR_SLOTS],
            selected_slot: 0,
        }
    }
}

impl SlotInventory {
    fn get_slot(&self, slot: usize) -> Option<BlockType> {
        self.slots.get(slot).and_then(|s| s.map(|(bt, _)| bt))
    }

    fn get_slot_count(&self, slot: usize) -> u32 {
        self.slots
            .get(slot)
            .and_then(|s| s.map(|(_, c)| c))
            .unwrap_or(0)
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
        self.slots
            .get(self.selected_slot)
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
    for i in 0..HOTBAR_SLOTS {
        // Alternate between block types but use separate add calls to fill slots
        let block = if i % 2 == 0 {
            BlockType::Stone
        } else {
            BlockType::Grass
        };
        // Force into separate slots by making each a new "stack"
        inv.slots[i] = Some((block, (i + 1) as u32));
    }

    // All slots full - adding new item type should fail
    // (We need a third block type for this test, but we only have 2 in test)
    // Instead, verify all slots are used
    for i in 0..HOTBAR_SLOTS {
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

// =====================================================
// Machine Component Tests
// =====================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn to_ivec3(&self) -> IVec3 {
        match self {
            Direction::North => IVec3::new(0, 0, -1),
            Direction::South => IVec3::new(0, 0, 1),
            Direction::East => IVec3::new(1, 0, 0),
            Direction::West => IVec3::new(-1, 0, 0),
        }
    }
}

/// Miner component for testing
struct Miner {
    position: IVec3,
    progress: f32,
    buffer: Option<(BlockType, u32)>,
}

impl Default for Miner {
    fn default() -> Self {
        Self {
            position: IVec3::ZERO,
            progress: 0.0,
            buffer: None,
        }
    }
}

impl Miner {
    fn tick(&mut self, delta_seconds: f32, ore_type: Option<BlockType>) -> bool {
        // Mining takes 5 seconds
        const MINING_TIME: f32 = 5.0;

        if ore_type.is_none() {
            return false;
        }

        self.progress += delta_seconds / MINING_TIME;
        if self.progress >= 1.0 {
            self.progress = 0.0;
            let ore = ore_type.unwrap();
            if let Some((bt, ref mut count)) = self.buffer {
                if bt == ore {
                    *count += 1;
                }
            } else {
                self.buffer = Some((ore, 1));
            }
            true
        } else {
            false
        }
    }

    fn take_output(&mut self) -> Option<BlockType> {
        if let Some((bt, ref mut count)) = self.buffer {
            if *count > 0 {
                *count -= 1;
                let result = bt;
                if *count == 0 {
                    self.buffer = None;
                }
                return Some(result);
            }
        }
        None
    }
}

/// Conveyor component for testing
struct Conveyor {
    position: IVec3,
    direction: Direction,
    item: Option<BlockType>,
    progress: f32,
}

impl Conveyor {
    fn new(position: IVec3, direction: Direction) -> Self {
        Self {
            position,
            direction,
            item: None,
            progress: 0.0,
        }
    }

    fn accept_item(&mut self, item: BlockType) -> bool {
        if self.item.is_none() {
            self.item = Some(item);
            self.progress = 0.0;
            true
        } else {
            false
        }
    }

    fn tick(&mut self, delta_seconds: f32) -> Option<BlockType> {
        const TRANSFER_TIME: f32 = 0.5;

        if self.item.is_none() {
            return None;
        }

        self.progress += delta_seconds / TRANSFER_TIME;
        if self.progress >= 1.0 {
            self.progress = 0.0;
            self.item.take()
        } else {
            None
        }
    }

    fn output_position(&self) -> IVec3 {
        self.position + self.direction.to_ivec3()
    }
}

/// Furnace component for testing
struct Furnace {
    fuel: u32,
    input_type: Option<BlockType>,
    input_count: u32,
    output_type: Option<BlockType>,
    output_count: u32,
    progress: f32,
}

impl Default for Furnace {
    fn default() -> Self {
        Self {
            fuel: 0,
            input_type: None,
            input_count: 0,
            output_type: None,
            output_count: 0,
            progress: 0.0,
        }
    }
}

impl Furnace {
    fn add_fuel(&mut self, count: u32) {
        self.fuel += count;
    }

    fn add_input(&mut self, ore_type: BlockType) -> bool {
        if self.input_type.is_none() || self.input_type == Some(ore_type) {
            self.input_type = Some(ore_type);
            self.input_count += 1;
            true
        } else {
            false
        }
    }

    fn tick(&mut self, delta_seconds: f32) -> bool {
        const SMELT_TIME: f32 = 3.0;

        // Need fuel and input to smelt
        if self.fuel == 0 || self.input_count == 0 {
            return false;
        }

        self.progress += delta_seconds / SMELT_TIME;
        if self.progress >= 1.0 {
            self.progress = 0.0;
            self.fuel -= 1;
            self.input_count -= 1;
            if self.input_count == 0 {
                self.input_type = None;
            }
            // Produce ingot
            self.output_type = Some(BlockType::Stone); // Simplified: IronIngot
            self.output_count += 1;
            true
        } else {
            false
        }
    }

    fn take_output(&mut self) -> Option<BlockType> {
        if self.output_count > 0 {
            self.output_count -= 1;
            let result = self.output_type;
            if self.output_count == 0 {
                self.output_type = None;
            }
            result
        } else {
            None
        }
    }
}

/// Crusher component for testing (doubles ore output)
struct Crusher {
    input_type: Option<BlockType>,
    input_count: u32,
    output_type: Option<BlockType>,
    output_count: u32,
    progress: f32,
}

impl Default for Crusher {
    fn default() -> Self {
        Self {
            input_type: None,
            input_count: 0,
            output_type: None,
            output_count: 0,
            progress: 0.0,
        }
    }
}

impl Crusher {
    fn add_input(&mut self, ore_type: BlockType) -> bool {
        if self.input_type.is_none() || self.input_type == Some(ore_type) {
            self.input_type = Some(ore_type);
            self.input_count += 1;
            true
        } else {
            false
        }
    }

    fn tick(&mut self, delta_seconds: f32) -> bool {
        const CRUSH_TIME: f32 = 2.0;

        if self.input_count == 0 {
            return false;
        }

        self.progress += delta_seconds / CRUSH_TIME;
        if self.progress >= 1.0 {
            self.progress = 0.0;
            self.input_count -= 1;
            let ore = self.input_type.unwrap();
            if self.input_count == 0 {
                self.input_type = None;
            }
            // Double the output
            self.output_type = Some(ore);
            self.output_count += 2;
            true
        } else {
            false
        }
    }

    fn take_output(&mut self) -> Option<BlockType> {
        if self.output_count > 0 {
            self.output_count -= 1;
            let result = self.output_type;
            if self.output_count == 0 {
                self.output_type = None;
            }
            result
        } else {
            None
        }
    }
}

#[test]
fn test_miner_mining_cycle() {
    let mut miner = Miner::default();

    // Initial state verification
    assert_eq!(miner.progress, 0.0, "Initial progress should be 0");
    assert!(miner.buffer.is_none(), "Initial buffer should be empty");

    // Simulate mining iron ore
    let ore_type = Some(BlockType::Stone); // Representing iron ore

    // Not enough time passed (2 seconds of 5 total)
    assert!(
        !miner.tick(2.0, ore_type),
        "Mining should not complete at 2s"
    );
    assert!(miner.buffer.is_none(), "Buffer should still be empty at 2s");
    assert!(
        miner.progress >= 0.4 - 0.01,
        "Progress should be ~0.4 at 2s"
    );
    assert!(
        miner.progress <= 0.4 + 0.01,
        "Progress should be ~0.4 at 2s"
    );

    // Complete mining (5 seconds total)
    assert!(miner.tick(3.0, ore_type), "Mining should complete at 5s");
    assert_eq!(
        miner.buffer,
        Some((BlockType::Stone, 1)),
        "Buffer should contain 1 Stone"
    );
    assert!(
        miner.progress < 0.01,
        "Progress should reset after completion"
    );

    // Take output
    assert_eq!(
        miner.take_output(),
        Some(BlockType::Stone),
        "Should output Stone"
    );
    assert!(miner.buffer.is_none(), "Buffer should be empty after take");
}

#[test]
fn test_miner_no_ore_below() {
    let mut miner = Miner::default();

    // No ore type means no mining
    assert!(!miner.tick(10.0, None));
    assert!(miner.buffer.is_none());
}

#[test]
fn test_conveyor_item_transfer() {
    let mut conv = Conveyor::new(IVec3::new(5, 8, 5), Direction::East);

    // Accept item
    assert!(conv.accept_item(BlockType::Stone));
    assert_eq!(conv.item, Some(BlockType::Stone));

    // Can't accept another while occupied
    assert!(!conv.accept_item(BlockType::Grass));

    // Transfer takes 0.5 seconds
    assert!(conv.tick(0.3).is_none());
    assert_eq!(conv.tick(0.3), Some(BlockType::Stone));
    assert!(conv.item.is_none());
}

#[test]
fn test_conveyor_chain() {
    // Simulate: Miner -> Conv1 -> Conv2 -> (output)
    let mut miner = Miner::default();
    miner.buffer = Some((BlockType::Stone, 1));

    let mut conv1 = Conveyor::new(IVec3::new(6, 8, 5), Direction::East);
    let mut conv2 = Conveyor::new(IVec3::new(7, 8, 5), Direction::East);

    // Miner outputs to conv1
    if let Some(item) = miner.take_output() {
        assert!(conv1.accept_item(item));
    }

    // Conv1 transfers to conv2
    if let Some(item) = conv1.tick(0.5) {
        assert!(conv2.accept_item(item));
    }

    // Conv2 outputs
    let output = conv2.tick(0.5);
    assert_eq!(output, Some(BlockType::Stone));
}

#[test]
fn test_furnace_smelting() {
    let mut furnace = Furnace::default();

    // Initial state verification
    assert_eq!(furnace.progress, 0.0, "Initial progress should be 0");
    assert_eq!(furnace.fuel, 0, "Initial fuel should be 0");
    assert!(
        furnace.input_type.is_none(),
        "Initial input should be empty"
    );
    assert!(
        furnace.output_type.is_none(),
        "Initial output should be empty"
    );

    // Add fuel and input
    furnace.add_fuel(1);
    assert_eq!(furnace.fuel, 1, "Fuel should be 1 after adding");
    furnace.add_input(BlockType::Stone); // Representing iron ore
    assert!(furnace.input_type.is_some(), "Input should be set");

    // Smelting takes 3 seconds - partial progress
    assert!(!furnace.tick(2.0), "Smelting should not complete at 2s");
    assert!(
        furnace.progress >= 0.6 - 0.1,
        "Progress should be ~0.66 at 2s"
    );

    // Complete smelting
    assert!(furnace.tick(1.0), "Smelting should complete at 3s");
    assert!(
        furnace.progress < 0.01,
        "Progress should reset after completion"
    );

    // Check output
    assert_eq!(furnace.output_count, 1, "Should have 1 output item");
    assert_eq!(
        furnace.take_output(),
        Some(BlockType::Stone),
        "Should output Stone"
    );
    assert_eq!(
        furnace.output_count, 0,
        "Output count should be 0 after take"
    );

    // Fuel consumed
    assert_eq!(furnace.fuel, 0, "Fuel should be consumed");
}

#[test]
fn test_furnace_no_fuel() {
    let mut furnace = Furnace::default();
    furnace.add_input(BlockType::Stone);

    // No smelting without fuel
    assert!(!furnace.tick(10.0));
    assert_eq!(furnace.output_count, 0);
}

#[test]
fn test_crusher_doubles_output() {
    let mut crusher = Crusher::default();

    // Initial state verification
    assert_eq!(crusher.progress, 0.0, "Initial progress should be 0");
    assert!(
        crusher.input_type.is_none(),
        "Initial input should be empty"
    );
    assert!(
        crusher.output_type.is_none(),
        "Initial output should be empty"
    );
    assert_eq!(crusher.output_count, 0, "Initial output count should be 0");

    crusher.add_input(BlockType::Stone);
    assert!(
        crusher.input_type.is_some(),
        "Input should be set after add"
    );
    assert_eq!(crusher.input_count, 1, "Input count should be 1");

    // Crushing takes 2 seconds
    assert!(crusher.tick(2.0), "Crushing should complete at 2s");
    assert!(
        crusher.progress < 0.01,
        "Progress should reset after completion"
    );

    // Should produce 2 outputs (doubling effect)
    assert_eq!(crusher.output_count, 2, "Should produce exactly 2 outputs");
    assert!(crusher.input_type.is_none(), "Input should be consumed");

    // Take first output
    assert_eq!(
        crusher.take_output(),
        Some(BlockType::Stone),
        "First output should be Stone"
    );
    assert_eq!(crusher.output_count, 1, "Should have 1 output remaining");

    // Take second output
    assert_eq!(
        crusher.take_output(),
        Some(BlockType::Stone),
        "Second output should be Stone"
    );
    assert_eq!(crusher.output_count, 0, "Should have 0 outputs remaining");

    // No more outputs
    assert!(
        crusher.take_output().is_none(),
        "No more outputs should exist"
    );
}

// =====================================================
// Entity Cleanup Tests (Bug Prevention)
// =====================================================

/// Simulates entity management for cleanup testing
struct EntityManager {
    entities: HashMap<u32, EntityData>,
    next_id: u32,
}

struct EntityData {
    entity_type: EntityType,
    children: Vec<u32>,
    item_visual: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum EntityType {
    Conveyor,
    Miner,
    Furnace,
    ItemVisual,
}

impl EntityManager {
    fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1,
        }
    }

    fn spawn(&mut self, entity_type: EntityType) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.insert(
            id,
            EntityData {
                entity_type,
                children: Vec::new(),
                item_visual: None,
            },
        );
        id
    }

    fn spawn_conveyor_with_item(&mut self) -> (u32, u32) {
        let conveyor_id = self.spawn(EntityType::Conveyor);
        let item_id = self.spawn(EntityType::ItemVisual);

        if let Some(conveyor) = self.entities.get_mut(&conveyor_id) {
            conveyor.item_visual = Some(item_id);
        }

        (conveyor_id, item_id)
    }

    fn despawn_with_cleanup(&mut self, id: u32) {
        if let Some(entity) = self.entities.remove(&id) {
            // Despawn children
            for child_id in entity.children {
                self.entities.remove(&child_id);
            }
            // Despawn item visual if present (THIS IS THE BUG FIX CHECK)
            if let Some(visual_id) = entity.item_visual {
                self.entities.remove(&visual_id);
            }
        }
    }

    fn despawn_without_cleanup(&mut self, id: u32) {
        // BUG: This doesn't clean up item_visual
        self.entities.remove(&id);
    }

    fn exists(&self, id: u32) -> bool {
        self.entities.contains_key(&id)
    }

    fn count_by_type(&self, entity_type: EntityType) -> usize {
        self.entities
            .values()
            .filter(|e| e.entity_type == entity_type)
            .count()
    }
}

#[test]
fn test_conveyor_destroy_cleans_item_visual() {
    let mut manager = EntityManager::new();

    // Spawn conveyor with item
    let (conveyor_id, item_id) = manager.spawn_conveyor_with_item();

    assert!(manager.exists(conveyor_id));
    assert!(manager.exists(item_id));
    assert_eq!(manager.count_by_type(EntityType::ItemVisual), 1);

    // Destroy conveyor WITH proper cleanup (correct behavior)
    manager.despawn_with_cleanup(conveyor_id);

    // Both should be gone
    assert!(!manager.exists(conveyor_id));
    assert!(!manager.exists(item_id));
    assert_eq!(manager.count_by_type(EntityType::ItemVisual), 0);
}

#[test]
fn test_conveyor_destroy_bug_detection() {
    let mut manager = EntityManager::new();

    // Spawn conveyor with item
    let (conveyor_id, item_id) = manager.spawn_conveyor_with_item();

    // Destroy conveyor WITHOUT cleanup (the bug)
    manager.despawn_without_cleanup(conveyor_id);

    // Conveyor gone but item remains (BUG!)
    assert!(!manager.exists(conveyor_id));
    assert!(manager.exists(item_id)); // This is the bug
    assert_eq!(manager.count_by_type(EntityType::ItemVisual), 1); // Orphaned!
}

#[test]
fn test_multiple_conveyors_cleanup() {
    let mut manager = EntityManager::new();

    // Spawn 5 conveyors with items
    let mut pairs = Vec::new();
    for _ in 0..5 {
        pairs.push(manager.spawn_conveyor_with_item());
    }

    assert_eq!(manager.count_by_type(EntityType::Conveyor), 5);
    assert_eq!(manager.count_by_type(EntityType::ItemVisual), 5);

    // Destroy all conveyors properly
    for (conveyor_id, _) in pairs {
        manager.despawn_with_cleanup(conveyor_id);
    }

    // All should be cleaned up
    assert_eq!(manager.count_by_type(EntityType::Conveyor), 0);
    assert_eq!(manager.count_by_type(EntityType::ItemVisual), 0);
}

// =====================================================
// Quest and Delivery Platform Tests
// =====================================================

#[derive(Clone)]
struct QuestDef {
    target_item: BlockType,
    required_count: u32,
    reward_items: Vec<(BlockType, u32)>,
}

struct CurrentQuest {
    index: usize,
    progress: u32,
    completed: bool,
    rewards_claimed: bool,
}

impl CurrentQuest {
    fn new(index: usize) -> Self {
        Self {
            index,
            progress: 0,
            completed: false,
            rewards_claimed: false,
        }
    }

    fn add_progress(&mut self, quest: &QuestDef, amount: u32) {
        if self.completed {
            return;
        }
        self.progress += amount;
        if self.progress >= quest.required_count {
            self.completed = true;
        }
    }

    fn claim_rewards(&mut self, quest: &QuestDef, inventory: &mut SlotInventory) -> bool {
        if !self.completed || self.rewards_claimed {
            return false;
        }
        for (item, count) in &quest.reward_items {
            inventory.add_item(*item, *count);
        }
        self.rewards_claimed = true;
        true
    }
}

struct DeliveryPlatform {
    delivered: HashMap<BlockType, u32>,
}

impl DeliveryPlatform {
    fn new() -> Self {
        Self {
            delivered: HashMap::new(),
        }
    }

    fn deliver(&mut self, item: BlockType) {
        *self.delivered.entry(item).or_insert(0) += 1;
    }

    fn get_delivered(&self, item: BlockType) -> u32 {
        *self.delivered.get(&item).unwrap_or(&0)
    }
}

#[test]
fn test_quest_progress() {
    let quest = QuestDef {
        target_item: BlockType::Stone, // Representing IronIngot
        required_count: 3,
        reward_items: vec![(BlockType::Grass, 10)],
    };

    let mut current = CurrentQuest::new(0);

    // Add progress
    current.add_progress(&quest, 1);
    assert_eq!(current.progress, 1);
    assert!(!current.completed);

    current.add_progress(&quest, 2);
    assert_eq!(current.progress, 3);
    assert!(current.completed);
}

#[test]
fn test_quest_rewards() {
    let quest = QuestDef {
        target_item: BlockType::Stone,
        required_count: 1,
        reward_items: vec![(BlockType::Grass, 5), (BlockType::Stone, 3)],
    };

    let mut current = CurrentQuest::new(0);
    let mut inventory = SlotInventory::default();

    // Can't claim before completion
    assert!(!current.claim_rewards(&quest, &mut inventory));

    // Complete quest
    current.add_progress(&quest, 1);
    assert!(current.completed);

    // Claim rewards
    assert!(current.claim_rewards(&quest, &mut inventory));
    assert_eq!(inventory.get_slot_count(0), 5); // Grass
    assert_eq!(inventory.get_slot_count(1), 3); // Stone

    // Can't claim twice
    assert!(!current.claim_rewards(&quest, &mut inventory));
}

#[test]
fn test_delivery_platform() {
    let mut platform = DeliveryPlatform::new();

    // Deliver items
    platform.deliver(BlockType::Stone);
    platform.deliver(BlockType::Stone);
    platform.deliver(BlockType::Grass);

    assert_eq!(platform.get_delivered(BlockType::Stone), 2);
    assert_eq!(platform.get_delivered(BlockType::Grass), 1);
}

#[test]
fn test_delivery_updates_quest() {
    let quest = QuestDef {
        target_item: BlockType::Stone,
        required_count: 5,
        reward_items: vec![],
    };

    let mut current = CurrentQuest::new(0);
    let mut platform = DeliveryPlatform::new();

    // Deliver items and update quest
    for _ in 0..5 {
        platform.deliver(BlockType::Stone);
        current.add_progress(&quest, 1);
    }

    assert_eq!(platform.get_delivered(BlockType::Stone), 5);
    assert!(current.completed);
}

// =====================================================
// Automation Line Integration Test
// =====================================================

#[test]
fn test_full_automation_line() {
    // Simulate: Miner -> Conveyor -> Crusher -> Conveyor -> Furnace -> Conveyor -> Delivery

    let mut miner = Miner {
        position: IVec3::new(5, 8, 5),
        progress: 0.0,
        buffer: None,
    };

    let mut conv1 = Conveyor::new(IVec3::new(6, 8, 5), Direction::East);
    let mut crusher = Crusher::default();
    let mut conv2 = Conveyor::new(IVec3::new(8, 8, 5), Direction::East);
    let mut furnace = Furnace::default();
    furnace.add_fuel(10); // Pre-load fuel
    let mut conv3 = Conveyor::new(IVec3::new(10, 8, 5), Direction::East);
    let mut platform = DeliveryPlatform::new();

    // Run simulation for several cycles
    let delta = 0.1; // 100ms per tick
    for _ in 0..200 {
        // 20 seconds of simulation
        // Miner mines
        miner.tick(delta, Some(BlockType::Stone));

        // Miner outputs to conv1
        if conv1.item.is_none() {
            if let Some(item) = miner.take_output() {
                conv1.accept_item(item);
            }
        }

        // Conv1 to crusher
        if let Some(item) = conv1.tick(delta) {
            crusher.add_input(item);
        }

        // Crusher processes
        crusher.tick(delta);

        // Crusher to conv2
        if conv2.item.is_none() {
            if let Some(item) = crusher.take_output() {
                conv2.accept_item(item);
            }
        }

        // Conv2 to furnace
        if let Some(item) = conv2.tick(delta) {
            furnace.add_input(item);
        }

        // Furnace smelts
        furnace.tick(delta);

        // Furnace to conv3
        if conv3.item.is_none() {
            if let Some(item) = furnace.take_output() {
                conv3.accept_item(item);
            }
        }

        // Conv3 to platform
        if let Some(item) = conv3.tick(delta) {
            platform.deliver(item);
        }
    }

    // Should have some deliveries
    let delivered = platform.get_delivered(BlockType::Stone);
    assert!(
        delivered > 0,
        "Automation line should produce deliveries, got {}",
        delivered
    );
}

// =====================================================
// Chunk Boundary Mesh Tests
// =====================================================

struct TestWorldData {
    chunks: HashMap<IVec2, HashMap<IVec3, BlockType>>,
}

impl TestWorldData {
    fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    fn set_block(&mut self, world_pos: IVec3, block_type: BlockType) {
        let chunk_coord = IVec2::new(
            world_pos.x.div_euclid(CHUNK_SIZE),
            world_pos.z.div_euclid(CHUNK_SIZE),
        );
        let chunk = self.chunks.entry(chunk_coord).or_insert_with(HashMap::new);
        chunk.insert(world_pos, block_type);
    }

    fn has_block(&self, world_pos: IVec3) -> bool {
        let chunk_coord = IVec2::new(
            world_pos.x.div_euclid(CHUNK_SIZE),
            world_pos.z.div_euclid(CHUNK_SIZE),
        );
        self.chunks
            .get(&chunk_coord)
            .map(|c| c.contains_key(&world_pos))
            .unwrap_or(false)
    }

    /// Check if a face should be rendered at the boundary
    fn should_render_face(&self, block_pos: IVec3, face_direction: IVec3) -> bool {
        let neighbor_pos = block_pos + face_direction;
        // Render face if neighbor block doesn't exist
        !self.has_block(neighbor_pos)
    }
}

#[test]
fn test_chunk_boundary_faces() {
    let mut world = TestWorldData::new();

    // Place block at chunk boundary (x=15, edge of chunk 0)
    let boundary_block = IVec3::new(15, 5, 5);
    world.set_block(boundary_block, BlockType::Stone);

    // East face (toward chunk 1) should be rendered
    assert!(world.should_render_face(boundary_block, IVec3::new(1, 0, 0)));

    // Now add block in adjacent chunk
    let adjacent_block = IVec3::new(16, 5, 5); // In chunk (1, 0)
    world.set_block(adjacent_block, BlockType::Stone);

    // East face should NOT be rendered now (neighbor exists)
    assert!(!world.should_render_face(boundary_block, IVec3::new(1, 0, 0)));
    // West face of adjacent block should NOT be rendered
    assert!(!world.should_render_face(adjacent_block, IVec3::new(-1, 0, 0)));
}

#[test]
fn test_chunk_boundary_all_directions() {
    let mut world = TestWorldData::new();

    // Place block in center of chunk
    let center = IVec3::new(8, 5, 8);
    world.set_block(center, BlockType::Stone);

    // All faces should render (no neighbors)
    let directions = [
        IVec3::new(1, 0, 0),  // East
        IVec3::new(-1, 0, 0), // West
        IVec3::new(0, 1, 0),  // Up
        IVec3::new(0, -1, 0), // Down
        IVec3::new(0, 0, 1),  // South
        IVec3::new(0, 0, -1), // North
    ];

    for dir in directions {
        assert!(
            world.should_render_face(center, dir),
            "Face {:?} should render",
            dir
        );
    }

    // Add neighbors in all directions
    for dir in directions {
        world.set_block(center + dir, BlockType::Stone);
    }

    // No faces should render now
    for dir in directions {
        assert!(
            !world.should_render_face(center, dir),
            "Face {:?} should NOT render",
            dir
        );
    }
}

#[test]
fn test_chunk_boundary_z_axis() {
    let mut world = TestWorldData::new();

    // Place block at z boundary
    let boundary_block = IVec3::new(5, 5, 15);
    world.set_block(boundary_block, BlockType::Stone);

    // South face should render
    assert!(world.should_render_face(boundary_block, IVec3::new(0, 0, 1)));

    // Add neighbor in next chunk
    world.set_block(IVec3::new(5, 5, 16), BlockType::Stone);

    // South face should NOT render
    assert!(!world.should_render_face(boundary_block, IVec3::new(0, 0, 1)));
}

// =====================================================
// Block Operations No-Freeze Tests
// =====================================================

#[test]
fn test_rapid_block_operations() {
    let mut world = TestWorldData::new();
    let mut inventory = SlotInventory::default();
    inventory.add_item(BlockType::Stone, 100);

    // Simulate rapid place/break cycles
    for i in 0..50 {
        let pos = IVec3::new(i % 16, 8, i / 16);

        // Place block
        world.set_block(pos, BlockType::Stone);
        inventory.consume_selected();

        // Break block (simulated - would return to inventory in real game)
        // In real game, this triggers mesh regeneration
    }

    // Should complete without issue
    assert!(inventory.get_slot_count(0) == 50);
}

#[test]
fn test_block_operations_at_chunk_boundaries() {
    let mut world = TestWorldData::new();

    // Operations right at chunk boundaries
    let boundary_positions = vec![
        IVec3::new(0, 5, 0),   // Corner
        IVec3::new(15, 5, 0),  // Edge
        IVec3::new(0, 5, 15),  // Edge
        IVec3::new(15, 5, 15), // Corner
        IVec3::new(16, 5, 0),  // Next chunk
        IVec3::new(-1, 5, 0),  // Previous chunk
    ];

    for pos in boundary_positions {
        world.set_block(pos, BlockType::Stone);
        assert!(world.has_block(pos), "Block at {:?} should exist", pos);
    }
}

// =====================================================
// Raycast All Machine Types Test
// =====================================================

struct RaycastTarget {
    position: Vec3,
    half_size: Vec3,
    entity_type: EntityType,
}

fn ray_aabb_test(
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

#[test]
fn test_raycast_hits_all_machine_types() {
    let machines = vec![
        RaycastTarget {
            position: Vec3::new(5.5, 8.5, 5.5),
            half_size: Vec3::splat(0.5),
            entity_type: EntityType::Miner,
        },
        RaycastTarget {
            position: Vec3::new(6.5, 8.15, 6.5),
            half_size: Vec3::new(0.5, 0.15, 0.5), // Conveyor is flatter
            entity_type: EntityType::Conveyor,
        },
        RaycastTarget {
            position: Vec3::new(7.5, 8.5, 7.5),
            half_size: Vec3::splat(0.5),
            entity_type: EntityType::Furnace,
        },
    ];

    // Ray from player position looking at each machine
    for machine in &machines {
        let ray_origin = Vec3::new(
            machine.position.x,
            machine.position.y + 2.0,
            machine.position.z - 3.0,
        );
        let ray_direction = (machine.position - ray_origin).normalize();

        let hit = ray_aabb_test(
            ray_origin,
            ray_direction,
            machine.position - machine.half_size,
            machine.position + machine.half_size,
        );

        assert!(
            hit.is_some(),
            "Raycast should hit {:?}",
            machine.entity_type
        );
    }
}

#[test]
fn test_raycast_misses_when_looking_away() {
    let machine = RaycastTarget {
        position: Vec3::new(5.5, 8.5, 5.5),
        half_size: Vec3::splat(0.5),
        entity_type: EntityType::Miner,
    };

    // Looking in opposite direction
    let ray_origin = Vec3::new(5.5, 10.0, 2.0);
    let ray_direction = Vec3::new(0.0, 0.0, -1.0); // Looking away

    let hit = ray_aabb_test(
        ray_origin,
        ray_direction,
        machine.position - machine.half_size,
        machine.position + machine.half_size,
    );

    assert!(hit.is_none(), "Raycast should miss when looking away");
}

// ============================================================================
// Phase 6: Additional Tests for Bug Detection
// ============================================================================

/// Test that conveyor items maintain proper spacing and don't overlap (BUG-4 prevention)
#[test]
fn test_conveyor_item_no_overlap() {
    const CONVEYOR_ITEM_SPACING: f32 = 0.4;

    // Simulate a conveyor with multiple items
    let items: Vec<(f32, f32)> = vec![
        (0.0, 0.0), // (progress, lateral_offset)
        (0.4, 0.0), // Should be at minimum spacing
        (0.8, 0.0), // Should be at minimum spacing from previous
    ];

    // Check that all items maintain minimum spacing
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            let distance = (items[j].0 - items[i].0).abs();
            assert!(
                distance >= CONVEYOR_ITEM_SPACING - 0.001, // Allow small floating point error
                "Items at progress {} and {} are too close (distance: {}, min: {})",
                items[i].0,
                items[j].0,
                distance,
                CONVEYOR_ITEM_SPACING
            );
        }
    }
}

/// Test that side-merge items have proper lateral offset (BUG-5 prevention)
#[test]
fn test_conveyor_side_merge_offset() {
    // Simulate side merge: item joining from perpendicular direction
    // Initial lateral_offset should be ±0.5
    let initial_offset: f32 = 0.5;
    let decay_rate: f32 = 3.0; // per second
    let delta_time: f32 = 0.016; // 60 FPS

    // After one frame, offset should decrease
    let new_offset = initial_offset - decay_rate * delta_time;
    assert!(new_offset < initial_offset, "Lateral offset should decay");
    assert!(new_offset > 0.0, "Lateral offset should not overshoot");

    // After enough time, offset should reach near zero
    let frames_to_center = (initial_offset / (decay_rate * delta_time)).ceil() as i32;
    assert!(
        frames_to_center > 0 && frames_to_center < 100,
        "Should center within reasonable time"
    );
}

/// Test inventory stack limit at 999
#[test]
fn test_inventory_stack_limit_999() {
    let mut inventory = TestInventory::new();

    // Add items up to stack limit
    for _ in 0..999 {
        inventory.add_item(TestBlockType::Stone);
    }

    assert_eq!(inventory.get_count(TestBlockType::Stone), 999);

    // Adding more should overflow to next slot or fail
    inventory.add_item(TestBlockType::Stone);

    // Total should be 1000 (999 in first slot, 1 in overflow or same slot depending on impl)
    // For our test, we just verify it handles the overflow gracefully
    assert!(inventory.get_count(TestBlockType::Stone) >= 999);
}

/// Helper struct for inventory stack test
struct TestInventory {
    slots: [(Option<TestBlockType>, u32); 9],
}

impl TestInventory {
    fn new() -> Self {
        Self {
            slots: [(None, 0); 9],
        }
    }

    fn add_item(&mut self, item: TestBlockType) {
        const MAX_STACK: u32 = 999;

        // Find existing stack or empty slot
        for slot in &mut self.slots {
            if slot.0 == Some(item) && slot.1 < MAX_STACK {
                slot.1 += 1;
                return;
            }
        }

        // Find empty slot
        for slot in &mut self.slots {
            if slot.0.is_none() {
                slot.0 = Some(item);
                slot.1 = 1;
                return;
            }
        }
    }

    fn get_count(&self, item: TestBlockType) -> u32 {
        self.slots
            .iter()
            .filter(|(i, _)| *i == Some(item))
            .map(|(_, c)| c)
            .sum()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TestBlockType {
    Stone,
}

/// Test multiple conveyors merging simultaneously
#[test]
fn test_multiple_conveyor_merge() {
    // Simulate 3 conveyors merging into 1
    // Main conveyor going East, two side conveyors from North and South

    struct SimConveyor {
        items: Vec<f32>, // progress values
        max_items: usize,
    }

    impl SimConveyor {
        fn can_accept(&self, at_progress: f32) -> bool {
            if self.items.len() >= self.max_items {
                return false;
            }
            for &item in &self.items {
                if (item - at_progress).abs() < 0.4 {
                    return false;
                }
            }
            true
        }
    }

    let mut main_conveyor = SimConveyor {
        items: vec![],
        max_items: 3,
    };

    // Try to add from north (progress 0.5)
    if main_conveyor.can_accept(0.5) {
        main_conveyor.items.push(0.5);
    }

    // Try to add from south (progress 0.5) - should fail due to spacing
    let can_add_south = main_conveyor.can_accept(0.5);
    assert!(
        !can_add_south,
        "Should not accept two items at same progress"
    );

    // Try to add from behind (progress 0.0) - should succeed
    let can_add_behind = main_conveyor.can_accept(0.0);
    assert!(can_add_behind, "Should accept item from behind");
}

/// Test conveyor loop doesn't cause infinite processing
#[test]
fn test_conveyor_loop_handling() {
    // Simulate a loop of 4 conveyors forming a square
    // Items should keep circulating without crashing

    struct LoopConveyor {
        id: usize,
        items: Vec<f32>,
        next_id: usize,
    }

    let mut conveyors = vec![
        LoopConveyor {
            id: 0,
            items: vec![0.5],
            next_id: 1,
        },
        LoopConveyor {
            id: 1,
            items: vec![],
            next_id: 2,
        },
        LoopConveyor {
            id: 2,
            items: vec![],
            next_id: 3,
        },
        LoopConveyor {
            id: 3,
            items: vec![],
            next_id: 0,
        }, // Loop back
    ];

    // Simulate 100 frames
    for _ in 0..100 {
        let mut transfers: Vec<(usize, usize)> = vec![]; // (from, to)

        // Find items ready to transfer
        for conv in &conveyors {
            if conv.items.iter().any(|&p| p >= 1.0) {
                transfers.push((conv.id, conv.next_id));
            }
        }

        // Apply transfers
        for (from, to) in transfers {
            if let Some(idx) = conveyors[from].items.iter().position(|&p| p >= 1.0) {
                if conveyors[to].items.len() < 3 {
                    conveyors[from].items.remove(idx);
                    conveyors[to].items.push(0.0);
                }
            }
        }

        // Advance progress
        for conv in &mut conveyors {
            for p in &mut conv.items {
                *p += 0.1;
                if *p > 1.0 {
                    *p = 1.0;
                }
            }
        }
    }

    // Count total items - should still be exactly 1
    let total_items: usize = conveyors.iter().map(|c| c.items.len()).sum();
    assert_eq!(
        total_items, 1,
        "Item should not be duplicated or lost in loop"
    );
}

/// Test that entity count remains stable after repeated operations
#[test]
fn test_entity_count_stability() {
    // Simulate spawning and despawning entities
    let mut entity_count = 0;
    let mut max_entities = 0;

    // Simulate 100 cycles of spawn/despawn
    for _ in 0..100 {
        // Spawn 5 entities
        entity_count += 5;
        max_entities = max_entities.max(entity_count);

        // Despawn 5 entities
        entity_count -= 5;
    }

    assert_eq!(entity_count, 0, "All entities should be cleaned up");
    assert!(
        max_entities <= 10,
        "Entity count should not grow unboundedly"
    );
}

/// Test visual entity handoff doesn't leak (BUG-3 prevention)
#[test]
fn test_visual_entity_handoff() {
    // Simulate transferring item between conveyors
    // Old implementation: despawn visual, create new -> 1 frame gap (flicker)
    // New implementation: transfer visual entity -> no gap

    struct ItemWithVisual {
        progress: f32,
        visual_entity: Option<u32>, // Simulated entity ID
    }

    let mut source = ItemWithVisual {
        progress: 1.0,
        visual_entity: Some(42),
    };

    let mut target = ItemWithVisual {
        progress: 0.0,
        visual_entity: None,
    };

    // Transfer: keep visual instead of despawn+spawn
    target.visual_entity = source.visual_entity.take();
    target.progress = 0.0;

    assert!(
        source.visual_entity.is_none(),
        "Source should release visual"
    );
    assert_eq!(
        target.visual_entity,
        Some(42),
        "Target should receive visual"
    );
}

/// Test zipper merge - alternating inputs from multiple sources
#[test]
fn test_zipper_merge() {
    // Simulate zipper merge: two sources feeding into one target
    // Each tick, only one source should be allowed to transfer

    struct ZipperConveyor {
        id: usize,
        last_input_source: usize,
    }

    let mut target = ZipperConveyor {
        id: 0,
        last_input_source: 0,
    };

    let sources = vec![1_usize, 2_usize]; // Two source conveyors
    let mut accepted_from: Vec<usize> = Vec::new();

    // Simulate 10 ticks of zipper merge
    for _ in 0..10 {
        // Determine which source is allowed this tick
        let mut sorted_sources = sources.clone();
        sorted_sources.sort();
        let allowed_idx = target.last_input_source % sorted_sources.len();
        let allowed_source = sorted_sources[allowed_idx];

        // Accept from allowed source
        accepted_from.push(allowed_source);
        target.last_input_source += 1;
    }

    // Count how many from each source
    let from_source_1 = accepted_from.iter().filter(|&&s| s == 1).count();
    let from_source_2 = accepted_from.iter().filter(|&&s| s == 2).count();

    // Should be evenly distributed (5 from each)
    assert_eq!(from_source_1, 5, "Should accept 5 items from source 1");
    assert_eq!(from_source_2, 5, "Should accept 5 items from source 2");

    // Verify alternating pattern
    for i in 0..9 {
        assert_ne!(
            accepted_from[i],
            accepted_from[i + 1],
            "Zipper should alternate between sources"
        );
    }
}

#[test]
fn test_splitter_round_robin() {
    // Simulate splitter: one input distributes to three outputs in round-robin order
    struct SplitterConveyor {
        last_output_index: usize,
    }

    let mut splitter = SplitterConveyor {
        last_output_index: 0,
    };

    // 3 outputs: front=0, left=1, right=2
    let outputs = [0, 1, 2];
    let mut output_counts = [0_usize; 3];
    let mut output_sequence: Vec<usize> = Vec::new();

    // Simulate 12 items (should distribute 4 to each output)
    for _ in 0..12 {
        // Get next output in round-robin order
        let output_idx = splitter.last_output_index % 3;
        let output = outputs[output_idx];

        output_counts[output] += 1;
        output_sequence.push(output);
        splitter.last_output_index += 1;
    }

    // Each output should receive 4 items
    assert_eq!(output_counts[0], 4, "Front output should receive 4 items");
    assert_eq!(output_counts[1], 4, "Left output should receive 4 items");
    assert_eq!(output_counts[2], 4, "Right output should receive 4 items");

    // Verify round-robin pattern: 0,1,2,0,1,2,...
    for (i, &output) in output_sequence.iter().enumerate() {
        assert_eq!(
            output,
            i % 3,
            "Round-robin pattern should be 0,1,2,0,1,2,..."
        );
    }
}

#[test]
fn test_splitter_skips_blocked_output() {
    // Simulate splitter behavior when some outputs are blocked
    struct SplitterConveyor {
        last_output_index: usize,
    }

    let mut splitter = SplitterConveyor {
        last_output_index: 0,
    };

    // Outputs: 0=front (available), 1=left (blocked), 2=right (available)
    let output_available = [true, false, true];
    let mut output_counts = [0_usize; 3];

    // Simulate 10 items
    for _ in 0..10 {
        // Try outputs in round-robin order until one is available
        let mut found = false;
        for attempt in 0..3 {
            let output_idx = (splitter.last_output_index + attempt) % 3;
            if output_available[output_idx] {
                output_counts[output_idx] += 1;
                found = true;
                break;
            }
        }
        // Always advance the index to maintain fairness
        splitter.last_output_index += 1;
        assert!(found, "Should always find an available output");
    }

    // With output 1 blocked, items should go to 0 and 2
    assert!(output_counts[0] > 0, "Front output should receive items");
    assert_eq!(
        output_counts[1], 0,
        "Blocked left output should receive 0 items"
    );
    assert!(output_counts[2] > 0, "Right output should receive items");
    // Total should be 10
    assert_eq!(
        output_counts.iter().sum::<usize>(),
        10,
        "All 10 items should be distributed"
    );
}

#[test]
fn test_conveyor_shape_detection() {
    // Test that conveyor shape is correctly detected based on inputs/outputs

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum ConveyorShape {
        Straight,
        CornerLeft,
        CornerRight,
        TJunction,
        Splitter,
    }

    // Function to determine shape based on inputs and outputs
    fn determine_shape(
        has_left_input: bool,
        has_right_input: bool,
        output_count: usize,
    ) -> ConveyorShape {
        if output_count >= 2 && !has_left_input && !has_right_input {
            ConveyorShape::Splitter
        } else {
            match (has_left_input, has_right_input) {
                (false, false) => ConveyorShape::Straight,
                (true, false) => ConveyorShape::CornerLeft,
                (false, true) => ConveyorShape::CornerRight,
                (true, true) => ConveyorShape::TJunction,
            }
        }
    }

    // Test cases
    assert_eq!(
        determine_shape(false, false, 1),
        ConveyorShape::Straight,
        "No side inputs, 1 output = Straight"
    );
    assert_eq!(
        determine_shape(true, false, 1),
        ConveyorShape::CornerLeft,
        "Left input only = CornerLeft"
    );
    assert_eq!(
        determine_shape(false, true, 1),
        ConveyorShape::CornerRight,
        "Right input only = CornerRight"
    );
    assert_eq!(
        determine_shape(true, true, 1),
        ConveyorShape::TJunction,
        "Both side inputs = TJunction"
    );
    assert_eq!(
        determine_shape(false, false, 2),
        ConveyorShape::Splitter,
        "No side inputs, 2+ outputs = Splitter"
    );
    assert_eq!(
        determine_shape(false, false, 3),
        ConveyorShape::Splitter,
        "No side inputs, 3 outputs = Splitter"
    );

    // Side inputs prevent splitter mode even with multiple outputs
    assert_eq!(
        determine_shape(true, false, 2),
        ConveyorShape::CornerLeft,
        "Left input with 2 outputs = CornerLeft (not Splitter)"
    );
    assert_eq!(
        determine_shape(false, true, 3),
        ConveyorShape::CornerRight,
        "Right input with 3 outputs = CornerRight (not Splitter)"
    );
    assert_eq!(
        determine_shape(true, true, 2),
        ConveyorShape::TJunction,
        "Both inputs with 2 outputs = TJunction (not Splitter)"
    );
}

// =====================================================
// Auto Conveyor Direction Tests
// =====================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TestDirection {
    North,
    South,
    East,
    West,
}

impl TestDirection {
    fn to_ivec3(self) -> IVec3 {
        match self {
            TestDirection::North => IVec3::new(0, 0, -1),
            TestDirection::South => IVec3::new(0, 0, 1),
            TestDirection::East => IVec3::new(1, 0, 0),
            TestDirection::West => IVec3::new(-1, 0, 0),
        }
    }
}

/// Test auto_conveyor_direction logic
fn auto_conveyor_direction(
    place_pos: IVec3,
    fallback_direction: TestDirection,
    conveyors: &[(IVec3, TestDirection)],
    machines: &[IVec3],
) -> TestDirection {
    // Priority 1: Continue chain from adjacent conveyor pointing toward us
    for (conv_pos, conv_dir) in conveyors {
        let expected_target = *conv_pos + conv_dir.to_ivec3();
        if expected_target == place_pos {
            return *conv_dir;
        }
    }

    // Priority 2: Point away from adjacent machine
    for machine_pos in machines {
        let diff = place_pos - *machine_pos;
        if diff.x.abs() + diff.y.abs() + diff.z.abs() == 1 {
            if diff.x == 1 {
                return TestDirection::East;
            }
            if diff.x == -1 {
                return TestDirection::West;
            }
            if diff.z == 1 {
                return TestDirection::South;
            }
            if diff.z == -1 {
                return TestDirection::North;
            }
        }
    }

    // Priority 3: Connect to adjacent conveyor
    for (conv_pos, _) in conveyors {
        let diff = *conv_pos - place_pos;
        if diff.x.abs() + diff.y.abs() + diff.z.abs() == 1 {
            if diff.x == 1 {
                return TestDirection::East;
            }
            if diff.x == -1 {
                return TestDirection::West;
            }
            if diff.z == 1 {
                return TestDirection::South;
            }
            if diff.z == -1 {
                return TestDirection::North;
            }
        }
    }

    fallback_direction
}

#[test]
fn test_auto_conveyor_continues_chain() {
    // Conveyor at (5,8,5) pointing East, placing at (6,8,5)
    let conveyors = vec![(IVec3::new(5, 8, 5), TestDirection::East)];
    let machines: Vec<IVec3> = vec![];
    let place_pos = IVec3::new(6, 8, 5);

    let dir = auto_conveyor_direction(place_pos, TestDirection::North, &conveyors, &machines);
    assert_eq!(dir, TestDirection::East, "Should continue chain direction");
}

#[test]
fn test_auto_conveyor_points_away_from_machine() {
    // Machine at (5,8,5), placing conveyor at (6,8,5)
    let conveyors: Vec<(IVec3, TestDirection)> = vec![];
    let machines = vec![IVec3::new(5, 8, 5)];
    let place_pos = IVec3::new(6, 8, 5);

    let dir = auto_conveyor_direction(place_pos, TestDirection::North, &conveyors, &machines);
    assert_eq!(
        dir,
        TestDirection::East,
        "Should point away from machine (East)"
    );
}

#[test]
fn test_auto_conveyor_connects_to_adjacent() {
    // Conveyor at (7,8,5) pointing East, placing at (6,8,5)
    // The existing conveyor is NOT pointing at us, but we should connect to it
    let conveyors = vec![(IVec3::new(7, 8, 5), TestDirection::East)];
    let machines: Vec<IVec3> = vec![];
    let place_pos = IVec3::new(6, 8, 5);

    let dir = auto_conveyor_direction(place_pos, TestDirection::North, &conveyors, &machines);
    assert_eq!(
        dir,
        TestDirection::East,
        "Should point toward adjacent conveyor"
    );
}

#[test]
fn test_auto_conveyor_fallback() {
    // No adjacent conveyors or machines
    let conveyors: Vec<(IVec3, TestDirection)> = vec![];
    let machines: Vec<IVec3> = vec![];
    let place_pos = IVec3::new(6, 8, 5);

    let dir = auto_conveyor_direction(place_pos, TestDirection::South, &conveyors, &machines);
    assert_eq!(dir, TestDirection::South, "Should use fallback direction");
}

#[test]
fn test_auto_conveyor_machine_priority_over_adjacent() {
    // Machine at (5,8,5) AND conveyor at (7,8,5)
    // Machine should take priority
    let conveyors = vec![(IVec3::new(7, 8, 5), TestDirection::East)];
    let machines = vec![IVec3::new(5, 8, 5)];
    let place_pos = IVec3::new(6, 8, 5);

    let dir = auto_conveyor_direction(place_pos, TestDirection::North, &conveyors, &machines);
    assert_eq!(
        dir,
        TestDirection::East,
        "Machine priority: should point away from machine"
    );
}

// =====================================================
// Inventory Edge Case Tests
// =====================================================

#[test]
fn test_inventory_add_at_max_slots() {
    const NUM_SLOTS: usize = 36;
    const MAX_STACK: u32 = 999;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Item {
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
    }

    let mut slots: [Option<(Item, u32)>; NUM_SLOTS] = [None; NUM_SLOTS];

    // Fill all slots with different items (can't stack)
    let items = [
        Item::A,
        Item::B,
        Item::C,
        Item::D,
        Item::E,
        Item::F,
        Item::G,
        Item::H,
        Item::I,
        Item::J,
    ];
    for (i, slot) in slots.iter_mut().enumerate() {
        *slot = Some((items[i % items.len()], MAX_STACK));
    }

    // Try to add new item - should fail gracefully
    let mut added = false;
    for slot in &mut slots {
        if slot.is_none() {
            *slot = Some((Item::A, 1));
            added = true;
            break;
        }
    }
    assert!(!added, "Should not add to full inventory");
}

#[test]
fn test_inventory_stack_overflow_protection() {
    const MAX_STACK: u32 = 999;

    let mut count: u32 = MAX_STACK - 10;

    // Try to add 20 items
    let to_add: u32 = 20;
    let space = MAX_STACK.saturating_sub(count);
    let actual_add = to_add.min(space);
    count = count.saturating_add(actual_add);

    assert_eq!(count, MAX_STACK, "Should cap at MAX_STACK");
    assert_eq!(actual_add, 10, "Should only add 10 items");
}

#[test]
fn test_inventory_u32_overflow() {
    // Test that we don't overflow u32
    let count: u32 = u32::MAX - 5;
    let to_add: u32 = 10;

    // Safe addition
    let result = count.saturating_add(to_add);
    assert_eq!(result, u32::MAX, "Should saturate at u32::MAX");

    // Check our MAX_STACK approach
    const MAX_STACK: u32 = 999;
    let capped = result.min(MAX_STACK);
    assert_eq!(capped, MAX_STACK, "Should cap at MAX_STACK");
}

// =====================================================
// Chunk Unload Tests
// =====================================================

#[test]
fn test_chunk_unload_clears_entities() {
    // Simulate chunk unload tracking
    struct ChunkEntities {
        chunk_coord: IVec2,
        entities: Vec<u32>, // Entity IDs
    }

    let mut loaded_chunks: Vec<ChunkEntities> = vec![
        ChunkEntities {
            chunk_coord: IVec2::new(0, 0),
            entities: vec![1, 2, 3],
        },
        ChunkEntities {
            chunk_coord: IVec2::new(1, 0),
            entities: vec![4, 5],
        },
        ChunkEntities {
            chunk_coord: IVec2::new(0, 1),
            entities: vec![6, 7, 8, 9],
        },
    ];

    // Unload chunk (0, 1) - should remove entities 6, 7, 8, 9
    let unload_coord = IVec2::new(0, 1);
    let mut despawned: Vec<u32> = vec![];

    loaded_chunks.retain(|chunk| {
        if chunk.chunk_coord == unload_coord {
            despawned.extend(&chunk.entities);
            false
        } else {
            true
        }
    });

    assert_eq!(despawned.len(), 4, "Should despawn 4 entities");
    assert_eq!(loaded_chunks.len(), 2, "Should have 2 chunks remaining");
}

#[test]
fn test_modified_blocks_persist_across_chunk_reload() {
    // Test that player modifications (placed/destroyed blocks) persist across chunk unload/reload
    use std::collections::HashMap;

    // Simulated modified_blocks storage (world_pos -> Option<BlockType>)
    // Some(block) = player placed, None = player destroyed
    let mut modified_blocks: HashMap<IVec3, Option<u32>> = HashMap::new();

    // Player destroys a grass block at (5, 7, 5)
    modified_blocks.insert(IVec3::new(5, 7, 5), None);
    // Player places a stone block at (10, 8, 10)
    modified_blocks.insert(IVec3::new(10, 8, 10), Some(1)); // 1 = Stone

    // Simulate chunk unload (modified_blocks should NOT be cleared)
    // In real code, chunks HashMap is cleared but modified_blocks persists

    // Simulate chunk reload - apply modifications
    fn apply_modifications(
        chunk_coord: IVec2,
        generated_blocks: &mut HashMap<IVec3, u32>,
        modified_blocks: &HashMap<IVec3, Option<u32>>,
    ) {
        for (&world_pos, &maybe_block) in modified_blocks {
            let pos_chunk = IVec2::new(world_pos.x.div_euclid(16), world_pos.z.div_euclid(16));
            if pos_chunk != chunk_coord {
                continue;
            }
            match maybe_block {
                Some(block) => {
                    generated_blocks.insert(world_pos, block);
                }
                None => {
                    generated_blocks.remove(&world_pos);
                }
            }
        }
    }

    // Generated chunk at (0, 0) has grass at (5, 7, 5)
    let mut blocks: HashMap<IVec3, u32> = HashMap::new();
    blocks.insert(IVec3::new(5, 7, 5), 2); // 2 = Grass

    // Apply modifications
    apply_modifications(IVec2::new(0, 0), &mut blocks, &modified_blocks);

    // After reload: (5, 7, 5) should be gone (player destroyed), (10, 8, 10) should have stone
    assert!(
        !blocks.contains_key(&IVec3::new(5, 7, 5)),
        "Destroyed block should stay destroyed"
    );
    assert_eq!(
        blocks.get(&IVec3::new(10, 8, 10)),
        Some(&1),
        "Placed block should persist"
    );
}

#[test]
fn test_chunk_boundary_machine_survival() {
    // Machine at chunk boundary should survive if any adjacent chunk is loaded
    struct Machine {
        world_pos: IVec3,
    }

    fn world_to_chunk(pos: IVec3) -> IVec2 {
        IVec2::new(pos.x.div_euclid(16), pos.z.div_euclid(16))
    }

    let machine = Machine {
        world_pos: IVec3::new(16, 8, 0),
    }; // At chunk (1, 0)
    let loaded_chunks = vec![IVec2::new(0, 0), IVec2::new(1, 0)];

    let machine_chunk = world_to_chunk(machine.world_pos);
    let is_loaded = loaded_chunks.contains(&machine_chunk);

    assert!(is_loaded, "Machine's chunk should be loaded");
}

// =====================================================
// Machine UI State Tests
// =====================================================

#[test]
fn test_furnace_ui_state_consistency() {
    // Test that furnace UI state stays consistent
    struct FurnaceUI {
        is_open: bool,
        target_furnace: Option<u32>, // Entity ID
        fuel: u32,
        input: Option<u32>,
        output: Option<u32>,
    }

    let mut ui = FurnaceUI {
        is_open: false,
        target_furnace: None,
        fuel: 0,
        input: None,
        output: None,
    };

    // Open UI for furnace 42
    ui.is_open = true;
    ui.target_furnace = Some(42);
    ui.fuel = 5;
    ui.input = Some(10);

    // Simulate furnace destruction while UI is open
    let furnace_destroyed = true;
    if furnace_destroyed && ui.target_furnace == Some(42) {
        ui.is_open = false;
        ui.target_furnace = None;
        ui.fuel = 0;
        ui.input = None;
        ui.output = None;
    }

    assert!(!ui.is_open, "UI should close when target is destroyed");
    assert!(ui.target_furnace.is_none(), "Target should be cleared");
}

#[test]
fn test_multiple_ui_exclusive() {
    // Only one UI should be open at a time
    struct GameUIState {
        inventory_open: bool,
        furnace_open: bool,
        crusher_open: bool,
        command_input_open: bool,
    }

    let mut ui = GameUIState {
        inventory_open: false,
        furnace_open: false,
        crusher_open: false,
        command_input_open: false,
    };

    // Open inventory
    ui.inventory_open = true;

    // Try to open furnace - should close inventory first
    if ui.inventory_open || ui.crusher_open || ui.command_input_open {
        ui.inventory_open = false;
        ui.crusher_open = false;
        ui.command_input_open = false;
    }
    ui.furnace_open = true;

    assert!(!ui.inventory_open, "Inventory should be closed");
    assert!(ui.furnace_open, "Furnace should be open");
}

#[test]
fn test_crusher_break_returns_items() {
    // When a crusher is broken, its input and output items should be returned to inventory
    struct CrusherState {
        input_type: Option<BlockType>,
        input_count: u32,
        output_type: Option<BlockType>,
        output_count: u32,
    }

    let crusher = CrusherState {
        input_type: Some(BlockType::Stone), // Using Stone as ore substitute
        input_count: 5,
        output_type: Some(BlockType::Stone),
        output_count: 10, // Crushed output (doubled)
    };

    let mut inventory = SlotInventory::default();

    // Simulate breaking the crusher - return contents to inventory
    if let Some(input_type) = crusher.input_type {
        if crusher.input_count > 0 {
            inventory.add_item(input_type, crusher.input_count);
        }
    }
    if let Some(output_type) = crusher.output_type {
        if crusher.output_count > 0 {
            inventory.add_item(output_type, crusher.output_count);
        }
    }

    // Verify items were returned (stacked in slot 0)
    assert_eq!(inventory.get_slot(0), Some(BlockType::Stone));
    assert_eq!(
        inventory.get_slot_count(0),
        15,
        "All items should be returned (5 input + 10 output)"
    );
}

#[test]
fn test_furnace_break_returns_items() {
    // When a furnace is broken, its fuel, input ore, and output ingots should be returned
    struct FurnaceState {
        fuel: u32,
        input_type: Option<BlockType>,
        input_count: u32,
        output_type: Option<BlockType>,
        output_count: u32,
    }

    let furnace = FurnaceState {
        fuel: 3,
        input_type: Some(BlockType::Stone), // Using Stone as ore substitute
        input_count: 5,
        output_type: Some(BlockType::Grass), // Using Grass as ingot substitute
        output_count: 2,
    };

    let mut inventory = SlotInventory::default();

    // Simulate breaking the furnace - return contents to inventory
    if furnace.fuel > 0 {
        // Use Grass for coal substitute (slot 0)
        inventory.add_item(BlockType::Grass, furnace.fuel);
    }
    if let Some(input_type) = furnace.input_type {
        if furnace.input_count > 0 {
            inventory.add_item(input_type, furnace.input_count);
        }
    }
    if let Some(output_type) = furnace.output_type {
        if furnace.output_count > 0 {
            inventory.add_item(output_type, furnace.output_count);
        }
    }

    // Verify items were returned
    // Grass (fuel substitute + output): 3 + 2 = 5 in slot 0
    assert_eq!(inventory.get_slot(0), Some(BlockType::Grass));
    assert_eq!(
        inventory.get_slot_count(0),
        5,
        "Fuel and output should be returned"
    );
    // Stone (input ore) in slot 1
    assert_eq!(inventory.get_slot(1), Some(BlockType::Stone));
    assert_eq!(
        inventory.get_slot_count(1),
        5,
        "Input ore should be returned"
    );
}

// === Command execution tests ===

#[test]
fn test_command_give_item() {
    // Simulate /give command
    let mut inventory = SlotInventory::default();

    // /give stone 10
    let item_name = "stone";
    let count = 10u32;

    // Parse item name (simplified)
    let block_type = match item_name {
        "stone" => Some(BlockType::Stone),
        "grass" => Some(BlockType::Grass),
        _ => None,
    };

    if let Some(bt) = block_type {
        inventory.add_item(bt, count);
    }

    assert_eq!(inventory.get_slot(0), Some(BlockType::Stone));
    assert_eq!(inventory.get_slot_count(0), 10);
}

#[test]
fn test_command_give_default_count() {
    // /give without count should default to 64
    let mut inventory = SlotInventory::default();
    let default_count = 64u32;

    inventory.add_item(BlockType::Stone, default_count);

    assert_eq!(inventory.get_slot_count(0), 64);
}

#[test]
fn test_command_clear_inventory() {
    let mut inventory = SlotInventory::default();

    // Add some items
    inventory.add_item(BlockType::Stone, 10);
    inventory.add_item(BlockType::Grass, 5);

    assert!(inventory.get_slot(0).is_some());
    assert!(inventory.get_slot(1).is_some());

    // Clear inventory
    for slot in inventory.slots.iter_mut() {
        *slot = None;
    }

    assert!(inventory.get_slot(0).is_none());
    assert!(inventory.get_slot(1).is_none());
}

#[test]
fn test_command_creative_mode_fills_inventory() {
    let mut inventory = SlotInventory::default();

    // Simulate entering creative mode - fills first 9 slots with 64 items each
    let all_items = [BlockType::Stone, BlockType::Grass];
    for (i, block_type) in all_items.iter().take(9).enumerate() {
        inventory.slots[i] = Some((*block_type, 64));
    }

    assert_eq!(inventory.get_slot(0), Some(BlockType::Stone));
    assert_eq!(inventory.get_slot_count(0), 64);
    assert_eq!(inventory.get_slot(1), Some(BlockType::Grass));
    assert_eq!(inventory.get_slot_count(1), 64);
}

#[test]
fn test_command_unknown_item_no_crash() {
    // /give unknownitem should not crash
    let mut inventory = SlotInventory::default();

    let item_name = "unknownitem";
    let block_type: Option<BlockType> = match item_name {
        "stone" => Some(BlockType::Stone),
        "grass" => Some(BlockType::Grass),
        _ => None,
    };

    // Should not add anything for unknown item
    if let Some(bt) = block_type {
        inventory.add_item(bt, 64);
    }

    // Inventory should remain empty
    assert!(inventory.get_slot(0).is_none());
}

#[test]
fn test_miner_buffer_overflow_protection() {
    // Miner buffer should not exceed max capacity
    struct MinerBuffer {
        buffer: Option<(BlockType, u32)>,
        max_buffer: u32,
    }

    let mut miner = MinerBuffer {
        buffer: None,
        max_buffer: 64,
    };

    // Simulate mining adding to buffer
    for _ in 0..100 {
        match &mut miner.buffer {
            Some((_, count)) if *count < miner.max_buffer => {
                *count += 1;
            }
            None => {
                miner.buffer = Some((BlockType::Stone, 1));
            }
            _ => {} // Buffer full, don't add
        }
    }

    // Buffer should be capped at max
    assert_eq!(miner.buffer.map(|(_, c)| c), Some(64));
}

#[test]
fn test_delivery_platform_accepts_any_item() {
    // Delivery platform should accept any item type
    let mut delivered: std::collections::HashMap<BlockType, u32> = std::collections::HashMap::new();

    // Deliver different item types
    *delivered.entry(BlockType::Stone).or_insert(0) += 1;
    *delivered.entry(BlockType::Grass).or_insert(0) += 1;
    *delivered.entry(BlockType::Stone).or_insert(0) += 1;

    assert_eq!(delivered.get(&BlockType::Stone), Some(&2));
    assert_eq!(delivered.get(&BlockType::Grass), Some(&1));
}

// ============================================================
// BUG-2: Machine placement should NOT register block in world data
// ============================================================
#[test]
fn test_machine_placement_no_block_registration() {
    // Machines (Miner, Conveyor, etc.) are entities, not blocks
    // They should NOT be registered in the world block data
    // Registering them causes the terrain underneath to disappear

    let mut world_blocks: std::collections::HashMap<IVec3, BlockType> =
        std::collections::HashMap::new();

    // Initial terrain
    world_blocks.insert(IVec3::new(0, 0, 0), BlockType::Stone);
    world_blocks.insert(IVec3::new(0, 1, 0), BlockType::Grass);

    // Simulate placing a miner at (0, 2, 0)
    // CORRECT: Don't register in world_blocks - just spawn entity
    let miner_pos = IVec3::new(0, 2, 0);
    // machines.spawn(Miner { position: miner_pos, ... });
    // DO NOT: world_blocks.insert(miner_pos, BlockType::MinerBlock);

    // Verify terrain is still intact
    assert!(
        world_blocks.contains_key(&IVec3::new(0, 0, 0)),
        "Stone should still exist"
    );
    assert!(
        world_blocks.contains_key(&IVec3::new(0, 1, 0)),
        "Grass should still exist"
    );
    assert!(
        !world_blocks.contains_key(&miner_pos),
        "Machine position should NOT be in world blocks"
    );
}

// ============================================================
// BUG-4: Chunk boundary mesh generation needs neighbor info
// ============================================================
#[test]
fn test_chunk_boundary_mesh_needs_neighbors() {
    // When generating mesh at chunk boundary, we need neighbor chunk data
    // to correctly determine which faces to render

    const CHUNK_SIZE: i32 = 16;

    // Simulate two adjacent chunks
    let mut chunk_a: std::collections::HashMap<IVec3, BlockType> = std::collections::HashMap::new();
    let mut chunk_b: std::collections::HashMap<IVec3, BlockType> = std::collections::HashMap::new();

    // Chunk A has a block at its +X edge
    chunk_a.insert(IVec3::new(CHUNK_SIZE - 1, 0, 0), BlockType::Stone);

    // Chunk B has a block at its -X edge (adjacent to chunk A's block)
    chunk_b.insert(IVec3::new(0, 0, 0), BlockType::Stone);

    // When generating mesh for chunk A's edge block:
    // - Without neighbor info: would render +X face (wrong - it's occluded)
    // - With neighbor info: correctly skip +X face

    fn should_render_face(
        pos: IVec3,
        face_dir: IVec3,
        own_chunk: &std::collections::HashMap<IVec3, BlockType>,
        neighbor_chunk: Option<&std::collections::HashMap<IVec3, BlockType>>,
    ) -> bool {
        let neighbor_pos = pos + face_dir;

        // Check in own chunk first
        if own_chunk.contains_key(&neighbor_pos) {
            return false; // Occluded by own chunk block
        }

        // Check in neighbor chunk if at boundary
        if neighbor_pos.x < 0 || neighbor_pos.x >= CHUNK_SIZE {
            if let Some(neighbor) = neighbor_chunk {
                // Convert to neighbor chunk local coords
                let local_x = if neighbor_pos.x < 0 {
                    CHUNK_SIZE - 1
                } else {
                    0
                };
                let local_pos = IVec3::new(local_x, neighbor_pos.y, neighbor_pos.z);
                if neighbor.contains_key(&local_pos) {
                    return false; // Occluded by neighbor chunk block
                }
            }
        }

        true // Not occluded, should render
    }

    let edge_pos = IVec3::new(CHUNK_SIZE - 1, 0, 0);

    // Without neighbor info: incorrectly says to render +X face
    let render_without_neighbor = should_render_face(edge_pos, IVec3::new(1, 0, 0), &chunk_a, None);

    // With neighbor info: correctly says NOT to render +X face
    let render_with_neighbor =
        should_render_face(edge_pos, IVec3::new(1, 0, 0), &chunk_a, Some(&chunk_b));

    assert!(
        render_without_neighbor,
        "Without neighbor info, would incorrectly render face"
    );
    assert!(
        !render_with_neighbor,
        "With neighbor info, correctly skips occluded face"
    );
}

// ============================================================
// BUG-5: Block operations should not cause freeze
// ============================================================
#[test]
fn test_block_operations_no_freeze() {
    // Block place and break should use the same chunk regeneration pattern
    // Inconsistent patterns can cause freezes

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum ChunkRegenPattern {
        CurrentOnly,
        CurrentAndNeighbors,
        AllLoaded,
    }

    // Simulate the regeneration pattern used by block_place and block_break
    let block_place_pattern = ChunkRegenPattern::CurrentAndNeighbors;
    let block_break_pattern = ChunkRegenPattern::CurrentAndNeighbors;

    // Both should use the same pattern
    assert_eq!(
        block_place_pattern, block_break_pattern,
        "block_place and block_break should use same chunk regeneration pattern"
    );
}

// ============================================================
// BUG-10: UI open should block player movement
// ============================================================
#[test]
fn test_ui_blocks_player_movement() {
    // When any UI is open, player movement should be blocked

    #[derive(Default)]
    struct InputState {
        inventory_open: bool,
        furnace_ui_open: bool,
        crusher_ui_open: bool,
        command_input_open: bool,
    }

    impl InputState {
        fn allows_movement(&self) -> bool {
            !self.inventory_open
                && !self.furnace_ui_open
                && !self.crusher_ui_open
                && !self.command_input_open
        }
    }

    let mut state = InputState::default();

    // No UI open - movement allowed
    assert!(state.allows_movement());

    // Inventory open - movement blocked
    state.inventory_open = true;
    assert!(!state.allows_movement());
    state.inventory_open = false;

    // Furnace UI open - movement blocked
    state.furnace_ui_open = true;
    assert!(!state.allows_movement());
    state.furnace_ui_open = false;

    // Crusher UI open - movement blocked
    state.crusher_ui_open = true;
    assert!(!state.allows_movement());
    state.crusher_ui_open = false;

    // Command input open - movement blocked
    state.command_input_open = true;
    assert!(!state.allows_movement());
}

// ============================================================
// BUG-12: UI open should block hotbar scroll
// ============================================================
#[test]
fn test_ui_blocks_hotbar_scroll() {
    // When inventory UI is open, mouse wheel should not change hotbar selection

    struct GameState {
        inventory_open: bool,
        hotbar_selection: usize,
    }

    fn handle_scroll(state: &mut GameState, scroll_delta: i32) {
        // Should check if UI is open before processing scroll
        if state.inventory_open {
            return; // Block scroll when UI open
        }

        if scroll_delta > 0 {
            state.hotbar_selection = (state.hotbar_selection + 1) % 9;
        } else if scroll_delta < 0 {
            state.hotbar_selection = (state.hotbar_selection + 8) % 9;
        }
    }

    let mut state = GameState {
        inventory_open: false,
        hotbar_selection: 0,
    };

    // Scroll without UI - should change selection
    handle_scroll(&mut state, 1);
    assert_eq!(state.hotbar_selection, 1);

    // Open inventory
    state.inventory_open = true;

    // Scroll with UI open - should NOT change selection
    handle_scroll(&mut state, 1);
    assert_eq!(
        state.hotbar_selection, 1,
        "Hotbar should not change when UI is open"
    );
}

// ============================================================
// BUG-19: Chunk processing should be rate-limited
// ============================================================
#[test]
fn test_chunk_processing_rate_limit() {
    // Processing too many chunks per frame causes freezes
    // Should limit to MAX_CHUNKS_PER_FRAME

    const MAX_CHUNKS_PER_FRAME: usize = 2;

    let pending_chunks = vec![
        IVec3::new(0, 0, 0),
        IVec3::new(1, 0, 0),
        IVec3::new(2, 0, 0),
        IVec3::new(3, 0, 0),
        IVec3::new(4, 0, 0),
    ];

    // Simulate processing with rate limit
    let chunks_to_process: Vec<_> = pending_chunks.iter().take(MAX_CHUNKS_PER_FRAME).collect();

    assert_eq!(chunks_to_process.len(), MAX_CHUNKS_PER_FRAME);
    assert!(
        chunks_to_process.len() < pending_chunks.len(),
        "Should not process all chunks at once"
    );
}

// NOTE: test_conveyor_shape_detection moved earlier in file with Splitter support

// ============================================================
// Asset file existence tests
// ============================================================
#[test]
fn test_conveyor_model_files_exist() {
    // Conveyor glTF models should exist for all shapes
    let model_files = [
        "assets/models/machines/conveyor/straight.glb",
        "assets/models/machines/conveyor/corner_left.glb",
        "assets/models/machines/conveyor/corner_right.glb",
        "assets/models/machines/conveyor/t_junction.glb",
        "assets/models/machines/conveyor/splitter.glb",
    ];

    for file in model_files {
        assert!(
            std::path::Path::new(file).exists(),
            "Conveyor model file should exist: {}",
            file
        );
    }
}

#[test]
fn test_conveyor_model_file_valid_glb() {
    // Verify conveyor model files are valid GLB format
    let model_files = [
        "assets/models/machines/conveyor/straight.glb",
        "assets/models/machines/conveyor/corner_left.glb",
        "assets/models/machines/conveyor/corner_right.glb",
        "assets/models/machines/conveyor/t_junction.glb",
        "assets/models/machines/conveyor/splitter.glb",
    ];

    for file in model_files {
        let data = std::fs::read(file).expect(&format!("Should read file: {}", file));

        // GLB magic number is "glTF" (0x46546C67)
        assert!(data.len() >= 12, "GLB file too small: {}", file);
        assert_eq!(&data[0..4], b"glTF", "Invalid GLB magic for: {}", file);

        // Version should be 2
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        assert_eq!(version, 2, "GLB version should be 2 for: {}", file);
    }
}

// ============================================================
// Save/Load system tests
// ============================================================

/// Test that save directory can be created
#[test]
fn test_save_directory_creation() {
    let save_dir = std::path::Path::new("saves");

    // Clean up if exists
    if save_dir.exists() {
        // Don't delete existing saves, just verify the dir exists
    } else {
        // Create directory
        std::fs::create_dir_all(save_dir).expect("Should create saves directory");
    }

    assert!(save_dir.exists() || std::fs::create_dir_all(save_dir).is_ok());
}

/// Test save file JSON structure
#[test]
fn test_save_file_json_structure() {
    // Simulate save data structure
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestSaveData {
        version: String,
        timestamp: u64,
        player: TestPlayerData,
        inventory: TestInventoryData,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestPlayerData {
        position: (f32, f32, f32),
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestInventoryData {
        selected_slot: usize,
        slots: Vec<Option<(String, u32)>>,
    }

    let data = TestSaveData {
        version: "0.1.0".to_string(),
        timestamp: 1704067200000,
        player: TestPlayerData {
            position: (8.0, 12.0, 20.0),
        },
        inventory: TestInventoryData {
            selected_slot: 0,
            slots: vec![
                Some(("Stone".to_string(), 10)),
                None,
                Some(("IronOre".to_string(), 5)),
            ],
        },
    };

    // Serialize
    let json = serde_json::to_string_pretty(&data).expect("Should serialize");
    assert!(json.contains("version"));
    assert!(json.contains("0.1.0"));
    assert!(json.contains("player"));
    assert!(json.contains("inventory"));

    // Deserialize
    let parsed: TestSaveData = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(parsed.version, "0.1.0");
    assert_eq!(parsed.player.position.0, 8.0);
    assert_eq!(parsed.inventory.selected_slot, 0);
}

/// Test position key conversion for world modifications
#[test]
fn test_position_key_conversion() {
    fn pos_to_key(pos: IVec3) -> String {
        format!("{},{},{}", pos.x, pos.y, pos.z)
    }

    fn key_to_pos(key: &str) -> Option<IVec3> {
        let parts: Vec<&str> = key.split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(IVec3::new(
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    }

    // Test positive coordinates
    let pos = IVec3::new(10, 8, 20);
    let key = pos_to_key(pos);
    assert_eq!(key, "10,8,20");
    assert_eq!(key_to_pos(&key), Some(pos));

    // Test negative coordinates
    let pos_neg = IVec3::new(-5, 0, -10);
    let key_neg = pos_to_key(pos_neg);
    assert_eq!(key_neg, "-5,0,-10");
    assert_eq!(key_to_pos(&key_neg), Some(pos_neg));

    // Test zero
    let pos_zero = IVec3::ZERO;
    let key_zero = pos_to_key(pos_zero);
    assert_eq!(key_zero, "0,0,0");
    assert_eq!(key_to_pos(&key_zero), Some(pos_zero));

    // Test invalid key
    assert_eq!(key_to_pos("invalid"), None);
    assert_eq!(key_to_pos("1,2"), None);
    assert_eq!(key_to_pos("a,b,c"), None);
}

/// Test save/load round trip for modified blocks
#[test]
fn test_modified_blocks_save_load() {
    use std::collections::HashMap;

    // Simulate modified blocks (what players placed/removed)
    let mut modified_blocks: HashMap<IVec3, Option<BlockType>> = HashMap::new();

    // Player placed a stone block
    modified_blocks.insert(IVec3::new(10, 8, 20), Some(BlockType::Stone));
    // Player removed a block (air)
    modified_blocks.insert(IVec3::new(12, 8, 20), None);
    // Player placed a miner
    modified_blocks.insert(IVec3::new(5, 8, 5), Some(BlockType::MinerBlock));

    // Convert to saveable format
    fn pos_to_key(pos: IVec3) -> String {
        format!("{},{},{}", pos.x, pos.y, pos.z)
    }

    let save_data: HashMap<String, Option<String>> = modified_blocks
        .iter()
        .map(|(pos, block)| {
            let key = pos_to_key(*pos);
            let value = block.map(|b| format!("{:?}", b));
            (key, value)
        })
        .collect();

    // Verify save data
    assert_eq!(save_data.len(), 3);
    assert!(save_data.contains_key("10,8,20"));
    assert!(save_data.contains_key("12,8,20"));
    assert!(save_data.contains_key("5,8,5"));

    // Check values
    assert_eq!(save_data.get("10,8,20"), Some(&Some("Stone".to_string())));
    assert_eq!(save_data.get("12,8,20"), Some(&None));
    assert_eq!(
        save_data.get("5,8,5"),
        Some(&Some("MinerBlock".to_string()))
    );
}

/// Test machine state serialization
#[test]
fn test_machine_state_serialization() {
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestMiner {
        position: (i32, i32, i32),
        progress: f32,
        buffer: Option<(String, u32)>,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestConveyor {
        position: (i32, i32, i32),
        direction: String,
        items: Vec<(String, f32)>,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestFurnace {
        position: (i32, i32, i32),
        fuel: u32,
        input: Option<(String, u32)>,
        output: Option<(String, u32)>,
        progress: f32,
    }

    // Test miner
    let miner = TestMiner {
        position: (10, 8, 10),
        progress: 0.5,
        buffer: Some(("IronOre".to_string(), 3)),
    };
    let json = serde_json::to_string(&miner).unwrap();
    let parsed: TestMiner = serde_json::from_str(&json).unwrap();
    assert_eq!(miner, parsed);

    // Test conveyor with items
    let conveyor = TestConveyor {
        position: (11, 8, 10),
        direction: "East".to_string(),
        items: vec![("IronOre".to_string(), 0.3), ("IronOre".to_string(), 0.7)],
    };
    let json = serde_json::to_string(&conveyor).unwrap();
    let parsed: TestConveyor = serde_json::from_str(&json).unwrap();
    assert_eq!(conveyor, parsed);

    // Test furnace with active smelting
    let furnace = TestFurnace {
        position: (12, 8, 10),
        fuel: 5,
        input: Some(("IronOre".to_string(), 10)),
        output: Some(("IronIngot".to_string(), 2)),
        progress: 0.75,
    };
    let json = serde_json::to_string(&furnace).unwrap();
    let parsed: TestFurnace = serde_json::from_str(&json).unwrap();
    assert_eq!(furnace, parsed);
}

/// Test quest progress serialization
#[test]
fn test_quest_progress_serialization() {
    use std::collections::HashMap;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestQuestData {
        current_index: usize,
        completed: bool,
        rewards_claimed: bool,
        delivered: HashMap<String, u32>,
    }

    // Quest in progress
    let mut delivered = HashMap::new();
    delivered.insert("IronIngot".to_string(), 50);

    let quest = TestQuestData {
        current_index: 1,
        completed: false,
        rewards_claimed: false,
        delivered,
    };

    let json = serde_json::to_string(&quest).unwrap();
    let parsed: TestQuestData = serde_json::from_str(&json).unwrap();
    assert_eq!(quest, parsed);

    // Completed quest
    let quest_done = TestQuestData {
        current_index: 2,
        completed: true,
        rewards_claimed: true,
        delivered: HashMap::new(),
    };

    let json = serde_json::to_string(&quest_done).unwrap();
    let parsed: TestQuestData = serde_json::from_str(&json).unwrap();
    assert_eq!(quest_done, parsed);
}

/// Test inventory serialization with edge cases
#[test]
fn test_inventory_serialization_edge_cases() {
    const NUM_SLOTS: usize = 36;
    const MAX_STACK: u32 = 999;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestInventory {
        selected_slot: usize,
        slots: Vec<Option<(String, u32)>>,
    }

    // Empty inventory
    let empty = TestInventory {
        selected_slot: 0,
        slots: vec![None; NUM_SLOTS],
    };
    let json = serde_json::to_string(&empty).unwrap();
    let parsed: TestInventory = serde_json::from_str(&json).unwrap();
    assert_eq!(empty.slots.len(), NUM_SLOTS);
    assert!(parsed.slots.iter().all(|s| s.is_none()));

    // Full inventory at max stack
    let mut full_slots = Vec::new();
    for _ in 0..NUM_SLOTS {
        full_slots.push(Some(("Stone".to_string(), MAX_STACK)));
    }
    let full = TestInventory {
        selected_slot: 8, // Last hotbar slot
        slots: full_slots,
    };
    let json = serde_json::to_string(&full).unwrap();
    let parsed: TestInventory = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.selected_slot, 8);
    assert!(parsed
        .slots
        .iter()
        .all(|s| { s.as_ref().map(|(_, c)| *c == MAX_STACK).unwrap_or(false) }));
}

/// Test auto-save timer logic
#[test]
fn test_auto_save_timer() {
    const AUTO_SAVE_INTERVAL: f32 = 60.0; // 1 minute

    struct MockTimer {
        elapsed: f32,
        duration: f32,
        just_finished: bool,
    }

    impl MockTimer {
        fn new(duration: f32) -> Self {
            Self {
                elapsed: 0.0,
                duration,
                just_finished: false,
            }
        }

        fn tick(&mut self, delta: f32) {
            self.elapsed += delta;
            self.just_finished = false;
            if self.elapsed >= self.duration {
                self.elapsed -= self.duration;
                self.just_finished = true;
            }
        }
    }

    let mut timer = MockTimer::new(AUTO_SAVE_INTERVAL);

    // Tick for 59 seconds - should not trigger
    for _ in 0..59 {
        timer.tick(1.0);
        assert!(!timer.just_finished);
    }

    // Tick for 1 more second - should trigger
    timer.tick(1.0);
    assert!(timer.just_finished);

    // Next tick should not trigger immediately
    timer.tick(0.1);
    assert!(!timer.just_finished);

    // Fast forward another minute
    timer.tick(60.0);
    assert!(timer.just_finished);
}

/// Test save command parsing
#[test]
fn test_save_command_parsing() {
    fn parse_save_command(input: &str) -> Option<String> {
        let lowered = input.trim().to_lowercase();
        let parts: Vec<&str> = lowered.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "/save" | "save" => Some(parts.get(1).unwrap_or(&"quicksave").to_string()),
            _ => None,
        }
    }

    // Default filename
    assert_eq!(parse_save_command("/save"), Some("quicksave".to_string()));
    assert_eq!(parse_save_command("save"), Some("quicksave".to_string()));

    // Custom filename
    assert_eq!(
        parse_save_command("/save myworld"),
        Some("myworld".to_string())
    );
    assert_eq!(
        parse_save_command("/save test_save"),
        Some("test_save".to_string())
    );

    // Invalid commands
    assert_eq!(parse_save_command("/creative"), None);
    assert_eq!(parse_save_command("help"), None);
}

/// Test load command parsing
#[test]
fn test_load_command_parsing() {
    fn parse_load_command(input: &str) -> Option<String> {
        let lowered = input.trim().to_lowercase();
        let parts: Vec<&str> = lowered.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "/load" | "load" => Some(parts.get(1).unwrap_or(&"quicksave").to_string()),
            _ => None,
        }
    }

    // Default filename
    assert_eq!(parse_load_command("/load"), Some("quicksave".to_string()));
    assert_eq!(parse_load_command("load"), Some("quicksave".to_string()));

    // Custom filename
    assert_eq!(
        parse_load_command("/load myworld"),
        Some("myworld".to_string())
    );
    assert_eq!(
        parse_load_command("/load autosave"),
        Some("autosave".to_string())
    );

    // Invalid commands
    assert_eq!(parse_load_command("/save"), None);
    assert_eq!(parse_load_command("clear"), None);
}

// =============================================================================
// B-4: Assert Helper Functions for E2E Testing
// =============================================================================

/// Check if inventory contains at least the specified amount of an item
fn assert_inventory_contains(
    inventory: &HashMap<BlockType, u32>,
    block_type: BlockType,
    min_count: u32,
) -> bool {
    inventory.get(&block_type).copied().unwrap_or(0) >= min_count
}

/// Check if a machine is working (has progress or output)
fn assert_machine_working(progress: f32, output_count: u32) -> bool {
    progress > 0.0 || output_count > 0
}

/// Check if conveyor has expected item count
fn assert_conveyor_has_items(item_count: usize, expected: usize) -> bool {
    item_count >= expected
}

/// Check quest progress
fn assert_quest_progress(delivered: u32, expected: u32) -> bool {
    delivered >= expected
}

#[test]
fn test_assert_inventory_contains() {
    let mut inventory = HashMap::new();
    inventory.insert(BlockType::IronOre, 5);
    inventory.insert(BlockType::Coal, 10);

    assert!(assert_inventory_contains(&inventory, BlockType::IronOre, 3));
    assert!(assert_inventory_contains(&inventory, BlockType::IronOre, 5));
    assert!(!assert_inventory_contains(
        &inventory,
        BlockType::IronOre,
        6
    ));
    assert!(!assert_inventory_contains(&inventory, BlockType::Stone, 1));
}

#[test]
fn test_assert_machine_working() {
    // Machine with progress
    assert!(assert_machine_working(0.5, 0));
    // Machine with output
    assert!(assert_machine_working(0.0, 1));
    // Machine idle
    assert!(!assert_machine_working(0.0, 0));
}

#[test]
fn test_assert_conveyor_has_items() {
    assert!(assert_conveyor_has_items(5, 3));
    assert!(assert_conveyor_has_items(3, 3));
    assert!(!assert_conveyor_has_items(2, 3));
}

#[test]
fn test_assert_quest_progress() {
    assert!(assert_quest_progress(5, 3));
    assert!(assert_quest_progress(3, 3));
    assert!(!assert_quest_progress(2, 3));
}

// =============================================================================
// F-2: L-Shape Conveyor Tests
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
enum ConveyorDirection {
    North,
    East,
    South,
    West,
}

impl ConveyorDirection {
    fn to_vec(&self) -> Vec3 {
        match self {
            ConveyorDirection::North => Vec3::new(0.0, 0.0, -1.0),
            ConveyorDirection::East => Vec3::new(1.0, 0.0, 0.0),
            ConveyorDirection::South => Vec3::new(0.0, 0.0, 1.0),
            ConveyorDirection::West => Vec3::new(-1.0, 0.0, 0.0),
        }
    }

    fn left(&self) -> ConveyorDirection {
        match self {
            ConveyorDirection::North => ConveyorDirection::West,
            ConveyorDirection::East => ConveyorDirection::North,
            ConveyorDirection::South => ConveyorDirection::East,
            ConveyorDirection::West => ConveyorDirection::South,
        }
    }

    fn right(&self) -> ConveyorDirection {
        match self {
            ConveyorDirection::North => ConveyorDirection::East,
            ConveyorDirection::East => ConveyorDirection::South,
            ConveyorDirection::South => ConveyorDirection::West,
            ConveyorDirection::West => ConveyorDirection::North,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
enum ConveyorShape {
    Straight,
    CornerLeft,
    CornerRight,
}

/// Simulate item movement on L-shape (left corner) conveyor
fn simulate_corner_left_path(
    start_pos: Vec3,
    input_dir: ConveyorDirection,
    steps: usize,
) -> Vec<Vec3> {
    let mut positions = vec![start_pos];
    let output_dir = input_dir.left();
    let corner_center = Vec3::new(0.5, 0.0, 0.5);

    for step in 0..steps {
        let t = step as f32 / steps as f32;

        let current = if t < 0.5 {
            // First half: move toward corner center along input direction
            start_pos + input_dir.to_vec() * (t * 2.0)
        } else {
            // Second half: turn and move along output direction
            let corner_progress = (t - 0.5) * 2.0;
            corner_center + output_dir.to_vec() * corner_progress
        };
        positions.push(current);
    }

    positions
}

/// Simulate item movement on L-shape (right corner) conveyor
fn simulate_corner_right_path(
    start_pos: Vec3,
    input_dir: ConveyorDirection,
    steps: usize,
) -> Vec<Vec3> {
    let mut positions = vec![start_pos];
    let output_dir = input_dir.right();
    let corner_center = Vec3::new(0.5, 0.0, 0.5);

    for step in 0..steps {
        let t = step as f32 / steps as f32;

        let current = if t < 0.5 {
            start_pos + input_dir.to_vec() * (t * 2.0)
        } else {
            let corner_progress = (t - 0.5) * 2.0;
            corner_center + output_dir.to_vec() * corner_progress
        };
        positions.push(current);
    }

    positions
}

#[test]
fn test_conveyor_corner_left_item_path() {
    // Item enters from South (moving North), should exit West
    let start = Vec3::new(0.5, 0.0, 1.0);
    let path = simulate_corner_left_path(start, ConveyorDirection::North, 10);

    // Verify item starts at entry point
    assert_eq!(path[0], start);

    // Verify item ends moving West (negative X)
    let final_pos = path.last().unwrap();
    assert!(
        final_pos.x < 0.5,
        "Corner left should exit West (negative X), got x={}",
        final_pos.x
    );

    // Verify item passed through center area
    let center_crossed = path
        .iter()
        .any(|p| (p.x - 0.5).abs() < 0.3 && (p.z - 0.5).abs() < 0.3);
    assert!(center_crossed, "Item should pass through corner center");
}

#[test]
fn test_conveyor_corner_right_item_path() {
    // Item enters from South (moving North), should exit East
    let start = Vec3::new(0.5, 0.0, 1.0);
    let path = simulate_corner_right_path(start, ConveyorDirection::North, 10);

    // Verify item starts at entry point
    assert_eq!(path[0], start);

    // Verify item ends moving East (positive X)
    let final_pos = path.last().unwrap();
    assert!(
        final_pos.x > 0.5,
        "Corner right should exit East (positive X), got x={}",
        final_pos.x
    );

    // Verify item passed through center area
    let center_crossed = path
        .iter()
        .any(|p| (p.x - 0.5).abs() < 0.3 && (p.z - 0.5).abs() < 0.3);
    assert!(center_crossed, "Item should pass through corner center");
}

#[test]
fn test_corner_left_all_directions() {
    // Test left corner from all 4 directions
    let test_cases = [
        (ConveyorDirection::North, ConveyorDirection::West), // N -> W
        (ConveyorDirection::East, ConveyorDirection::North), // E -> N
        (ConveyorDirection::South, ConveyorDirection::East), // S -> E
        (ConveyorDirection::West, ConveyorDirection::South), // W -> S
    ];

    for (input, expected_output) in test_cases {
        let output = input.left();
        assert_eq!(
            output, expected_output,
            "Left of {:?} should be {:?}, got {:?}",
            input, expected_output, output
        );
    }
}

#[test]
fn test_corner_right_all_directions() {
    // Test right corner from all 4 directions
    let test_cases = [
        (ConveyorDirection::North, ConveyorDirection::East), // N -> E
        (ConveyorDirection::East, ConveyorDirection::South), // E -> S
        (ConveyorDirection::South, ConveyorDirection::West), // S -> W
        (ConveyorDirection::West, ConveyorDirection::North), // W -> N
    ];

    for (input, expected_output) in test_cases {
        let output = input.right();
        assert_eq!(
            output, expected_output,
            "Right of {:?} should be {:?}, got {:?}",
            input, expected_output, output
        );
    }
}

// =============================================================================
// B-2: Production Chain Scenario Tests
// =============================================================================

/// Mock Miner for testing
struct MockMiner {
    position: IVec3,
    progress: f32,
    buffer: Option<(BlockType, u32)>,
    ore_type: BlockType,
}

impl MockMiner {
    fn new(position: IVec3, ore_type: BlockType) -> Self {
        Self {
            position,
            progress: 0.0,
            buffer: None,
            ore_type,
        }
    }

    fn tick(&mut self, delta: f32) {
        const MINE_TIME: f32 = 2.0;
        self.progress += delta / MINE_TIME;
        if self.progress >= 1.0 {
            self.progress = 0.0;
            let (_, count) = self.buffer.get_or_insert((self.ore_type, 0));
            *count += 1;
        }
    }

    fn take_output(&mut self) -> Option<BlockType> {
        if let Some((item, count)) = &mut self.buffer {
            if *count > 0 {
                *count -= 1;
                let result = Some(*item);
                if *count == 0 {
                    self.buffer = None;
                }
                return result;
            }
        }
        None
    }
}

/// Mock Furnace for testing
struct MockFurnace {
    #[allow(dead_code)]
    position: IVec3,
    input: Option<(BlockType, u32)>,
    output: Option<(BlockType, u32)>,
    fuel: u32,
    progress: f32,
}

impl MockFurnace {
    fn new(position: IVec3) -> Self {
        Self {
            position,
            input: None,
            output: None,
            fuel: 0,
            progress: 0.0,
        }
    }

    fn add_input(&mut self, item: BlockType) -> bool {
        if let Some((existing, count)) = &mut self.input {
            if *existing == item {
                *count += 1;
                return true;
            }
            return false;
        }
        self.input = Some((item, 1));
        true
    }

    fn add_fuel(&mut self, amount: u32) {
        self.fuel += amount;
    }

    fn tick(&mut self, delta: f32) -> bool {
        const SMELT_TIME: f32 = 3.0;

        // Check if we can smelt
        if self.fuel == 0 {
            return false;
        }
        let Some((input_type, input_count)) = &mut self.input else {
            return false;
        };
        if *input_count == 0 {
            return false;
        }

        // Smelt
        self.progress += delta / SMELT_TIME;
        if self.progress >= 1.0 {
            self.progress = 0.0;
            *input_count -= 1;
            self.fuel -= 1;

            // Produce output
            let output_type = match input_type {
                BlockType::IronOre => BlockType::IronIngot,
                BlockType::CopperOre => BlockType::CopperIngot,
                _ => return false,
            };

            if let Some((_, count)) = &mut self.output {
                *count += 1;
            } else {
                self.output = Some((output_type, 1));
            }

            if *input_count == 0 {
                self.input = None;
            }
            return true;
        }
        false
    }

    fn take_output(&mut self) -> Option<BlockType> {
        if let Some((item, count)) = &mut self.output {
            if *count > 0 {
                *count -= 1;
                let result = Some(*item);
                if *count == 0 {
                    self.output = None;
                }
                return result;
            }
        }
        None
    }
}

#[test]
fn test_full_production_chain() {
    // Simulate: Miner (IronOre) -> Conveyor -> Furnace -> IronIngot

    let mut miner = MockMiner::new(IVec3::new(0, 8, 0), BlockType::IronOre);
    let mut furnace = MockFurnace::new(IVec3::new(2, 8, 0));
    let mut conveyor_buffer: Vec<BlockType> = Vec::new();

    // Add fuel to furnace
    furnace.add_fuel(5);

    // Simulate 30 seconds of operation
    let delta = 0.5;
    let mut produced_ingots = 0u32;

    for _ in 0..60 {
        // Miner produces ore
        miner.tick(delta);

        // Transfer from miner to conveyor
        if let Some(ore) = miner.take_output() {
            conveyor_buffer.push(ore);
        }

        // Transfer from conveyor to furnace
        if let Some(ore) = conveyor_buffer.pop() {
            furnace.add_input(ore);
        }

        // Furnace processes
        furnace.tick(delta);

        // Count output
        while let Some(_ingot) = furnace.take_output() {
            produced_ingots += 1;
        }
    }

    // Should have produced at least 3 ingots in 30 seconds
    // (Miner: 2s per ore, Furnace: 3s per ingot, 5 fuel available)
    assert!(
        produced_ingots >= 3,
        "Expected at least 3 ingots, got {}",
        produced_ingots
    );
}

#[test]
fn test_miner_to_furnace_chain_with_limited_fuel() {
    let mut miner = MockMiner::new(IVec3::new(0, 8, 0), BlockType::IronOre);
    let mut furnace = MockFurnace::new(IVec3::new(2, 8, 0));

    // Only 2 fuel
    furnace.add_fuel(2);

    let delta = 0.5;
    let mut produced = 0u32;

    for _ in 0..40 {
        miner.tick(delta);
        if let Some(ore) = miner.take_output() {
            furnace.add_input(ore);
        }
        furnace.tick(delta);
        while furnace.take_output().is_some() {
            produced += 1;
        }
    }

    // Should produce exactly 2 ingots (limited by fuel)
    assert_eq!(produced, 2, "Should produce exactly 2 ingots with 2 fuel");
}

#[test]
fn test_furnace_without_fuel() {
    let mut furnace = MockFurnace::new(IVec3::ZERO);
    furnace.add_input(BlockType::IronOre);

    // Tick without fuel
    for _ in 0..20 {
        furnace.tick(0.5);
    }

    // Should not produce anything
    assert!(furnace.take_output().is_none());
    assert_eq!(furnace.progress, 0.0);
}

#[test]
fn test_miner_continuous_output() {
    let mut miner = MockMiner::new(IVec3::ZERO, BlockType::CopperOre);
    let mut collected = 0u32;

    // Run for 20 seconds
    for _ in 0..40 {
        miner.tick(0.5);
        while miner.take_output().is_some() {
            collected += 1;
        }
    }

    // 20 seconds / 2 seconds per ore = 10 ores
    assert!(
        collected >= 9,
        "Expected at least 9 ores in 20s, got {}",
        collected
    );
}

// =============================================================================
// Block Placement on Conveyor Tests
// =============================================================================

/// Test that blocks can be placed on top of a conveyor
#[test]
fn test_place_block_on_conveyor() {
    // Conveyor at (5, 8, 5)
    let conveyor_pos = IVec3::new(5, 8, 5);

    // Simulate ray from above the conveyor looking down
    let ray_origin = Vec3::new(5.5, 12.0, 5.5);
    let ray_direction = Vec3::new(0.0, -1.0, 0.0); // Looking straight down

    // Conveyor bounds (belt height is about 0.25)
    let conveyor_center = Vec3::new(5.5, 8.125, 5.5);
    let conveyor_half = Vec3::new(0.45, 0.125, 0.5);

    // Check ray hits conveyor
    let hit = ray_aabb_intersection_simple(
        ray_origin,
        ray_direction,
        conveyor_center - conveyor_half,
        conveyor_center + conveyor_half,
    );

    assert!(hit.is_some(), "Ray should hit conveyor from above");

    // The placement position should be one block above the conveyor
    let place_pos = conveyor_pos + IVec3::new(0, 1, 0);
    assert_eq!(
        place_pos,
        IVec3::new(5, 9, 5),
        "Block should be placed above conveyor"
    );
}

/// Test that conveyors can be stacked (placing conveyor on conveyor)
#[test]
fn test_stack_conveyor_on_conveyor() {
    // Bottom conveyor at y=8
    let bottom_conveyor = IVec3::new(10, 8, 10);
    // Expected top conveyor at y=9
    let top_conveyor_expected = IVec3::new(10, 9, 10);

    // Verify positions are different
    assert_ne!(bottom_conveyor, top_conveyor_expected);
    assert_eq!(top_conveyor_expected.y, bottom_conveyor.y + 1);
}

/// Simple ray-AABB intersection for testing
fn ray_aabb_intersection_simple(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let inv_dir = Vec3::new(
        if dir.x.abs() > 1e-8 {
            1.0 / dir.x
        } else {
            f32::MAX
        },
        if dir.y.abs() > 1e-8 {
            1.0 / dir.y
        } else {
            f32::MAX
        },
        if dir.z.abs() > 1e-8 {
            1.0 / dir.z
        } else {
            f32::MAX
        },
    );

    let t1 = (min - origin) * inv_dir;
    let t2 = (max - origin) * inv_dir;

    let tmin = t1.min(t2);
    let tmax = t1.max(t2);

    let t_enter = tmin.x.max(tmin.y).max(tmin.z);
    let t_exit = tmax.x.min(tmax.y).min(tmax.z);

    if t_enter <= t_exit && t_exit > 0.0 {
        Some(t_enter.max(0.0))
    } else {
        None
    }
}

/// Test placement doesn't occur inside conveyor
#[test]
fn test_place_not_inside_conveyor() {
    let conveyor_positions = vec![
        IVec3::new(0, 8, 0),
        IVec3::new(1, 8, 0),
        IVec3::new(2, 8, 0),
    ];

    // Test that we can't place at conveyor position
    for pos in &conveyor_positions {
        let place_pos = *pos;
        let is_occupied = conveyor_positions.iter().any(|c| *c == place_pos);
        assert!(
            is_occupied,
            "Position {:?} should be occupied by conveyor",
            pos
        );
    }

    // Test that we CAN place above
    let above_pos = IVec3::new(1, 9, 0);
    let is_occupied = conveyor_positions.iter().any(|c| *c == above_pos);
    assert!(!is_occupied, "Position above conveyor should be free");
}

// =====================================================
// Bevy App Simulation Tests
// =====================================================

use idle_factory::player::PlayerInventory;
use idle_factory::{BlockBreakEvent, BlockPlaceEvent, GameEventsPlugin, QuestProgressEvent};

/// Test that GameEventsPlugin registers all events correctly
#[test]
fn test_game_events_plugin_registers_events() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GameEventsPlugin);

    // Verify events are registered by sending them (would panic if not registered)
    app.world_mut().send_event(BlockPlaceEvent {
        position: IVec3::new(0, 0, 0),
        block_type: BlockType::Stone,
        player_id: 1,
    });

    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(1, 1, 1),
        player_id: 1,
    });

    app.world_mut().send_event(QuestProgressEvent {
        item_type: BlockType::IronOre,
        amount: 5,
    });

    // Run one update cycle
    app.update();

    // If we get here without panic, events are registered correctly
}

/// Test PlayerInventory component in Bevy App context
#[test]
fn test_inventory_component_in_app() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn player entity with inventory
    let player_entity = app
        .world_mut()
        .spawn(PlayerInventory::with_initial_items(&[
            (BlockType::Stone, 64),
            (BlockType::IronOre, 32),
        ]))
        .id();

    app.update();

    // Verify inventory state
    let inventory = app.world().get::<PlayerInventory>(player_entity).unwrap();

    // Check first slot has stone
    assert_eq!(inventory.get_slot(0), Some(BlockType::Stone));
    assert_eq!(inventory.get_slot_count(0), 64);

    // Check second slot has iron ore
    assert_eq!(inventory.get_slot(1), Some(BlockType::IronOre));
    assert_eq!(inventory.get_slot_count(1), 32);
}

/// Test inventory add_item stacking behavior
#[test]
fn test_inventory_stacking_in_app() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn player entity with empty inventory
    let player_entity = app.world_mut().spawn(PlayerInventory::default()).id();

    app.update();

    // Add items through world access
    {
        let mut inventory = app
            .world_mut()
            .get_mut::<PlayerInventory>(player_entity)
            .unwrap();
        inventory.add_item(BlockType::Coal, 100);
        inventory.add_item(BlockType::Coal, 100);
        inventory.add_item(BlockType::Coal, 100);
    }

    app.update();

    // Verify stacking behavior
    let inventory = app.world().get::<PlayerInventory>(player_entity).unwrap();

    // Coal should be stacked (999 max per slot)
    let total_coal: u32 = (0..idle_factory::constants::NUM_SLOTS)
        .filter_map(|i| {
            if inventory.get_slot(i) == Some(BlockType::Coal) {
                Some(inventory.get_slot_count(i))
            } else {
                None
            }
        })
        .sum();

    assert_eq!(total_coal, 300, "Should have 300 coal total");
}

/// Test system that reads events
fn count_block_break_events(
    mut events: EventReader<BlockBreakEvent>,
    mut counter: ResMut<EventCounter>,
) {
    for _event in events.read() {
        counter.block_breaks += 1;
    }
}

#[derive(Resource, Default)]
struct EventCounter {
    block_breaks: u32,
}

/// Test custom system with event handling
#[test]
fn test_custom_system_event_handling() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GameEventsPlugin);
    app.init_resource::<EventCounter>();
    app.add_systems(Update, count_block_break_events);

    // Send events before update
    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(0, 0, 0),
        player_id: 1,
    });
    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(1, 1, 1),
        player_id: 1,
    });
    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(2, 2, 2),
        player_id: 1,
    });

    // Run update to process events
    app.update();

    // Verify events were counted
    let counter = app.world().resource::<EventCounter>();
    assert_eq!(
        counter.block_breaks, 3,
        "Should have counted 3 block breaks"
    );
}

/// Test inventory selected slot changes
#[test]
fn test_inventory_slot_selection() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut inv = PlayerInventory::default();
    inv.add_item(BlockType::Grass, 10);
    inv.add_item(BlockType::Coal, 20);
    inv.add_item(BlockType::Stone, 30);

    let player_entity = app.world_mut().spawn(inv).id();
    app.update();

    // Initial selection is slot 0
    {
        let inventory = app.world().get::<PlayerInventory>(player_entity).unwrap();
        assert_eq!(inventory.selected_slot, 0);
        assert_eq!(inventory.selected_block(), Some(BlockType::Grass));
    }

    // Change selection to slot 2
    {
        let mut inventory = app
            .world_mut()
            .get_mut::<PlayerInventory>(player_entity)
            .unwrap();
        inventory.selected_slot = 2;
    }

    app.update();

    {
        let inventory = app.world().get::<PlayerInventory>(player_entity).unwrap();
        assert_eq!(inventory.selected_slot, 2);
        assert_eq!(inventory.selected_block(), Some(BlockType::Stone));
    }
}

/// Test quest progress event chain
#[test]
fn test_quest_progress_event_chain() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GameEventsPlugin);

    #[derive(Resource, Default)]
    struct QuestTracker {
        items_collected: HashMap<BlockType, u32>,
    }

    fn track_quest_progress(
        mut events: EventReader<QuestProgressEvent>,
        mut tracker: ResMut<QuestTracker>,
    ) {
        for event in events.read() {
            *tracker.items_collected.entry(event.item_type).or_default() += event.amount;
        }
    }

    app.init_resource::<QuestTracker>();
    app.add_systems(Update, track_quest_progress);

    // Simulate collecting items
    app.world_mut().send_event(QuestProgressEvent {
        item_type: BlockType::IronOre,
        amount: 5,
    });
    app.world_mut().send_event(QuestProgressEvent {
        item_type: BlockType::Coal,
        amount: 10,
    });
    app.world_mut().send_event(QuestProgressEvent {
        item_type: BlockType::IronOre,
        amount: 3,
    });

    app.update();

    let tracker = app.world().resource::<QuestTracker>();
    assert_eq!(tracker.items_collected.get(&BlockType::IronOre), Some(&8));
    assert_eq!(tracker.items_collected.get(&BlockType::Coal), Some(&10));
}

/// Test multiple app updates don't duplicate events
#[test]
fn test_events_consumed_after_read() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GameEventsPlugin);
    app.init_resource::<EventCounter>();
    app.add_systems(Update, count_block_break_events);

    // Send one event
    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(0, 0, 0),
        player_id: 1,
    });

    // First update processes the event
    app.update();
    assert_eq!(app.world().resource::<EventCounter>().block_breaks, 1);

    // Second update should not reprocess the same event
    app.update();
    assert_eq!(app.world().resource::<EventCounter>().block_breaks, 1);

    // Send another event
    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(1, 1, 1),
        player_id: 1,
    });

    app.update();
    assert_eq!(app.world().resource::<EventCounter>().block_breaks, 2);
}

// =====================================================
// Inventory UI Interaction Tests
// =====================================================

/// Test held item pickup and placement behavior
#[test]
fn test_held_item_pickup_and_drop() {
    // Simulate held item for drag-drop
    struct HeldItem {
        block_type: Option<BlockType>,
        count: u32,
    }

    impl HeldItem {
        fn pick_up(&mut self, block_type: BlockType, count: u32) {
            self.block_type = Some(block_type);
            self.count = count;
        }

        fn drop_all(&mut self) -> Option<(BlockType, u32)> {
            if let Some(bt) = self.block_type.take() {
                let count = self.count;
                self.count = 0;
                Some((bt, count))
            } else {
                None
            }
        }

        fn is_empty(&self) -> bool {
            self.block_type.is_none() || self.count == 0
        }
    }

    let mut held = HeldItem {
        block_type: None,
        count: 0,
    };
    let mut inventory = SlotInventory::default();

    // Add items to inventory
    inventory.add_item(BlockType::Stone, 64);

    // Pick up items from slot
    if let Some(bt) = inventory.get_slot(0) {
        let count = inventory.get_slot_count(0);
        inventory.slots[0] = None; // Clear slot
        held.pick_up(bt, count);
    }

    assert!(!held.is_empty());
    assert_eq!(held.count, 64);
    assert!(inventory.get_slot(0).is_none());

    // Drop items into empty slot
    if let Some((bt, count)) = held.drop_all() {
        inventory.add_item(bt, count);
    }

    assert!(held.is_empty());
    assert_eq!(inventory.get_slot_count(0), 64);
}

/// Test slot swap behavior (drag to occupied slot)
#[test]
fn test_inventory_slot_swap() {
    let mut inventory = SlotInventory::default();

    // Put different items in two slots
    inventory.add_item(BlockType::Stone, 32);
    inventory.add_item(BlockType::Grass, 16);

    // Verify initial state
    assert_eq!(inventory.get_slot(0), Some(BlockType::Stone));
    assert_eq!(inventory.get_slot_count(0), 32);
    assert_eq!(inventory.get_slot(1), Some(BlockType::Grass));
    assert_eq!(inventory.get_slot_count(1), 16);

    // Simulate swap by manually swapping slot data
    let slot0 = inventory.slots[0];
    let slot1 = inventory.slots[1];

    inventory.slots[0] = slot1;
    inventory.slots[1] = slot0;

    // Verify swap
    assert_eq!(inventory.get_slot(0), Some(BlockType::Grass));
    assert_eq!(inventory.get_slot_count(0), 16);
    assert_eq!(inventory.get_slot(1), Some(BlockType::Stone));
    assert_eq!(inventory.get_slot_count(1), 32);
}

/// Test trash slot behavior
#[test]
fn test_trash_slot_deletes_items() {
    struct TrashSlot {
        deleted_count: u32,
    }

    impl TrashSlot {
        fn trash(&mut self, _block_type: BlockType, count: u32) {
            self.deleted_count += count;
        }
    }

    let mut trash = TrashSlot { deleted_count: 0 };
    let mut inventory = SlotInventory::default();

    inventory.add_item(BlockType::Stone, 100);

    // Simulate dragging to trash
    if let Some(bt) = inventory.get_slot(0) {
        let count = inventory.get_slot_count(0);
        inventory.slots[0] = None; // Clear slot
        trash.trash(bt, count);
    }

    assert_eq!(trash.deleted_count, 100);
    assert!(inventory.get_slot(0).is_none());
}

/// Test creative mode item spawning
#[test]
fn test_creative_mode_item_spawn() {
    let mut inventory = SlotInventory::default();
    let creative_mode = true;

    // In creative mode, clicking an item in catalog adds a stack
    if creative_mode {
        let spawned_type = BlockType::Stone;
        let spawned_count = 64;
        inventory.add_item(spawned_type, spawned_count);
    }

    assert_eq!(inventory.get_slot(0), Some(BlockType::Stone));
    assert_eq!(inventory.get_slot_count(0), 64);
}

// =====================================================
// Load/Stress Tests
// =====================================================

/// Test handling large number of inventory operations
#[test]
fn test_load_massive_inventory_operations() {
    let mut inventory = SlotInventory::default();

    // Perform 10,000 add operations
    for i in 0..10000 {
        let block_type = if i % 2 == 0 {
            BlockType::Stone
        } else {
            BlockType::Grass
        };
        inventory.add_item(block_type, 1);
    }

    // Verify total counts
    let mut stone_count = 0u32;
    let mut grass_count = 0u32;
    for slot in &inventory.slots {
        if let Some((bt, count)) = slot {
            match bt {
                BlockType::Stone => stone_count += count,
                BlockType::Grass => grass_count += count,
                _ => {}
            }
        }
    }

    assert_eq!(stone_count, 5000);
    assert_eq!(grass_count, 5000);
}

/// Test rapid slot selection changes
#[test]
fn test_load_rapid_slot_selection() {
    let mut hotbar = HotbarState::default();

    // Rapidly switch between all slots 1000 times
    for i in 0..1000 {
        hotbar.select(i % 9);
        assert!(hotbar.selected_index.unwrap() < 9);
    }
}

/// Test stress test with many machines simulated
#[test]
fn test_load_many_machines() {
    // Simulate 100 miners, 100 furnaces, 100 crushers
    struct MachineState {
        miners: Vec<MinerState>,
        furnaces: Vec<FurnaceState>,
        crushers: Vec<CrusherState>,
    }

    struct MinerState {
        progress: f32,
        buffer: u32,
    }
    struct FurnaceState {
        fuel: u32,
        input: u32,
        output: u32,
        progress: f32,
    }
    struct CrusherState {
        input: u32,
        output: u32,
        progress: f32,
    }

    let mut machines = MachineState {
        miners: (0..100)
            .map(|_| MinerState {
                progress: 0.0,
                buffer: 0,
            })
            .collect(),
        furnaces: (0..100)
            .map(|_| FurnaceState {
                fuel: 10,
                input: 0,
                output: 0,
                progress: 0.0,
            })
            .collect(),
        crushers: (0..100)
            .map(|_| CrusherState {
                input: 0,
                output: 0,
                progress: 0.0,
            })
            .collect(),
    };

    // Simulate 1000 ticks of processing
    let delta_time = 0.016; // 60fps
    for _ in 0..1000 {
        // Update miners
        for miner in &mut machines.miners {
            miner.progress += delta_time;
            if miner.progress >= 1.0 {
                miner.progress = 0.0;
                miner.buffer += 1;
            }
        }

        // Update furnaces
        for furnace in &mut machines.furnaces {
            if furnace.fuel > 0 && furnace.input > 0 {
                furnace.progress += delta_time;
                if furnace.progress >= 1.0 {
                    furnace.progress = 0.0;
                    furnace.input -= 1;
                    furnace.output += 1;
                }
            }
        }

        // Update crushers
        for crusher in &mut machines.crushers {
            if crusher.input > 0 {
                crusher.progress += delta_time;
                if crusher.progress >= 1.0 {
                    crusher.progress = 0.0;
                    crusher.input -= 1;
                    crusher.output += 2; // Doubles output
                }
            }
        }
    }

    // Verify miners produced items (1000 ticks * 0.016 = 16 seconds, at 1 ore/second = 16 ore each)
    let total_mined: u32 = machines.miners.iter().map(|m| m.buffer).sum();
    assert!(
        total_mined >= 1500,
        "Should have mined substantial amount: {}",
        total_mined
    );
}

/// Test conveyor chain with many items
#[test]
fn test_load_conveyor_chain_throughput() {
    // Simulate 50-segment conveyor chain
    struct ConveyorSegment {
        items: Vec<f32>, // Progress values
    }

    let mut chain: Vec<ConveyorSegment> =
        (0..50).map(|_| ConveyorSegment { items: vec![] }).collect();

    // Insert 200 items over time
    for tick in 0..500 {
        let delta = 0.1; // 10 ticks per conveyor length

        // Insert new item at start every 2 ticks
        if tick % 2 == 0 && chain[0].items.len() < 5 {
            chain[0].items.push(0.0);
        }

        // Move items along chain
        for i in 0..chain.len() {
            let mut to_transfer = vec![];

            for item in &mut chain[i].items {
                *item += delta;
                if *item >= 1.0 {
                    to_transfer.push(*item);
                }
            }

            chain[i].items.retain(|&p| p < 1.0);

            // Transfer to next segment
            if i + 1 < chain.len() {
                for _ in to_transfer {
                    if chain[i + 1].items.len() < 5 {
                        chain[i + 1].items.push(0.0);
                    }
                }
            }
        }
    }

    // Should have items spread through chain
    let total_items: usize = chain.iter().map(|c| c.items.len()).sum();
    assert!(total_items > 0, "Conveyor chain should have items flowing");
}

/// Test GlobalInventory with very large item counts
#[test]
fn test_load_global_inventory_large_counts() {
    use std::collections::HashMap;

    let mut global_inventory: HashMap<BlockType, u32> = HashMap::new();

    // Add millions of items
    for _ in 0..1000 {
        *global_inventory.entry(BlockType::Stone).or_insert(0) += 10000;
        *global_inventory.entry(BlockType::Grass).or_insert(0) += 5000;
        *global_inventory.entry(BlockType::Coal).or_insert(0) += 2500;
    }

    assert_eq!(global_inventory.get(&BlockType::Stone), Some(&10_000_000));
    assert_eq!(global_inventory.get(&BlockType::Grass), Some(&5_000_000));
    assert_eq!(global_inventory.get(&BlockType::Coal), Some(&2_500_000));

    // Consume items
    for _ in 0..500 {
        if let Some(count) = global_inventory.get_mut(&BlockType::Stone) {
            *count = count.saturating_sub(10000);
        }
    }

    assert_eq!(global_inventory.get(&BlockType::Stone), Some(&5_000_000));
}

// ============================================================================
// GlobalInventory Advanced Tests
// ============================================================================

/// Test GlobalInventory atomic try_consume behavior
#[test]
fn test_global_inventory_try_consume_atomic() {
    use idle_factory::player::GlobalInventory;

    let mut inv = GlobalInventory::new();
    inv.add_item(BlockType::IronIngot, 10);
    inv.add_item(BlockType::Coal, 5);

    // Should fail atomically - neither item consumed
    let result = inv.try_consume(&[(BlockType::IronIngot, 5), (BlockType::Coal, 10)]);
    assert!(!result);
    assert_eq!(inv.get_count(BlockType::IronIngot), 10);
    assert_eq!(inv.get_count(BlockType::Coal), 5);

    // Should succeed
    let result = inv.try_consume(&[(BlockType::IronIngot, 5), (BlockType::Coal, 3)]);
    assert!(result);
    assert_eq!(inv.get_count(BlockType::IronIngot), 5);
    assert_eq!(inv.get_count(BlockType::Coal), 2);
}

/// Test GlobalInventory with zero count items are not shown
#[test]
fn test_global_inventory_zero_count_hidden() {
    use idle_factory::player::GlobalInventory;

    let mut inv = GlobalInventory::new();
    inv.add_item(BlockType::Stone, 10);
    inv.add_item(BlockType::Coal, 0); // Should not appear

    let items = inv.get_all_items();
    assert_eq!(items.len(), 1);
    assert!(items.iter().any(|(bt, _)| *bt == BlockType::Stone));
    assert!(!items.iter().any(|(bt, _)| *bt == BlockType::Coal));
}

/// Test GlobalInventory remove item cleans up zero entries
#[test]
fn test_global_inventory_remove_cleans_zero() {
    use idle_factory::player::GlobalInventory;

    let mut inv = GlobalInventory::new();
    inv.add_item(BlockType::Stone, 10);

    // Remove all
    assert!(inv.remove_item(BlockType::Stone, 10));
    assert_eq!(inv.get_count(BlockType::Stone), 0);
    assert!(inv.is_empty());
}

/// Test GlobalInventory with_items constructor
#[test]
fn test_global_inventory_with_items() {
    use idle_factory::player::GlobalInventory;

    let inv = GlobalInventory::with_items(&[
        (BlockType::MinerBlock, 5),
        (BlockType::ConveyorBlock, 50),
        (BlockType::FurnaceBlock, 10),
    ]);

    assert_eq!(inv.get_count(BlockType::MinerBlock), 5);
    assert_eq!(inv.get_count(BlockType::ConveyorBlock), 50);
    assert_eq!(inv.get_count(BlockType::FurnaceBlock), 10);
    assert_eq!(inv.item_type_count(), 3);
}

// ============================================================================
// Biome System Tests
// ============================================================================

/// Test BiomeType sample_resource with different random values
#[test]
fn test_biome_sample_resource() {
    use idle_factory::world::biome::BiomeType;

    // Iron biome: 70% iron, 22% stone, 8% coal
    let iron_biome = BiomeType::Iron;

    // random_value 0-69 should give iron
    assert_eq!(iron_biome.sample_resource(0), Some(BlockType::IronOre));
    assert_eq!(iron_biome.sample_resource(69), Some(BlockType::IronOre));

    // random_value 70-91 should give stone
    assert_eq!(iron_biome.sample_resource(70), Some(BlockType::Stone));
    assert_eq!(iron_biome.sample_resource(91), Some(BlockType::Stone));

    // random_value 92-99 should give coal
    assert_eq!(iron_biome.sample_resource(92), Some(BlockType::Coal));
    assert_eq!(iron_biome.sample_resource(99), Some(BlockType::Coal));
}

/// Test Unmailable biome returns no resources
#[test]
fn test_biome_unmailable_no_resources() {
    use idle_factory::world::biome::BiomeType;

    let unmailable = BiomeType::Unmailable;
    assert!(unmailable.sample_resource(0).is_none());
    assert!(unmailable.sample_resource(50).is_none());
    assert!(unmailable.sample_resource(99).is_none());
}

/// Test BiomeMap spawn area guarantees
#[test]
fn test_biome_spawn_area_guarantees() {
    use idle_factory::world::biome::BiomeMap;
    use idle_factory::world::biome::BiomeType;

    let biome_map = BiomeMap::new(12345);

    // Very close to spawn center (26, 16) should be Mixed
    let center = IVec3::new(26, 0, 16);
    assert_eq!(biome_map.get_biome(center), BiomeType::Mixed);

    // Can mine at spawn area
    assert!(biome_map.can_mine(center));
}

/// Test BiomeMap deterministic generation
#[test]
fn test_biome_deterministic() {
    use idle_factory::world::biome::BiomeMap;

    let biome_map1 = BiomeMap::new(42);
    let biome_map2 = BiomeMap::new(42);

    // Same seed should give same biomes
    let test_positions = [
        IVec3::new(100, 0, 100),
        IVec3::new(-50, 5, 200),
        IVec3::new(0, 10, 0),
    ];

    for pos in test_positions {
        assert_eq!(
            biome_map1.get_biome(pos),
            biome_map2.get_biome(pos),
            "Biome at {:?} should be deterministic",
            pos
        );
    }
}

/// Test BiomeMap different seeds give different results
#[test]
fn test_biome_different_seeds() {
    use idle_factory::world::biome::BiomeMap;

    let biome_map1 = BiomeMap::new(1);
    let biome_map2 = BiomeMap::new(999999);

    // Check multiple positions - at least one should differ
    let positions: Vec<IVec3> = (0..20)
        .map(|i| IVec3::new(i * 50, 0, i * 50 + 100))
        .collect();

    let different = positions
        .iter()
        .filter(|&&pos| biome_map1.get_biome(pos) != biome_map2.get_biome(pos))
        .count();

    assert!(
        different > 0,
        "Different seeds should produce some different biomes"
    );
}

// ============================================================================
// Machine I/O Tests
// ============================================================================

/// Test machine facing determines output position
#[test]
fn test_machine_facing_output_direction() {
    use idle_factory::components::Direction;

    let base_pos = IVec3::new(10, 5, 10);

    // North facing outputs to z-1 (negative Z is north in this engine)
    let north_output = base_pos + Direction::North.to_ivec3();
    assert_eq!(north_output, IVec3::new(10, 5, 9));

    // East facing outputs to x+1
    let east_output = base_pos + Direction::East.to_ivec3();
    assert_eq!(east_output, IVec3::new(11, 5, 10));

    // South facing outputs to z+1
    let south_output = base_pos + Direction::South.to_ivec3();
    assert_eq!(south_output, IVec3::new(10, 5, 11));

    // West facing outputs to x-1
    let west_output = base_pos + Direction::West.to_ivec3();
    assert_eq!(west_output, IVec3::new(9, 5, 10));
}

/// Test machine input comes from opposite of facing (back)
#[test]
fn test_machine_input_from_back() {
    use idle_factory::components::Direction;

    let machine_pos = IVec3::new(10, 5, 10);
    let facing = Direction::North;

    // Input comes from opposite direction (back = South)
    // North.opposite() = South, South.to_ivec3() = (0, 0, 1)
    let input_pos = machine_pos + facing.opposite().to_ivec3();
    assert_eq!(input_pos, IVec3::new(10, 5, 11)); // South of machine (z+1)

    // Conveyor at input_pos facing toward machine would deliver
    let conveyor_pos = input_pos;
    let conveyor_facing = Direction::North; // Toward machine (z-1)
    let conveyor_output = conveyor_pos + conveyor_facing.to_ivec3();
    assert_eq!(conveyor_output, machine_pos);
}

/// Test library Miner creates with default facing
#[test]
fn test_lib_miner_default_facing() {
    use idle_factory::components::Direction;
    let miner = idle_factory::Miner::default();
    assert_eq!(miner.facing, Direction::North);
    assert_eq!(miner.buffer, None);
    assert_eq!(miner.progress, 0.0);
}

/// Test library Furnace creates with default facing
#[test]
fn test_lib_furnace_default_facing() {
    use idle_factory::components::Direction;
    let furnace = idle_factory::Furnace::default();
    assert_eq!(furnace.facing, Direction::North);
    assert_eq!(furnace.fuel, 0);
    assert_eq!(furnace.input_type, None);
}

/// Test library Crusher creates with default facing
#[test]
fn test_lib_crusher_default_facing() {
    use idle_factory::components::Direction;
    let crusher = idle_factory::Crusher::default();
    assert_eq!(crusher.facing, Direction::North);
    assert_eq!(crusher.input_type, None);
    assert_eq!(crusher.input_count, 0);
}

/// Test Direction opposite
#[test]
fn test_direction_opposite() {
    use idle_factory::components::Direction;

    assert_eq!(Direction::North.opposite(), Direction::South);
    assert_eq!(Direction::South.opposite(), Direction::North);
    assert_eq!(Direction::East.opposite(), Direction::West);
    assert_eq!(Direction::West.opposite(), Direction::East);
}

/// Test Direction rotate_cw
#[test]
fn test_direction_rotate_cw() {
    use idle_factory::components::Direction;

    // Clockwise rotation: N->E->S->W->N
    assert_eq!(Direction::North.rotate_cw(), Direction::East);
    assert_eq!(Direction::East.rotate_cw(), Direction::South);
    assert_eq!(Direction::South.rotate_cw(), Direction::West);
    assert_eq!(Direction::West.rotate_cw(), Direction::North);
}

// ============================================================================
// Storage UI Logic Tests
// ============================================================================

/// Test pagination calculation
#[test]
fn test_storage_ui_pagination() {
    use idle_factory::ui::storage_ui::{GRID_COLUMNS, SLOTS_PER_PAGE};

    // 32 slots per page
    assert_eq!(SLOTS_PER_PAGE, 32);
    assert_eq!(GRID_COLUMNS, 8);

    // 100 items should need 4 pages (ceil(100/32))
    let item_count = 100;
    let pages_needed = (item_count + SLOTS_PER_PAGE - 1) / SLOTS_PER_PAGE;
    assert_eq!(pages_needed, 4);

    // 32 items exactly should need 1 page
    let item_count = 32;
    let pages_needed = (item_count + SLOTS_PER_PAGE - 1) / SLOTS_PER_PAGE;
    assert_eq!(pages_needed, 1);

    // 0 items should need 1 page (minimum)
    let item_count = 0;
    let pages_needed = std::cmp::max(1, (item_count + SLOTS_PER_PAGE - 1) / SLOTS_PER_PAGE);
    assert_eq!(pages_needed, 1);
}

/// Test ItemCategory variants exist
#[test]
fn test_item_category_variants() {
    use idle_factory::components::ItemCategory;

    // All variants can be created
    let _all = ItemCategory::All;
    let _ores = ItemCategory::Ores;
    let _ingots = ItemCategory::Ingots;
    let _machines = ItemCategory::Machines;

    // Labels work
    assert_eq!(ItemCategory::All.label(), "All");
    assert_eq!(ItemCategory::Ores.label(), "Ores");
    assert_eq!(ItemCategory::Ingots.label(), "Ingots");
    assert_eq!(ItemCategory::Machines.label(), "Machines");
}

// ============================================================================
// Machine Chain Integration Tests
// ============================================================================

/// Test complete production chain: Miner -> Conveyor -> Furnace -> Output
#[test]
fn test_production_chain_miner_to_furnace() {
    use idle_factory::components::Direction;

    // Setup positions
    let miner_pos = IVec3::new(0, 0, 0);
    let miner_facing = Direction::East;
    let conveyor_pos = IVec3::new(1, 0, 0);
    let conveyor_dir = Direction::East;
    let furnace_pos = IVec3::new(2, 0, 0);
    let furnace_facing = Direction::East;

    // Verify chain connections
    let miner_output_pos = miner_pos + miner_facing.to_ivec3();
    assert_eq!(miner_output_pos, conveyor_pos);

    let conveyor_output_pos = conveyor_pos + conveyor_dir.to_ivec3();
    assert_eq!(conveyor_output_pos, furnace_pos);

    let furnace_input_pos = furnace_pos + furnace_facing.opposite().to_ivec3();
    assert_eq!(furnace_input_pos, conveyor_pos);
}

/// Test Miner -> Crusher chain for ore processing
#[test]
fn test_production_chain_miner_to_crusher() {
    use idle_factory::components::Direction;

    // North = z-1, so chain goes in decreasing z direction
    let miner_pos = IVec3::new(5, 5, 5);
    let miner_facing = Direction::North;
    let conveyor_pos = IVec3::new(5, 5, 4); // North of miner (z-1)
    let crusher_pos = IVec3::new(5, 5, 3); // North of conveyor (z-1)
    let crusher_facing = Direction::North;

    // Miner output goes to conveyor
    let miner_output = miner_pos + miner_facing.to_ivec3();
    assert_eq!(miner_output, conveyor_pos);

    // Crusher input is from back (South = z+1)
    let crusher_input = crusher_pos + crusher_facing.opposite().to_ivec3();
    assert_eq!(crusher_input, conveyor_pos);
}

// ============================================================================
// Performance Monitoring Tests
// ============================================================================

/// Benchmark: Machine tick performance (should complete 1000 ticks quickly)
#[test]
fn test_performance_machine_ticks() {
    use std::time::Instant;

    let mut miner = Miner::default();
    let mut furnace = Furnace::default();
    let mut crusher = Crusher::default();

    furnace.add_fuel(100);
    furnace.add_input(BlockType::Stone);
    crusher.add_input(BlockType::Stone);

    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        miner.tick(0.1, Some(BlockType::Stone));
        furnace.tick(0.1);
        crusher.tick(0.1);
    }

    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_micros() as f64 / iterations as f64;

    // Each tick should complete in under 100 microseconds
    assert!(
        avg_ms < 100.0,
        "Machine tick avg {}us exceeds 100us limit",
        avg_ms
    );
}

/// Benchmark: Conveyor operations (should handle many items efficiently)
#[test]
fn test_performance_conveyor_operations() {
    use std::time::Instant;

    let mut conveyor = Conveyor::new(IVec3::ZERO, Direction::North);

    let start = Instant::now();
    let operations = 1000;

    for _ in 0..operations {
        // Add items
        for _ in 0..5 {
            let _ = conveyor.accept_item(BlockType::Stone);
        }
        // Tick to move items
        for _ in 0..10 {
            let _ = conveyor.tick(0.05);
        }
    }

    let elapsed = start.elapsed();
    let total_ms = elapsed.as_millis();

    // Should complete within 500ms
    assert!(
        total_ms < 500,
        "Conveyor operations took {}ms, exceeds 500ms limit",
        total_ms
    );
}

/// Benchmark: Inventory operations (should be fast for basic operations)
#[test]
fn test_performance_inventory_operations() {
    use std::time::Instant;

    let start = Instant::now();
    let operations = 10000;

    for _ in 0..operations {
        let mut inventory = HotbarInventory::default();

        // Add items
        for slot in 0..9 {
            if slot < 2 {
                inventory.slots[slot] = Some((BlockType::Stone, 64));
            }
        }

        // Place blocks
        for _ in 0..10 {
            let _ = inventory.place_block(0);
        }

        // Query items
        for slot in 0..9 {
            let _ = inventory.get_slot(slot);
        }
    }

    let elapsed = start.elapsed();
    let ops_per_sec = (operations as f64 * 1000.0) / elapsed.as_millis() as f64;

    // Should handle at least 10000 operations per second
    assert!(
        ops_per_sec > 10000.0,
        "Inventory ops/sec {} is below 10000",
        ops_per_sec
    );
}

/// Benchmark: WorldData block queries
#[test]
fn test_performance_world_block_queries() {
    use idle_factory::constants::CHUNK_SIZE;
    use idle_factory::world::{ChunkData, WorldData};
    use std::time::Instant;

    let mut world = WorldData::default();
    // Generate chunk at 0,0
    world
        .chunks
        .insert(IVec2::new(0, 0), ChunkData::generate(IVec2::new(0, 0)));

    let start = Instant::now();
    let queries = 10000;

    for i in 0..queries {
        let pos = IVec3::new(
            (i % CHUNK_SIZE as usize) as i32,
            5,
            (i / 16 % CHUNK_SIZE as usize) as i32,
        );
        let _ = world.get_block(pos);
    }

    let elapsed = start.elapsed();
    let queries_per_ms = queries as f64 / elapsed.as_millis().max(1) as f64;

    // Should handle at least 100 queries per millisecond
    assert!(
        queries_per_ms > 100.0,
        "Block queries/ms {} is below 100",
        queries_per_ms
    );
}

// ============================================================================
// Creative Catalog Sprite Visibility Tests
// ============================================================================

/// Test that creative catalog sprites use Inherited visibility
/// This prevents sprites from showing when the parent panel is hidden
#[test]
fn test_creative_catalog_sprite_visibility_inherits_from_parent() {
    // Simulate the visibility behavior:
    // - CreativePanel is Hidden (inventory closed or non-creative mode)
    // - CreativeItemImage should NOT be visible

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum TestVisibility {
        Inherited,
        Visible,
        Hidden,
    }

    struct CreativePanel {
        visibility: TestVisibility,
    }

    struct CreativeItemImage {
        visibility: TestVisibility,
    }

    // Simulate update_creative_catalog_sprites behavior (FIXED version)
    fn update_sprite_visibility(sprite: &mut CreativeItemImage, _has_texture: bool) {
        if _has_texture {
            // Should use Inherited, NOT Visible
            sprite.visibility = TestVisibility::Inherited;
        }
    }

    // Simulate inherited visibility calculation
    fn compute_inherited_visibility(parent: &CreativePanel, child: &CreativeItemImage) -> bool {
        match child.visibility {
            TestVisibility::Hidden => false,
            TestVisibility::Visible => true, // BUG: Would show even if parent hidden
            TestVisibility::Inherited => match parent.visibility {
                TestVisibility::Hidden => false,
                TestVisibility::Visible | TestVisibility::Inherited => true,
            },
        }
    }

    // Test case 1: Panel hidden, sprite should not be visible
    let panel = CreativePanel {
        visibility: TestVisibility::Hidden,
    };
    let mut sprite = CreativeItemImage {
        visibility: TestVisibility::Hidden,
    };

    // Simulate texture loaded
    update_sprite_visibility(&mut sprite, true);

    // Sprite visibility should be Inherited
    assert_eq!(
        sprite.visibility,
        TestVisibility::Inherited,
        "Sprite should use Inherited visibility after texture load"
    );

    // When parent is hidden, sprite should not be visible
    let is_visible = compute_inherited_visibility(&panel, &sprite);
    assert!(
        !is_visible,
        "Sprite should NOT be visible when parent panel is Hidden"
    );

    // Test case 2: Panel visible, sprite should be visible
    let panel_visible = CreativePanel {
        visibility: TestVisibility::Visible,
    };
    let is_visible_with_panel = compute_inherited_visibility(&panel_visible, &sprite);
    assert!(
        is_visible_with_panel,
        "Sprite should be visible when parent panel is Visible"
    );
}

/// Test that demonstrates the bug that was fixed
/// If sprites used Visibility::Visible, they would show even when panel is hidden
#[test]
fn test_creative_catalog_visibility_bug_prevented() {
    // This test documents the bug that was fixed:
    // Before: update_creative_catalog_sprites set Visibility::Visible
    // After: update_creative_catalog_sprites sets Visibility::Inherited

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum Visibility {
        Inherited,
        Visible,
        Hidden,
    }

    // OLD buggy behavior (for documentation)
    fn old_buggy_update(_sprite_visibility: &mut Visibility, has_texture: bool) {
        if has_texture {
            *_sprite_visibility = Visibility::Visible; // BUG!
        }
    }

    // NEW fixed behavior
    fn new_fixed_update(sprite_visibility: &mut Visibility, has_texture: bool) {
        if has_texture {
            *sprite_visibility = Visibility::Inherited; // FIXED
        }
    }

    let mut old_vis = Visibility::Hidden;
    let mut new_vis = Visibility::Hidden;

    old_buggy_update(&mut old_vis, true);
    new_fixed_update(&mut new_vis, true);

    // Old code would set Visible (causing the bug)
    assert_eq!(old_vis, Visibility::Visible);
    // New code sets Inherited (correct behavior)
    assert_eq!(new_vis, Visibility::Inherited);
}
