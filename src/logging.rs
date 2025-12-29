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
//! - `info!(block_type = ?BlockType::Stone, pos = ?IVec3::new(1,2,3), "Block placed")`

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
        // Just log that we're starting
        tracing::info!("🎮 Idle Factory - Native logging initialized");
    }
}

/// Game event categories for structured logging
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum LogCategory {
    Block,
    Machine,
    Inventory,
    Quest,
    Chunk,
    Ui,
    Input,
}

impl std::fmt::Display for LogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogCategory::Block => write!(f, "BLOCK"),
            LogCategory::Machine => write!(f, "MACHINE"),
            LogCategory::Inventory => write!(f, "INVENTORY"),
            LogCategory::Quest => write!(f, "QUEST"),
            LogCategory::Chunk => write!(f, "CHUNK"),
            LogCategory::Ui => write!(f, "UI"),
            LogCategory::Input => write!(f, "INPUT"),
        }
    }
}

/// Log buffer for WASM export (used via JavaScript interop)
#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct GameLogBuffer {
    pub entries: Vec<LogEntry>,
    pub max_entries: usize,
}

#[allow(dead_code)]
impl GameLogBuffer {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            max_entries,
        }
    }

    pub fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LogEntry {
    pub timestamp: f64,
    pub category: String,
    pub level: String,
    pub message: String,
}

/// Plugin to initialize logging
pub struct GameLoggingPlugin;

impl Plugin for GameLoggingPlugin {
    fn build(&self, app: &mut App) {
        init_logging();
        app.insert_resource(GameLogBuffer::new(1000));
    }
}

/// Macro for game event logging with category
#[macro_export]
macro_rules! game_log {
    ($category:expr, $level:ident, $($arg:tt)*) => {
        tracing::$level!(category = %$category, $($arg)*);
    };
}

/// Convenience macros for each category
#[macro_export]
macro_rules! log_block {
    ($($arg:tt)*) => {
        tracing::info!(category = "BLOCK", $($arg)*);
    };
}

#[macro_export]
macro_rules! log_machine {
    ($($arg:tt)*) => {
        tracing::info!(category = "MACHINE", $($arg)*);
    };
}

#[macro_export]
macro_rules! log_inventory {
    ($($arg:tt)*) => {
        tracing::debug!(category = "INVENTORY", $($arg)*);
    };
}

#[macro_export]
macro_rules! log_quest {
    ($($arg:tt)*) => {
        tracing::info!(category = "QUEST", $($arg)*);
    };
}

#[macro_export]
macro_rules! log_chunk {
    ($($arg:tt)*) => {
        tracing::debug!(category = "CHUNK", $($arg)*);
    };
}

#[macro_export]
macro_rules! log_ui {
    ($($arg:tt)*) => {
        tracing::debug!(category = "UI", $($arg)*);
    };
}
