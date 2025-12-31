//! Machine component definitions

use crate::block_type::BlockType;
use bevy::prelude::*;
use std::collections::HashMap;

/// Maximum stack size for machine slots
pub const MAX_MACHINE_STACK: u32 = 64;

/// Furnace component - smelts ore into ingots
#[derive(Component)]
pub struct Furnace {
    /// Fuel slot (coal)
    pub fuel: u32,
    /// Input slot - stores ore type and count
    pub input_type: Option<BlockType>,
    pub input_count: u32,
    /// Output slot - stores ingot type and count
    pub output_type: Option<BlockType>,
    pub output_count: u32,
    /// Smelting progress (0.0-1.0)
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
    /// Get smelt output for an ore type
    pub fn get_smelt_output(ore: BlockType) -> Option<BlockType> {
        match ore {
            BlockType::IronOre => Some(BlockType::IronIngot),
            BlockType::CopperOre => Some(BlockType::CopperIngot),
            _ => None,
        }
    }

    /// Check if this ore type can be added to input (same type or empty, within stack limit)
    pub fn can_add_input(&self, ore: BlockType) -> bool {
        let type_ok = self.input_type.is_none() || self.input_type == Some(ore);
        let count_ok = self.input_count < MAX_MACHINE_STACK;
        type_ok && count_ok
    }
}

/// Miner component - automatically mines blocks below
#[derive(Component)]
pub struct Miner {
    /// World position of this miner
    pub position: IVec3,
    /// Mining progress (0.0-1.0)
    pub progress: f32,
    /// Buffer of mined items (block type, count)
    pub buffer: Option<(BlockType, u32)>,
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

/// Crusher component - doubles ore output
#[derive(Component)]
pub struct Crusher {
    /// World position of this crusher
    pub position: IVec3,
    /// Input ore type and count
    pub input_type: Option<BlockType>,
    pub input_count: u32,
    /// Output ore type and count (doubled)
    pub output_type: Option<BlockType>,
    pub output_count: u32,
    /// Processing progress (0.0-1.0)
    pub progress: f32,
}

impl Default for Crusher {
    fn default() -> Self {
        Self {
            position: IVec3::ZERO,
            input_type: None,
            input_count: 0,
            output_type: None,
            output_count: 0,
            progress: 0.0,
        }
    }
}

impl Crusher {
    /// Check if this ore can be crushed
    pub fn can_crush(ore: BlockType) -> bool {
        matches!(ore, BlockType::IronOre | BlockType::CopperOre)
    }
}

/// Delivery platform - accepts items for delivery quests
#[derive(Component, Default)]
pub struct DeliveryPlatform {
    /// Total items delivered (by type)
    pub delivered: HashMap<BlockType, u32>,
}

/// Currently interacting furnace entity
#[derive(Resource, Default)]
pub struct InteractingFurnace(pub Option<Entity>);

/// Currently interacting crusher entity
#[derive(Resource, Default)]
pub struct InteractingCrusher(pub Option<Entity>);

/// Currently interacting miner entity
#[derive(Resource, Default)]
pub struct InteractingMiner(pub Option<Entity>);

/// Slot type for machine UI (Furnace/Crusher)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MachineSlotType {
    Fuel,
    Input,
    Output,
}

/// Machine UI slot button
#[derive(Component)]
pub struct MachineSlotButton(pub MachineSlotType);

/// Machine UI progress bar fill
#[derive(Component)]
pub struct MachineProgressBar;

/// Machine UI slot count text
#[derive(Component)]
pub struct MachineSlotCount(pub MachineSlotType);

/// Marker for furnace UI
#[derive(Component)]
pub struct FurnaceUI;

/// Marker for furnace UI text
#[derive(Component)]
pub struct FurnaceUIText;

/// Marker for miner UI panel
#[derive(Component)]
pub struct MinerUI;

/// Marker for crusher UI panel
#[derive(Component)]
pub struct CrusherUI;

/// Crusher UI progress bar fill
#[derive(Component)]
pub struct CrusherProgressBar;

/// Crusher UI slot button
#[derive(Component)]
pub struct CrusherSlotButton(pub MachineSlotType);

/// Crusher UI slot count text
#[derive(Component)]
pub struct CrusherSlotCount(pub MachineSlotType);

/// Miner UI buffer slot button (take buffer contents)
#[derive(Component)]
pub struct MinerBufferButton;

/// Miner UI clear button (discard buffer)
#[derive(Component)]
pub struct MinerClearButton;

/// Miner UI buffer count text
#[derive(Component)]
pub struct MinerBufferCountText;

/// Marker for conveyor item visual
#[derive(Component)]
pub struct ConveyorItemVisual;

/// Marker for delivery platform UI
#[derive(Component)]
pub struct DeliveryUI;

/// Marker for delivery UI text
#[derive(Component)]
pub struct DeliveryUIText;
