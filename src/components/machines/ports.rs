//! Port-related components: InputPort, OutputPort, PortDirection, ItemAcceptor, ItemEjector, Crafter, MachineInventory, PowerConsumer

use bevy::prelude::*;

use crate::core::ItemId;

/// Port direction relative to machine facing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDirection {
    Front,
    Back,
    Left,
    Right,
}

/// Type of port for different resource types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PortType {
    /// Item input/output port
    Item,
    /// Fluid input/output port
    Fluid,
    /// Power input/output port
    Power,
    /// Signal input/output port
    Signal,
}

/// Machine port side relative to machine facing (6 directions)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MachinePortSide {
    /// North side (top in 2D, back in 3D)
    North,
    /// East side (right)
    East,
    /// South side (bottom in 2D, front in 3D)
    South,
    /// West side (left)
    West,
    /// Top side (up)
    Top,
    /// Bottom side (down)
    Bottom,
}

/// Machine I/O port definition for machine interfaces
///
/// An MachineIoPort represents a single connection point on a machine that can handle
/// different types of resources (item, fluid, power, signal).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct MachineIoPort {
    /// Side of the machine (North, East, South, West, Top, Bottom)
    pub side: MachinePortSide,
    /// Type of resource this port handles
    pub port_type: PortType,
    /// Slot ID within the machine's interface
    pub slot_id: usize,
}

#[allow(dead_code)]
impl MachineIoPort {
    /// Create a new MachineIoPort with the specified side, type, and slot
    ///
    /// # Arguments
    ///
    /// - `side`: The side of the machine
    /// - `port_type`: The type of resource this port handles
    /// - `slot_id`: The slot ID within the machine's interface
    ///
    /// # Returns
    ///
    /// A new MachineIoPort instance
    pub fn new(side: MachinePortSide, port_type: PortType, slot_id: usize) -> Self {
        Self {
            side,
            port_type,
            slot_id,
        }
    }

    /// Check if this port can handle the specified resource type
    pub fn can_handle(&self, port_type: PortType) -> bool {
        self.port_type == port_type
    }
}

impl Default for MachineIoPort {
    fn default() -> Self {
        Self {
            side: MachinePortSide::North,
            port_type: PortType::Signal,
            slot_id: 0,
        }
    }
}

/// Input port definition for machines that accept items
#[derive(Clone, Debug)]
pub struct InputPort {
    /// Direction relative to machine facing (Back = behind machine)
    pub direction: MachinePortSide,
    /// Optional filter for accepted item types (ItemId-based)
    pub filter: Option<Vec<ItemId>>,
}

/// Output port definition for machines that eject items
#[derive(Clone, Debug)]
pub struct OutputPort {
    /// Direction relative to machine facing (Front = in front of machine)
    pub direction: MachinePortSide,
}

impl Default for ItemAcceptor {
    fn default() -> Self {
        Self {
            ports: vec![InputPort {
                direction: MachinePortSide::North,
                filter: None,
            }],
        }
    }
}

/// Component for machines that accept items through input ports
#[derive(Component, Clone, Debug)]
pub struct ItemAcceptor {
    /// Input ports
    pub ports: Vec<InputPort>,
}

impl Default for ItemEjector {
    fn default() -> Self {
        Self {
            ports: vec![OutputPort {
                direction: MachinePortSide::South,
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

#[cfg(test)]
mod port_tests {
    use super::*;

    #[test]
    fn test_port_type() {
        let item_port = PortType::Item;
        let fluid_port = PortType::Fluid;
        let power_port = PortType::Power;
        let signal_port = PortType::Signal;

        assert_ne!(item_port, fluid_port);
        assert_ne!(item_port, power_port);
        assert_ne!(item_port, signal_port);
        assert_ne!(fluid_port, power_port);
        assert_ne!(fluid_port, signal_port);
        assert_ne!(power_port, signal_port);
    }

    #[test]
    fn test_port_side_equality() {
        let north = MachinePortSide::North;
        let east = MachinePortSide::East;
        let south = MachinePortSide::South;
        let west = MachinePortSide::West;
        let top = MachinePortSide::Top;
        let bottom = MachinePortSide::Bottom;

        assert_eq!(north, north);
        assert_eq!(east, east);
        assert_ne!(north, east);
        assert_ne!(south, west);
        assert_ne!(top, bottom);
    }

    #[test]
    fn test_port_type_all_variants() {
        // Test all 4 variants
        let _item = PortType::Item;
        let _fluid = PortType::Fluid;
        let _power = PortType::Power;
        let _signal = PortType::Signal;

        // Verify all variants can be created
        assert!(matches!(_item, PortType::Item));
        assert!(matches!(_fluid, PortType::Fluid));
        assert!(matches!(_power, PortType::Power));
        assert!(matches!(_signal, PortType::Signal));
    }

    #[test]
    fn test_machine_io_port_new() {
        let port = MachineIoPort::new(MachinePortSide::North, PortType::Power, 0);

        assert_eq!(port.side, MachinePortSide::North);
        assert_eq!(port.port_type, PortType::Power);
        assert_eq!(port.slot_id, 0);
    }

    #[test]
    fn test_machine_io_port_type_handling() {
        let item_port = MachineIoPort::new(MachinePortSide::North, PortType::Item, 0);
        let power_port = MachineIoPort::new(MachinePortSide::North, PortType::Power, 0);

        assert!(item_port.can_handle(PortType::Item));
        assert!(!item_port.can_handle(PortType::Fluid));
        assert!(!power_port.can_handle(PortType::Item));
        assert!(power_port.can_handle(PortType::Power));
    }

    #[test]
    fn test_machine_io_port_equality() {
        let port1 = MachineIoPort::new(MachinePortSide::North, PortType::Power, 0);
        let port2 = MachineIoPort::new(MachinePortSide::North, PortType::Power, 0);
        let port3 = MachineIoPort::new(MachinePortSide::East, PortType::Power, 0);
        let port4 = MachineIoPort::new(MachinePortSide::North, PortType::Fluid, 0);
        let port5 = MachineIoPort::new(MachinePortSide::North, PortType::Power, 1);

        assert_eq!(port1, port2);
        assert_ne!(port1, port3);
        assert_ne!(port1, port4);
        assert_ne!(port1, port5);
    }
}
