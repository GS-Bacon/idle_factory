//! Save/Load system implementations

use super::format as save;
use crate::components::{LoadGameEvent, SaveGameEvent};
use crate::components::{MachineBundle, *};
use crate::core::{items, ItemId};
use crate::game_spec::{CRUSHER, FURNACE, MINER};
use crate::player::{LocalPlatformInventory, LocalPlayer, PlatformInventory, PlayerInventory};
use crate::world::WorldData;
use crate::{Direction, BLOCK_SIZE};
use bevy::prelude::*;
use tracing::info;

/// Collect all game state into SaveDataV2 (string ID format)
#[allow(clippy::too_many_arguments)]
pub fn collect_save_data(
    player_query: &Query<&Transform, With<Player>>,
    camera_query: &Query<&PlayerCamera>,
    inventory: &PlayerInventory,
    world_data: &WorldData,
    machine_query: &Query<&Machine>,
    conveyor_query: &Query<&Conveyor>,
    _delivery_query: &Query<&DeliveryPlatform>,
    current_quest: &CurrentQuest,
    creative_mode: &CreativeMode,
    platform_inventory: &PlatformInventory,
) -> save::SaveDataV2 {
    use save::*;

    // Get current timestamp
    let timestamp = {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    };

    // Collect player data
    let player_data = if let Ok(transform) = player_query.single() {
        let rotation = camera_query
            .single()
            .map(|c| CameraRotation {
                pitch: c.pitch,
                yaw: c.yaw,
            })
            .unwrap_or(CameraRotation {
                pitch: 0.0,
                yaw: 0.0,
            });
        PlayerSaveData {
            position: transform.translation.into(),
            rotation,
        }
    } else {
        PlayerSaveData {
            position: Vec3Save {
                x: 8.0,
                y: 12.0,
                z: 20.0,
            },
            rotation: CameraRotation {
                pitch: 0.0,
                yaw: 0.0,
            },
        }
    };

    // Helper to convert ItemId to string ID
    fn item_id_to_string(id: ItemId) -> String {
        id.name().unwrap_or("base:unknown").to_string()
    }

    // Collect inventory data (V2 format with string IDs)
    let inventory_data = InventorySaveDataV2 {
        selected_slot: inventory.selected_slot,
        slots: inventory
            .slots
            .iter()
            .map(|slot| {
                slot.map(|(item_id, count)| ItemStackV2 {
                    item_id: item_id_to_string(item_id),
                    count,
                })
            })
            .collect(),
    };

    // Collect world modifications (V2 format with string IDs)
    let modified_blocks: std::collections::HashMap<String, Option<String>> = world_data
        .modified_blocks
        .iter()
        .map(|(pos, block)| {
            (
                WorldSaveDataV2::pos_to_key(*pos),
                block.and_then(|b| b.name().map(|s| s.to_string())),
            )
        })
        .collect();

    let world_save = WorldSaveDataV2 { modified_blocks };

    // Collect machines (V2 format)
    let mut machines = Vec::new();

    // All machines (Miner, Furnace, Crusher) using Machine component
    for machine in machine_query.iter() {
        let machine_id = machine.spec.item_id();
        if machine_id == items::miner_block() {
            let buffer = machine
                .slots
                .outputs
                .first()
                .and_then(|s| s.item_id.map(|id| (id, s.count)));
            machines.push(MachineSaveDataV2::Miner(MinerSaveDataV2 {
                position: machine.position.into(),
                progress: machine.progress,
                buffer: buffer.map(|(id, count)| ItemStackV2 {
                    item_id: item_id_to_string(id),
                    count,
                }),
            }));
        } else if machine_id == items::furnace_block() {
            let input = machine
                .slots
                .inputs
                .first()
                .and_then(|s| s.item_id.map(|id| (id, s.count)));
            let output = machine
                .slots
                .outputs
                .first()
                .and_then(|s| s.item_id.map(|id| (id, s.count)));
            machines.push(MachineSaveDataV2::Furnace(FurnaceSaveDataV2 {
                position: machine.position.into(),
                fuel: machine.slots.fuel,
                input: input.map(|(id, count)| ItemStackV2 {
                    item_id: item_id_to_string(id),
                    count,
                }),
                output: output.map(|(id, count)| ItemStackV2 {
                    item_id: item_id_to_string(id),
                    count,
                }),
                progress: machine.progress,
            }));
        } else if machine_id == items::crusher_block() {
            let input = machine
                .slots
                .inputs
                .first()
                .and_then(|s| s.item_id.map(|id| (id, s.count)));
            let output = machine
                .slots
                .outputs
                .first()
                .and_then(|s| s.item_id.map(|id| (id, s.count)));
            machines.push(MachineSaveDataV2::Crusher(CrusherSaveDataV2 {
                position: machine.position.into(),
                input: input.map(|(id, count)| ItemStackV2 {
                    item_id: item_id_to_string(id),
                    count,
                }),
                output: output.map(|(id, count)| ItemStackV2 {
                    item_id: item_id_to_string(id),
                    count,
                }),
                progress: machine.progress,
            }));
        }
    }

    // Conveyors (V2 format)
    for conveyor in conveyor_query.iter() {
        let direction = match conveyor.direction {
            Direction::North => DirectionSave::North,
            Direction::South => DirectionSave::South,
            Direction::East => DirectionSave::East,
            Direction::West => DirectionSave::West,
        };
        let shape = match conveyor.shape {
            ConveyorShape::Straight => ConveyorShapeSave::Straight,
            ConveyorShape::CornerLeft => ConveyorShapeSave::CornerLeft,
            ConveyorShape::CornerRight => ConveyorShapeSave::CornerRight,
            ConveyorShape::TJunction => ConveyorShapeSave::TJunction,
            ConveyorShape::Splitter => ConveyorShapeSave::Splitter,
        };
        let items: Vec<ConveyorItemSaveV2> = conveyor
            .items
            .iter()
            .map(|item| ConveyorItemSaveV2 {
                item_id: item_id_to_string(item.item_id),
                progress: item.progress,
                lateral_offset: item.lateral_offset,
            })
            .collect();

        machines.push(MachineSaveDataV2::Conveyor(ConveyorSaveDataV2 {
            position: conveyor.position.into(),
            direction,
            shape,
            items,
            last_output_index: conveyor.last_output_index,
            last_input_source: conveyor.last_input_source,
        }));
    }

    // Collect quest data (V2 format with string IDs)
    let quest_data = QuestSaveDataV2 {
        current_index: current_quest.index,
        completed: current_quest.completed,
        rewards_claimed: current_quest.rewards_claimed,
        delivered: std::collections::HashMap::new(),
    };

    // Game mode
    let mode_data = GameModeSaveData {
        creative: creative_mode.enabled,
    };

    // Save PlatformInventory items (V2 format with string IDs)
    let platform_inventory_data = PlatformInventorySaveDataV2 {
        items: platform_inventory
            .items_by_id()
            .iter()
            .map(|(id, count)| (item_id_to_string(*id), *count))
            .collect(),
    };

    SaveDataV2 {
        version: save::SAVE_VERSION.to_string(),
        timestamp,
        player: player_data,
        inventory: inventory_data,
        platform_inventory: platform_inventory_data,
        world: world_save,
        machines,
        quests: quest_data,
        mode: mode_data,
    }
}

