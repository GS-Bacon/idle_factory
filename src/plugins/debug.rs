//! Debug Plugin
//!
//! Groups debug-related systems:
//! - FPS display in window title
//! - Debug HUD (F3 toggle)
//! - E2E state export for automated testing

use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use crate::components::DebugHudState;
use crate::systems::{
    update_window_title_fps, toggle_debug_hud, update_debug_hud,
    export_e2e_state, E2EExportConfig,
};

/// Plugin that adds debug functionality
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .init_resource::<DebugHudState>()
            .init_resource::<E2EExportConfig>()
            .add_systems(
                Update,
                (
                    update_window_title_fps,
                    toggle_debug_hud,
                    update_debug_hud,
                    export_e2e_state,
                ),
            );
    }
}
