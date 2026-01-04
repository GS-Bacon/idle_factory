//! Event handlers for command system
//!
//! Handles events dispatched by command executor:
//! - Teleport, Look, SetBlock for player/camera control
//! - SpawnMachine for E2E testing
//! - DebugConveyor for debugging

use crate::components::*;
use crate::events::SpawnMachineEvent;
use crate::world::WorldData;
use crate::{
    BlockType, Conveyor, ConveyorShape, ConveyorVisual, Crusher, Direction, Furnace, MachineModels,
    Miner, BLOCK_SIZE,
};
use bevy::prelude::*;
use tracing::info;

use super::{
    AssertMachineEvent, DebugEvent, DebugEventType, LookEvent, MachineAssertType, ScreenshotEvent,
    SetBlockEvent, TeleportEvent,
};
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

/// Handle teleport events
pub fn handle_teleport_event(
    mut events: EventReader<TeleportEvent>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    for event in events.read() {
        info!("TeleportEvent received: {:?}", event.position);
        match player_query.get_single_mut() {
            Ok(mut transform) => {
                transform.translation = event.position;
                info!("Teleported to {:?}", event.position);
            }
            Err(e) => {
                info!("Failed to teleport: {:?}", e);
            }
        }
    }
}

/// Handle look events
pub fn handle_look_event(
    mut events: EventReader<LookEvent>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    for event in events.read() {
        info!(
            "LookEvent received: pitch={:.2} yaw={:.2}",
            event.pitch, event.yaw
        );
        match camera_query.get_single_mut() {
            Ok((mut camera_transform, mut camera)) => {
                camera.pitch = event.pitch;
                camera.yaw = event.yaw;
                // Apply rotation immediately to Transform
                camera_transform.rotation = Quat::from_rotation_x(camera.pitch);
                // Also update player yaw
                if let Ok(mut player_transform) = player_query.get_single_mut() {
                    player_transform.rotation = Quat::from_rotation_y(camera.yaw);
                }
                info!(
                    "Camera updated: pitch={:.2} yaw={:.2}",
                    event.pitch, event.yaw
                );
            }
            Err(e) => {
                info!("Failed to get camera: {:?}", e);
            }
        }
    }
}

/// Handle setblock events
pub fn handle_setblock_event(
    mut events: EventReader<SetBlockEvent>,
    mut world_data: ResMut<WorldData>,
) {
    for event in events.read() {
        world_data.set_block(event.position, event.block_type);
        info!(
            "Set block at {:?} to {:?}",
            event.position, event.block_type
        );
    }
}

