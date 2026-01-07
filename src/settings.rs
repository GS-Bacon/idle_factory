//! Game settings system
//!
//! Provides user-configurable settings with persistence.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Settings file name
const SETTINGS_FILE: &str = "settings.json";

/// User-configurable game settings
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Mouse sensitivity (0.0001 - 0.01)
    pub mouse_sensitivity: f32,
    /// View distance in chunks (1 - 8)
    pub view_distance: i32,
    /// Master volume (0.0 - 1.0)
    pub master_volume: f32,
    /// Sound effects volume (0.0 - 1.0)
    pub sfx_volume: f32,
    /// Music volume (0.0 - 1.0)
    pub music_volume: f32,
    /// Enable shadows
    pub shadows_enabled: bool,
    /// Vertical sync
    pub vsync_enabled: bool,
    /// Fullscreen mode
    pub fullscreen: bool,
    /// Field of view (45 - 120)
    pub fov: f32,
    /// Invert Y axis
    pub invert_y: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.002,
            view_distance: 3,
            master_volume: 1.0,
            sfx_volume: 1.0,
            music_volume: 0.5,
            shadows_enabled: true,
            vsync_enabled: true,
            fullscreen: false,
            fov: 70.0,
            invert_y: false,
        }
    }
}

impl GameSettings {
    /// Get the settings file path
    fn settings_path() -> PathBuf {
        // Use project directory for development, or user config dir in production
        #[cfg(debug_assertions)]
        {
            PathBuf::from(SETTINGS_FILE)
        }
        #[cfg(not(debug_assertions))]
        {
            dirs::config_dir()
                .map(|p| p.join("idle_factory").join(SETTINGS_FILE))
                .unwrap_or_else(|| PathBuf::from(SETTINGS_FILE))
        }
    }

    /// Load settings from file, or return default if not found
    pub fn load() -> Self {
        let path = Self::settings_path();
        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(settings) => {
                    tracing::info!("Settings loaded from {:?}", path);
                    settings
                }
                Err(e) => {
                    tracing::warn!("Failed to parse settings: {}, using defaults", e);
                    Self::default()
                }
            },
            Err(_) => {
                tracing::info!("No settings file found, using defaults");
                Self::default()
            }
        }
    }

    /// Save settings to file
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::settings_path();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        tracing::info!("Settings saved to {:?}", path);
        Ok(())
    }

    /// Clamp all settings to valid ranges
    pub fn validate(&mut self) {
        self.mouse_sensitivity = self.mouse_sensitivity.clamp(0.0001, 0.01);
        self.view_distance = self.view_distance.clamp(1, 8);
        self.master_volume = self.master_volume.clamp(0.0, 1.0);
        self.sfx_volume = self.sfx_volume.clamp(0.0, 1.0);
        self.music_volume = self.music_volume.clamp(0.0, 1.0);
        self.fov = self.fov.clamp(45.0, 120.0);
    }

    /// Get effective mouse sensitivity (with invert Y option)
    pub fn effective_sensitivity(&self) -> (f32, f32) {
        let y_mult = if self.invert_y { -1.0 } else { 1.0 };
        (self.mouse_sensitivity, self.mouse_sensitivity * y_mult)
    }

    /// Get effective SFX volume (master * sfx)
    pub fn effective_sfx_volume(&self) -> f32 {
        self.master_volume * self.sfx_volume
    }

    /// Get effective music volume (master * music)
    pub fn effective_music_volume(&self) -> f32 {
        self.master_volume * self.music_volume
    }
}

/// Event sent when settings are changed
#[derive(Event)]
pub struct SettingsChangedEvent;

/// Plugin that manages game settings
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = GameSettings::load();
        app.insert_resource(settings)
            .add_event::<SettingsChangedEvent>()
            .add_systems(Update, (auto_save_settings, apply_settings_immediately));
    }
}

/// Track if settings need saving
#[derive(Resource, Default)]
struct SettingsDirty {
    dirty: bool,
    save_timer: f32,
}

