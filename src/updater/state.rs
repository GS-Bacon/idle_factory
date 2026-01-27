//! Update state and events for the auto-updater system.

use bevy::prelude::*;

/// Current phase of the update process.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum UpdatePhase {
    /// No update activity.
    #[default]
    Idle,
    /// Checking for updates.
    Checking,
    /// Update is available.
    Available {
        version: String,
        release_notes: String,
        download_url: String,
    },
    /// Already up to date.
    UpToDate,
    /// Update check failed.
    Failed(String),
}

/// Resource tracking the current update state.
#[derive(Resource)]
pub struct UpdateState {
    /// Current phase of the update process.
    pub phase: UpdatePhase,
    /// Last time we checked for updates.
    pub last_check: Option<std::time::Instant>,
    /// Minimum interval between checks (to avoid rate limiting).
    pub check_interval: std::time::Duration,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            phase: UpdatePhase::Idle,
            last_check: None,
            check_interval: std::time::Duration::from_secs(3600), // 1 hour
        }
    }
}

impl UpdateState {
    /// Check if enough time has passed since the last check.
    pub fn can_check(&self) -> bool {
        match self.last_check {
            None => true,
            Some(last) => last.elapsed() >= self.check_interval,
        }
    }
}

/// Event to trigger an update check.
#[derive(Message)]
pub struct CheckForUpdateEvent;

/// Event to start the update (launches external updater).
#[derive(Message)]
pub struct StartUpdateEvent;

/// Event to cancel/dismiss the update notification.
#[derive(Message)]
pub struct CancelUpdateEvent;

/// Event to restart the application (unused with external updater, kept for compatibility).
#[derive(Message)]
pub struct RestartAppEvent;

/// Event sent when update check completes.
#[derive(Message)]
#[allow(dead_code)]
pub struct UpdateCheckCompleteEvent {
    pub result: UpdateCheckResult,
}

/// Result of an update check.
#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    /// New version available.
    Available {
        version: String,
        release_notes: String,
        download_url: String,
    },
    /// Already up to date.
    UpToDate,
    /// Check failed.
    Error(String),
}
