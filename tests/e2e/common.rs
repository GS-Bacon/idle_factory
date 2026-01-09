//! Common test helpers and types for E2E tests

#![allow(dead_code)]

use bevy::prelude::*;
use idle_factory::constants::{CHUNK_SIZE, HOTBAR_SLOTS};
use idle_factory::core::items;
use idle_factory::ItemId;
use std::collections::HashMap;

// ============================================================================
// Basic Test Types
// ============================================================================

/// Simple inventory for tests (not the real game inventory)
#[derive(Resource, Default)]
pub struct TestInventory {
    pub items: HashMap<ItemId, u32>,
}

/// Simple chunk data for tests
#[derive(Resource)]
pub struct TestChunkData {
    pub blocks: HashMap<IVec3, ItemId>,
}

impl Default for TestChunkData {
    fn default() -> Self {
        let mut blocks = HashMap::new();
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..8 {
                    let item_id = if y == 7 {
                        items::grass()
                    } else {
                        items::stone()
                    };
                    blocks.insert(IVec3::new(x, y, z), item_id);
                }
            }
        }
        Self { blocks }
    }
}

// ============================================================================
// Slot-based Inventory (matching game implementation)
// ============================================================================

/// Slot-based inventory for tests
#[derive(Clone)]
pub struct SlotInventory {
    pub slots: [Option<(ItemId, u32)>; HOTBAR_SLOTS],
    pub selected_slot: usize,
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
    pub fn get_slot(&self, slot: usize) -> Option<ItemId> {
        self.slots.get(slot).and_then(|s| s.map(|(bt, _)| bt))
    }

    pub fn get_slot_count(&self, slot: usize) -> u32 {
        self.slots
            .get(slot)
            .and_then(|s| s.map(|(_, c)| c))
            .unwrap_or(0)
    }

    pub fn selected_block(&self) -> Option<ItemId> {
        self.get_slot(self.selected_slot)
    }

    pub fn add_item(&mut self, block_type: ItemId, amount: u32) -> bool {
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

    pub fn consume_selected(&mut self) -> Option<ItemId> {
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

    pub fn consume_item(&mut self, block_type: ItemId, amount: u32) -> bool {
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

    pub fn has_selected(&self) -> bool {
        self.slots
            .get(self.selected_slot)
            .and_then(|s| s.as_ref())
            .map(|(_, c)| *c > 0)
            .unwrap_or(false)
    }
}

// ============================================================================
// Hotbar State
// ============================================================================

#[derive(Resource, Default)]
pub struct HotbarState {
    pub selected_index: Option<usize>,
}

impl HotbarState {
    pub fn select(&mut self, index: usize) {
        if index < 9 {
            self.selected_index = Some(index);
        }
    }

    pub fn deselect(&mut self) {
        self.selected_index = None;
    }
}

// ============================================================================
// Hotbar Inventory
// ============================================================================

#[derive(Resource)]
pub struct HotbarInventory {
    pub slots: Vec<Option<(ItemId, u32)>>,
}

impl Default for HotbarInventory {
    fn default() -> Self {
        Self {
            slots: vec![
                Some((items::stone(), 64)),
                Some((items::grass(), 32)),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        }
    }
}

impl HotbarInventory {
    pub fn place_block(&mut self, slot: usize) -> Option<ItemId> {
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

    pub fn get_slot(&self, slot: usize) -> Option<(ItemId, u32)> {
        if slot >= 9 {
            return None;
        }
        self.slots[slot]
    }
}

// ============================================================================
// Simple UI State
// ============================================================================

#[derive(Resource, Default)]
pub struct TestUIState {
    pub furnace_ui_open: bool,
}

impl TestUIState {
    pub fn toggle_furnace_ui(&mut self) {
        self.furnace_ui_open = !self.furnace_ui_open;
    }

    pub fn close_ui(&mut self) {
        self.furnace_ui_open = false;
    }
}

// ============================================================================
// Ray-AABB Intersection Helper
// ============================================================================

pub fn ray_aabb_intersection(
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
