//! Machine component tests

use bevy::prelude::*;
use idle_factory::core::items;
use idle_factory::ItemId;
use std::collections::HashMap;

// ============================================================================
// Test Machine Types
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn to_ivec3(&self) -> IVec3 {
        match self {
            Direction::North => IVec3::new(0, 0, -1),
            Direction::South => IVec3::new(0, 0, 1),
            Direction::East => IVec3::new(1, 0, 0),
            Direction::West => IVec3::new(-1, 0, 0),
        }
    }
}

/// Miner component for testing
pub struct Miner {
    pub position: IVec3,
    pub progress: f32,
    pub buffer: Option<(ItemId, u32)>,
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
    pub fn tick(&mut self, delta_seconds: f32, ore_type: Option<ItemId>) -> bool {
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

    pub fn take_output(&mut self) -> Option<ItemId> {
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
pub struct Conveyor {
    pub position: IVec3,
    pub direction: Direction,
    pub item: Option<ItemId>,
    pub progress: f32,
}

impl Conveyor {
    pub fn new(position: IVec3, direction: Direction) -> Self {
        Self {
            position,
            direction,
            item: None,
            progress: 0.0,
        }
    }

    pub fn accept_item(&mut self, item: ItemId) -> bool {
        if self.item.is_none() {
            self.item = Some(item);
            self.progress = 0.0;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self, delta_seconds: f32) -> Option<ItemId> {
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

    #[allow(dead_code)]
    pub fn output_position(&self) -> IVec3 {
        self.position + self.direction.to_ivec3()
    }
}

/// Furnace component for testing
pub struct Furnace {
    pub fuel: u32,
    pub input_type: Option<ItemId>,
    pub input_count: u32,
    pub output_type: Option<ItemId>,
    pub output_count: u32,
    pub progress: f32,
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
    pub fn add_fuel(&mut self, count: u32) {
        self.fuel += count;
    }

    pub fn add_input(&mut self, ore_type: ItemId) -> bool {
        if self.input_type.is_none() || self.input_type == Some(ore_type) {
            self.input_type = Some(ore_type);
            self.input_count += 1;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self, delta_seconds: f32) -> bool {
        const SMELT_TIME: f32 = 3.0;

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
            self.output_type = Some(items::stone());
            self.output_count += 1;
            true
        } else {
            false
        }
    }

    pub fn take_output(&mut self) -> Option<ItemId> {
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

/// Crusher component for testing
pub struct Crusher {
    pub input_type: Option<ItemId>,
    pub input_count: u32,
    pub output_type: Option<ItemId>,
    pub output_count: u32,
    pub progress: f32,
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
    pub fn add_input(&mut self, ore_type: ItemId) -> bool {
        if self.input_type.is_none() || self.input_type == Some(ore_type) {
            self.input_type = Some(ore_type);
            self.input_count += 1;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self, delta_seconds: f32) -> bool {
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
            self.output_type = Some(ore);
            self.output_count += 2;
            true
        } else {
            false
        }
    }

    pub fn take_output(&mut self) -> Option<ItemId> {
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

// ============================================================================
// Entity Cleanup Types
// ============================================================================

pub struct EntityManager {
    entities: HashMap<u32, EntityData>,
    next_id: u32,
}

struct EntityData {
    entity_type: EntityType,
    #[allow(dead_code)]
    children: Vec<u32>,
    item_visual: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EntityType {
    Conveyor,
    #[allow(dead_code)]
    Miner,
    #[allow(dead_code)]
    Furnace,
    ItemVisual,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn spawn(&mut self, entity_type: EntityType) -> u32 {
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

    pub fn spawn_conveyor_with_item(&mut self) -> (u32, u32) {
        let conveyor_id = self.spawn(EntityType::Conveyor);
        let item_id = self.spawn(EntityType::ItemVisual);

        if let Some(conveyor) = self.entities.get_mut(&conveyor_id) {
            conveyor.item_visual = Some(item_id);
        }

        (conveyor_id, item_id)
    }

    pub fn despawn_with_cleanup(&mut self, id: u32) {
        if let Some(entity) = self.entities.remove(&id) {
            for child_id in entity.children {
                self.entities.remove(&child_id);
            }
            if let Some(visual_id) = entity.item_visual {
                self.entities.remove(&visual_id);
            }
        }
    }

    pub fn despawn_without_cleanup(&mut self, id: u32) {
        self.entities.remove(&id);
    }

    pub fn exists(&self, id: u32) -> bool {
        self.entities.contains_key(&id)
    }

    pub fn count_by_type(&self, entity_type: EntityType) -> usize {
        self.entities
            .values()
            .filter(|e| e.entity_type == entity_type)
            .count()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_miner_mining_cycle() {
    let mut miner = Miner::default();

    assert_eq!(miner.progress, 0.0);
    assert!(miner.buffer.is_none());

    let ore_type = Some(items::stone());

    assert!(!miner.tick(2.0, ore_type));
    assert!(miner.buffer.is_none());
    assert!(miner.progress >= 0.4 - 0.01);
    assert!(miner.progress <= 0.4 + 0.01);

    assert!(miner.tick(3.0, ore_type));
    assert_eq!(miner.buffer, Some((items::stone(), 1)));
    assert!(miner.progress < 0.01);

    assert_eq!(miner.take_output(), Some(items::stone()));
    assert!(miner.buffer.is_none());
}

#[test]
fn test_miner_no_ore_below() {
    let mut miner = Miner::default();

    assert!(!miner.tick(10.0, None));
    assert!(miner.buffer.is_none());
}

#[test]
fn test_conveyor_item_transfer() {
    let mut conv = Conveyor::new(IVec3::new(5, 8, 5), Direction::East);

    assert!(conv.accept_item(items::stone()));
    assert_eq!(conv.item, Some(items::stone()));

    assert!(!conv.accept_item(items::grass()));

    assert!(conv.tick(0.3).is_none());
    assert_eq!(conv.tick(0.3), Some(items::stone()));
    assert!(conv.item.is_none());
}

#[test]
fn test_conveyor_chain() {
    let mut miner = Miner::default();
    miner.buffer = Some((items::stone(), 1));

    let mut conv1 = Conveyor::new(IVec3::new(6, 8, 5), Direction::East);
    let mut conv2 = Conveyor::new(IVec3::new(7, 8, 5), Direction::East);

    if let Some(item) = miner.take_output() {
        assert!(conv1.accept_item(item));
    }

    if let Some(item) = conv1.tick(0.5) {
        assert!(conv2.accept_item(item));
    }

    let output = conv2.tick(0.5);
    assert_eq!(output, Some(items::stone()));
}

#[test]
fn test_furnace_smelting() {
    let mut furnace = Furnace::default();

    assert_eq!(furnace.progress, 0.0);
    assert_eq!(furnace.fuel, 0);
    assert!(furnace.input_type.is_none());
    assert!(furnace.output_type.is_none());

    furnace.add_fuel(1);
    assert_eq!(furnace.fuel, 1);
    furnace.add_input(items::stone());
    assert!(furnace.input_type.is_some());

    assert!(!furnace.tick(2.0));
    assert!(furnace.progress >= 0.6 - 0.1);

    assert!(furnace.tick(1.0));
    assert!(furnace.progress < 0.01);

    assert_eq!(furnace.output_count, 1);
    assert_eq!(furnace.take_output(), Some(items::stone()));
    assert_eq!(furnace.output_count, 0);
    assert_eq!(furnace.fuel, 0);
}

#[test]
fn test_furnace_no_fuel() {
    let mut furnace = Furnace::default();
    furnace.add_input(items::stone());

    assert!(!furnace.tick(10.0));
    assert_eq!(furnace.output_count, 0);
}

#[test]
fn test_crusher_doubles_output() {
    let mut crusher = Crusher::default();

    assert_eq!(crusher.progress, 0.0);
    assert!(crusher.input_type.is_none());
    assert!(crusher.output_type.is_none());
    assert_eq!(crusher.output_count, 0);

    crusher.add_input(items::stone());
    assert!(crusher.input_type.is_some());
    assert_eq!(crusher.input_count, 1);

    assert!(crusher.tick(2.0));
    assert!(crusher.progress < 0.01);

    assert_eq!(crusher.output_count, 2);
    assert!(crusher.input_type.is_none());

    assert_eq!(crusher.take_output(), Some(items::stone()));
    assert_eq!(crusher.output_count, 1);

    assert_eq!(crusher.take_output(), Some(items::stone()));
    assert_eq!(crusher.output_count, 0);

    assert!(crusher.take_output().is_none());
}

#[test]
fn test_conveyor_destroy_cleans_item_visual() {
    let mut manager = EntityManager::new();

    let (conveyor_id, item_id) = manager.spawn_conveyor_with_item();

    assert!(manager.exists(conveyor_id));
    assert!(manager.exists(item_id));
    assert_eq!(manager.count_by_type(EntityType::ItemVisual), 1);

    manager.despawn_with_cleanup(conveyor_id);

    assert!(!manager.exists(conveyor_id));
    assert!(!manager.exists(item_id));
    assert_eq!(manager.count_by_type(EntityType::ItemVisual), 0);
}

#[test]
fn test_conveyor_destroy_bug_detection() {
    let mut manager = EntityManager::new();

    let (conveyor_id, item_id) = manager.spawn_conveyor_with_item();

    manager.despawn_without_cleanup(conveyor_id);

    assert!(!manager.exists(conveyor_id));
    assert!(manager.exists(item_id)); // This is the bug
    assert_eq!(manager.count_by_type(EntityType::ItemVisual), 1);
}

#[test]
fn test_multiple_conveyors_cleanup() {
    let mut manager = EntityManager::new();

    let mut pairs = Vec::new();
    for _ in 0..5 {
        pairs.push(manager.spawn_conveyor_with_item());
    }

    assert_eq!(manager.count_by_type(EntityType::Conveyor), 5);
    assert_eq!(manager.count_by_type(EntityType::ItemVisual), 5);

    for (conveyor_id, _) in pairs {
        manager.despawn_with_cleanup(conveyor_id);
    }

    assert_eq!(manager.count_by_type(EntityType::Conveyor), 0);
    assert_eq!(manager.count_by_type(EntityType::ItemVisual), 0);
}
