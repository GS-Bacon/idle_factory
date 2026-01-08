//! Game state dump functionality for debugging and test failure analysis

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::components::{Conveyor, Machine, PlayerCamera};
use crate::player::{LocalPlayer, PlayerInventory};
use crate::world::WorldData;

/// Serializable snapshot of game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateDump {
    /// Timestamp of the dump (Unix milliseconds)
    pub timestamp: i64,
    /// Player state
    pub player: PlayerStateDump,
    /// Inventory state
    pub inventory: InventoryDump,
    /// Machine states
    pub machines: MachinesDump,
    /// World statistics
    pub world_stats: WorldStatsDump,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStateDump {
    pub position: [f32; 3],
    pub rotation: [f32; 2], // pitch, yaw
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryDump {
    pub selected_slot: usize,
    pub slots: Vec<Option<ItemStackDump>>,
    pub total_items: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStackDump {
    pub item: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachinesDump {
    /// All machines (unified via Machine component)
    pub machines: Vec<MachineDump>,
    pub conveyors: Vec<ConveyorDump>,
}

/// Unified machine dump (replaces MinerDump/FurnaceDump/CrusherDump)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineDump {
    pub machine_type: String,
    pub position: [i32; 3],
    pub facing: String,
    pub progress: f32,
    pub slot_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConveyorDump {
    pub position: [i32; 3],
    pub direction: String,
    pub shape: String,
    pub item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStatsDump {
    pub chunk_count: usize,
    pub modified_blocks: usize,
    pub block_type_counts: HashMap<String, u32>,
}

/// Dump the current game state from the Bevy World
pub fn dump_game_state(world: &mut World) -> GameStateDump {
    let timestamp = chrono::Utc::now().timestamp_millis();

    GameStateDump {
        timestamp,
        player: extract_player_state(world),
        inventory: extract_inventory(world),
        machines: extract_machines(world),
        world_stats: extract_world_stats(world),
    }
}

fn extract_player_state(world: &mut World) -> PlayerStateDump {
    let mut position = [0.0, 0.0, 0.0];
    let mut rotation = [0.0, 0.0];

    // Find player camera component for rotation
    let mut camera_query = world.query::<&PlayerCamera>();
    if let Some(camera) = camera_query.iter(world).next() {
        rotation = [camera.pitch, camera.yaw];
    }

    // Find camera transform for position
    let mut transform_query = world.query::<(&Camera3d, &Transform)>();
    if let Some((_, transform)) = transform_query.iter(world).next() {
        position = [
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
        ];
    }

    PlayerStateDump { position, rotation }
}

fn extract_inventory(world: &World) -> InventoryDump {
    let mut slots = Vec::new();
    let mut selected_slot = 0;
    let mut total_items = 0u32;

    // Get inventory via LocalPlayer + PlayerInventory
    if let Some(local_player) = world.get_resource::<LocalPlayer>() {
        if let Some(inventory) = world.get::<PlayerInventory>(local_player.0) {
            selected_slot = inventory.selected_slot;
            for slot in &inventory.slots {
                if let Some((item_id, count)) = slot {
                    total_items += count;
                    slots.push(Some(ItemStackDump {
                        item: item_id.name().unwrap_or("unknown").to_string(),
                        count: *count,
                    }));
                } else {
                    slots.push(None);
                }
            }
        }
    }

    InventoryDump {
        selected_slot,
        slots,
        total_items,
    }
}

fn extract_machines(world: &mut World) -> MachinesDump {
    let mut machines = Vec::new();
    let mut conveyors = Vec::new();

    // Extract unified machines (new Machine component)
    let mut machine_query = world.query::<&Machine>();
    for machine in machine_query.iter(world) {
        machines.push(MachineDump {
            machine_type: machine.spec.id.to_string(),
            position: [machine.position.x, machine.position.y, machine.position.z],
            facing: format!("{:?}", machine.facing),
            progress: machine.progress,
            slot_count: machine.slots.inputs.len() + machine.slots.outputs.len(),
        });
    }

    // Extract conveyors
    let mut conveyor_query = world.query::<&Conveyor>();
    for conveyor in conveyor_query.iter(world) {
        conveyors.push(ConveyorDump {
            position: [
                conveyor.position.x,
                conveyor.position.y,
                conveyor.position.z,
            ],
            direction: format!("{:?}", conveyor.direction),
            shape: format!("{:?}", conveyor.shape),
            item_count: conveyor.items.len(),
        });
    }

    MachinesDump {
        machines,
        conveyors,
    }
}

fn extract_world_stats(world: &World) -> WorldStatsDump {
    let mut chunk_count = 0;
    let mut modified_blocks = 0;
    let mut block_type_counts: HashMap<String, u32> = HashMap::new();

    if let Some(world_data) = world.get_resource::<WorldData>() {
        chunk_count = world_data.chunks.len();
        modified_blocks = world_data.modified_blocks.len();

        // Count block types in modified blocks
        for item_id in world_data.modified_blocks.values().flatten() {
            *block_type_counts
                .entry(item_id.name().unwrap_or("unknown").to_string())
                .or_insert(0) += 1;
        }
    }

    WorldStatsDump {
        chunk_count,
        modified_blocks,
        block_type_counts,
    }
}

impl GameStateDump {
    /// Save the dump to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(path, json)
    }

    /// Save with auto-generated filename based on timestamp
    pub fn save_with_timestamp(&self) -> std::io::Result<String> {
        let logs_dir = Path::new("logs");
        if !logs_dir.exists() {
            fs::create_dir_all(logs_dir)?;
        }

        let datetime = chrono::DateTime::from_timestamp_millis(self.timestamp)
            .unwrap_or_else(chrono::Utc::now);
        let filename = format!("logs/state_dump_{}.json", datetime.format("%Y%m%d_%H%M%S"));
        self.save_to_file(&filename)?;
        Ok(filename)
    }

    /// Load a dump from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let json = fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Quick summary for logging
    pub fn summary(&self) -> String {
        format!(
            "State dump: {} machines, {} conveyors, {} items in inventory, {} chunks",
            self.machines.machines.len(),
            self.machines.conveyors.len(),
            self.inventory.total_items,
            self.world_stats.chunk_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_dump_serialization() {
        let dump = GameStateDump {
            timestamp: 1704067200000,
            player: PlayerStateDump {
                position: [10.0, 20.0, 30.0],
                rotation: [0.5, 1.0],
            },
            inventory: InventoryDump {
                selected_slot: 0,
                slots: vec![
                    Some(ItemStackDump {
                        item: "Stone".to_string(),
                        count: 64,
                    }),
                    None,
                ],
                total_items: 64,
            },
            machines: MachinesDump {
                machines: vec![MachineDump {
                    machine_type: "miner".to_string(),
                    position: [5, 10, 5],
                    facing: "North".to_string(),
                    progress: 0.5,
                    slot_count: 1,
                }],
                conveyors: vec![],
            },
            world_stats: WorldStatsDump {
                chunk_count: 9,
                modified_blocks: 10,
                block_type_counts: HashMap::new(),
            },
        };

        let json = serde_json::to_string(&dump).expect("serialization should succeed");
        let restored: GameStateDump =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(restored.timestamp, dump.timestamp);
        assert_eq!(restored.machines.machines.len(), 1);
        assert_eq!(restored.inventory.total_items, 64);
    }

    #[test]
    fn test_summary() {
        let dump = GameStateDump {
            timestamp: 0,
            player: PlayerStateDump {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0],
            },
            inventory: InventoryDump {
                selected_slot: 0,
                slots: vec![],
                total_items: 100,
            },
            machines: MachinesDump {
                machines: vec![MachineDump {
                    machine_type: "miner".to_string(),
                    position: [0, 0, 0],
                    facing: "North".to_string(),
                    progress: 0.0,
                    slot_count: 1,
                }],
                conveyors: vec![ConveyorDump {
                    position: [0, 0, 0],
                    direction: "North".to_string(),
                    shape: "Straight".to_string(),
                    item_count: 0,
                }],
            },
            world_stats: WorldStatsDump {
                chunk_count: 5,
                modified_blocks: 0,
                block_type_counts: HashMap::new(),
            },
        };

        let summary = dump.summary();
        assert!(summary.contains("1 machines"));
        assert!(summary.contains("1 conveyors"));
        assert!(summary.contains("100 items"));
        assert!(summary.contains("5 chunks"));
    }
}