/// Handle spawn machine events - creates machine entities directly (for E2E testing)
#[allow(clippy::too_many_arguments)]
pub fn handle_spawn_machine_event(
    mut commands: Commands,
    mut events: EventReader<SpawnMachineEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    machine_models: Res<MachineModels>,
) {
    for event in events.read() {
        let pos = event.position;
        let world_pos = Vec3::new(
            pos.x as f32 * BLOCK_SIZE + 0.5,
            pos.y as f32 * BLOCK_SIZE + 0.5,
            pos.z as f32 * BLOCK_SIZE + 0.5,
        );

        match event.machine_type {
            BlockType::ConveyorBlock => {
                // Direction from event or default to North
                let direction = match event.direction.unwrap_or(0) {
                    0 => Direction::North,
                    1 => Direction::East,
                    2 => Direction::South,
                    3 => Direction::West,
                    _ => Direction::North,
                };

                let conveyor_pos = Vec3::new(
                    pos.x as f32 * BLOCK_SIZE + 0.5,
                    pos.y as f32 * BLOCK_SIZE, // Conveyor sits on top of block
                    pos.z as f32 * BLOCK_SIZE + 0.5,
                );

                if let Some(model_handle) =
                    machine_models.get_conveyor_model(ConveyorShape::Straight)
                {
                    commands.spawn((
                        SceneRoot(model_handle),
                        Transform::from_translation(conveyor_pos)
                            .with_rotation(direction.to_rotation()),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Conveyor {
                            position: pos,
                            direction,
                            output_direction: direction,
                            items: Vec::new(),
                            last_output_index: 0,
                            last_input_source: 0,
                            shape: ConveyorShape::Straight,
                        },
                        ConveyorVisual,
                    ));
                } else {
                    // Fallback to procedural mesh
                    let mesh =
                        meshes.add(Cuboid::new(BLOCK_SIZE * 0.9, BLOCK_SIZE * 0.15, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: BlockType::ConveyorBlock.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::from_translation(conveyor_pos)
                            .with_rotation(direction.to_rotation()),
                        Conveyor {
                            position: pos,
                            direction,
                            output_direction: direction,
                            items: Vec::new(),
                            last_output_index: 0,
                            last_input_source: 0,
                            shape: ConveyorShape::Straight,
                        },
                        ConveyorVisual,
                    ));
                }
                info!("Spawned conveyor at {:?} facing {:?}", pos, direction);
            }
            BlockType::MinerBlock => {
                let transform = Transform::from_translation(world_pos);

                if let Some(model) = machine_models.miner.clone() {
                    commands.spawn((
                        SceneRoot(model),
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Miner {
                            position: pos,
                            ..default()
                        },
                    ));
                } else {
                    let mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: BlockType::MinerBlock.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        transform,
                        Miner {
                            position: pos,
                            ..default()
                        },
                    ));
                }
                info!("Spawned miner at {:?}", pos);
            }
            BlockType::FurnaceBlock => {
                let transform = Transform::from_translation(world_pos);

                if let Some(model) = machine_models.furnace.clone() {
                    commands.spawn((
                        SceneRoot(model),
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Furnace::default(),
                    ));
                } else {
                    let mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: BlockType::FurnaceBlock.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        transform,
                        Furnace::default(),
                    ));
                }
                info!("Spawned furnace at {:?}", pos);
            }
            BlockType::CrusherBlock => {
                let transform = Transform::from_translation(world_pos);

                if let Some(model) = machine_models.crusher.clone() {
                    commands.spawn((
                        SceneRoot(model),
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Crusher {
                            position: pos,
                            facing: Direction::North, // Default for spawned machines
                            ..default()
                        },
                    ));
                } else {
                    let mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: BlockType::CrusherBlock.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        transform,
                        Crusher {
                            position: pos,
                            facing: Direction::North, // Default for spawned machines
                            ..default()
                        },
                    ));
                }
                info!("Spawned crusher at {:?}", pos);
            }
            _ => {
                info!("Cannot spawn {:?} as machine", event.machine_type);
            }
        }
    }
}

