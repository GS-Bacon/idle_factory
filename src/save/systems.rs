//! Save/Load system implementations

use super::format as save;
use crate::components::*;
use crate::components::{Furnace, LoadGameEvent, SaveGameEvent};
use crate::player::Inventory;
use crate::world::WorldData;
use crate::{BlockType, Direction, BLOCK_SIZE};
use bevy::prelude::*;
use tracing::info;

/// Collect all game state into SaveData
#[allow(clippy::too_many_arguments)]
pub fn collect_save_data(
    player_query: &Query<&Transform, With<Player>>,
    camera_query: &Query<&PlayerCamera>,
    inventory: &Inventory,
    world_data: &WorldData,
    miner_query: &Query<&Miner>,
    conveyor_query: &Query<&Conveyor>,
    furnace_query: &Query<(&Furnace, &GlobalTransform)>,
    crusher_query: &Query<&Crusher>,
    _delivery_query: &Query<&DeliveryPlatform>,
    current_quest: &CurrentQuest,
    creative_mode: &CreativeMode,
    global_inventory: &crate::player::GlobalInventory,
) -> save::SaveData {
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
    let player_data = if let Ok(transform) = player_query.get_single() {
        let rotation = camera_query
            .get_single()
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

    // Collect inventory data
    let inventory_data = InventorySaveData {
        selected_slot: inventory.selected_slot,
        slots: inventory
            .slots
            .iter()
            .map(|slot| {
                slot.map(|(bt, count)| ItemStack {
                    item_type: bt.into(),
                    count,
                })
            })
            .collect(),
    };

    // Collect world modifications
    let modified_blocks: std::collections::HashMap<String, Option<BlockTypeSave>> = world_data
        .modified_blocks
        .iter()
        .map(|(pos, block)| (WorldSaveData::pos_to_key(*pos), block.map(|b| b.into())))
        .collect();

    let world_save = WorldSaveData { modified_blocks };

    // Collect machines
    let mut machines = Vec::new();

    // Miners
    for miner in miner_query.iter() {
        machines.push(MachineSaveData::Miner(MinerSaveData {
            position: miner.position.into(),
            progress: miner.progress,
            buffer: miner.buffer.map(|(bt, count)| ItemStack {
                item_type: bt.into(),
                count,
            }),
        }));
    }

    // Conveyors
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
        let items: Vec<ConveyorItemSave> = conveyor
            .items
            .iter()
            .map(|item| ConveyorItemSave {
                item_type: item.block_type.into(),
                progress: item.progress,
                lateral_offset: item.lateral_offset,
            })
            .collect();

        machines.push(MachineSaveData::Conveyor(ConveyorSaveData {
            position: conveyor.position.into(),
            direction,
            shape,
            items,
            last_output_index: conveyor.last_output_index,
            last_input_source: conveyor.last_input_source,
        }));
    }

    // Furnaces
    for (furnace, transform) in furnace_query.iter() {
        let pos = crate::world_to_grid(transform.translation());
        machines.push(MachineSaveData::Furnace(FurnaceSaveData {
            position: pos.into(),
            fuel: furnace.fuel,
            input: furnace.input_type.map(|bt| ItemStack {
                item_type: bt.into(),
                count: furnace.input_count,
            }),
            output: furnace.output_type.map(|bt| ItemStack {
                item_type: bt.into(),
                count: furnace.output_count,
            }),
            progress: furnace.progress,
        }));
    }

    // Crushers
    for crusher in crusher_query.iter() {
        machines.push(MachineSaveData::Crusher(CrusherSaveData {
            position: crusher.position.into(),
            input: crusher.input_type.map(|bt| ItemStack {
                item_type: bt.into(),
                count: crusher.input_count,
            }),
            output: crusher.output_type.map(|bt| ItemStack {
                item_type: bt.into(),
                count: crusher.output_count,
            }),
            progress: crusher.progress,
        }));
    }

    // Collect quest data (delivered items are now stored in GlobalInventory, saved via global_inventory field)
    let delivered: std::collections::HashMap<BlockTypeSave, u32> = std::collections::HashMap::new();

    let quest_data = QuestSaveData {
        current_index: current_quest.index,
        completed: current_quest.completed,
        rewards_claimed: current_quest.rewards_claimed,
        delivered,
    };

    // Game mode
    let mode_data = GameModeSaveData {
        creative: creative_mode.enabled,
    };

    // Save GlobalInventory items
    let global_inventory_data = GlobalInventorySaveData {
        items: global_inventory
            .items()
            .iter()
            .map(|(bt, count)| ((*bt).into(), *count))
            .collect(),
    };

    SaveData {
        version: save::SAVE_VERSION.to_string(),
        timestamp,
        player: player_data,
        inventory: inventory_data,
        global_inventory: global_inventory_data,
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
    mut save_events: EventWriter<SaveGameEvent>,
) {
    auto_save_timer.timer.tick(time.delta());

    if auto_save_timer.timer.just_finished() {
        save_events.send(SaveGameEvent {
            filename: "autosave".to_string(),
        });
        info!("Auto-save triggered");
    }
}

/// Handle save game events
#[allow(clippy::too_many_arguments)]
pub fn handle_save_event(
    mut events: EventReader<SaveGameEvent>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&PlayerCamera>,
    inventory: Res<Inventory>,
    world_data: Res<WorldData>,
    miner_query: Query<&Miner>,
    conveyor_query: Query<&Conveyor>,
    furnace_query: Query<(&Furnace, &GlobalTransform)>,
    crusher_query: Query<&Crusher>,
    delivery_query: Query<&DeliveryPlatform>,
    current_quest: Res<CurrentQuest>,
    creative_mode: Res<CreativeMode>,
    global_inventory: Res<crate::player::GlobalInventory>,
    mut save_load_state: ResMut<SaveLoadState>,
) {
    for event in events.read() {
        let save_data = collect_save_data(
            &player_query,
            &camera_query,
            &inventory,
            &world_data,
            &miner_query,
            &conveyor_query,
            &furnace_query,
            &crusher_query,
            &delivery_query,
            &current_quest,
            &creative_mode,
            &global_inventory,
        );

        match save::save_game(&save_data, &event.filename) {
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

/// Handle load game events
/// Note: This function uses create_conveyor_mesh from main.rs which needs to be made public
/// or moved to a shared module. For now, we'll keep this in main.rs until full refactor.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_load_event(
    mut events: EventReader<LoadGameEvent>,
    mut save_load_state: ResMut<SaveLoadState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<&mut PlayerCamera>,
    mut inventory: ResMut<Inventory>,
    mut world_data: ResMut<WorldData>,
    mut current_quest: ResMut<CurrentQuest>,
    mut creative_mode: ResMut<CreativeMode>,
    mut global_inventory: ResMut<crate::player::GlobalInventory>,
    // All machine entities to despawn (combined query)
    machine_entities: Query<
        Entity,
        Or<(With<Miner>, With<Conveyor>, With<Furnace>, With<Crusher>)>,
    >,
) {
    for event in events.read() {
        match save::load_game(&event.filename) {
            Ok(data) => {
                // Apply player position
                if let Ok(mut transform) = player_query.get_single_mut() {
                    transform.translation = data.player.position.into();
                }

                // Apply camera rotation
                if let Ok(mut camera) = camera_query.get_single_mut() {
                    camera.pitch = data.player.rotation.pitch;
                    camera.yaw = data.player.rotation.yaw;
                }

                // Apply inventory
                inventory.selected_slot = data.inventory.selected_slot;
                for (i, slot) in data.inventory.slots.iter().enumerate() {
                    if i < inventory.slots.len() {
                        inventory.slots[i] =
                            slot.as_ref().map(|s| (s.item_type.clone().into(), s.count));
                    }
                }

                // Migrate old global_inventory items into regular inventory (v0.2 -> unified)
                if !data.global_inventory.items.is_empty() {
                    info!("[SAVE] Migrating old global_inventory items to unified inventory");
                    for (bt_save, count) in &data.global_inventory.items {
                        let bt: BlockType = bt_save.clone().into();
                        inventory.add_item(bt, *count);
                    }
                }

                // Apply world modifications
                world_data.modified_blocks.clear();
                for (key, block_opt) in &data.world.modified_blocks {
                    if let Some(pos) = save::WorldSaveData::key_to_pos(key) {
                        world_data
                            .modified_blocks
                            .insert(pos, block_opt.as_ref().map(|b| b.clone().into()));
                    }
                }

                // Despawn existing machines
                for entity in machine_entities.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Spawn machines from save data
                for machine in &data.machines {
                    match machine {
                        save::MachineSaveData::Miner(miner_data) => {
                            let pos: IVec3 = miner_data.position.into();
                            let world_pos = Vec3::new(
                                pos.x as f32 + 0.5,
                                pos.y as f32 + 0.5,
                                pos.z as f32 + 0.5,
                            );

                            let cube_mesh =
                                meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                            commands.spawn((
                                Miner {
                                    position: pos,
                                    facing: Direction::North, // Default for old saves
                                    progress: miner_data.progress,
                                    buffer: miner_data
                                        .buffer
                                        .as_ref()
                                        .map(|b| (b.item_type.clone().into(), b.count)),
                                    tick_count: 0,
                                },
                                Mesh3d(cube_mesh),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: BlockType::MinerBlock.color(),
                                    ..default()
                                })),
                                Transform::from_translation(world_pos),
                            ));
                        }
                        save::MachineSaveData::Conveyor(conveyor_data) => {
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
                                .map(|item| {
                                    ConveyorItem {
                                        block_type: item.item_type.clone().into(),
                                        progress: item.progress,
                                        visual_entity: None, // Will be created by update_conveyor_item_visuals
                                        lateral_offset: item.lateral_offset,
                                    }
                                })
                                .collect();

                            // Use simple cuboid mesh for now (create_conveyor_mesh would need to be moved)
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
                                    base_color: BlockType::ConveyorBlock.color(),
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
                        save::MachineSaveData::Furnace(furnace_data) => {
                            let pos: IVec3 = furnace_data.position.into();
                            let world_pos = Vec3::new(
                                pos.x as f32 + 0.5,
                                pos.y as f32 + 0.5,
                                pos.z as f32 + 0.5,
                            );

                            let cube_mesh =
                                meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                            commands.spawn((
                                Furnace {
                                    position: pos,
                                    facing: Direction::North, // Default for old saves
                                    fuel: furnace_data.fuel,
                                    input_type: furnace_data
                                        .input
                                        .as_ref()
                                        .map(|s| s.item_type.clone().into()),
                                    input_count: furnace_data
                                        .input
                                        .as_ref()
                                        .map(|s| s.count)
                                        .unwrap_or(0),
                                    output_type: furnace_data
                                        .output
                                        .as_ref()
                                        .map(|s| s.item_type.clone().into()),
                                    output_count: furnace_data
                                        .output
                                        .as_ref()
                                        .map(|s| s.count)
                                        .unwrap_or(0),
                                    progress: furnace_data.progress,
                                },
                                Mesh3d(cube_mesh),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: BlockType::FurnaceBlock.color(),
                                    ..default()
                                })),
                                Transform::from_translation(world_pos),
                            ));
                        }
                        save::MachineSaveData::Crusher(crusher_data) => {
                            let pos: IVec3 = crusher_data.position.into();
                            let world_pos = Vec3::new(
                                pos.x as f32 + 0.5,
                                pos.y as f32 + 0.5,
                                pos.z as f32 + 0.5,
                            );

                            let cube_mesh =
                                meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                            commands.spawn((
                                Crusher {
                                    position: pos,
                                    facing: Direction::North, // Default for old saves
                                    input_type: crusher_data
                                        .input
                                        .as_ref()
                                        .map(|s| s.item_type.clone().into()),
                                    input_count: crusher_data
                                        .input
                                        .as_ref()
                                        .map(|s| s.count)
                                        .unwrap_or(0),
                                    output_type: crusher_data
                                        .output
                                        .as_ref()
                                        .map(|s| s.item_type.clone().into()),
                                    output_count: crusher_data
                                        .output
                                        .as_ref()
                                        .map(|s| s.count)
                                        .unwrap_or(0),
                                    progress: crusher_data.progress,
                                },
                                Mesh3d(cube_mesh),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: BlockType::CrusherBlock.color(),
                                    ..default()
                                })),
                                Transform::from_translation(world_pos),
                            ));
                        }
                    }
                }

                // Apply quest progress
                current_quest.index = data.quests.current_index;
                current_quest.completed = data.quests.completed;
                current_quest.rewards_claimed = data.quests.rewards_claimed;

                // Restore GlobalInventory (from legacy quests.delivered for backward compatibility)
                let mut restored_items = std::collections::HashMap::new();
                for (bt, count) in &data.quests.delivered {
                    let block_type: BlockType = bt.clone().into();
                    *restored_items.entry(block_type).or_insert(0) += count;
                }
                // Also restore from global_inventory.items if present
                for (bt, count) in &data.global_inventory.items {
                    let block_type: BlockType = bt.clone().into();
                    *restored_items.entry(block_type).or_insert(0) += count;
                }
                global_inventory.set_items(restored_items);

                // Apply game mode
                creative_mode.enabled = data.mode.creative;

                let msg = format!("Game loaded from '{}'", event.filename);
                info!("{}", msg);
                save_load_state.last_message = Some(msg);

                // Force chunk reload by clearing chunks (they will regenerate with modified_blocks applied)
                // Note: This is a simple approach; a more sophisticated one would only reload affected chunks
                world_data.chunks.clear();
            }
            Err(e) => {
                let msg = format!("Failed to load: {}", e);
                info!("{}", msg);
                save_load_state.last_message = Some(msg);
            }
        }
    }
}