/// Convert Direction from save format
pub fn direction_from_save(dir: save::DirectionSave) -> Direction {
    match dir {
        save::DirectionSave::North => Direction::North,
        save::DirectionSave::South => Direction::South,
        save::DirectionSave::East => Direction::East,
        save::DirectionSave::West => Direction::West,
    }
}

/// Convert ConveyorShape from save format
pub fn conveyor_shape_from_save(shape: save::ConveyorShapeSave) -> ConveyorShape {
    match shape {
        save::ConveyorShapeSave::Straight => ConveyorShape::Straight,
        save::ConveyorShapeSave::CornerLeft => ConveyorShape::CornerLeft,
        save::ConveyorShapeSave::CornerRight => ConveyorShape::CornerRight,
        save::ConveyorShapeSave::TJunction => ConveyorShape::TJunction,
        save::ConveyorShapeSave::Splitter => ConveyorShape::Splitter,
    }
}

/// Auto-save system - saves game every minute
pub fn auto_save_system(
    time: Res<Time>,
    mut auto_save_timer: ResMut<save::AutoSaveTimer>,
    mut save_events: MessageWriter<SaveGameEvent>,
) {
    auto_save_timer.timer.tick(time.delta());

    if auto_save_timer.timer.just_finished() {
        save_events.write(SaveGameEvent {
            filename: "autosave".to_string(),
        });
        info!("Auto-save triggered");
    }
}

