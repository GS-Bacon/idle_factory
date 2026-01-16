//! MachineDescriptor and MachineCategory for UI auto-generation

/// Machine category for grouping in UI
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineCategory {
    /// Resource production (miner)
    Production,
    /// Item processing (furnace, crusher)
    Processing,
    /// Item logistics (conveyor, inserter)
    Logistics,
    /// Storage
    Storage,
}

/// Machine metadata for UI auto-generation
#[derive(Clone, Debug)]
pub struct MachineDescriptor {
    /// Unique machine ID
    pub id: &'static str,
    /// Display name (localized)
    pub display_name: &'static str,
    /// Category for UI grouping
    pub category: MachineCategory,
    /// Number of input slots
    pub input_slots: u8,
    /// Number of output slots
    pub output_slots: u8,
    /// Whether machine has a fuel slot
    pub has_fuel_slot: bool,
    /// Whether machine has recipe selection
    pub has_recipe_select: bool,
    /// Power consumption (None = no power required)
    pub power_consumption: Option<f32>,
}

impl MachineDescriptor {
    /// Miner descriptor
    pub const MINER: Self = Self {
        id: "miner",
        display_name: "採掘機",
        category: MachineCategory::Production,
        input_slots: 0,
        output_slots: 1,
        has_fuel_slot: false,
        has_recipe_select: false,
        power_consumption: None,
    };

    /// Furnace descriptor
    pub const FURNACE: Self = Self {
        id: "furnace",
        display_name: "精錬炉",
        category: MachineCategory::Processing,
        input_slots: 1,
        output_slots: 1,
        has_fuel_slot: true,
        has_recipe_select: false,
        power_consumption: None,
    };

    /// Crusher descriptor
    pub const CRUSHER: Self = Self {
        id: "crusher",
        display_name: "粉砕機",
        category: MachineCategory::Processing,
        input_slots: 1,
        output_slots: 1,
        has_fuel_slot: false,
        has_recipe_select: false,
        power_consumption: None,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_descriptor_constants() {
        assert_eq!(MachineDescriptor::MINER.id, "miner");
        assert_eq!(MachineDescriptor::FURNACE.has_fuel_slot, true);
        assert_eq!(MachineDescriptor::CRUSHER.input_slots, 1);
    }
}
