//! Command execution logic
//!
//! Parses and executes slash commands like /creative, /give, /tp, etc.

use crate::components::{CreativeMode, LoadGameEvent, SaveGameEvent};
use crate::core::{items, ItemId};
use crate::events::SpawnMachineEvent;
use crate::player::PlayerInventory;
use crate::utils::parse_item_name;
use bevy::prelude::*;
use tracing::info;

use super::{
    AssertMachineEvent, DebugEvent, DebugEventType, LookEvent, MachineAssertType, ScreenshotEvent,
    SetBlockEvent, TeleportEvent,
};

/// Execute a command
#[allow(clippy::too_many_arguments)]
pub fn execute_command(
    command: &str,
    creative_mode: &mut ResMut<CreativeMode>,
    inventory: &mut Mut<PlayerInventory>,
    save_events: &mut EventWriter<SaveGameEvent>,
    load_events: &mut EventWriter<LoadGameEvent>,
    tp_events: &mut EventWriter<TeleportEvent>,
    look_events: &mut EventWriter<LookEvent>,
    setblock_events: &mut EventWriter<SetBlockEvent>,
    spawn_machine_events: &mut EventWriter<SpawnMachineEvent>,
    debug_events: &mut EventWriter<DebugEvent>,
    assert_machine_events: &mut EventWriter<AssertMachineEvent>,
    screenshot_events: &mut EventWriter<ScreenshotEvent>,
) {
    info!("execute_command called with: '{}'", command);
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        info!("Command is empty, returning");
        return;
    }

    info!("Command parts: {:?}", parts);
    match parts[0] {
        "/creative" | "creative" => {
            creative_mode.enabled = true;
            info!("Creative mode enabled");
        }
        "/survival" | "survival" => {
            creative_mode.enabled = false;
            info!("Survival mode enabled");
        }
        "/give" | "give" => {
            // /give <item> [count]
            if parts.len() >= 2 {
                let item_name = parts[1].to_lowercase();
                let count: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(64);

                if let Some(item_id) = parse_item_name(&item_name) {
                    inventory.add_item_by_id(item_id, count);
                    info!("Gave {:?} x{}", item_id.name(), count);
                }
            }
        }
        "/clear" | "clear" => {
            // Clear inventory
            for slot in inventory.slots.iter_mut() {
                *slot = None;
            }
            info!("Inventory cleared");
        }
        "/save" | "save" => {
            // /save [filename]
            let filename = parts.get(1).unwrap_or(&"quicksave").to_string();
            // Security: prevent path traversal
            if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
                tracing::error!("Invalid filename: path traversal not allowed");
                return;
            }
            save_events.send(SaveGameEvent { filename });
        }
        "/load" | "load" => {
            // /load [filename]
            let filename = parts.get(1).unwrap_or(&"quicksave").to_string();
            // Security: prevent path traversal
            if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
                tracing::error!("Invalid filename: path traversal not allowed");
                return;
            }
            load_events.send(LoadGameEvent { filename });
        }
        "/help" | "help" => {
            info!("Commands: /creative, /survival, /give <item> [count], /clear, /save [name], /load [name], /tp x y z, /look pitch yaw, /setblock x y z type");
        }
        "/tp" | "tp" => {
            // /tp x y z - Teleport player
            if parts.len() >= 4 {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(12.0);
                let z: f32 = parts[3].parse().unwrap_or(0.0);
                // Security: prevent NaN/Infinity
                if !x.is_finite() || !y.is_finite() || !z.is_finite() {
                    tracing::error!("Invalid coordinates: NaN/Infinity not allowed");
                    return;
                }
                tp_events.send(TeleportEvent {
                    position: Vec3::new(x, y, z),
                });
                info!("Teleporting to ({}, {}, {})", x, y, z);
            } else {
                info!("Usage: /tp x y z");
            }
        }
        "/look" | "look" => {
            // /look pitch yaw - Set camera direction (in degrees)
            if parts.len() >= 3 {
                let pitch_deg: f32 = parts[1].parse().unwrap_or(0.0);
                let yaw_deg: f32 = parts[2].parse().unwrap_or(0.0);
                // Security: prevent NaN/Infinity
                if !pitch_deg.is_finite() || !yaw_deg.is_finite() {
                    tracing::error!("Invalid angles: NaN/Infinity not allowed");
                    return;
                }
                let pitch = pitch_deg.to_radians();
                let yaw = yaw_deg.to_radians();
                look_events.send(LookEvent { pitch, yaw });
                info!("Looking at pitch={:.1}° yaw={:.1}°", pitch_deg, yaw_deg);
            } else {
                info!("Usage: /look pitch_deg yaw_deg");
            }
        }
        "/setblock" | "setblock" => {
            // /setblock x y z blocktype - Place a block
            if parts.len() >= 5 {
                let x: i32 = parts[1].parse().unwrap_or(0);
                let y: i32 = parts[2].parse().unwrap_or(0);
                let z: i32 = parts[3].parse().unwrap_or(0);
                let block_name = parts[4].to_lowercase();
                if let Some(item_id) = parse_item_name(&block_name) {
                    setblock_events.send(SetBlockEvent {
                        position: IVec3::new(x, y, z),
                        block_type: item_id,
                    });
                    info!(
                        "Setting block at ({}, {}, {}) to {:?}",
                        x,
                        y,
                        z,
                        item_id.name()
                    );
                } else {
                    info!("Unknown block type: {}", block_name);
                }
            } else {
                info!("Usage: /setblock x y z blocktype");
            }
        }
        "/spawn" | "spawn" => {
            // /spawn x y z machine [direction] - Spawn a machine entity (E2E testing)
            // direction: 0=North, 1=East, 2=South, 3=West (for conveyors)
            if parts.len() >= 5 {
                let x: i32 = parts[1].parse().unwrap_or(0);
                let y: i32 = parts[2].parse().unwrap_or(0);
                let z: i32 = parts[3].parse().unwrap_or(0);
                let machine_name = parts[4].to_lowercase();
                let direction: Option<u8> = parts.get(5).and_then(|s| s.parse().ok());

                if let Some(machine_id) = parse_item_name(&machine_name) {
                    spawn_machine_events.send(SpawnMachineEvent {
                        position: IVec3::new(x, y, z),
                        machine_id,
                        direction,
                    });
                    info!("Spawning {:?} at ({}, {}, {})", machine_id.name(), x, y, z);
                } else {
                    info!("Unknown machine type: {}", machine_name);
                }
            } else {
                info!("Usage: /spawn x y z machine [direction]");
            }
        }
        "/spawn_line" | "spawn_line" => {
            // /spawn_line start_x start_z direction count [machine]
            // Spawn a line of machines for E2E testing
            if parts.len() >= 5 {
                let start_x: i32 = parts[1].parse().unwrap_or(0);
                let start_z: i32 = parts[2].parse().unwrap_or(0);
                let dir: u8 = parts[3].parse().unwrap_or(0);
                let count: u32 = parts[4].parse().unwrap_or(5);
                let machine_id: ItemId = parts
                    .get(5)
                    .and_then(|s| parse_item_name(&s.to_lowercase()))
                    .unwrap_or_else(items::conveyor_block);

                let y = 8; // Default height (surface level)
                let (dx, dz) = match dir {
                    0 => (0, -1), // North
                    1 => (1, 0),  // East
                    2 => (0, 1),  // South
                    3 => (-1, 0), // West
                    _ => (0, -1),
                };

                for i in 0..count {
                    let x = start_x + dx * i as i32;
                    let z = start_z + dz * i as i32;
                    spawn_machine_events.send(SpawnMachineEvent {
                        position: IVec3::new(x, y, z),
                        machine_id,
                        direction: Some(dir),
                    });
                }
                info!(
                    "Spawned {} {:?} machines starting at ({}, {})",
                    count,
                    machine_id.name(),
                    start_x,
                    start_z
                );
            } else {
                info!("Usage: /spawn_line start_x start_z direction count [machine]");
            }
        }
        "/test" | "test" => {
            // /test [scenario] - Run E2E test scenarios
            match parts.get(1).map(|s| s.as_ref()) {
                Some("production") => {
                    use crate::core::items;
                    // Production line: Miner -> Conveyor x3 -> Furnace
                    // Place miner on iron ore
                    spawn_machine_events.send(SpawnMachineEvent {
                        position: IVec3::new(0, 8, 0),
                        machine_id: items::miner_block(),
                        direction: None,
                    });
                    // Conveyors from miner to furnace
                    for i in 1..4 {
                        spawn_machine_events.send(SpawnMachineEvent {
                            position: IVec3::new(i, 8, 0),
                            machine_id: items::conveyor_block(),
                            direction: Some(1), // East
                        });
                    }
                    // Furnace at the end
                    spawn_machine_events.send(SpawnMachineEvent {
                        position: IVec3::new(4, 8, 0),
                        machine_id: items::furnace_block(),
                        direction: None,
                    });
                    // Give coal for furnace
                    inventory.add_item_by_id(items::coal(), 16);
                    info!("Production test: Miner -> 3x Conveyor -> Furnace spawned at y=8");
                }
                Some("stress") => {
                    use crate::core::items;
                    // Stress test: 10x10 conveyor grid
                    for x in 0..10 {
                        for z in 0..10 {
                            spawn_machine_events.send(SpawnMachineEvent {
                                position: IVec3::new(x, 8, z),
                                machine_id: items::conveyor_block(),
                                direction: Some(1), // East
                            });
                        }
                    }
                    info!("Stress test: 100 conveyors spawned");
                }
                _ => {
                    info!("Usage: /test [production|stress]");
                    info!("  production - Miner + Conveyor + Furnace line");
                    info!("  stress - 10x10 conveyor grid");
                }
            }
        }
        "/assert" | "assert" => {
            // /assert inventory <item> <min_count> - Check inventory
            // /assert machine working - Check if machines are working
            match parts.get(1).map(|s| s.as_ref()) {
                Some("inventory") => {
                    if parts.len() >= 4 {
                        let item_name = parts[2].to_lowercase();
                        let min_count: u32 = parts[3].parse().unwrap_or(1);

                        if let Some(item_id) = parse_item_name(&item_name) {
                            let actual = inventory.get_total_count_by_id(item_id);
                            if actual >= min_count {
                                info!(
                                    "✓ PASS: {:?} >= {} (actual: {})",
                                    item_id.name(),
                                    min_count,
                                    actual
                                );
                            } else {
                                tracing::error!(
                                    "✗ FAIL: {:?} < {} (actual: {})",
                                    item_id.name(),
                                    min_count,
                                    actual
                                );
                            }
                        } else {
                            tracing::error!("Unknown item: {}", item_name);
                        }
                    } else {
                        info!("Usage: /assert inventory <item> <min_count>");
                    }
                }
                Some("slot") => {
                    // /assert slot <index> <item> <count>
                    if parts.len() >= 5 {
                        let slot_idx: usize = parts[2].parse().unwrap_or(0);
                        let item_name = parts[3].to_lowercase();
                        let expected_count: u32 = parts[4].parse().unwrap_or(1);

                        if let Some(expected_type) = parse_item_name(&item_name) {
                            if slot_idx < inventory.slots.len() {
                                if let Some((actual_type, actual_count)) = inventory.slots[slot_idx]
                                {
                                    if actual_type == expected_type
                                        && actual_count >= expected_count
                                    {
                                        info!(
                                            "✓ PASS: slot {} = {} x{}",
                                            slot_idx,
                                            actual_type.name().unwrap_or("unknown"),
                                            actual_count
                                        );
                                    } else {
                                        tracing::error!(
                                            "✗ FAIL: slot {} = {} x{} (expected {} x{})",
                                            slot_idx,
                                            actual_type.name().unwrap_or("unknown"),
                                            actual_count,
                                            expected_type.name().unwrap_or("unknown"),
                                            expected_count
                                        );
                                    }
                                } else {
                                    tracing::error!("✗ FAIL: slot {} is empty", slot_idx);
                                }
                            } else {
                                tracing::error!("Invalid slot index: {}", slot_idx);
                            }
                        }
                    } else {
                        info!("Usage: /assert slot <index> <item> <count>");
                    }
                }
                Some("machine") => {
                    // /assert machine miner working - Check if miner is working
                    // /assert machine conveyor items - Check if conveyors have items
                    // /assert machine <type> count <min> - Check machine count
                    match parts.get(2).map(|s| s.as_ref()) {
                        Some("miner") if parts.get(3).map(|s| s.as_ref()) == Some("working") => {
                            assert_machine_events.send(AssertMachineEvent {
                                assert_type: MachineAssertType::MinerWorking,
                            });
                        }
                        Some("conveyor") if parts.get(3).map(|s| s.as_ref()) == Some("items") => {
                            assert_machine_events.send(AssertMachineEvent {
                                assert_type: MachineAssertType::ConveyorHasItems,
                            });
                        }
                        Some(machine_name) if parts.get(3).map(|s| s.as_ref()) == Some("count") => {
                            if let Some(block_type) = parse_item_name(machine_name) {
                                let min_count: u32 =
                                    parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
                                assert_machine_events.send(AssertMachineEvent {
                                    assert_type: MachineAssertType::MachineCount {
                                        machine: block_type,
                                        min_count,
                                    },
                                });
                            } else {
                                tracing::error!("Unknown machine type: {}", machine_name);
                            }
                        }
                        _ => {
                            info!("Usage: /assert machine <subcommand>");
                            info!("  /assert machine miner working");
                            info!("  /assert machine conveyor items");
                            info!("  /assert machine <type> count <min>");
                        }
                    }
                }
                _ => {
                    info!("Usage: /assert [inventory|slot|machine] ...");
                    info!("  /assert inventory <item> <min_count>");
                    info!("  /assert slot <index> <item> <count>");
                    info!("  /assert machine miner working");
                    info!("  /assert machine conveyor items");
                    info!("  /assert machine <type> count <min>");
                }
            }
        }
        "/debug_conveyor" | "debug_conveyor" => {
            debug_events.send(DebugEvent {
                debug_type: DebugEventType::Conveyor,
            });
            info!("Dumping conveyor debug info...");
        }
        "/debug_machine" | "debug_machine" => {
            debug_events.send(DebugEvent {
                debug_type: DebugEventType::Machine,
            });
            info!("Dumping machine debug info...");
        }
        "/debug_connection" | "debug_connection" => {
            debug_events.send(DebugEvent {
                debug_type: DebugEventType::Connection,
            });
            info!("Dumping connection debug info...");
        }
        "/screenshot" | "screenshot" => {
            // /screenshot [filename] - Capture game screen
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let filename = parts
                .get(1)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("screenshot_{}", timestamp));

            // Security: prevent path traversal
            if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
                tracing::error!("Invalid filename: path traversal not allowed");
                return;
            }

            screenshot_events.send(ScreenshotEvent {
                filename: filename.clone(),
            });
            info!("Taking screenshot: {}.png", filename);
        }
        _ => {
            info!("Unknown command: {}", command);
        }
    }
}
