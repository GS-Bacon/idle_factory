//! Power system plugin for M3
//!
//! This module handles the power grid calculation and power management.

mod grid_calc;
mod generator_tick;

use bevy::prelude::*;

use crate::components::power::{PowerGrids, PowerProducer, PowerWire};
use crate::core::ItemId;
use crate::systems::power::grid_calc::PowerGridCache;

/// Power system plugin
pub struct PowerSystemPlugin;

impl Plugin for PowerSystemPlugin {
    fn build(&self, app: &mut App) {
        // Initialize power grids resource
        app.insert_resource(PowerGrids::default());
        app.insert_resource(PowerGridCache::default());

        // Add systems
        app.add_systems(bevy::prelude::FixedUpdate, grid_calc::calculate_power_grids);
        app.add_systems(bevy::prelude::FixedUpdate, generator_tick::update_generator_fuel);
    }
}
