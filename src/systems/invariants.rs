//! Runtime invariant checking for detecting "unplayable" bugs
//!
//! This module provides automatic detection of gameplay bugs that don't crash
//! but make the game unplayable, such as:
//! - Player embedded in blocks
//! - Player stuck (has input but can't move)
//! - Player falling through the world
//!
//! Violations are automatically saved to `logs/bugs.log`

use crate::components::{Player, PlayerPhysics};
use crate::constants::{PLAYER_HEIGHT, PLAYER_WIDTH};
use crate::input::{GameAction, InputManager};
use crate::world::WorldData;
use bevy::prelude::*;
use std::fs::{self, OpenOptions};
use std::io::Write;

/// Resource to track stuck detection state
#[derive(Resource, Default)]
pub struct StuckDetector {
    /// Number of consecutive frames with input but no movement
    pub stuck_frames: u32,
    /// Last known player position
    pub last_position: Vec3,
    /// Whether movement input is active this frame
    pub has_input: bool,
}

/// Resource to collect invariant violations
#[derive(Resource, Default)]
pub struct ViolationLog {
    /// List of current violations
    pub violations: Vec<InvariantViolation>,
    /// Frame count when last violation occurred
    pub last_violation_frame: u64,
    /// Count of violations saved to file (to avoid duplicates)
    pub saved_count: usize,
    /// Last saved violation string (for deduplication)
    pub last_saved_violation: String,
    /// Cooldown timer for same violation (seconds)
    pub cooldown_remaining: f32,
}

/// Types of invariant violations
#[derive(Debug, Clone)]
pub enum InvariantViolation {
    /// Player is inside a solid block
    PlayerEmbedded { player_pos: Vec3, block_pos: IVec3 },
    /// Player has input but can't move for too long
    PlayerStuck { player_pos: Vec3, stuck_frames: u32 },
    /// Player fell through the world
    PlayerFellThroughWorld { player_pos: Vec3 },
}

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvariantViolation::PlayerEmbedded {
                player_pos,
                block_pos,
            } => {
                write!(
                    f,
                    "EMBEDDED: Player at ({:.1},{:.1},{:.1}) inside block at ({},{},{})",
                    player_pos.x, player_pos.y, player_pos.z, block_pos.x, block_pos.y, block_pos.z
                )
            }
            InvariantViolation::PlayerStuck {
                player_pos,
                stuck_frames,
            } => {
                write!(
                    f,
                    "STUCK: Player at ({:.1},{:.1},{:.1}) stuck for {} frames",
                    player_pos.x, player_pos.y, player_pos.z, stuck_frames
                )
            }
            InvariantViolation::PlayerFellThroughWorld { player_pos } => {
                write!(
                    f,
                    "FELL: Player at ({:.1},{:.1},{:.1}) fell through world",
                    player_pos.x, player_pos.y, player_pos.z
                )
            }
        }
    }
}

/// Threshold for stuck detection (3 seconds at 60 FPS)
const STUCK_THRESHOLD_FRAMES: u32 = 180;
/// Minimum Y position before considered "fell through world"
const WORLD_FLOOR_Y: f32 = -10.0;
/// Minimum movement to not be considered stuck
const MOVEMENT_EPSILON: f32 = 0.01;

/// System to track movement input
pub fn track_movement_input(input: Res<InputManager>, mut stuck_detector: ResMut<StuckDetector>) {
    stuck_detector.has_input = input.pressed(GameAction::MoveForward)
        || input.pressed(GameAction::MoveBackward)
        || input.pressed(GameAction::MoveLeft)
        || input.pressed(GameAction::MoveRight);
}

/// System to detect if player is stuck
pub fn check_player_stuck(
    player_query: Query<(&Transform, &PlayerPhysics), With<Player>>,
    mut stuck_detector: ResMut<StuckDetector>,
    mut violation_log: ResMut<ViolationLog>,
) {
    let Ok((transform, physics)) = player_query.single() else {
        return;
    };

    let current_pos = transform.translation;
    let movement = (current_pos - stuck_detector.last_position).length();

    // Check if player has input but isn't moving
    if stuck_detector.has_input && movement < MOVEMENT_EPSILON && physics.velocity.length() < 0.1 {
        stuck_detector.stuck_frames += 1;

        if stuck_detector.stuck_frames >= STUCK_THRESHOLD_FRAMES {
            // Only log once when threshold is reached
            if stuck_detector.stuck_frames == STUCK_THRESHOLD_FRAMES {
                let violation = InvariantViolation::PlayerStuck {
                    player_pos: current_pos,
                    stuck_frames: stuck_detector.stuck_frames,
                };
                warn!("Invariant violation: {}", violation);
                violation_log.violations.push(violation);
            }
        }
    } else {
        // Reset stuck counter if player is moving or has no input
        stuck_detector.stuck_frames = 0;
    }

    stuck_detector.last_position = current_pos;
}

