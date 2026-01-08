//! Event handlers for command system
//!
//! Handles events dispatched by command executor:
//! - Teleport, Look, SetBlock for player/camera control
//! - SpawnMachine for E2E testing
//! - DebugConveyor for debugging

use crate::components::*;
use crate::events::SpawnMachineEvent;
use crate::game_spec::{CRUSHER, FURNACE, MINER};
use crate::world::WorldData;
use crate::{
    BlockType, Conveyor, ConveyorShape, ConveyorVisual, Direction, MachineModels, BLOCK_SIZE,
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
                        Machine::new(&MINER, pos, Direction::North),
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
                        Machine::new(&MINER, pos, Direction::North),
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
                        Machine::new(&FURNACE, pos, Direction::North),
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
                        Machine::new(&FURNACE, pos, Direction::North),
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
                        Machine::new(&CRUSHER, pos, Direction::North),
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
                        Machine::new(&CRUSHER, pos, Direction::North),
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
    machine_query: Query<(Entity, &Machine)>,
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
                            item.block_type_for_render().name(),
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

                let mut miner_count = 0;
                let mut furnace_count = 0;
                let mut crusher_count = 0;

                for (entity, machine) in machine_query.iter() {
                    match machine.spec.block_type {
                        BlockType::MinerBlock => {
                            let output =
                                machine.slots.outputs.first().and_then(|s| {
                                    s.block_type_for_render().map(|bt| (bt, s.count))
                                });
                            info!(
                                "Miner {:?}: pos={:?}, facing={:?}, progress={:.1}%, buffer={:?}",
                                entity,
                                machine.position,
                                machine.facing,
                                machine.progress * 100.0,
                                output.map(|(bt, count)| format!("{}x{}", bt.name(), count)),
                            );
                            miner_count += 1;
                        }
                        BlockType::FurnaceBlock => {
                            let input = machine.slots.inputs.first();
                            let output = machine.slots.outputs.first();
                            info!(
                                "Furnace {:?}: pos={:?}, facing={:?}, input={:?}x{}, output={:?}x{}, fuel={}, progress={:.1}%",
                                entity,
                                machine.position,
                                machine.facing,
                                input.and_then(|s| s.block_type_for_render()).map(|b| b.name()),
                                input.map(|s| s.count).unwrap_or(0),
                                output.and_then(|s| s.block_type_for_render()).map(|b| b.name()),
                                output.map(|s| s.count).unwrap_or(0),
                                machine.slots.fuel,
                                machine.progress * 100.0,
                            );
                            furnace_count += 1;
                        }
                        BlockType::CrusherBlock => {
                            let input = machine.slots.inputs.first();
                            let output = machine.slots.outputs.first();
                            info!(
                                "Crusher {:?}: pos={:?}, facing={:?}, input={:?}x{}, output={:?}x{}, progress={:.1}%",
                                entity,
                                machine.position,
                                machine.facing,
                                input.and_then(|s| s.block_type_for_render()).map(|b| b.name()),
                                input.map(|s| s.count).unwrap_or(0),
                                output.and_then(|s| s.block_type_for_render()).map(|b| b.name()),
                                output.map(|s| s.count).unwrap_or(0),
                                machine.progress * 100.0,
                            );
                            crusher_count += 1;
                        }
                        _ => {}
                    }
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

                for (entity, machine) in machine_query.iter() {
                    let input_pos = machine.position + machine.facing.opposite().to_ivec3();
                    let output_pos = machine.position + machine.facing.to_ivec3();
                    let input_connected = conveyor_positions.get(&input_pos);
                    let output_connected = conveyor_positions.get(&output_pos);

                    let machine_name = match machine.spec.block_type {
                        BlockType::MinerBlock => "Miner",
                        BlockType::FurnaceBlock => "Furnace",
                        BlockType::CrusherBlock => "Crusher",
                        _ => "Unknown",
                    };

                    info!(
                        "{} {:?} @ {:?} facing {:?}",
                        machine_name, entity, machine.position, machine.facing
                    );
                    info!(
                        "  input {:?}: {:?}",
                        input_pos,
                        input_connected
                            .map(|e| format!("{:?}", e))
                            .unwrap_or_else(|| "none".to_string())
                    );
                    info!(
                        "  output {:?}: {:?}",
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
    machine_query: Query<&Machine>,
    conveyor_query: Query<&Conveyor>,
) {
    for event in events.read() {
        match event.assert_type {
            MachineAssertType::MinerWorking => {
                let working_miners: Vec<_> = machine_query
                    .iter()
                    .filter(|m| {
                        m.spec.block_type == BlockType::MinerBlock
                            && (m.progress > 0.0
                                || m.slots
                                    .outputs
                                    .first()
                                    .map(|s| s.item_id.is_some())
                                    .unwrap_or(false))
                    })
                    .collect();
                let total_miners = machine_query
                    .iter()
                    .filter(|m| m.spec.block_type == BlockType::MinerBlock)
                    .count();
                if !working_miners.is_empty() {
                    info!("PASS: {} miner(s) working", working_miners.len());
                } else {
                    tracing::error!("FAIL: No miners working (total: {})", total_miners);
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
                        "PASS: {} item(s) on {} conveyor(s)",
                        total_items,
                        conveyors_with_items.len()
                    );
                } else {
                    tracing::error!(
                        "FAIL: No items on conveyors (total conveyors: {})",
                        conveyor_query.iter().count()
                    );
                }
            }
            MachineAssertType::MachineCount { machine, min_count } => {
                let count = machine_query
                    .iter()
                    .filter(|m| m.spec.block_type == machine)
                    .count() as u32;
                if count >= min_count {
                    info!(
                        "PASS: {} {:?}(s) found (min: {})",
                        count, machine, min_count
                    );
                } else {
                    tracing::error!(
                        "FAIL: Only {} {:?}(s) found (min: {})",
                        count,
                        machine,
                        min_count
                    );
                }
            }
        }
    }
}

/// Handle screenshot events
pub fn handle_screenshot_event(mut events: EventReader<ScreenshotEvent>, mut commands: Commands) {
    for event in events.read() {
        info!("Taking screenshot: {}", event.filename);
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(event.filename.clone()));
    }
}
