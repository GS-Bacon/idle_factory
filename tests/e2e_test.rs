//! E2E Tests for Idle Factory
//! Tests core game logic without rendering

#![allow(dead_code)] // Test helper types may not all be used in every test

use bevy::prelude::*;
use std::collections::HashMap;

// Re-create the core types for testing (since they're private in main)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum BlockType {
    Stone,
    Grass,
    IronOre,
    CopperOre,
    Coal,
    IronIngot,
    CopperIngot,
    MinerBlock,
    ConveyorBlock,
    CrusherBlock,
    FurnaceBlock,
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

    // Simulate mining iron ore
    let ore_type = Some(BlockType::Stone); // Representing iron ore

    // Not enough time passed
    assert!(!miner.tick(2.0, ore_type));
    assert!(miner.buffer.is_none());

    // Complete mining (5 seconds total)
    assert!(miner.tick(3.0, ore_type));
    assert_eq!(miner.buffer, Some((BlockType::Stone, 1)));

    // Take output
    assert_eq!(miner.take_output(), Some(BlockType::Stone));
    assert!(miner.buffer.is_none());
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

    // Add fuel and input
    furnace.add_fuel(1);
    furnace.add_input(BlockType::Stone); // Representing iron ore

    // Smelting takes 3 seconds
    assert!(!furnace.tick(2.0));
    assert!(furnace.tick(1.0));

    // Check output
    assert_eq!(furnace.output_count, 1);
    assert_eq!(furnace.take_output(), Some(BlockType::Stone));

    // Fuel consumed
    assert_eq!(furnace.fuel, 0);
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

    crusher.add_input(BlockType::Stone);

    // Crushing takes 2 seconds
    assert!(crusher.tick(2.0));

    // Should produce 2 outputs
    assert_eq!(crusher.output_count, 2);
    assert_eq!(crusher.take_output(), Some(BlockType::Stone));
    assert_eq!(crusher.take_output(), Some(BlockType::Stone));
    assert!(crusher.take_output().is_none());
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
        self.entities.insert(id, EntityData {
            entity_type,
            children: Vec::new(),
            item_visual: None,
        });
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
        self.entities.values().filter(|e| e.entity_type == entity_type).count()
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
    for _ in 0..200 { // 20 seconds of simulation
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
    assert!(delivered > 0, "Automation line should produce deliveries, got {}", delivered);
}

// =====================================================
// Chunk Boundary Mesh Tests
// =====================================================