/// Handle save game events
#[allow(clippy::too_many_arguments)]
pub fn handle_save_event(
    mut events: MessageReader<SaveGameEvent>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&PlayerCamera>,
    local_player: Option<Res<LocalPlayer>>,
    inventory_query: Query<&PlayerInventory>,
    world_data: Res<WorldData>,
    machine_query: Query<&Machine>,
    conveyor_query: Query<&Conveyor>,
    delivery_query: Query<&DeliveryPlatform>,
    current_quest: Res<CurrentQuest>,
    creative_mode: Res<CreativeMode>,
    platform_inventory: LocalPlatformInventory,
    mut save_load_state: ResMut<SaveLoadState>,
) {
    // Get local player's inventory
    let Some(local_player) = local_player else {
        return;
    };
    let Ok(inventory) = inventory_query.get(local_player.0) else {
        return;
    };

    // Get platform inventory
    let Some(platform_inv) = platform_inventory.get() else {
        info!("[SAVE] No platform inventory found");
        return;
    };

    for event in events.read() {
        let save_data = collect_save_data(
            &player_query,
            &camera_query,
            inventory,
            &world_data,
            &machine_query,
            &conveyor_query,
            &delivery_query,
            &current_quest,
            &creative_mode,
            platform_inv,
        );

        match save::native::save_game_v2(&save_data, &event.filename) {
            Ok(()) => {
                let msg = format!("Game saved to '{}'", event.filename);
                info!("{}", msg);
                save_load_state.last_message = Some(msg);
            }
            Err(e) => {
                let msg = format!("Failed to save: {}", e);
                info!("{}", msg);
                save_load_state.last_message = Some(msg);
            }
        }
    }
}

/// Helper to parse string ID to ItemId
fn string_id_to_item_id(s: &str) -> Option<ItemId> {
    crate::core::items::by_name(s)
}

