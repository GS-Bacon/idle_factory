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
//! - Native: Logs are written to `logs/game_YYYYMMDD_HHMMSS.log`
//! - WASM: Logs go to browser console, capture via Playwright

use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use tracing_appender::non_blocking::WorkerGuard;
#[cfg(not(target_arch = "wasm32"))]
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Resource to hold the log file guard (keeps writer alive)
#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
#[allow(dead_code)]
pub struct LogFileGuard(WorkerGuard);

/// Initialize logging for the current platform
#[cfg(not(target_arch = "wasm32"))]
pub fn init_logging() -> Option<WorkerGuard> {
    // Create logs directory
    let logs_dir = std::path::Path::new("logs");
    if !logs_dir.exists() {
        let _ = fs::create_dir_all(logs_dir);
    }

    // Generate timestamped log filename
    let now = chrono::Local::now();
    let log_filename = format!("game_{}.log", now.format("%Y%m%d_%H%M%S"));

    // Create file appender
    let file_appender = tracing_appender::rolling::never("logs", &log_filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Set up subscriber with both console and file output
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,wgpu=warn,bevy_render=warn,bevy_ecs=warn"));

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)  // No ANSI colors in file
        .with_target(true)
        .with_thread_ids(true);

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    tracing::info!("Idle Factory - Logging initialized");
    tracing::info!("Log file: logs/{}", log_filename);

    Some(guard)
}

#[cfg(target_arch = "wasm32")]
pub fn init_logging() {
    // WASM: Send logs to browser console
    tracing_wasm::set_as_global_default();
    tracing::info!("Idle Factory - WASM logging initialized");
}

/// Plugin to initialize logging (WASM only)
#[cfg(target_arch = "wasm32")]
pub struct GameLoggingPlugin;

#[cfg(target_arch = "wasm32")]
impl Plugin for GameLoggingPlugin {
    fn build(&self, _app: &mut App) {
        init_logging();
    }
}
