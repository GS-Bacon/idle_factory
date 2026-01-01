//! Machine systems plugin
//!
//! Consolidates all machine-related systems:
//! - Furnace, Crusher, Miner interaction
//! - Machine processing and output
//! - Conveyor transport
//! - Machine UI updates

use bevy::prelude::*;

use crate::components::{
    ConveyorRotationOffset, InteractingCrusher, InteractingFurnace, InteractingMiner,
    MachineModels,
};
use crate::systems::{
    // Machine interaction systems
    conveyor_transfer,
    crusher_interact,
    crusher_output,
    crusher_processing,
    crusher_ui_input,
    furnace_interact,
    furnace_output,
    furnace_smelting,
    furnace_ui_input,
    miner_interact,
    miner_mining,
    miner_output,
    miner_ui_input,
    miner_visual_feedback,
    update_conveyor_item_visuals,
    // Machine UI systems
    update_crusher_ui,
    update_furnace_ui,
    update_miner_ui,
};

/// Plugin that organizes all machine-related systems
pub struct MachineSystemsPlugin;

impl Plugin for MachineSystemsPlugin {
    fn build(&self, app: &mut App) {
        // Machine-related resources
        app.init_resource::<InteractingFurnace>()
            .init_resource::<InteractingCrusher>()
            .init_resource::<InteractingMiner>()
            .init_resource::<MachineModels>()
            .init_resource::<ConveyorRotationOffset>();

        // Machine interaction systems
        app.add_systems(
            Update,
            (
                furnace_interact,
                furnace_ui_input,
                furnace_smelting,
                crusher_interact,
                crusher_ui_input,
                miner_interact,
                miner_ui_input,
            ),
        );

        // Machine processing systems
        app.add_systems(
            Update,
            (
                miner_mining,
                miner_visual_feedback,
                miner_output,
                crusher_processing,
                crusher_output,
                furnace_output,
                conveyor_transfer,
                update_conveyor_item_visuals,
            ),
        );

        // Machine UI update systems
        app.add_systems(
            Update,
            (update_furnace_ui, update_crusher_ui, update_miner_ui),
        );
    }
}