/// Auto-save settings when changed (with debounce)
fn auto_save_settings(
    settings: Res<GameSettings>,
    mut dirty: Local<SettingsDirty>,
    mut events: EventReader<SettingsChangedEvent>,
    time: Res<Time>,
) {
    // Mark dirty when settings changed
    for _ in events.read() {
        dirty.dirty = true;
        dirty.save_timer = 1.0; // Wait 1 second before saving
    }

    // Countdown and save
    if dirty.dirty {
        dirty.save_timer -= time.delta_secs();
        if dirty.save_timer <= 0.0 {
            if let Err(e) = settings.save() {
                tracing::error!("Failed to save settings: {}", e);
            }
            dirty.dirty = false;
        }
    }
}

/// Apply settings changes immediately to the game
pub fn apply_settings_immediately(
    settings: Res<GameSettings>,
    mut events: EventReader<SettingsChangedEvent>,
    mut windows: Query<&mut Window>,
    mut projection_query: Query<&mut Projection>,
) {
    // Only process if there were changes
    if events.read().next().is_none() {
        return;
    }
    // Drain remaining events
    for _ in events.read() {}

    // Apply window settings
    if let Ok(mut window) = windows.get_single_mut() {
        // VSync
        window.present_mode = if settings.vsync_enabled {
            bevy::window::PresentMode::AutoVsync
        } else {
            bevy::window::PresentMode::AutoNoVsync
        };

        // Fullscreen
        window.mode = if settings.fullscreen {
            bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
        } else {
            bevy::window::WindowMode::Windowed
        };
    }

    // Apply FOV to camera
    for mut projection in projection_query.iter_mut() {
        if let Projection::Perspective(ref mut persp) = *projection {
            persp.fov = settings.fov.to_radians();
        }
    }

    tracing::info!(
        "Settings applied: vsync={}, fullscreen={}, fov={}",
        settings.vsync_enabled,
        settings.fullscreen,
        settings.fov
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = GameSettings::default();
        assert_eq!(settings.mouse_sensitivity, 0.002);
        assert_eq!(settings.view_distance, 3);
        assert!((settings.master_volume - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_settings_validate() {
        let mut settings = GameSettings {
            mouse_sensitivity: 0.1, // Too high
            view_distance: 100,     // Too high
            master_volume: 2.0,     // Too high
            sfx_volume: -0.5,       // Too low
            music_volume: 0.5,
            shadows_enabled: true,
            vsync_enabled: true,
            fullscreen: false,
            fov: 200.0, // Too high
            invert_y: false,
        };

        settings.validate();

        assert_eq!(settings.mouse_sensitivity, 0.01);
        assert_eq!(settings.view_distance, 8);
        assert!((settings.master_volume - 1.0).abs() < f32::EPSILON);
        assert!((settings.sfx_volume - 0.0).abs() < f32::EPSILON);
        assert!((settings.fov - 120.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_settings_effective_volume() {
        let settings = GameSettings {
            master_volume: 0.5,
            sfx_volume: 0.8,
            music_volume: 0.6,
            ..Default::default()
        };

        assert!((settings.effective_sfx_volume() - 0.4).abs() < f32::EPSILON);
        assert!((settings.effective_music_volume() - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_settings_invert_y() {
        let mut settings = GameSettings::default();

        settings.invert_y = false;
        let (x, y) = settings.effective_sensitivity();
        assert!((y - 0.002).abs() < f32::EPSILON);
        assert!((x - y).abs() < f32::EPSILON);

        settings.invert_y = true;
        let (x, y) = settings.effective_sensitivity();
        assert!((y - (-0.002)).abs() < f32::EPSILON);
        assert!((x - (-y)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = GameSettings::default();
        let json = serde_json::to_string(&settings).expect("should serialize");
        let parsed: GameSettings = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(settings.mouse_sensitivity, parsed.mouse_sensitivity);
        assert_eq!(settings.view_distance, parsed.view_distance);
        assert_eq!(settings.fullscreen, parsed.fullscreen);
    }
}
