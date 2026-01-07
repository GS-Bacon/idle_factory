//! Machine specification
//!
//! All machines are defined as `MachineSpec`.
//! UI is automatically generated from the spec.

use crate::BlockType;

use super::recipes::MachineType;

/// Machine facing direction (player's direction at placement)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum MachineFacing {
    North, // -Z
    South, // +Z
    East,  // +X
    West,  // -X
}

/// I/O port position
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PortSide {
    Front, // Front of machine
    Back,  // Back of machine
    #[allow(dead_code)]
    Left, // Left side
    #[allow(dead_code)]
    Right, // Right side
    #[allow(dead_code)]
    Top, // Top of machine
    #[allow(dead_code)]
    Bottom, // Bottom (unused but for future)
}

/// I/O port definition
#[derive(Clone, Copy, Debug)]
pub struct IoPort {
    /// Port position
    pub side: PortSide,
    /// Is input port (false = output)
    pub is_input: bool,
    /// Slot ID (for multiple inputs/outputs)
    pub slot_id: u8,
}

// =============================================================================
// UI Slot Definitions (for auto-generated UI)
// =============================================================================

/// UI slot type
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiSlotType {
    /// Input slot for materials
    Input,
    /// Output slot for products
    Output,
    /// Fuel slot (coal etc.)
    Fuel,
}

/// UI slot definition for auto-generated machine UI
#[derive(Clone, Copy, Debug)]
pub struct UiSlotDef {
    /// Slot type
    pub slot_type: UiSlotType,
    /// Slot ID (matches IoPort.slot_id)
    pub slot_id: u8,
    /// Display label (e.g., "鉱石", "燃料", "出力")
    pub label: &'static str,
}

impl UiSlotDef {
    pub const fn new(slot_type: UiSlotType, slot_id: u8, label: &'static str) -> Self {
        Self {
            slot_type,
            slot_id,
            label,
        }
    }
}

/// Machine processing type
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessType {
    /// Recipe-based processing (furnace, crusher, assembler)
    Recipe(MachineType),
    /// Auto-generates resources from terrain (miner)
    AutoGenerate,
    /// Transfer only, no processing (conveyor) - not a machine UI
    #[allow(dead_code)]
    Transfer,
}

/// Machine specification
#[derive(Clone, Debug)]
pub struct MachineSpec {
    /// Machine ID
    pub id: &'static str,
    /// Display name
    pub name: &'static str,
    /// Corresponding BlockType
    pub block_type: BlockType,
    /// I/O ports (for conveyor connections)
    pub ports: &'static [IoPort],
    /// Internal buffer size (per slot)
    pub buffer_size: u32,
    /// Base processing time (seconds)
    pub process_time: f32,
    /// Requires fuel
    pub requires_fuel: bool,
    /// Auto-generates resources (like miner)
    #[allow(dead_code)]
    pub auto_generate: bool,
    /// UI slot definitions (for auto-generated UI)
    pub ui_slots: &'static [UiSlotDef],
    /// Processing type
    pub process_type: ProcessType,
}

// =============================================================================
// Machine Definitions
// =============================================================================

/// Miner - auto-generates resources from terrain
pub const MINER: MachineSpec = MachineSpec {
    id: "miner",
    name: "採掘機",
    block_type: BlockType::MinerBlock,
    ports: &[IoPort {
        side: PortSide::Front,
        is_input: false,
        slot_id: 0,
    }],
    buffer_size: 64,
    process_time: 1.5,
    requires_fuel: false,
    auto_generate: true,
    ui_slots: &[UiSlotDef::new(UiSlotType::Output, 0, "出力")],
    process_type: ProcessType::AutoGenerate,
};

