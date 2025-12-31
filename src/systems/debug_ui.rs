//! Debug HUD systems

use crate::components::*;
use crate::world::WorldData;
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;

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

    let chunk_count = world_data.chunks.len();
    let mode_str = if creative_mode.enabled { "Creative" } else { "Survival" };
    let pause_str = if cursor_state.paused { " [PAUSED]" } else { "" };

    text.0 = format!(
        "FPS: {:.0}\nPos: {}\nDir: {}\nChunks: {}\nMode: {}{}",
        fps, pos_str, dir_str, chunk_count, mode_str, pause_str
    );
}
