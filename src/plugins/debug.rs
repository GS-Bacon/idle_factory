//! Debug Plugin
//!
//! Groups debug-related systems:
//! - Version/build ID display in window title
//! - Debug HUD (F3 toggle) with FPS
//! - E2E state export for automated testing (debug only)
//! - Runtime invariant checking for playability bugs (debug only)

use bevy::prelude::*;

use crate::components::DebugHudState;
use crate::systems::{toggle_debug_hud, update_biome_hud, update_debug_hud, update_window_title};

#[cfg(debug_assertions)]
use crate::systems::{export_e2e_state, E2EExportConfig, InvariantCheckPlugin};
#[cfg(debug_assertions)]
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

/// Plugin that adds debug functionality
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Debug-only: FPS diagnostics, invariant checks, E2E export
        #[cfg(debug_assertions)]
        {
            app.add_plugins(FrameTimeDiagnosticsPlugin)
                .add_plugins(InvariantCheckPlugin)
                .init_resource::<E2EExportConfig>()
                .add_systems(Update, export_e2e_state);
        }

        // Always active: window title, debug HUD, biome HUD
        app.init_resource::<DebugHudState>().add_systems(
            Update,
            (
                update_window_title,
                toggle_debug_hud,
                update_debug_hud,
                update_biome_hud,
            ),
        );
    }
}
