//! Command processing for test API

use bevy::prelude::*;

use crate::core::{items, ItemId};
use crate::input::GameAction;
use crate::systems::command::{SetBlockEvent, TeleportEvent};

use super::config::ModApiServer;

/// Process queued commands from test.send_command
/// This runs as a separate system to avoid parameter limit in process_server_messages
pub fn process_test_command_queue(
    mut server: Option<ResMut<ModApiServer>>,
    mut teleport_writer: EventWriter<TeleportEvent>,
    mut setblock_writer: EventWriter<SetBlockEvent>,
    mut tutorial_shown: Option<ResMut<crate::components::TutorialShown>>,
) {
    let Some(ref mut server) = server else { return };

    for cmd in server.command_queue.drain(..) {
        tracing::info!("Processing command: {}", cmd);
        parse_and_execute_command(
            &cmd,
            &mut teleport_writer,
            &mut setblock_writer,
            &mut tutorial_shown,
        );
    }
}

/// Parse a command string and execute it
pub fn parse_and_execute_command(
    cmd: &str,
    teleport_writer: &mut EventWriter<TeleportEvent>,
    setblock_writer: &mut EventWriter<SetBlockEvent>,
    tutorial_shown: &mut Option<ResMut<crate::components::TutorialShown>>,
) {
    let cmd = cmd.trim();
    let cmd = cmd.strip_prefix('/').unwrap_or(cmd);
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    if parts.is_empty() {
        tracing::warn!("Empty command");
        return;
    }

    match parts[0] {
        "tp" | "teleport" => {
            // /tp x y z
            if parts.len() < 4 {
                tracing::warn!("tp requires 3 coordinates: /tp x y z");
                return;
            }
            let x: f32 = parts[1].parse().unwrap_or(0.0);
            let y: f32 = parts[2].parse().unwrap_or(0.0);
            let z: f32 = parts[3].parse().unwrap_or(0.0);
            teleport_writer.send(TeleportEvent {
                position: Vec3::new(x, y, z),
            });
            tracing::info!("Teleport to ({}, {}, {})", x, y, z);
        }
        "setblock" => {
            // /setblock x y z item_id
            if parts.len() < 5 {
                tracing::warn!("setblock requires 4 args: /setblock x y z item_id");
                return;
            }
            let x: i32 = parts[1].parse().unwrap_or(0);
            let y: i32 = parts[2].parse().unwrap_or(0);
            let z: i32 = parts[3].parse().unwrap_or(0);
            let item_id_str = parts[4];
            // Try to get ItemId from interner, fall back to stone if not found
            let item_id = items::interner()
                .get(item_id_str)
                .map(ItemId::from_raw)
                .unwrap_or_else(items::stone);
            setblock_writer.send(SetBlockEvent {
                position: IVec3::new(x, y, z),
                block_type: item_id,
            });
            tracing::info!("SetBlock at ({}, {}, {}) = {}", x, y, z, item_id_str);
        }
        "dismiss_tutorial" => {
            // /dismiss_tutorial - Force dismiss tutorial
            if let Some(tutorial) = tutorial_shown.as_mut() {
                tutorial.0 = true;
                tracing::info!("Tutorial dismissed via API");
            } else {
                tracing::warn!("TutorialShown resource not available");
            }
        }
        _ => {
            tracing::warn!("Unknown command: {}", parts[0]);
        }
    }
}

/// Parse GameAction from string
pub fn parse_game_action(s: &str) -> Option<GameAction> {
    match s {
        "MoveForward" => Some(GameAction::MoveForward),
        "MoveBackward" => Some(GameAction::MoveBackward),
        "MoveLeft" => Some(GameAction::MoveLeft),
        "MoveRight" => Some(GameAction::MoveRight),
        "Jump" => Some(GameAction::Jump),
        "Descend" => Some(GameAction::Descend),
        "LookUp" => Some(GameAction::LookUp),
        "LookDown" => Some(GameAction::LookDown),
        "LookLeft" => Some(GameAction::LookLeft),
        "LookRight" => Some(GameAction::LookRight),
        "ToggleInventory" => Some(GameAction::ToggleInventory),
        "TogglePause" => Some(GameAction::TogglePause),
        "ToggleQuest" => Some(GameAction::ToggleQuest),
        "OpenCommand" => Some(GameAction::OpenCommand),
        "CloseUI" => Some(GameAction::CloseUI),
        "Confirm" => Some(GameAction::Confirm),
        "Cancel" => Some(GameAction::Cancel),
        "Hotbar1" => Some(GameAction::Hotbar1),
        "Hotbar2" => Some(GameAction::Hotbar2),
        "Hotbar3" => Some(GameAction::Hotbar3),
        "Hotbar4" => Some(GameAction::Hotbar4),
        "Hotbar5" => Some(GameAction::Hotbar5),
        "Hotbar6" => Some(GameAction::Hotbar6),
        "Hotbar7" => Some(GameAction::Hotbar7),
        "Hotbar8" => Some(GameAction::Hotbar8),
        "Hotbar9" => Some(GameAction::Hotbar9),
        "PrimaryAction" => Some(GameAction::PrimaryAction),
        "SecondaryAction" => Some(GameAction::SecondaryAction),
        "RotateBlock" => Some(GameAction::RotateBlock),
        "ModifierShift" => Some(GameAction::ModifierShift),
        "ToggleDebug" => Some(GameAction::ToggleDebug),
        "DeleteChar" => Some(GameAction::DeleteChar),
        _ => None,
    }
}