/// Handle debug events - dump states based on debug type
pub fn handle_debug_event(
    mut events: EventReader<DebugEvent>,
    conveyor_query: Query<(Entity, &Conveyor, &GlobalTransform)>,
    miner_query: Query<(Entity, &Miner)>,
    furnace_query: Query<(Entity, &Furnace)>,
    crusher_query: Query<(Entity, &Crusher)>,
) {
    for event in events.read() {
        match event.debug_type {
            DebugEventType::Conveyor => {
                info!("=== Conveyor Debug Dump ===");
                let mut count = 0;
                for (entity, conveyor, transform) in conveyor_query.iter() {
                    info!(
                        "Conveyor {:?}: pos={:?}, dir={:?}, shape={:?}, items={}, last_input={}, world_pos={:.1},{:.1},{:.1}",
                        entity,
                        conveyor.position,
                        conveyor.direction,
                        conveyor.shape,
                        conveyor.items.len(),
                        conveyor.last_input_source,
                        transform.translation().x,
                        transform.translation().y,
                        transform.translation().z,
                    );
                    for (i, item) in conveyor.items.iter().enumerate() {
                        info!(
                            "  Item {}: {} @ progress={:.2}, lateral={:.2}",
                            i,
                            item.block_type.name(),
                            item.progress,
                            item.lateral_offset
                        );
                    }
                    count += 1;
                }
                info!("=== Total: {} conveyors ===", count);
            }
            DebugEventType::Machine => {
                info!("=== Machine Debug Dump ===");

                // Miners
                let miner_count = miner_query.iter().count();
                info!("--- Miners ({}) ---", miner_count);
                for (entity, miner) in miner_query.iter() {
                    info!(
                        "  {:?}: pos={:?}, facing={:?}, progress={:.1}%, buffer={:?}",
                        entity,
                        miner.position,
                        miner.facing,
                        miner.progress * 100.0,
                        miner
                            .buffer
                            .map(|(bt, count)| format!("{}x{}", bt.name(), count)),
                    );
                }

                // Furnaces
                let furnace_count = furnace_query.iter().count();
                info!("--- Furnaces ({}) ---", furnace_count);
                for (entity, furnace) in furnace_query.iter() {
                    info!(
                        "  {:?}: pos={:?}, facing={:?}, input={:?}x{}, output={:?}x{}, fuel={}, progress={:.1}%",
                        entity,
                        furnace.position,
                        furnace.facing,
                        furnace.input_type.map(|b| b.name()),
                        furnace.input_count,
                        furnace.output_type.map(|b| b.name()),
                        furnace.output_count,
                        furnace.fuel,
                        furnace.progress * 100.0,
                    );
                }

                // Crushers
                let crusher_count = crusher_query.iter().count();
                info!("--- Crushers ({}) ---", crusher_count);
                for (entity, crusher) in crusher_query.iter() {
                    info!(
                        "  {:?}: pos={:?}, facing={:?}, input={:?}x{}, output={:?}x{}, progress={:.1}%",
                        entity,
                        crusher.position,
                        crusher.facing,
                        crusher.input_type.map(|b| b.name()),
                        crusher.input_count,
                        crusher.output_type.map(|b| b.name()),
                        crusher.output_count,
                        crusher.progress * 100.0,
                    );
                }

                info!(
                    "=== Total: {} miners, {} furnaces, {} crushers ===",
                    miner_count, furnace_count, crusher_count
                );
            }
            DebugEventType::Connection => {
                info!("=== Connection Debug Dump ===");

                // Build position map for conveyors
                let conveyor_positions: std::collections::HashMap<IVec3, Entity> = conveyor_query
                    .iter()
                    .map(|(e, c, _)| (c.position, e))
                    .collect();

                // Check miner output connections
                info!("--- Miner Outputs ---");
                for (entity, miner) in miner_query.iter() {
                    let output_pos = miner.position + miner.facing.to_ivec3();
                    let connected = conveyor_positions.get(&output_pos);
                    info!(
                        "  {:?} @ {:?} facing {:?} -> output {:?}: {:?}",
                        entity,
                        miner.position,
                        miner.facing,
                        output_pos,
                        connected
                            .map(|e| format!("{:?}", e))
                            .unwrap_or_else(|| "none".to_string())
                    );
                }

                // Check furnace input/output connections
                info!("--- Furnace Connections ---");
                for (entity, furnace) in furnace_query.iter() {
                    let input_pos = furnace.position + furnace.facing.opposite().to_ivec3();
                    let output_pos = furnace.position + furnace.facing.to_ivec3();
                    let input_connected = conveyor_positions.get(&input_pos);
                    let output_connected = conveyor_positions.get(&output_pos);
                    info!(
                        "  {:?} @ {:?} facing {:?}",
                        entity, furnace.position, furnace.facing
                    );
                    info!(
                        "    input {:?}: {:?}",
                        input_pos,
                        input_connected
                            .map(|e| format!("{:?}", e))
                            .unwrap_or_else(|| "none".to_string())
                    );
                    info!(
                        "    output {:?}: {:?}",
                        output_pos,
                        output_connected
                            .map(|e| format!("{:?}", e))
                            .unwrap_or_else(|| "none".to_string())
                    );
                }

                // Check crusher input/output connections
                info!("--- Crusher Connections ---");
                for (entity, crusher) in crusher_query.iter() {
                    let input_pos = crusher.position + crusher.facing.opposite().to_ivec3();
                    let output_pos = crusher.position + crusher.facing.to_ivec3();
                    let input_connected = conveyor_positions.get(&input_pos);
                    let output_connected = conveyor_positions.get(&output_pos);
                    info!(
                        "  {:?} @ {:?} facing {:?}",
                        entity, crusher.position, crusher.facing
                    );
                    info!(
                        "    input {:?}: {:?}",
                        input_pos,
                        input_connected
                            .map(|e| format!("{:?}", e))
                            .unwrap_or_else(|| "none".to_string())
                    );
                    info!(
                        "    output {:?}: {:?}",
                        output_pos,
                        output_connected
                            .map(|e| format!("{:?}", e))
                            .unwrap_or_else(|| "none".to_string())
                    );
                }

                info!("=== End Connection Debug ===");
            }
        }
    }
}