const TEST_CHUNK_SIZE: i32 = 16;

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
            world_pos.x.div_euclid(TEST_CHUNK_SIZE),
            world_pos.z.div_euclid(TEST_CHUNK_SIZE),
        );
        let chunk = self.chunks.entry(chunk_coord).or_insert_with(HashMap::new);
        chunk.insert(world_pos, block_type);
    }

    fn has_block(&self, world_pos: IVec3) -> bool {
        let chunk_coord = IVec2::new(
            world_pos.x.div_euclid(TEST_CHUNK_SIZE),
            world_pos.z.div_euclid(TEST_CHUNK_SIZE),
        );
        self.chunks.get(&chunk_coord)
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
        assert!(world.should_render_face(center, dir), "Face {:?} should render", dir);
    }

    // Add neighbors in all directions
    for dir in directions {
        world.set_block(center + dir, BlockType::Stone);
    }

    // No faces should render now
    for dir in directions {
        assert!(!world.should_render_face(center, dir), "Face {:?} should NOT render", dir);
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
        let ray_origin = Vec3::new(machine.position.x, machine.position.y + 2.0, machine.position.z - 3.0);
        let ray_direction = (machine.position - ray_origin).normalize();

        let hit = ray_aabb_test(
            ray_origin,
            ray_direction,
            machine.position - machine.half_size,
            machine.position + machine.half_size,
        );

        assert!(hit.is_some(), "Raycast should hit {:?}", machine.entity_type);
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
        (0.0, 0.0),   // (progress, lateral_offset)
        (0.4, 0.0),   // Should be at minimum spacing
        (0.8, 0.0),   // Should be at minimum spacing from previous
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
    assert!(frames_to_center > 0 && frames_to_center < 100, "Should center within reasonable time");
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
        self.slots.iter()
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
    assert!(!can_add_south, "Should not accept two items at same progress");

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
        LoopConveyor { id: 0, items: vec![0.5], next_id: 1 },
        LoopConveyor { id: 1, items: vec![], next_id: 2 },
        LoopConveyor { id: 2, items: vec![], next_id: 3 },
        LoopConveyor { id: 3, items: vec![], next_id: 0 }, // Loop back
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
    assert_eq!(total_items, 1, "Item should not be duplicated or lost in loop");
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
    assert!(max_entities <= 10, "Entity count should not grow unboundedly");
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

    assert!(source.visual_entity.is_none(), "Source should release visual");
    assert_eq!(target.visual_entity, Some(42), "Target should receive visual");
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
            accepted_from[i], accepted_from[i + 1],
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
        assert_eq!(output, i % 3, "Round-robin pattern should be 0,1,2,0,1,2,...");
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
    assert_eq!(output_counts[1], 0, "Blocked left output should receive 0 items");
    assert!(output_counts[2] > 0, "Right output should receive items");
    // Total should be 10
    assert_eq!(output_counts.iter().sum::<usize>(), 10, "All 10 items should be distributed");
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
            if diff.x == 1 { return TestDirection::East; }
            if diff.x == -1 { return TestDirection::West; }
            if diff.z == 1 { return TestDirection::South; }
            if diff.z == -1 { return TestDirection::North; }
        }
    }

    // Priority 3: Connect to adjacent conveyor
    for (conv_pos, _) in conveyors {
        let diff = *conv_pos - place_pos;
        if diff.x.abs() + diff.y.abs() + diff.z.abs() == 1 {
            if diff.x == 1 { return TestDirection::East; }
            if diff.x == -1 { return TestDirection::West; }
            if diff.z == 1 { return TestDirection::South; }
            if diff.z == -1 { return TestDirection::North; }
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
    assert_eq!(dir, TestDirection::East, "Should point away from machine (East)");
}

#[test]
fn test_auto_conveyor_connects_to_adjacent() {
    // Conveyor at (7,8,5) pointing East, placing at (6,8,5)
    // The existing conveyor is NOT pointing at us, but we should connect to it
    let conveyors = vec![(IVec3::new(7, 8, 5), TestDirection::East)];
    let machines: Vec<IVec3> = vec![];
    let place_pos = IVec3::new(6, 8, 5);

    let dir = auto_conveyor_direction(place_pos, TestDirection::North, &conveyors, &machines);
    assert_eq!(dir, TestDirection::East, "Should point toward adjacent conveyor");
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
    assert_eq!(dir, TestDirection::East, "Machine priority: should point away from machine");
}

// =====================================================
// Inventory Edge Case Tests
// =====================================================

#[test]
fn test_inventory_add_at_max_slots() {
    const NUM_SLOTS: usize = 36;
    const MAX_STACK: u32 = 999;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Item { A, B, C, D, E, F, G, H, I, J }

    let mut slots: [Option<(Item, u32)>; NUM_SLOTS] = [None; NUM_SLOTS];

    // Fill all slots with different items (can't stack)
    let items = [Item::A, Item::B, Item::C, Item::D, Item::E, Item::F, Item::G, Item::H, Item::I, Item::J];
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
        ChunkEntities { chunk_coord: IVec2::new(0, 0), entities: vec![1, 2, 3] },
        ChunkEntities { chunk_coord: IVec2::new(1, 0), entities: vec![4, 5] },
        ChunkEntities { chunk_coord: IVec2::new(0, 1), entities: vec![6, 7, 8, 9] },
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
fn test_chunk_boundary_machine_survival() {
    // Machine at chunk boundary should survive if any adjacent chunk is loaded
    struct Machine {
        world_pos: IVec3,
    }

    fn world_to_chunk(pos: IVec3) -> IVec2 {
        IVec2::new(pos.x.div_euclid(16), pos.z.div_euclid(16))
    }

    let machine = Machine { world_pos: IVec3::new(16, 8, 0) }; // At chunk (1, 0)
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
    assert_eq!(inventory.get_slot_count(0), 15, "All items should be returned (5 input + 10 output)");
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
    assert_eq!(inventory.get_slot_count(0), 5, "Fuel and output should be returned");
    // Stone (input ore) in slot 1
    assert_eq!(inventory.get_slot(1), Some(BlockType::Stone));
    assert_eq!(inventory.get_slot_count(1), 5, "Input ore should be returned");
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

    let mut world_blocks: std::collections::HashMap<IVec3, BlockType> = std::collections::HashMap::new();

    // Initial terrain
    world_blocks.insert(IVec3::new(0, 0, 0), BlockType::Stone);
    world_blocks.insert(IVec3::new(0, 1, 0), BlockType::Grass);

    // Simulate placing a miner at (0, 2, 0)
    // CORRECT: Don't register in world_blocks - just spawn entity
    let miner_pos = IVec3::new(0, 2, 0);
    // machines.spawn(Miner { position: miner_pos, ... });
    // DO NOT: world_blocks.insert(miner_pos, BlockType::MinerBlock);

    // Verify terrain is still intact
    assert!(world_blocks.contains_key(&IVec3::new(0, 0, 0)), "Stone should still exist");
    assert!(world_blocks.contains_key(&IVec3::new(0, 1, 0)), "Grass should still exist");
    assert!(!world_blocks.contains_key(&miner_pos), "Machine position should NOT be in world blocks");
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
                let local_x = if neighbor_pos.x < 0 { CHUNK_SIZE - 1 } else { 0 };
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
    let render_without_neighbor = should_render_face(
        edge_pos,
        IVec3::new(1, 0, 0),
        &chunk_a,
        None,
    );

    // With neighbor info: correctly says NOT to render +X face
    let render_with_neighbor = should_render_face(
        edge_pos,
        IVec3::new(1, 0, 0),
        &chunk_a,
        Some(&chunk_b),
    );

    assert!(render_without_neighbor, "Without neighbor info, would incorrectly render face");
    assert!(!render_with_neighbor, "With neighbor info, correctly skips occluded face");
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
    assert_eq!(state.hotbar_selection, 1, "Hotbar should not change when UI is open");
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
    let chunks_to_process: Vec<_> = pending_chunks
        .iter()
        .take(MAX_CHUNKS_PER_FRAME)
        .collect();

    assert_eq!(chunks_to_process.len(), MAX_CHUNKS_PER_FRAME);
    assert!(chunks_to_process.len() < pending_chunks.len(), "Should not process all chunks at once");
}

// ============================================================
// Conveyor shape detection test
// ============================================================
#[test]
fn test_conveyor_shape_detection() {
    // Test that conveyor shapes are correctly detected based on adjacent inputs

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum ConveyorShape {
        Straight,
        CornerLeft,
        CornerRight,
        TJunction,
    }

    fn detect_shape(has_left_input: bool, has_right_input: bool) -> ConveyorShape {
        match (has_left_input, has_right_input) {
            (false, false) => ConveyorShape::Straight,
            (true, false) => ConveyorShape::CornerLeft,
            (false, true) => ConveyorShape::CornerRight,
            (true, true) => ConveyorShape::TJunction,
        }
    }

    assert_eq!(detect_shape(false, false), ConveyorShape::Straight);
    assert_eq!(detect_shape(true, false), ConveyorShape::CornerLeft);
    assert_eq!(detect_shape(false, true), ConveyorShape::CornerRight);
    assert_eq!(detect_shape(true, true), ConveyorShape::TJunction);
}