/// Furnace - smelts ore into ingots (requires fuel)
pub const FURNACE: MachineSpec = MachineSpec {
    id: "furnace",
    name: "精錬炉",
    block_type: BlockType::FurnaceBlock,
    ports: &[
        IoPort {
            side: PortSide::Back,
            is_input: true,
            slot_id: 0, // 鉱石入力
        },
        IoPort {
            side: PortSide::Left,
            is_input: true,
            slot_id: 1, // 燃料入力（左）
        },
        IoPort {
            side: PortSide::Right,
            is_input: true,
            slot_id: 1, // 燃料入力（右）
        },
        IoPort {
            side: PortSide::Front,
            is_input: false,
            slot_id: 0,
        },
    ],
    buffer_size: 64,
    process_time: 2.0,
    requires_fuel: true,
    auto_generate: false,
    ui_slots: &[
        UiSlotDef::new(UiSlotType::Input, 0, "鉱石"),
        UiSlotDef::new(UiSlotType::Fuel, 1, "燃料"),
        UiSlotDef::new(UiSlotType::Output, 0, "出力"),
    ],
    process_type: ProcessType::Recipe(MachineType::Furnace),
};

/// Crusher - crushes ore into dust (doubles output)
pub const CRUSHER: MachineSpec = MachineSpec {
    id: "crusher",
    name: "粉砕機",
    block_type: BlockType::CrusherBlock,
    ports: &[
        IoPort {
            side: PortSide::Back,
            is_input: true,
            slot_id: 0,
        },
        IoPort {
            side: PortSide::Front,
            is_input: false,
            slot_id: 0,
        },
    ],
    buffer_size: 64,
    process_time: 1.5,
    requires_fuel: false,
    auto_generate: false,
    ui_slots: &[
        UiSlotDef::new(UiSlotType::Input, 0, "入力"),
        UiSlotDef::new(UiSlotType::Output, 0, "出力"),
    ],
    process_type: ProcessType::Recipe(MachineType::Crusher),
};

/// Assembler - crafts machines and components
pub const ASSEMBLER: MachineSpec = MachineSpec {
    id: "assembler",
    name: "組立機",
    block_type: BlockType::AssemblerBlock,
    ports: &[
        IoPort {
            side: PortSide::Back,
            is_input: true,
            slot_id: 0, // 主素材入力
        },
        IoPort {
            side: PortSide::Left,
            is_input: true,
            slot_id: 1, // 副素材入力（左）
        },
        IoPort {
            side: PortSide::Right,
            is_input: true,
            slot_id: 1, // 副素材入力（右）
        },
        IoPort {
            side: PortSide::Front,
            is_input: false,
            slot_id: 0,
        },
    ],
    buffer_size: 32,
    process_time: 3.0,
    requires_fuel: false,
    auto_generate: false,
    ui_slots: &[
        UiSlotDef::new(UiSlotType::Input, 0, "主素材"),
        UiSlotDef::new(UiSlotType::Input, 1, "副素材"),
        UiSlotDef::new(UiSlotType::Output, 0, "出力"),
    ],
    process_type: ProcessType::Recipe(MachineType::Assembler),
};

/// All machines
pub const ALL_MACHINES: &[&MachineSpec] = &[&MINER, &FURNACE, &CRUSHER, &ASSEMBLER];

/// Get machine spec from BlockType
pub fn get_machine_spec(block_type: BlockType) -> Option<&'static MachineSpec> {
    ALL_MACHINES
        .iter()
        .find(|m| m.block_type == block_type)
        .copied()
}

/// Get input ports for a machine
pub fn get_input_ports(spec: &MachineSpec) -> impl Iterator<Item = &IoPort> {
    spec.ports.iter().filter(|p| p.is_input)
}

/// Get output ports for a machine
pub fn get_output_ports(spec: &MachineSpec) -> impl Iterator<Item = &IoPort> {
    spec.ports.iter().filter(|p| !p.is_input)
}

// =============================================================================
// Machine Connection Spec
// =============================================================================

/// Enable direct machine-to-machine connection
#[allow(dead_code)]
pub const ENABLE_DIRECT_CONNECTION: bool = true;

/// Direct connection transfer interval (seconds/item)
#[allow(dead_code)]
pub const DIRECT_TRANSFER_INTERVAL: f32 = 0.0;

/// Wait when receiver buffer is full
#[allow(dead_code)]
pub const WAIT_ON_FULL_BUFFER: bool = true;

// =============================================================================
// Machine Rendering Spec
// =============================================================================

/// Render machines as blocks
#[allow(dead_code)]
pub const RENDER_AS_BLOCK: bool = true;