/// Handle assert machine events - verify machine states for E2E testing
pub fn handle_assert_machine_event(
    mut events: EventReader<AssertMachineEvent>,
    miner_query: Query<&Miner>,
    conveyor_query: Query<&Conveyor>,
    crusher_query: Query<&Crusher>,
    furnace_query: Query<&Furnace>,
) {
    for event in events.read() {
        match event.assert_type {
            MachineAssertType::MinerWorking => {
                let working_miners: Vec<_> = miner_query
                    .iter()
                    .filter(|m| m.progress > 0.0 || m.buffer.is_some())
                    .collect();
                if !working_miners.is_empty() {
                    info!("✓ PASS: {} miner(s) working", working_miners.len());
                } else {
                    tracing::error!(
                        "✗ FAIL: No miners working (total: {})",
                        miner_query.iter().count()
                    );
                }
            }
            MachineAssertType::ConveyorHasItems => {
                let conveyors_with_items: Vec<_> = conveyor_query
                    .iter()
                    .filter(|c| !c.items.is_empty())
                    .collect();
                let total_items: usize = conveyors_with_items.iter().map(|c| c.items.len()).sum();
                if total_items > 0 {
                    info!(
                        "✓ PASS: {} item(s) on {} conveyor(s)",
                        total_items,
                        conveyors_with_items.len()
                    );
                } else {
                    tracing::error!(
                        "✗ FAIL: No items on conveyors (total conveyors: {})",
                        conveyor_query.iter().count()
                    );
                }
            }
            MachineAssertType::MachineCount { machine, min_count } => {
                let actual_count = match machine {
                    BlockType::MinerBlock => miner_query.iter().count(),
                    BlockType::ConveyorBlock => conveyor_query.iter().count(),
                    BlockType::CrusherBlock => crusher_query.iter().count(),
                    BlockType::FurnaceBlock => furnace_query.iter().count(),
                    _ => 0,
                };
                if actual_count as u32 >= min_count {
                    info!(
                        "✓ PASS: {} count {} >= {}",
                        machine.name(),
                        actual_count,
                        min_count
                    );
                } else {
                    tracing::error!(
                        "✗ FAIL: {} count {} < {}",
                        machine.name(),
                        actual_count,
                        min_count
                    );
                }
            }
        }
    }
}

/// Handle screenshot events - capture game screen using Bevy's Screenshot system
pub fn handle_screenshot_event(mut commands: Commands, mut events: EventReader<ScreenshotEvent>) {
    for event in events.read() {
        // Ensure screenshots directory exists
        let _ = std::fs::create_dir_all("screenshots/game");

        let path = format!("screenshots/game/{}.png", event.filename);

        // Spawn Screenshot entity with observer to save to disk
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));

        info!("Screenshot scheduled: {}", path);
    }
}
