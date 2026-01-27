//! Save/Load systems plugin
//!
//! Consolidates all save-related systems, events, and resources.

use bevy::prelude::*;

use crate::save::AutoSaveTimer;
use crate::systems::{auto_save_system, handle_load_event, handle_save_event};
use crate::{LoadGameEvent, SaveGameEvent, SaveLoadState};

/// Plugin for save/load functionality
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        // Save resources
        app.init_resource::<AutoSaveTimer>()
            .init_resource::<SaveLoadState>();

        // Save events
        app.add_message::<SaveGameEvent>()
            .add_message::<LoadGameEvent>();

        // Save systems
        app.add_systems(
            Update,
            (auto_save_system, handle_save_event, handle_load_event),
        );
    }
}
