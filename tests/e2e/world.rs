//! World and chunk tests

use super::common::*;
use bevy::prelude::*;
use idle_factory::core::items;

#[test]
fn test_world_generation() {
    let chunk = TestChunkData::default();

    let expected_blocks = 16 * 16 * 8;
    assert_eq!(chunk.blocks.len(), expected_blocks);

    let grass_count = chunk
        .blocks
        .values()
        .filter(|&&b| b == items::grass())
        .count();
    let stone_count = chunk
        .blocks
        .values()
        .filter(|&&b| b == items::stone())
        .count();

    assert_eq!(grass_count, 16 * 16);
    assert_eq!(stone_count, 16 * 16 * 7);
}

#[test]
fn test_block_mining_adds_to_inventory() {
    let mut chunk = TestChunkData::default();
    let mut inventory = TestInventory::default();

    let block_pos = IVec3::new(0, 7, 0);
    if let Some(block_type) = chunk.blocks.remove(&block_pos) {
        *inventory.items.entry(block_type).or_insert(0) += 1;
    }

    assert!(!chunk.blocks.contains_key(&block_pos));
    assert_eq!(inventory.items.get(&items::grass()), Some(&1));
}

#[test]
fn test_multiple_blocks_mining() {
    let mut chunk = TestChunkData::default();
    let mut inventory = TestInventory::default();

    for x in 0..3 {
        let block_pos = IVec3::new(x, 7, 0);
        if let Some(block_type) = chunk.blocks.remove(&block_pos) {
            *inventory.items.entry(block_type).or_insert(0) += 1;
        }
    }

    for x in 0..2 {
        let block_pos = IVec3::new(x, 0, 0);
        if let Some(block_type) = chunk.blocks.remove(&block_pos) {
            *inventory.items.entry(block_type).or_insert(0) += 1;
        }
    }

    assert_eq!(inventory.items.get(&items::grass()), Some(&3));
    assert_eq!(inventory.items.get(&items::stone()), Some(&2));
}

#[test]
fn test_ray_aabb_intersection() {
    // Test hit from directly in front
    let result = ray_aabb_intersection(
        Vec3::new(0.0, 0.0, -5.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, 0.5),
    );
    assert!(result.is_some());
    let t = result.unwrap();
    assert!(t > 4.0 && t < 5.0);

    // Test miss
    let result = ray_aabb_intersection(
        Vec3::new(0.0, 0.0, -5.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, 0.5),
    );
    assert!(result.is_none());
}

#[test]
fn test_bevy_app_builds() {
    let _app = App::new().add_plugins(MinimalPlugins);
}

#[test]
fn test_block_break_no_freeze() {
    let mut chunk = TestChunkData::default();
    let mut inventory = TestInventory::default();

    for x in 0..10 {
        let block_pos = IVec3::new(x, 7, 0);
        if let Some(block_type) = chunk.blocks.remove(&block_pos) {
            *inventory.items.entry(block_type).or_insert(0) += 1;
        }
    }

    for x in 0..10 {
        assert!(!chunk.blocks.contains_key(&IVec3::new(x, 7, 0)));
    }
    assert_eq!(inventory.items.get(&items::grass()), Some(&10));
}
