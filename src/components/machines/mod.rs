//! Machine components: Miner, Conveyor, Furnace, Crusher

mod conveyor;
mod descriptor;
mod direction;
mod machine;
mod models;
mod ports;

// Re-export Direction (widely used)
pub use direction::Direction;

// Re-export Conveyor types
pub use conveyor::{Conveyor, ConveyorItem, ConveyorItemVisual, ConveyorShape, ConveyorVisual};

// Re-export Machine types
pub use machine::{
    can_crush_by_id, get_crush_output_by_id, get_smelt_output_by_id, Machine, MachineBundle,
    MachineSlot, MachineSlots,
};

// Re-export MachineModels resource
pub use models::MachineModels;

// Re-export Port types
pub use ports::{
    Crafter, InputPort, ItemAcceptor, ItemEjector, MachineInventory, OutputPort, PortDirection,
    PowerConsumer,
};

// Re-export Descriptor types
pub use descriptor::{MachineCategory, MachineDescriptor};
