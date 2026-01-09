//! Automation line and conveyor tests

use super::machines::*;
use bevy::prelude::*;
use idle_factory::constants::CHUNK_SIZE;
use idle_factory::core::items;
use idle_factory::ItemId;
use std::collections::HashMap;

// ============================================================================
// Test World Data (for chunk boundary tests)
// ============================================================================

struct TestWorldData {
    chunks: HashMap<IVec2, HashMap<IVec3, ItemId>>,
}

impl TestWorldData {
    fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    fn set_block(&mut self, world_pos: IVec3, item_id: ItemId) {
        let chunk_coord = IVec2::new(
            world_pos.x.div_euclid(CHUNK_SIZE),
            world_pos.z.div_euclid(CHUNK_SIZE),
        );
        let chunk = self.chunks.entry(chunk_coord).or_insert_with(HashMap::new);
        chunk.insert(world_pos, item_id);
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

    fn should_render_face(&self, block_pos: IVec3, face_direction: IVec3) -> bool {
        let neighbor_pos = block_pos + face_direction;
        !self.has_block(neighbor_pos)
    }
}

// ============================================================================
// Delivery Platform
// ============================================================================

struct DeliveryPlatform {
    delivered: HashMap<ItemId, u32>,
}

impl DeliveryPlatform {
    fn new() -> Self {
        Self {
            delivered: HashMap::new(),
        }
    }

    fn deliver(&mut self, item: ItemId) {
        *self.delivered.entry(item).or_insert(0) += 1;
    }

    fn get_delivered(&self, item: ItemId) -> u32 {
        *self.delivered.get(&item).unwrap_or(&0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_full_automation_line() {
    let mut miner = Miner {
        position: IVec3::new(5, 8, 5),
        progress: 0.0,
        buffer: None,
    };

    let mut conv1 = Conveyor::new(IVec3::new(6, 8, 5), Direction::East);
    let mut crusher = Crusher::default();
    let mut conv2 = Conveyor::new(IVec3::new(8, 8, 5), Direction::East);
    let mut furnace = Furnace::default();
    furnace.add_fuel(10);
    let mut conv3 = Conveyor::new(IVec3::new(10, 8, 5), Direction::East);
    let mut platform = DeliveryPlatform::new();

    let delta = 0.1;
    for _ in 0..200 {
        miner.tick(delta, Some(items::stone()));

        if conv1.item.is_none() {
            if let Some(item) = miner.take_output() {
                conv1.accept_item(item);
            }
        }

        if let Some(item) = conv1.tick(delta) {
            crusher.add_input(item);
        }

        crusher.tick(delta);

        if conv2.item.is_none() {
            if let Some(item) = crusher.take_output() {
                conv2.accept_item(item);
            }
        }

        if let Some(item) = conv2.tick(delta) {
            furnace.add_input(item);
        }

        furnace.tick(delta);

        if conv3.item.is_none() {
            if let Some(item) = furnace.take_output() {
                conv3.accept_item(item);
            }
        }

        if let Some(item) = conv3.tick(delta) {
            platform.deliver(item);
        }
    }

    let delivered = platform.get_delivered(items::stone());
    assert!(delivered > 0, "Automation line should produce deliveries");
}

#[test]
fn test_chunk_boundary_faces() {
    let mut world = TestWorldData::new();

    let boundary_block = IVec3::new(15, 5, 5);
    world.set_block(boundary_block, items::stone());

    assert!(world.should_render_face(boundary_block, IVec3::new(1, 0, 0)));

    let adjacent_block = IVec3::new(16, 5, 5);
    world.set_block(adjacent_block, items::stone());

    assert!(!world.should_render_face(boundary_block, IVec3::new(1, 0, 0)));
    assert!(!world.should_render_face(adjacent_block, IVec3::new(-1, 0, 0)));
}

#[test]
fn test_chunk_boundary_all_directions() {
    let mut world = TestWorldData::new();

    let center = IVec3::new(8, 5, 8);
    world.set_block(center, items::stone());

    // All 6 faces should render (no neighbors)
    let directions = [
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 0, 0),
        IVec3::new(0, 1, 0),
        IVec3::new(0, -1, 0),
        IVec3::new(0, 0, 1),
        IVec3::new(0, 0, -1),
    ];

    for dir in directions {
        assert!(
            world.should_render_face(center, dir),
            "Face {:?} should render",
            dir
        );
    }

    // Add all neighbors
    for dir in directions {
        world.set_block(center + dir, items::stone());
    }

    // No faces should render (all neighbors exist)
    for dir in directions {
        assert!(
            !world.should_render_face(center, dir),
            "Face {:?} should not render",
            dir
        );
    }
}

#[test]
fn test_chunk_boundary_z_axis() {
    let mut world = TestWorldData::new();

    let boundary_block = IVec3::new(8, 5, 15);
    world.set_block(boundary_block, items::stone());

    assert!(world.should_render_face(boundary_block, IVec3::new(0, 0, 1)));

    let adjacent_block = IVec3::new(8, 5, 16);
    world.set_block(adjacent_block, items::stone());

    assert!(!world.should_render_face(boundary_block, IVec3::new(0, 0, 1)));
}
