//! Command execution logic
//!
//! Parses and executes slash commands like /creative, /give, /tp, etc.

use crate::components::{CreativeMode, SaveGameEvent, LoadGameEvent};
use crate::events::SpawnMachineEvent;
use crate::player::Inventory;
use crate::BlockType;
use bevy::prelude::*;
use tracing::info;

use super::{TeleportEvent, LookEvent, SetBlockEvent, DebugConveyorEvent};

/// Parse item name to BlockType
pub fn parse_item_name(name: &str) -> Option<BlockType> {
    match name {
        "stone" => Some(BlockType::Stone),
        "grass" => Some(BlockType::Grass),
        "ironore" | "iron_ore" => Some(BlockType::IronOre),
        "copperore" | "copper_ore" => Some(BlockType::CopperOre),
        "coal" => Some(BlockType::Coal),
        "ironingot" | "iron_ingot" | "iron" => Some(BlockType::IronIngot),
        "copperingot" | "copper_ingot" | "copper" => Some(BlockType::CopperIngot),
        "miner" => Some(BlockType::MinerBlock),
        "conveyor" => Some(BlockType::ConveyorBlock),
        "crusher" => Some(BlockType::CrusherBlock),
        "furnace" => Some(BlockType::FurnaceBlock),
        _ => None,
    }
}

/// Execute a command
#[allow(clippy::too_many_arguments)]
pub fn execute_command(
    command: &str,
    creative_mode: &mut ResMut<CreativeMode>,
    inventory: &mut ResMut<Inventory>,
    save_events: &mut EventWriter<SaveGameEvent>,
    load_events: &mut EventWriter<LoadGameEvent>,
    tp_events: &mut EventWriter<TeleportEvent>,
    look_events: &mut EventWriter<LookEvent>,
    setblock_events: &mut EventWriter<SetBlockEvent>,
    spawn_machine_events: &mut EventWriter<SpawnMachineEvent>,
    debug_conveyor_events: &mut EventWriter<DebugConveyorEvent>,
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
            // Give all items when entering creative mode
            let all_items = [
                BlockType::Stone,
                BlockType::Grass,
                BlockType::IronOre,
                BlockType::Coal,
                BlockType::IronIngot,
                BlockType::CopperOre,
                BlockType::CopperIngot,
                BlockType::MinerBlock,
                BlockType::ConveyorBlock,
                BlockType::CrusherBlock,
            ];
            for (i, block_type) in all_items.iter().take(9).enumerate() {
                inventory.slots[i] = Some((*block_type, 64));
            }
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

                if let Some(block_type) = parse_item_name(&item_name) {
                    inventory.add_item(block_type, count);
                    info!("Gave {} x{}", block_type.name(), count);
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
            save_events.send(SaveGameEvent { filename });
        }
        "/load" | "load" => {
            // /load [filename]
            let filename = parts.get(1).unwrap_or(&"quicksave").to_string();
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
                tp_events.send(TeleportEvent { position: Vec3::new(x, y, z) });
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
                if let Some(block_type) = parse_item_name(&block_name) {
                    setblock_events.send(SetBlockEvent {
                        position: IVec3::new(x, y, z),
                        block_type,
                    });
                    info!("Setting block at ({}, {}, {}) to {}", x, y, z, block_type.name());
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

                if let Some(machine_type) = parse_item_name(&machine_name) {
                    spawn_machine_events.send(SpawnMachineEvent {
                        position: IVec3::new(x, y, z),
                        machine_type,
                        direction,
                    });
                    info!("Spawning {} at ({}, {}, {})", machine_type.name(), x, y, z);
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
                let machine = parts.get(5).and_then(|s| parse_item_name(&s.to_lowercase())).unwrap_or(BlockType::ConveyorBlock);

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
                        machine_type: machine,
                        direction: Some(dir),
                    });
                }
                info!("Spawned {} {} machines starting at ({}, {})", count, machine.name(), start_x, start_z);
            } else {
                info!("Usage: /spawn_line start_x start_z direction count [machine]");
            }
        }
        "/test" | "test" => {
            // /test [scenario] - Run E2E test scenarios
            match parts.get(1).map(|s| s.as_ref()) {
                Some("production") => {
                    // Production line: Miner -> Conveyor x3 -> Furnace
                    // Place miner on iron ore
                    spawn_machine_events.send(SpawnMachineEvent {
                        position: IVec3::new(0, 8, 0),
                        machine_type: BlockType::MinerBlock,
                        direction: None,
                    });
                    // Conveyors from miner to furnace
                    for i in 1..4 {
                        spawn_machine_events.send(SpawnMachineEvent {
                            position: IVec3::new(i, 8, 0),
                            machine_type: BlockType::ConveyorBlock,
                            direction: Some(1), // East
                        });
                    }
                    // Furnace at the end
                    spawn_machine_events.send(SpawnMachineEvent {
                        position: IVec3::new(4, 8, 0),
                        machine_type: BlockType::FurnaceBlock,
                        direction: None,
                    });
                    // Give coal for furnace
                    inventory.add_item(BlockType::Coal, 16);
                    info!("Production test: Miner -> 3x Conveyor -> Furnace spawned at y=8");
                }
                Some("stress") => {
                    // Stress test: 10x10 conveyor grid
                    for x in 0..10 {
                        for z in 0..10 {
                            spawn_machine_events.send(SpawnMachineEvent {
                                position: IVec3::new(x, 8, z),
                                machine_type: BlockType::ConveyorBlock,
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

                        if let Some(block_type) = parse_item_name(&item_name) {
                            let actual = inventory.get_item_count(block_type);
                            if actual >= min_count {
                                info!("✓ PASS: {} >= {} (actual: {})", block_type.name(), min_count, actual);
                            } else {
                                info!("✗ FAIL: {} < {} (actual: {})", block_type.name(), min_count, actual);
                            }
                        } else {
                            info!("Unknown item: {}", item_name);
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
                                if let Some((actual_type, actual_count)) = inventory.slots[slot_idx] {
                                    if actual_type == expected_type && actual_count >= expected_count {
                                        info!("✓ PASS: slot {} = {} x{}", slot_idx, actual_type.name(), actual_count);
                                    } else {
                                        info!("✗ FAIL: slot {} = {} x{} (expected {} x{})", slot_idx, actual_type.name(), actual_count, expected_type.name(), expected_count);
                                    }
                                } else {
                                    info!("✗ FAIL: slot {} is empty", slot_idx);
                                }
                            } else {
                                info!("Invalid slot index: {}", slot_idx);
                            }
                        }
                    } else {
                        info!("Usage: /assert slot <index> <item> <count>");
                    }
                }
                _ => {
                    info!("Usage: /assert [inventory|slot] ...");
                    info!("  /assert inventory <item> <min_count>");
                    info!("  /assert slot <index> <item> <count>");
                }
            }
        }
        "/debug_conveyor" | "debug_conveyor" => {
            // Trigger debug conveyor event (handled by separate system with Query access)
            debug_conveyor_events.send(DebugConveyorEvent);
            info!("Dumping conveyor debug info...");
        }
        _ => {
            info!("Unknown command: {}", command);
        }
    }
}
