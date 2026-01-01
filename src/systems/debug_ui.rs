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
#[allow(clippy::too_many_arguments)]
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
    conveyor_query: Query<&Conveyor>,
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

    // Get conveyor info at break target
    let conveyor_info = if let Some(break_pos) = target_block.break_target {
        conveyor_query
            .iter()
            .find(|conv| conv.position == break_pos)
            .map(|conv| format!("Shape: {:?}, Dir: {:?}", conv.shape, conv.direction))
            .unwrap_or_default()
    } else {
        String::new()
    };

    let chunk_count = world_data.chunks.len();
    let mode_str = if creative_mode.enabled { "Creative" } else { "Survival" };
    let pause_str = if cursor_state.paused { " [PAUSED]" } else { "" };

    let conveyor_line = if conveyor_info.is_empty() {
        String::new()
    } else {
        format!("\nConveyor: {}", conveyor_info)
    };

    text.0 = format!(
        "FPS: {:.0}\nPos: {}\nDir: {}\nTarget: {} ({})\nPlace: {}\nChunks: {}\nMode: {}{}{}",
        fps, pos_str, dir_str, break_str, block_type_str, place_str, chunk_count, mode_str, pause_str, conveyor_line
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
    // Extended fields for comprehensive E2E testing
    pub quest: E2EQuestState,
    pub conveyors: Vec<E2EConveyorInfo>,
    pub machines: Vec<E2EMachineInfo>,
    pub floating_blocks: Vec<[i32; 3]>,
}

/// Quest state for E2E testing
#[derive(Serialize, Default)]
pub struct E2EQuestState {
    pub index: usize,
    pub completed: bool,
    pub rewards_claimed: bool,
    pub description: String,
    pub required_item: String,
    pub required_amount: u32,
    pub delivered_amount: u32,
}

/// Conveyor info for E2E testing
#[derive(Serialize)]
pub struct E2EConveyorInfo {
    pub position: [i32; 3],
    pub direction: String,
    pub shape: String,
    pub item_count: usize,
}

/// Machine info for E2E testing
#[derive(Serialize)]
pub struct E2EMachineInfo {
    pub position: [i32; 3],
    pub machine_type: String,
    /// Processing progress (0.0-1.0)
    pub progress: f32,
    /// Input slot: (item_name, count)
    pub input: Option<(String, u32)>,
    /// Output slot: (item_name, count)
    pub output: Option<(String, u32)>,
    /// Fuel count (Furnace only)
    pub fuel: Option<u32>,
    /// Buffer info (Miner only)
    pub buffer: Option<(String, u32)>,
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
    current_quest: Res<CurrentQuest>,
    platform_query: Query<&DeliveryPlatform>,
    conveyor_query: Query<&Conveyor>,
    miner_query: Query<&Miner>,
    furnace_query: Query<(&Furnace, &Transform)>,
    crusher_query: Query<&Crusher>,
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

    // Collect quest state
    let quests = crate::systems::quest::get_quests();
    let quest = if current_quest.index < quests.len() {
        let q = &quests[current_quest.index];
        let delivered = platform_query
            .get_single()
            .map(|p| p.delivered.get(&q.required_item).copied().unwrap_or(0))
            .unwrap_or(0);
        E2EQuestState {
            index: current_quest.index,
            completed: current_quest.completed,
            rewards_claimed: current_quest.rewards_claimed,
            description: q.description.to_string(),
            required_item: format!("{:?}", q.required_item),
            required_amount: q.required_amount,
            delivered_amount: delivered,
        }
    } else {
        E2EQuestState::default()
    };

    // Collect conveyor info
    let conveyors: Vec<E2EConveyorInfo> = conveyor_query
        .iter()
        .map(|c| E2EConveyorInfo {
            position: [c.position.x, c.position.y, c.position.z],
            direction: format!("{:?}", c.direction),
            shape: format!("{:?}", c.shape),
            item_count: c.items.len(),
        })
        .collect();

    // Collect machine info with detailed state
    let mut machines: Vec<E2EMachineInfo> = Vec::new();
    for miner in miner_query.iter() {
        machines.push(E2EMachineInfo {
            position: [miner.position.x, miner.position.y, miner.position.z],
            machine_type: "Miner".to_string(),
            progress: miner.progress,
            input: None,
            output: None,
            fuel: None,
            buffer: miner.buffer.map(|(bt, count)| (format!("{:?}", bt), count)),
        });
    }
    for (furnace, transform) in furnace_query.iter() {
        let pos = transform.translation / crate::BLOCK_SIZE;
        machines.push(E2EMachineInfo {
            position: [pos.x as i32, pos.y as i32, pos.z as i32],
            machine_type: "Furnace".to_string(),
            progress: furnace.progress,
            input: furnace.input_type.map(|bt| (format!("{:?}", bt), furnace.input_count)),
            output: furnace.output_type.map(|bt| (format!("{:?}", bt), furnace.output_count)),
            fuel: Some(furnace.fuel),
            buffer: None,
        });
    }
    for crusher in crusher_query.iter() {
        machines.push(E2EMachineInfo {
            position: [crusher.position.x, crusher.position.y, crusher.position.z],
            machine_type: "Crusher".to_string(),
            progress: crusher.progress,
            input: crusher.input_type.map(|bt| (format!("{:?}", bt), crusher.input_count)),
            output: crusher.output_type.map(|bt| (format!("{:?}", bt), crusher.output_count)),
            fuel: None,
            buffer: None,
        });
    }

    // Check for floating blocks (machines not on solid ground)
    let floating_blocks = check_floating_blocks(&conveyors, &machines, &world_data);

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
        quest,
        conveyors,
        machines,
        floating_blocks,
    };

    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = fs::write(&config.path, json);
    }
}

/// Check if blocks are floating (not supported by anything below)
fn check_floating_blocks(
    conveyors: &[E2EConveyorInfo],
    machines: &[E2EMachineInfo],
    world_data: &WorldData,
) -> Vec<[i32; 3]> {
    let mut floating = Vec::new();

    // Check conveyors
    for c in conveyors {
        let pos = IVec3::new(c.position[0], c.position[1], c.position[2]);
        let below = pos - IVec3::Y;
        if !is_supported(below, world_data) {
            floating.push(c.position);
        }
    }

    // Check machines
    for m in machines {
        let pos = IVec3::new(m.position[0], m.position[1], m.position[2]);
        let below = pos - IVec3::Y;
        if !is_supported(below, world_data) {
            floating.push(m.position);
        }
    }

    floating
}

/// Check if a position is supported (has a solid block or is on terrain)
fn is_supported(pos: IVec3, world_data: &WorldData) -> bool {
    // Y=0 or below is always supported (ground level)
    if pos.y <= 8 {
        return true;
    }
    // Check if there's a solid block at this position
    // If get_block returns Some, there's a block (all block types are solid)
    world_data.get_block(pos).is_some()
}