/// Handle load game events (V2 format with string IDs)
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_load_event(
    mut events: MessageReader<LoadGameEvent>,
    mut save_load_state: ResMut<SaveLoadState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<&mut PlayerCamera>,
    local_player: Option<Res<LocalPlayer>>,
    mut inventory_query: Query<&mut PlayerInventory>,
    mut world_data: ResMut<WorldData>,
    mut current_quest: ResMut<CurrentQuest>,
    mut creative_mode: ResMut<CreativeMode>,
    mut platform_inventory: LocalPlatformInventory,
    // All machine entities to despawn (combined query)
    machine_entities: Query<Entity, Or<(With<Machine>, With<Conveyor>)>>,
) {
    // Get local player's inventory
    let Some(local_player) = local_player else {
        // If no local player, we can't load inventory
        for event in events.read() {
            let msg = format!("Failed to load '{}': No local player", event.filename);
            info!("{}", msg);
            save_load_state.last_message = Some(msg);
        }
        return;
    };
    let Ok(mut inventory) = inventory_query.get_mut(local_player.0) else {
        for event in events.read() {
            let msg = format!(
                "Failed to load '{}': Local player has no inventory",
                event.filename
            );
            info!("{}", msg);
            save_load_state.last_message = Some(msg);
        }
        return;
    };

    for event in events.read() {
        match save::native::load_game_v2(&event.filename) {
            Ok(data) => {
                // Apply player position
                if let Ok(mut transform) = player_query.single_mut() {
                    transform.translation = data.player.position.into();
                }

                // Apply camera rotation
                if let Ok(mut camera) = camera_query.single_mut() {
                    camera.pitch = data.player.rotation.pitch;
                    camera.yaw = data.player.rotation.yaw;
                }

                // Apply inventory (V2 format with string IDs)
                inventory.selected_slot = data.inventory.selected_slot;
                for (i, slot) in data.inventory.slots.iter().enumerate() {
                    if i < inventory.slots.len() {
                        inventory.slots[i] = slot
                            .as_ref()
                            .and_then(|s| string_id_to_item_id(&s.item_id).map(|id| (id, s.count)));
                    }
                }

                // Migrate platform_inventory items into platform inventory
                if !data.platform_inventory.items.is_empty() {
                    info!("[SAVE] Loading platform_inventory items to platform inventory");
                    for (item_id_str, count) in &data.platform_inventory.items {
                        if let Some(item_id) = string_id_to_item_id(item_id_str) {
                            platform_inventory.add_item(item_id, *count);
                        } else {
                            info!("[SAVE] Unknown item ID: {}, skipping", item_id_str);
                        }
                    }
                }

                // Apply world modifications (V2 format with string IDs)
                world_data.modified_blocks.clear();
                for (key, block_opt) in &data.world.modified_blocks {
                    if let Some(pos) = save::WorldSaveDataV2::key_to_pos(key) {
                        let block = block_opt.as_ref().and_then(|id| items::by_name(id));
                        world_data.modified_blocks.insert(pos, block);
                    }
                }

                // Despawn existing machines
                for entity in machine_entities.iter() {
                    commands.entity(entity).despawn();
                }

                // Spawn machines from save data (V2 format)
                for machine in &data.machines {
                    match machine {
                        save::MachineSaveDataV2::Miner(miner_data) => {
                            let pos: IVec3 = miner_data.position.into();

                            let cube_mesh =
                                meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                            let mut bundle =
                                MachineBundle::new_centered(&MINER, pos, Direction::North);
                            bundle.machine.progress = miner_data.progress;
                            if let Some(buffer) = &miner_data.buffer {
                                if let Some(output_slot) = bundle.machine.slots.outputs.first_mut()
                                {
                                    if let Some(item_id) = string_id_to_item_id(&buffer.item_id) {
                                        output_slot.item_id = Some(item_id);
                                        output_slot.count = buffer.count;
                                    }
                                }
                            }
                            commands.spawn((
                                Mesh3d(cube_mesh),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: items::miner_block().color(),
                                    ..default()
                                })),
                                bundle,
                            ));
                        }
                        save::MachineSaveDataV2::Conveyor(conveyor_data) => {
                            let pos: IVec3 = conveyor_data.position.into();
                            let direction = direction_from_save(conveyor_data.direction);
                            let shape = conveyor_shape_from_save(conveyor_data.shape);
                            let world_pos = Vec3::new(
                                pos.x as f32 + 0.5,
                                pos.y as f32 + 0.5,
                                pos.z as f32 + 0.5,
                            );

                            let items: Vec<ConveyorItem> = conveyor_data
                                .items
                                .iter()
                                .filter_map(|item| {
                                    string_id_to_item_id(&item.item_id).map(|item_id| {
                                        let mut ci = ConveyorItem::new(item_id, item.progress);
                                        ci.lateral_offset = item.lateral_offset;
                                        ci.previous_lateral_offset = item.lateral_offset;
                                        ci
                                    })
                                })
                                .collect();

                            // Use simple cuboid mesh for now
                            let mesh = meshes.add(Cuboid::new(
                                BLOCK_SIZE * 0.9,
                                BLOCK_SIZE * 0.2,
                                BLOCK_SIZE,
                            ));

                            commands.spawn((
                                Conveyor {
                                    position: pos,
                                    direction,
                                    output_direction: direction, // Will be updated by update_conveyor_shapes
                                    items,
                                    last_output_index: conveyor_data.last_output_index,
                                    last_input_source: conveyor_data.last_input_source,
                                    shape,
                                },
                                Mesh3d(mesh),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: items::conveyor_block().color(),
                                    ..default()
                                })),
                                Transform::from_translation(world_pos)
                                    .with_rotation(direction.to_rotation()),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ));
                        }
                        save::MachineSaveDataV2::Furnace(furnace_data) => {
                            let pos: IVec3 = furnace_data.position.into();

                            let cube_mesh =
                                meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                            let mut bundle =
                                MachineBundle::new_centered(&FURNACE, pos, Direction::North);
                            bundle.machine.slots.fuel = furnace_data.fuel;
                            bundle.machine.progress = furnace_data.progress;
                            if let Some(input) = &furnace_data.input {
                                if let Some(input_slot) = bundle.machine.slots.inputs.first_mut() {
                                    if let Some(item_id) = string_id_to_item_id(&input.item_id) {
                                        input_slot.item_id = Some(item_id);
                                        input_slot.count = input.count;
                                    }
                                }
                            }
                            if let Some(output) = &furnace_data.output {
                                if let Some(output_slot) = bundle.machine.slots.outputs.first_mut()
                                {
                                    if let Some(item_id) = string_id_to_item_id(&output.item_id) {
                                        output_slot.item_id = Some(item_id);
                                        output_slot.count = output.count;
                                    }
                                }
                            }
                            commands.spawn((
                                Mesh3d(cube_mesh),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: items::furnace_block().color(),
                                    ..default()
                                })),
                                bundle,
                            ));
                        }
                        save::MachineSaveDataV2::Crusher(crusher_data) => {
                            let pos: IVec3 = crusher_data.position.into();

                            let cube_mesh =
                                meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                            let mut bundle =
                                MachineBundle::new_centered(&CRUSHER, pos, Direction::North);
                            bundle.machine.progress = crusher_data.progress;
                            if let Some(input) = &crusher_data.input {
                                if let Some(input_slot) = bundle.machine.slots.inputs.first_mut() {
                                    if let Some(item_id) = string_id_to_item_id(&input.item_id) {
                                        input_slot.item_id = Some(item_id);
                                        input_slot.count = input.count;
                                    }
                                }
                            }
                            if let Some(output) = &crusher_data.output {
                                if let Some(output_slot) = bundle.machine.slots.outputs.first_mut()
                                {
                                    if let Some(item_id) = string_id_to_item_id(&output.item_id) {
                                        output_slot.item_id = Some(item_id);
                                        output_slot.count = output.count;
                                    }
                                }
                            }
                            commands.spawn((
                                Mesh3d(cube_mesh),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: items::crusher_block().color(),
                                    ..default()
                                })),
                                bundle,
                            ));
                        }
                    }
                }

                // Apply quest progress (V2 format)
                current_quest.index = data.quests.current_index;
                current_quest.completed = data.quests.completed;
                current_quest.rewards_claimed = data.quests.rewards_claimed;

                // Note: quests.delivered is now empty in V2 format
                // PlatformInventory is loaded from platform_inventory above

                // Apply game mode
                creative_mode.enabled = data.mode.creative;

                let msg = format!("Game loaded from '{}'", event.filename);
                info!("{}", msg);
                save_load_state.last_message = Some(msg);
            }
            Err(e) => {
                let msg = format!("Failed to load '{}': {}", event.filename, e);
                info!("{}", msg);
                save_load_state.last_message = Some(msg);
            }
        }
    }
}
