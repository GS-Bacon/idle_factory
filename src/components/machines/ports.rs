//! Port-related components: InputPort, OutputPort, PortDirection, ItemAcceptor, ItemEjector, Crafter, MachineInventory, PowerConsumer

use bevy::prelude::*;

use crate::core::ItemId;

/// Input port definition for machines that accept items
#[derive(Clone, Debug)]
pub struct InputPort {
    /// Direction relative to machine facing (Back = behind machine)
    pub direction: PortDirection,
    /// Optional filter for accepted item types (ItemId-based)
    pub filter: Option<Vec<ItemId>>,
}

/// Output port definition for machines that eject items
#[derive(Clone, Debug)]
pub struct OutputPort {
    /// Direction relative to machine facing (Front = in front of machine)
    pub direction: PortDirection,
}

/// Port direction relative to machine facing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDirection {
    Front,
    Back,
    Left,
    Right,
}

/// Component for machines that accept items through input ports
#[derive(Component, Clone, Debug)]
pub struct ItemAcceptor {
    /// Input ports
    pub ports: Vec<InputPort>,
}

impl Default for ItemAcceptor {
    fn default() -> Self {
        Self {
            ports: vec![InputPort {
                direction: PortDirection::Back,
                filter: None,
            }],
        }
    }
}

/// Component for machines that eject items through output ports
#[derive(Component, Clone, Debug)]
pub struct ItemEjector {
    /// Output ports
    pub ports: Vec<OutputPort>,
}

impl Default for ItemEjector {
    fn default() -> Self {
        Self {
            ports: vec![OutputPort {
                direction: PortDirection::Front,
            }],
        }
    }
}

/// Component for machines that process recipes
#[derive(Component, Clone, Debug, Default)]
pub struct Crafter {
    /// Current recipe ID being processed (if any)
    pub recipe_id: Option<&'static str>,
    /// Processing progress (0.0 to 1.0)
    pub progress: f32,
    /// Speed multiplier (1.0 = normal speed)
    pub speed_multiplier: f32,
}

/// Component for machine's item inventory (ItemId-based)
#[derive(Component, Clone, Debug, Default)]
pub struct MachineInventory {
    /// Input slots (ItemId, count)
    pub input_slots: Vec<Option<(ItemId, u32)>>,
    /// Output slots (ItemId, count)
    pub output_slots: Vec<Option<(ItemId, u32)>>,
    /// Fuel slot (for furnaces) (ItemId, count)
    pub fuel_slot: Option<(ItemId, u32)>,
}

impl MachineInventory {
    /// Create with specified slot counts
    pub fn new(input_count: usize, output_count: usize, _has_fuel: bool) -> Self {
        // Note: fuel_slot is always initialized as None; has_fuel is reserved for future use
        Self {
            input_slots: vec![None; input_count],
            output_slots: vec![None; output_count],
            fuel_slot: None,
        }
    }

    /// Get input item as ItemId
    pub fn input_item_id(&self, slot: usize) -> Option<ItemId> {
        self.input_slots.get(slot)?.as_ref().map(|(id, _)| *id)
    }

    /// Get output item as ItemId
    pub fn output_item_id(&self, slot: usize) -> Option<ItemId> {
        self.output_slots.get(slot)?.as_ref().map(|(id, _)| *id)
    }
}

/// Component for machines that consume power (future use)
#[derive(Component, Clone, Debug, Default)]
pub struct PowerConsumer {
    /// Required power per tick
    pub required_power: f32,
    /// Currently available power
    pub current_power: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_inventory_new() {
        let inv = MachineInventory::new(2, 1, true);
        assert_eq!(inv.input_slots.len(), 2);
        assert_eq!(inv.output_slots.len(), 1);
    }
}
