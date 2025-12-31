//! Debug HUD systems

use crate::components::*;
use crate::world::WorldData;
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use serde::Serialize;
use std::fs;

/// Update window title with FPS
pub fn update_window_title_fps(diagnostics: Res<DiagnosticsStore>, mut windows: Query<&mut Window>) {
    if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            if let Ok(mut window) = windows.get_single_mut() {
                window.title = format!("Idle Factory - FPS: {:.0}", value);
            }
        }
    }
}

/// Toggle debug HUD with F3 key
pub fn toggle_debug_hud(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugHudState>,
    debug_query: Query<Entity, With<DebugHudText>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        debug_state.visible = !debug_state.visible;

        if debug_state.visible {
            // Spawn debug HUD
            commands.spawn((
                Text::new("Debug Info"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                DebugHudText,
            ));
        } else {
            // Remove debug HUD
            for entity in debug_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Update debug HUD content
pub fn update_debug_hud(
    debug_state: Res<DebugHudState>,
    mut text_query: Query<&mut Text, With<DebugHudText>>,
    diagnostics: Res<DiagnosticsStore>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&PlayerCamera>,
    world_data: Res<WorldData>,
    creative_mode: Res<CreativeMode>,
    cursor_state: Res<CursorLockState>,
    target_block: Res<TargetBlock>,
) {
    if !debug_state.visible {
        return;
    }

    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let (pos_str, dir_str) = if let Ok(transform) = player_query.get_single() {
        let pos = transform.translation;
        let dir = if let Ok(camera) = camera_query.get_single() {
            format!("Pitch: {:.1}° Yaw: {:.1}°", camera.pitch.to_degrees(), camera.yaw.to_degrees())
        } else {
            "N/A".to_string()
        };
        (format!("X: {:.1} Y: {:.1} Z: {:.1}", pos.x, pos.y, pos.z), dir)
    } else {
        ("N/A".to_string(), "N/A".to_string())
    };

    // Target block info
    let break_str = target_block
        .break_target
        .map(|p| format!("({},{},{})", p.x, p.y, p.z))
        .unwrap_or_else(|| "None".to_string());
    let place_str = target_block
        .place_target
        .map(|p| format!("({},{},{})", p.x, p.y, p.z))
        .unwrap_or_else(|| "None".to_string());

    // Get block type at break target
    let block_type_str = if let Some(break_pos) = target_block.break_target {
        world_data
            .get_block(break_pos)
            .map(|b| format!("{:?}", b))
            .unwrap_or_else(|| "Air".to_string())
    } else {
        "N/A".to_string()
    };

    let chunk_count = world_data.chunks.len();
    let mode_str = if creative_mode.enabled { "Creative" } else { "Survival" };
    let pause_str = if cursor_state.paused { " [PAUSED]" } else { "" };

    text.0 = format!(
        "FPS: {:.0}\nPos: {}\nDir: {}\nTarget: {} ({})\nPlace: {}\nChunks: {}\nMode: {}{}",
        fps, pos_str, dir_str, break_str, block_type_str, place_str, chunk_count, mode_str, pause_str
    );
}

/// Game state for E2E testing - exported to JSON file
#[derive(Serialize)]
pub struct E2EGameState {
    pub player_pos: [f32; 3],
    pub camera_pitch: f32,
    pub camera_yaw: f32,
    pub target_break: Option<[i32; 3]>,
    pub target_place: Option<[i32; 3]>,
    pub target_block_type: String,
    pub creative_mode: bool,
    pub paused: bool,
    pub fps: f32,
}

/// Resource to control E2E state export
#[derive(Resource)]
pub struct E2EExportConfig {
    pub enabled: bool,
    pub path: String,
}

impl Default for E2EExportConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("E2E_EXPORT").is_ok(),
            path: std::env::var("E2E_EXPORT_PATH")
                .unwrap_or_else(|_| "e2e_state.json".to_string()),
        }
    }
}

/// Export game state to JSON file for E2E testing
/// Enable with E2E_EXPORT=1 environment variable
/// Customize path with E2E_EXPORT_PATH environment variable
pub fn export_e2e_state(
    config: Res<E2EExportConfig>,
    diagnostics: Res<DiagnosticsStore>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&PlayerCamera>,
    world_data: Res<WorldData>,
    creative_mode: Res<CreativeMode>,
    cursor_state: Res<CursorLockState>,
    target_block: Res<TargetBlock>,
) {
    if !config.enabled {
        return;
    }

    let fps = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0) as f32;

    let (player_pos, camera_pitch, camera_yaw) = if let Ok(transform) = player_query.get_single() {
        let pos = transform.translation;
        let (pitch, yaw) = if let Ok(camera) = camera_query.get_single() {
            (camera.pitch, camera.yaw)
        } else {
            (0.0, 0.0)
        };
        ([pos.x, pos.y, pos.z], pitch, yaw)
    } else {
        ([0.0, 0.0, 0.0], 0.0, 0.0)
    };

    let target_break = target_block.break_target.map(|p| [p.x, p.y, p.z]);
    let target_place = target_block.place_target.map(|p| [p.x, p.y, p.z]);

    let target_block_type = if let Some(break_pos) = target_block.break_target {
        world_data
            .get_block(break_pos)
            .map(|b| format!("{:?}", b))
            .unwrap_or_else(|| "Air".to_string())
    } else {
        "None".to_string()
    };

    let state = E2EGameState {
        player_pos,
        camera_pitch,
        camera_yaw,
        target_break,
        target_place,
        target_block_type,
        creative_mode: creative_mode.enabled,
        paused: cursor_state.paused,
        fps,
    };

    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = fs::write(&config.path, json);
    }
}
