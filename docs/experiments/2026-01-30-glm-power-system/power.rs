//! Power system components: PowerProducer, PowerWire, and PowerConsumer extension

use bevy::prelude::*;

/// Component for power producers (generators)
#[derive(Component, Clone, Debug, Default)]
pub struct PowerProducer {
    /// Power output capacity (watts per tick)
    pub output_watts: u32,
    /// Fuel slot (for fuel-based generators)
    pub fuel_slot: Option<PowerFuelSlot>,
    /// Operational state
    pub is_operational: bool,
}

/// Fuel slot for power generators
#[derive(Clone, Debug, Default)]
pub struct PowerFuelSlot {
    /// Current fuel item and count
    pub fuel: Option<(ItemId, u32)>,
    /// Fuel consumption rate (fuel units per tick)
    pub consumption_rate: f32,
    /// Startup delay after refueling (seconds)
    pub startup_delay: f32,
    /// Current startup timer
    pub startup_timer: f32,
}

/// Component for power wire blocks
#[derive(Component, Clone, Debug)]
pub struct PowerWire {
    /// Connections to adjacent positions
    pub connections: Vec<IVec3>,
    /// Associated power grid ID
    pub grid_id: u64,
}

impl Default for PowerWire {
    fn default() -> Self {
        Self {
            connections: Vec::new(),
            grid_id: 0,
        }
    }
}

/// Power grid resource for tracking networks
#[derive(Resource, Clone, Debug, Default)]
pub struct PowerGrids {
    /// All power grids
    pub grids: HashMap<u64, PowerGrid>,
    /// Next grid ID
    pub next_id: u64,
}

/// Represents a connected power network
#[derive(Clone, Debug)]
pub struct PowerGrid {
    /// Grid ID
    pub id: u64,
    /// Total generation capacity
    pub total_generation: u32,
    /// Total consumption
    pub total_consumption: u32,
    /// Connected producers
    pub producers: Vec<Entity>,
    /// Connected consumers
    pub consumers: Vec<Entity>,
    /// Connected wires
    pub wires: Vec<Entity>,
}

impl PowerGrid {
    /// Check if grid has sufficient power
    pub fn has_power(&self) -> bool {
        self.total_generation >= self.total_consumption
    }

    /// Get surplus power
    pub fn surplus(&self) -> i32 {
        self.total_generation as i32 - self.total_consumption as i32
    }
}

/// Power-related events
#[derive(Message, Debug)]
pub struct PowerGridChanged {
    pub grid_id: u64,
    pub change_type: PowerGridChangeType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerGridChangeType {
    GridCreated,
    GridSplit { new_grid_ids: Vec<u64> },
    GridMerged { merged_into_id: u64 },
    GeneratorAdded,
    GeneratorRemoved,
    ConsumerAdded,
    ConsumerRemoved,
    WireAdded,
    WireRemoved,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_grid_has_power() {
        let grid = PowerGrid {
            id: 0,
            total_generation: 100,
            total_consumption: 80,
            producers: vec![],
            consumers: vec![],
            wires: vec![],
        };
        assert!(grid.has_power());
        assert_eq!(grid.surplus(), 20);
    }

    #[test]
    fn test_power_grid_no_power() {
        let grid = PowerGrid {
            id: 0,
            total_generation: 50,
            total_consumption: 100,
            producers: vec![],
            consumers: vec![],
            wires: vec![],
        };
        assert!(!grid.has_power());
        assert_eq!(grid.surplus(), -50);
    }
}
