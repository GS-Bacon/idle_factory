//! Logging system for both WASM and native platforms
//!
//! Usage:
//! - `info!("message")` - General information
//! - `warn!("message")` - Warnings
//! - `error!("message")` - Errors
//! - `debug!("message")` - Debug info (filtered in release)
//! - `trace!("message")` - Verbose trace (filtered in release)
//!
//! Structured logging:
//! - `info!(category = "BLOCK", action = "place", ?pos, "Block placed")`
//!
//! Log collection:
//! - Native: Logs are written to `logs/game.log` via run.sh
//! - WASM: Logs go to browser console, capture via Playwright

use bevy::prelude::*;

/// Initialize logging for the current platform
pub fn init_logging() {
    #[cfg(target_arch = "wasm32")]
    {
        // WASM: Send logs to browser console
        tracing_wasm::set_as_global_default();
        tracing::info!("🎮 Idle Factory - WASM logging initialized");
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Native: Bevy's default logging is already set up via DefaultPlugins
        // Logs are captured to file via run.sh: cargo run 2>&1 | tee logs/game.log
        tracing::info!("🎮 Idle Factory - Native logging initialized");
    }
}

/// Plugin to initialize logging
pub struct GameLoggingPlugin;

impl Plugin for GameLoggingPlugin {
    fn build(&self, _app: &mut App) {
        init_logging();
    }
}
