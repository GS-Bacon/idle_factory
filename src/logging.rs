//! Logging system
//!
//! Usage:
//! - `info!("message")` - General information
//! - `warn!("message")` - Warnings
//! - `error!("message")` - Errors
//! - `debug!("message")` - Debug info (filtered in release)
//! - `trace!("message")` - Verbose trace (filtered in release)
//!
//! Game Event Logging (JSON format):
//! - Use `EventLogger` resource to log structured game events
//! - Events are written to `logs/events_YYYYMMDD_HHMMSS.jsonl`
//!
//! Log collection:
//! - Logs are written to `logs/game_YYYYMMDD_HHMMSS.log`
//!
//! Crash handling:
//! - Call `setup_crash_handler()` at the start of main() to capture panic backtraces
//! - Crash reports are written to `logs/crash.log`
//!
//! Log analysis:
//! - scripts/summarize_log.sh - AI-powered log summary
//! - scripts/detect_anomalies.sh - Anomaly detection

use bevy::prelude::*;
use serde::Serialize;
use std::backtrace::Backtrace;
use std::fs;
use std::io::Write;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Resource to hold the log file guard (keeps writer alive)
#[derive(Resource)]
#[allow(dead_code)]
pub struct LogFileGuard(WorkerGuard);

/// Set up a custom panic handler that captures backtraces and writes to crash.log
///
/// Call this at the very start of main(), before any other initialization.
pub fn setup_crash_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        // Force capture backtrace regardless of RUST_BACKTRACE setting
        let backtrace = Backtrace::force_capture();

        let crash_report = format!(
            "=== CRASH REPORT ===\n\
             Time: {:?}\n\
             Version: {}\n\
             OS: {} {}\n\
             \n\
             === PANIC INFO ===\n\
             {}\n\
             Location: {:?}\n\
             \n\
             === BACKTRACE ===\n\
             {}\n\
             \n\
             === END CRASH REPORT ===\n\n",
            std::time::SystemTime::now(),
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            panic_info,
            panic_info.location(),
            backtrace
        );

        // Ensure logs directory exists
        let logs_dir = std::path::Path::new("logs");
        if !logs_dir.exists() {
            let _ = fs::create_dir_all(logs_dir);
        }

        // Append to crash.log
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/crash.log")
        {
            let _ = file.write_all(crash_report.as_bytes());
        }

        // Also output to stderr for immediate visibility
        eprintln!("{}", crash_report);
    }));
}

/// Initialize logging
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
        .with_ansi(false) // No ANSI colors in file
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

    // Log system information
    log_system_info();

    Some(guard)
}

/// Log system information at startup
fn log_system_info() {
    use std::env;

    tracing::info!("=== System Information ===");
    tracing::info!("OS: {} {}", env::consts::OS, env::consts::ARCH);
    tracing::info!(
        "Rust version: {}",
        env!("CARGO_PKG_RUST_VERSION", "unknown")
    );
    tracing::info!("Game version: {}", env!("CARGO_PKG_VERSION"));

    // Current working directory
    if let Ok(cwd) = env::current_dir() {
        tracing::info!("Working directory: {}", cwd.display());
    }

    // Number of CPUs
    tracing::info!(
        "CPU cores: {}",
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    );

    tracing::info!("==========================");
}

// ============================================================================
// Game Event Logging (JSON format)
// Reserved for future structured logging integration
// ============================================================================

/// Game event categories
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Block,
    Machine,
    Item,
    Quest,
    Player,
    System,
}

/// A structured game event for JSON logging
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct GameEvent {
    pub timestamp: f64,
    pub category: EventCategory,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<[i32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Resource for logging game events in JSON format
#[allow(dead_code)]
#[derive(Resource)]
pub struct EventLogger {
    file: std::sync::Mutex<std::fs::File>,
}

#[allow(dead_code)]
impl EventLogger {
    /// Create a new event logger
    pub fn new() -> Option<Self> {
        use std::io::Write;

        let logs_dir = std::path::Path::new("logs");
        if !logs_dir.exists() {
            let _ = fs::create_dir_all(logs_dir);
        }

        let now = chrono::Local::now();
        let filename = format!("logs/events_{}.jsonl", now.format("%Y%m%d_%H%M%S"));

        match std::fs::File::create(&filename) {
            Ok(mut file) => {
                // Write header comment
                let _ = writeln!(file, "// Game events log - JSON Lines format");
                tracing::info!("Event log: {}", filename);
                Some(Self {
                    file: std::sync::Mutex::new(file),
                })
            }
            Err(e) => {
                tracing::warn!("Failed to create event log: {}", e);
                None
            }
        }
    }

    /// Log a game event
    pub fn log(&self, event: GameEvent) {
        use std::io::Write;

        if let Ok(mut file) = self.file.lock() {
            if let Ok(json) = serde_json::to_string(&event) {
                let _ = writeln!(file, "{}", json);
            }
        }
    }

    /// Log a simple event
    pub fn log_simple(&self, category: EventCategory, action: &str, details: Option<&str>) {
        self.log(GameEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            category,
            action: action.to_string(),
            position: None,
            entity: None,
            details: details.map(|s| s.to_string()),
        });
    }

    /// Log an event with position
    pub fn log_at(&self, category: EventCategory, action: &str, pos: IVec3, entity: Option<&str>) {
        self.log(GameEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            category,
            action: action.to_string(),
            position: Some([pos.x, pos.y, pos.z]),
            entity: entity.map(|s| s.to_string()),
            details: None,
        });
    }
}

impl Default for EventLogger {
    fn default() -> Self {
        Self::new().expect("Failed to create event logger")
    }
}
