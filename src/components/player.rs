//! Player-related components and resources

use bevy::prelude::*;

/// Player marker component
#[derive(Component)]
pub struct Player;

/// Player camera component with rotation state
#[derive(Component)]
pub struct PlayerCamera {
    /// Pitch (vertical rotation) in radians
    pub pitch: f32,
    /// Yaw (horizontal rotation) in radians
    pub yaw: f32,
}

/// Player physics state for survival mode
#[derive(Component)]
pub struct PlayerPhysics {
    /// Current velocity (y component is vertical velocity)
    pub velocity: Vec3,
    /// Whether player is on the ground
    pub on_ground: bool,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            on_ground: false,
        }
    }
}

/// Tracks cursor lock state and handles mouse input for both local and RDP environments
#[derive(Resource, Default)]
pub struct CursorLockState {
    pub was_locked: bool,
    pub skip_frames: u8,
    /// Last mouse position for calculating delta in RDP/absolute mode
    pub last_mouse_pos: Option<Vec2>,
    /// Skip next block break (used when resuming from pointer lock release)
    pub just_locked: bool,
    /// Game is paused (ESC pressed, waiting for click to resume)
    pub paused: bool,
    /// Skip inventory toggle this frame (used when closing machine UI with E key)
    pub skip_inventory_toggle: bool,
}

/// Timer for continuous block break/place operations
#[derive(Resource)]
pub struct ContinuousActionTimer {
    /// Timer for block breaking
    pub break_timer: Timer,
    /// Timer for block placing
    pub place_timer: Timer,
    /// Timer for inventory shift-click
    pub inventory_timer: Timer,
}

impl Default for ContinuousActionTimer {
    fn default() -> Self {
        Self {
            break_timer: Timer::from_seconds(0.15, TimerMode::Once),
            place_timer: Timer::from_seconds(0.15, TimerMode::Once),
            inventory_timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}
