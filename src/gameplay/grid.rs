use bevy::prelude::*;
use std::collections::HashMap;
use crate::gameplay::machines::{
    conveyor::Conveyor,
    miner::Miner,
    assembler::Assembler,
};

// --- Common Data Structures ---

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


// --- Machine-Specific Data ---

#[derive(Debug, Clone)]
pub enum Machine {
    Conveyor(Conveyor),
    Miner(Miner),
    Assembler(Assembler),
}

// The generic machine container on the grid
#[derive(Clone, Debug, Component)] // Added Component derive
pub struct MachineInstance {
    pub id: String, // Block ID, e.g., "conveyor", "miner"
    pub orientation: Direction,
    pub machine_type: Machine,
    pub power_node: Option<Entity>, // Placeholder for the kinetic power system
}


// --- Grid Resource ---

#[derive(Resource, Default)]
pub struct SimulationGrid {
    pub machines: HashMap<IVec3, MachineInstance>,
}