//! Game state dump functionality for debugging and test failure analysis

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::components::{Conveyor, Crusher, Furnace, Miner, PlayerCamera};
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
    pub miners: Vec<MinerDump>,
    pub conveyors: Vec<ConveyorDump>,
    pub furnaces: Vec<FurnaceDump>,
    pub crushers: Vec<CrusherDump>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerDump {
    pub position: [i32; 3],
    pub progress: f32,
    pub buffer_item: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConveyorDump {
    pub position: [i32; 3],
    pub direction: String,
    pub shape: String,
    pub item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FurnaceDump {
    pub position: [i32; 3],
    pub progress: f32,
    pub fuel: u32,
    pub has_input: bool,
    pub has_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrusherDump {
    pub position: [i32; 3],
    pub progress: f32,
    pub has_input: bool,
    pub has_output: bool,
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
                if let Some((block_type, count)) = slot {
                    total_items += count;
                    slots.push(Some(ItemStackDump {
                        item: block_type.name().to_string(),
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
    let mut miners = Vec::new();
    let mut conveyors = Vec::new();
    let mut furnaces = Vec::new();
    let mut crushers = Vec::new();

    // Extract miners
    let mut miner_query = world.query::<&Miner>();
    for miner in miner_query.iter(world) {
        miners.push(MinerDump {
            position: [miner.position.x, miner.position.y, miner.position.z],
            progress: miner.progress,
            buffer_item: miner.buffer.as_ref().map(|(bt, _)| bt.name().to_string()),
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

    // Extract furnaces
    let mut furnace_query = world.query::<&Furnace>();
    for furnace in furnace_query.iter(world) {
        furnaces.push(FurnaceDump {
            position: [furnace.position.x, furnace.position.y, furnace.position.z],
            progress: furnace.progress,
            fuel: furnace.fuel,
            has_input: furnace.input_type.is_some(),
            has_output: furnace.output_type.is_some(),
        });
    }

    // Extract crushers
    let mut crusher_query = world.query::<&Crusher>();
    for crusher in crusher_query.iter(world) {
        crushers.push(CrusherDump {
            position: [crusher.position.x, crusher.position.y, crusher.position.z],
            progress: crusher.progress,
            has_input: crusher.input_type.is_some(),
            has_output: crusher.output_type.is_some(),
        });
    }

    MachinesDump {
        miners,
        conveyors,
        furnaces,
        crushers,
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
        for block_type in world_data.modified_blocks.values().flatten() {
            *block_type_counts
                .entry(block_type.name().to_string())
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
            "State dump: {} miners, {} conveyors, {} furnaces, {} crushers, {} items in inventory, {} chunks",
            self.machines.miners.len(),
            self.machines.conveyors.len(),
            self.machines.furnaces.len(),
            self.machines.crushers.len(),
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
                miners: vec![MinerDump {
                    position: [5, 10, 5],
                    progress: 0.5,
                    buffer_item: Some("IronOre".to_string()),
                }],
                conveyors: vec![],
                furnaces: vec![],
                crushers: vec![],
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
        assert_eq!(restored.machines.miners.len(), 1);
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
                miners: vec![MinerDump {
                    position: [0, 0, 0],
                    progress: 0.0,
                    buffer_item: None,
                }],
                conveyors: vec![ConveyorDump {
                    position: [0, 0, 0],
                    direction: "North".to_string(),
                    shape: "Straight".to_string(),
                    item_count: 0,
                }],
                furnaces: vec![],
                crushers: vec![],
            },
            world_stats: WorldStatsDump {
                chunk_count: 5,
                modified_blocks: 0,
                block_type_counts: HashMap::new(),
            },
        };

        let summary = dump.summary();
        assert!(summary.contains("1 miners"));
        assert!(summary.contains("1 conveyors"));
        assert!(summary.contains("100 items"));
        assert!(summary.contains("5 chunks"));
    }
}