/// Machine block size
#[allow(dead_code)]
pub const MACHINE_BLOCK_SIZE: f32 = 1.0;

/// Show direction indicator
#[allow(dead_code)]
pub const SHOW_DIRECTION_INDICATOR: bool = true;

/// Show I/O ports visually
#[allow(dead_code)]
pub const SHOW_IO_PORTS: bool = true;

/// Machine state
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum MachineState {
    Idle,
    Working,
}

/// Smoke particle settings
pub mod smoke {
    #[allow(dead_code)]
    pub const ENABLED: bool = true;
    #[allow(dead_code)]
    pub const SMOKE_MACHINES: &[&str] = &["furnace", "crusher"];
    #[allow(dead_code)]
    pub const SPAWN_INTERVAL: f32 = 0.3;
    #[allow(dead_code)]
    pub const RISE_SPEED: f32 = 0.5;
    #[allow(dead_code)]
    pub const LIFETIME: f32 = 1.5;
    #[allow(dead_code)]
    pub const SIZE: f32 = 0.15;
    #[allow(dead_code)]
    pub const COLOR: (f32, f32, f32, f32) = (0.5, 0.5, 0.5, 0.6);
}

/// State visual settings
pub mod state_visuals {
    #[allow(dead_code)]
    pub const WORKING_BRIGHTNESS: f32 = 1.3;
    #[allow(dead_code)]
    pub const WORKING_EMISSIVE: bool = true;
    #[allow(dead_code)]
    pub const EMISSIVE_COLOR_FURNACE: (f32, f32, f32) = (1.0, 0.5, 0.1);
    #[allow(dead_code)]
    pub const EMISSIVE_COLOR_CRUSHER: (f32, f32, f32) = (0.3, 0.5, 1.0);
    #[allow(dead_code)]
    pub const EMISSIVE_COLOR_MINER: (f32, f32, f32) = (0.2, 0.8, 0.3);
    #[allow(dead_code)]
    pub const EMISSIVE_INTENSITY: f32 = 0.5;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_spec() {
        assert!(!ALL_MACHINES.is_empty());

        for machine in ALL_MACHINES {
            assert!(
                !machine.ports.is_empty(),
                "Machine {} should have ports",
                machine.id
            );
            assert!(
                machine.buffer_size > 0,
                "Machine {} should have buffer",
                machine.id
            );
        }

        let miner = get_machine_spec(BlockType::MinerBlock);
        assert!(miner.is_some());
        assert_eq!(miner.unwrap().id, "miner");

        let furnace = get_machine_spec(BlockType::FurnaceBlock);
        assert!(furnace.is_some());
        assert_eq!(furnace.unwrap().id, "furnace");

        let stone = get_machine_spec(BlockType::Stone);
        assert!(stone.is_none());
    }

    #[test]
    fn test_io_ports() {
        let miner_inputs: Vec<_> = get_input_ports(&MINER).collect();
        let miner_outputs: Vec<_> = get_output_ports(&MINER).collect();
        assert!(miner_inputs.is_empty());
        assert_eq!(miner_outputs.len(), 1);

        // Furnace: 3 inputs (ore + fuel left + fuel right), 1 output
        let furnace_inputs: Vec<_> = get_input_ports(&FURNACE).collect();
        let furnace_outputs: Vec<_> = get_output_ports(&FURNACE).collect();
        assert_eq!(furnace_inputs.len(), 3);
        assert_eq!(furnace_outputs.len(), 1);

        let crusher_inputs: Vec<_> = get_input_ports(&CRUSHER).collect();
        let crusher_outputs: Vec<_> = get_output_ports(&CRUSHER).collect();
        assert_eq!(crusher_inputs.len(), 1);
        assert_eq!(crusher_outputs.len(), 1);

        // Assembler: 3 inputs (main + sub left + sub right), 1 output
        let assembler_inputs: Vec<_> = get_input_ports(&ASSEMBLER).collect();
        let assembler_outputs: Vec<_> = get_output_ports(&ASSEMBLER).collect();
        assert_eq!(assembler_inputs.len(), 3);
        assert_eq!(assembler_outputs.len(), 1);
    }
}
