//! Auto-save timer and save slot utilities

use super::AUTO_SAVE_INTERVAL;
use bevy::prelude::*;

/// Auto-save timer resource
#[derive(Resource)]
pub struct AutoSaveTimer {
    pub timer: Timer,
}

impl Default for AutoSaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(AUTO_SAVE_INTERVAL, TimerMode::Repeating),
        }
    }
}

/// Save slot info for listing saves
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SaveSlotInfo {
    pub filename: String,
    pub timestamp: u64,
}