/// System to detect if player is embedded in blocks
pub fn check_player_embedded(
    player_query: Query<&Transform, With<Player>>,
    world_data: Res<WorldData>,
    mut violation_log: ResMut<ViolationLog>,
) {
    let Ok(transform) = player_query.single() else {
        return;
    };

    let player_pos = transform.translation;
    let half_width = PLAYER_WIDTH / 2.0;
    let half_height = PLAYER_HEIGHT / 2.0;

    // Check 8 corners of player AABB
    let offsets = [
        Vec3::new(-half_width, -half_height + 0.1, -half_width), // Bottom corners (slightly above feet)
        Vec3::new(half_width, -half_height + 0.1, -half_width),
        Vec3::new(-half_width, -half_height + 0.1, half_width),
        Vec3::new(half_width, -half_height + 0.1, half_width),
        Vec3::new(-half_width, half_height - 0.1, -half_width), // Top corners (slightly below head)
        Vec3::new(half_width, half_height - 0.1, -half_width),
        Vec3::new(-half_width, half_height - 0.1, half_width),
        Vec3::new(half_width, half_height - 0.1, half_width),
    ];

    for offset in offsets {
        let check_pos = player_pos + offset;
        let block_pos = crate::world_to_grid(check_pos);

        if world_data.get_block(block_pos).is_some() {
            let violation = InvariantViolation::PlayerEmbedded {
                player_pos,
                block_pos,
            };
            warn!("Invariant violation: {}", violation);
            violation_log.violations.push(violation);
            return; // Only report once per frame
        }
    }
}

/// System to detect if player fell through the world
pub fn check_player_fell(
    player_query: Query<&Transform, With<Player>>,
    mut violation_log: ResMut<ViolationLog>,
) {
    let Ok(transform) = player_query.single() else {
        return;
    };

    if transform.translation.y < WORLD_FLOOR_Y {
        let violation = InvariantViolation::PlayerFellThroughWorld {
            player_pos: transform.translation,
        };
        warn!("Invariant violation: {}", violation);
        violation_log.violations.push(violation);
    }
}

/// Clear violations that are older than 1 second (to avoid log spam)
pub fn clear_old_violations(mut violation_log: ResMut<ViolationLog>, time: Res<Time>) {
    // Keep only recent violations (last 10)
    let len = violation_log.violations.len();
    if len > 10 {
        let drain_count = len - 10;
        violation_log.violations.drain(0..drain_count);
        // Adjust saved_count since we drained some
        violation_log.saved_count = violation_log.saved_count.saturating_sub(drain_count);
    }
    let _ = time; // Suppress unused warning
}

/// Save new violations to bugs.log file (with deduplication)
pub fn save_violations_to_file(mut violation_log: ResMut<ViolationLog>, time: Res<Time>) {
    // Update cooldown
    violation_log.cooldown_remaining -= time.delta_secs();

    let saved_count = violation_log.saved_count;
    let new_violations = violation_log.violations.len().saturating_sub(saved_count);
    if new_violations == 0 {
        return;
    }

    // Collect violations to write (with deduplication check)
    let last_key = violation_log.last_saved_violation.clone();
    let cooldown = violation_log.cooldown_remaining;

    let to_write: Vec<_> = violation_log
        .violations
        .iter()
        .skip(saved_count)
        .filter_map(|v| {
            let key = get_violation_key(v);
            // Skip if same violation type within cooldown
            if key == last_key && cooldown > 0.0 {
                None
            } else {
                Some((key, format!("{}", v)))
            }
        })
        .collect();

    if to_write.is_empty() {
        violation_log.saved_count = violation_log.violations.len();
        return;
    }

    // Create logs directory if needed
    let _ = fs::create_dir_all("logs");

    // Open bugs.log in append mode
    let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/bugs.log")
    else {
        violation_log.saved_count = violation_log.violations.len();
        return;
    };

    // Get current timestamp
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    // Write violations
    let mut last_written_key = String::new();
    for (key, msg) in &to_write {
        let line = format!("[{}] {}\n", timestamp, msg);
        let _ = file.write_all(line.as_bytes());
        last_written_key = key.clone();
    }

    // Update state
    if !last_written_key.is_empty() {
        violation_log.last_saved_violation = last_written_key;
        violation_log.cooldown_remaining = 5.0; // 5 second cooldown
    }
    violation_log.saved_count = violation_log.violations.len();
}

/// Get a key for deduplication (violation type without exact coordinates)
fn get_violation_key(violation: &InvariantViolation) -> String {
    match violation {
        InvariantViolation::PlayerEmbedded { .. } => "EMBEDDED".to_string(),
        InvariantViolation::PlayerStuck { .. } => "STUCK".to_string(),
        InvariantViolation::PlayerFellThroughWorld { .. } => "FELL".to_string(),
    }
}

/// Plugin to add invariant checking systems
pub struct InvariantCheckPlugin;

impl Plugin for InvariantCheckPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StuckDetector>()
            .init_resource::<ViolationLog>()
            .add_systems(
                PostUpdate,
                (
                    track_movement_input,
                    check_player_stuck,
                    check_player_embedded,
                    check_player_fell,
                    save_violations_to_file,
                    clear_old_violations,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_display() {
        let violation = InvariantViolation::PlayerStuck {
            player_pos: Vec3::new(10.0, 20.0, 30.0),
            stuck_frames: 180,
        };
        let display = format!("{}", violation);
        assert!(display.contains("STUCK"));
        assert!(display.contains("180"));
    }

    #[test]
    fn test_stuck_detector_default() {
        let detector = StuckDetector::default();
        assert_eq!(detector.stuck_frames, 0);
        assert!(!detector.has_input);
    }

    #[test]
    fn test_violation_log_default() {
        let log = ViolationLog::default();
        assert!(log.violations.is_empty());
    }
}
