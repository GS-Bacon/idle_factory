//! Machine systems plugin (Phase C: Data-Driven)
//!
//! Consolidates all machine-related systems:
//! - Generic machine interaction (unified)
//! - Machine processing via generic_machine_tick
//! - Conveyor transport
//! - Generic machine UI

use bevy::prelude::*;

use crate::components::{ConveyorRotationOffset, InteractingMachine, MachineModels};
use crate::machines::{
    generic_machine_interact, generic_machine_tick, generic_machine_ui_input,
    machine_visual_feedback, update_generic_machine_ui,
};
use crate::systems::{conveyor_transfer, update_conveyor_item_visuals};

/// Plugin that organizes all machine-related systems
pub struct MachineSystemsPlugin;

impl Plugin for MachineSystemsPlugin {
    fn build(&self, app: &mut App) {
        // Machine-related resources
        app.init_resource::<InteractingMachine>()
            .init_resource::<MachineModels>()
            .init_resource::<ConveyorRotationOffset>();

        // Machine interaction systems (Phase C: generic)
        app.add_systems(Update, (generic_machine_interact, generic_machine_ui_input));

        // Machine processing systems (Phase C: generic only)
        app.add_systems(
            Update,
            (
                machine_visual_feedback,
                conveyor_transfer,
                update_conveyor_item_visuals,
                generic_machine_tick,
            ),
        );

        // Machine UI update systems (Phase C: generic)
        app.add_systems(Update, update_generic_machine_ui);
    }
}
