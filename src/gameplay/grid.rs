use bevy::prelude::*;
use std::collections::HashMap;

// 方角の定義
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    North, // Z-
    South, // Z+
    East,  // X+
    West,  // X-
}

impl Direction {
    pub fn to_ivec3(&self) -> IVec3 {
        match self {
            Direction::North => IVec3::new(0, 0, -1),
            Direction::South => IVec3::new(0, 0, 1),
            Direction::East  => IVec3::new(1, 0, 0),
            Direction::West  => IVec3::new(-1, 0, 0),
        }
    }
    
    pub fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East  => Direction::West,
            Direction::West  => Direction::East,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ItemSlot {
    pub item_id: String,
    pub count: u32,
    pub progress: f32,
    pub unique_id: u64,
    pub from_direction: Option<Direction>,
}

// ★修正: 1つだけにする。progressを含める。
#[derive(Clone, Debug)]
pub struct MachineInstance {
    pub id: String,
    pub orientation: Direction,
    pub inventory: Vec<ItemSlot>,
    pub progress: f32, // ★これが最新の定義
}

#[derive(Resource, Default)]
pub struct SimulationGrid {
    pub machines: HashMap<IVec3, MachineInstance>,
}