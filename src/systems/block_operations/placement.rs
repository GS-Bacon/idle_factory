//! Block placement system

use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::components::Machine;
use crate::events::game_events::{BlockPlaced, EventSource, MachineSpawned};
use crate::game_spec::{CRUSHER, FURNACE, MINER};
use crate::systems::TutorialEvent;
use crate::utils::{
    auto_conveyor_direction, dda_raycast, ray_aabb_intersection, ray_aabb_intersection_with_normal,
    yaw_to_direction,
};
use crate::world::{DirtyChunks, WorldData};
use crate::{
    BlockType, ContinuousActionTimer, Conveyor, ConveyorRotationOffset, ConveyorShape,
    ConveyorVisual, CreativeMode, DeliveryPlatform, Direction, InputStateResourcesWithCursor,
    MachineModels, PlayerCamera, BLOCK_SIZE, CONVEYOR_BELT_HEIGHT, CONVEYOR_BELT_WIDTH,
    PLATFORM_SIZE, REACH_DISTANCE,
};

use super::{BlockPlaceEvents, ChunkAssets, LocalPlayerInventory, MachinePlaceQueries};

#[allow(clippy::too_many_arguments)]
pub fn block_place(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &PlayerCamera)>,
    machines: MachinePlaceQueries,
    platform_query: Query<&Transform, With<DeliveryPlatform>>,
    mut world_data: ResMut<WorldData>,
    mut player_inventory: LocalPlayerInventory,
    mut dirty_chunks: ResMut<DirtyChunks>,
    mut chunk_assets: ChunkAssets,
    windows: Query<&Window>,
    creative_mode: Res<CreativeMode>,
    input_resources: InputStateResourcesWithCursor,
    mut action_timer: ResMut<ContinuousActionTimer>,
    mut rotation: ResMut<ConveyorRotationOffset>,
    machine_models: Res<MachineModels>,
    mut events: BlockPlaceEvents,
) {
    // Get player entity before consuming inventory
    let player_entity = player_inventory.entity();
    let Some(mut inventory) = player_inventory.get_mut() else {
        return;
    };
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    let input_state = input_resources.get_state();
    if !input_state.allows_block_actions() || !cursor_locked {
        return;
    }

    let can_place = mouse_button.just_pressed(MouseButton::Right)
        || (mouse_button.pressed(MouseButton::Right) && action_timer.place_timer.finished());
    if can_place {
        action_timer.place_timer.reset();
    }

    if !can_place {
        return;
    }

    let Some(selected_type) = inventory.selected_block() else {
        return;
    };

    // Don't allow placing non-placeable items (tools, ingots, etc.)
    if !selected_type.is_placeable() {
        return;
    }

    let Ok((camera_transform, player_camera)) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();
    let half_size = BLOCK_SIZE / 2.0;

    // Check conveyors for raycast hit - allow placing on top of them
    let mut conveyor_hit: Option<(IVec3, Vec3, f32)> = None;
    for conveyor in machines.conveyor.iter() {
        let conveyor_center = Vec3::new(
            conveyor.position.x as f32 * BLOCK_SIZE + 0.5,
            conveyor.position.y as f32 * BLOCK_SIZE + CONVEYOR_BELT_HEIGHT / 2.0,
            conveyor.position.z as f32 * BLOCK_SIZE + 0.5,
        );
        let conveyor_half = Vec3::new(
            BLOCK_SIZE * CONVEYOR_BELT_WIDTH / 2.0,
            CONVEYOR_BELT_HEIGHT / 2.0,
            BLOCK_SIZE / 2.0,
        );
        if let Some((t, normal)) = ray_aabb_intersection_with_normal(
            ray_origin,
            ray_direction,
            conveyor_center - conveyor_half,
            conveyor_center + conveyor_half,
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = conveyor_hit.is_none_or(|h| t < h.2);
                if is_closer {
                    conveyor_hit = Some((conveyor.position, normal, t));
                }
            }
        }
    }

    // Check if looking at any machine (miner, furnace, crusher) - if so, don't place
    for (_, machine_transform) in machines.machine.iter() {
        let machine_pos = machine_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            machine_pos - Vec3::splat(half_size),
            machine_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                return;
            }
        }
    }

    // Find closest block intersection with hit normal using DDA
    let mut closest_hit: Option<(IVec3, Vec3, f32)> = None;

    if let Some(hit) = dda_raycast(ray_origin, ray_direction, REACH_DISTANCE, |pos| {
        world_data.has_block(pos)
    }) {
        let normal = Vec3::new(
            hit.normal.x as f32,
            hit.normal.y as f32,
            hit.normal.z as f32,
        );
        closest_hit = Some((hit.position, normal, hit.distance));
    }

    // Also check DeliveryPlatform for raycast hit
    if let Ok(platform_transform) = platform_query.get_single() {
        let platform_center = platform_transform.translation;
        let platform_half_x = (PLATFORM_SIZE as f32 * BLOCK_SIZE) / 2.0;
        let platform_half_y = BLOCK_SIZE * 0.1;
        let platform_half_z = platform_half_x;

        let platform_min =
            platform_center - Vec3::new(platform_half_x, platform_half_y, platform_half_z);
        let platform_max =
            platform_center + Vec3::new(platform_half_x, platform_half_y, platform_half_z);

        if let Some((hit_t, normal)) =
            ray_aabb_intersection_with_normal(ray_origin, ray_direction, platform_min, platform_max)
        {
            if hit_t > 0.0 && hit_t < REACH_DISTANCE {
                let hit_point = ray_origin + ray_direction * hit_t;
                let hit_block_pos = crate::world_to_grid(hit_point);
                let is_closer = closest_hit.is_none_or(|h| hit_t < h.2);
                if is_closer {
                    closest_hit = Some((hit_block_pos, normal, hit_t));
                }
            }
        }
    }

    // Include conveyor hit if it's closer
    if let Some((conv_pos, conv_normal, conv_t)) = conveyor_hit {
        let is_closer = closest_hit.is_none_or(|h| conv_t < h.2);
        if is_closer {
            closest_hit = Some((conv_pos, conv_normal, conv_t));
        }
    }

    // Place block on the adjacent face
    if let Some((hit_pos, normal, _)) = closest_hit {
        let place_pos = hit_pos
            + IVec3::new(
                normal.x.round() as i32,
                normal.y.round() as i32,
                normal.z.round() as i32,
            );

        // Don't place if already occupied
        if world_data.has_block(place_pos) {
            return;
        }
        for conveyor in machines.conveyor.iter() {
            if conveyor.position == place_pos {
                return;
            }
        }
        for (machine, _) in machines.machine.iter() {
            if machine.position == place_pos {
                return;
            }
        }

        // Consume from inventory (unless in creative mode)
        if !creative_mode.enabled && !inventory.consume_item(selected_type, 1) {
            // Not enough in inventory
            return;
        }

        let chunk_coord = WorldData::world_to_chunk(place_pos);
        let player_facing = yaw_to_direction(player_camera.yaw);

        let facing_direction = if selected_type == BlockType::ConveyorBlock {
            let conveyors: Vec<(IVec3, Direction)> = machines
                .conveyor
                .iter()
                .map(|c| (c.position, c.direction))
                .collect();

            let mut machine_positions: Vec<IVec3> = Vec::new();
            for (machine, _) in machines.machine.iter() {
                machine_positions.push(machine.position);
            }

            let mut dir =
                auto_conveyor_direction(place_pos, player_facing, &conveyors, &machine_positions);
            for _ in 0..rotation.offset {
                dir = dir.rotate_cw();
            }
            dir
        } else if selected_type.is_machine() {
            // Apply rotation offset to all directional machines
            let mut dir = player_facing;
            for _ in 0..rotation.offset {
                dir = dir.rotate_cw();
            }
            dir
        } else {
            player_facing
        };

        match selected_type {
            BlockType::MinerBlock => {
                info!(
                    category = "MACHINE",
                    action = "place",
                    machine = "miner",
                    ?place_pos,
                    "Miner placed"
                );

                let entity = if let Some(model) = machine_models.miner.clone() {
                    // VOX model has origin at bottom center, so Y offset is 0
                    let model_transform = Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    ));
                    commands
                        .spawn((
                            SceneRoot(model),
                            model_transform.with_rotation(player_facing.to_rotation()),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            Machine::new(&MINER, place_pos, player_facing),
                        ))
                        .id()
                } else {
                    // Fallback cube mesh has center origin, so Y offset is +0.5
                    let cube_transform = Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE + 0.5,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    ));
                    let cube_mesh = chunk_assets
                        .meshes
                        .add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = chunk_assets.materials.add(StandardMaterial {
                        base_color: selected_type.color(),
                        ..default()
                    });
                    commands
                        .spawn((
                            Mesh3d(cube_mesh),
                            MeshMaterial3d(material),
                            cube_transform.with_rotation(player_facing.to_rotation()),
                            Machine::new(&MINER, place_pos, player_facing),
                        ))
                        .id()
                };
                // Send MachineSpawned event
                events.machine_spawned.send(MachineSpawned {
                    entity,
                    machine_type: BlockType::MinerBlock,
                    pos: place_pos,
                });
                // Tutorial event for miner placement
                events
                    .tutorial
                    .send(TutorialEvent::MachinePlaced(BlockType::MinerBlock));
            }
            BlockType::ConveyorBlock => {
                let front_pos = place_pos + facing_direction.to_ivec3();
                let mut final_shape = ConveyorShape::Straight;
                let final_direction = facing_direction;

                for conv in machines.conveyor.iter() {
                    if conv.position == front_pos {
                        let front_dir = conv.direction;

                        if front_dir != facing_direction {
                            let left_of_facing = facing_direction.left();
                            let right_of_facing = facing_direction.right();

                            if front_dir == left_of_facing {
                                final_shape = ConveyorShape::CornerLeft;
                            } else if front_dir == right_of_facing {
                                final_shape = ConveyorShape::CornerRight;
                            }
                        }
                        break;
                    }
                }

                info!(
                    category = "MACHINE",
                    action = "place",
                    machine = "conveyor",
                    ?place_pos,
                    ?final_direction,
                    ?final_shape,
                    "Conveyor placed"
                );

                let conveyor_pos = Vec3::new(
                    place_pos.x as f32 * BLOCK_SIZE + 0.5,
                    place_pos.y as f32 * BLOCK_SIZE,
                    place_pos.z as f32 * BLOCK_SIZE + 0.5,
                );

                let entity =
                    if let Some(model_handle) = machine_models.get_conveyor_model(final_shape) {
                        commands
                            .spawn((
                                SceneRoot(model_handle),
                                Transform::from_translation(conveyor_pos)
                                    .with_rotation(final_direction.to_rotation()),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                                Conveyor {
                                    position: place_pos,
                                    direction: final_direction,
                                    output_direction: final_direction, // Will be updated by update_conveyor_shapes
                                    items: Vec::new(),
                                    last_output_index: 0,
                                    last_input_source: 0,
                                    shape: final_shape,
                                },
                                ConveyorVisual,
                            ))
                            .id()
                    } else {
                        let conveyor_mesh = chunk_assets.meshes.add(Cuboid::new(
                            BLOCK_SIZE * CONVEYOR_BELT_WIDTH,
                            BLOCK_SIZE * CONVEYOR_BELT_HEIGHT,
                            BLOCK_SIZE,
                        ));
                        let material = chunk_assets.materials.add(StandardMaterial {
                            base_color: selected_type.color(),
                            ..default()
                        });
                        let arrow_mesh = chunk_assets.meshes.add(Cuboid::new(
                            BLOCK_SIZE * 0.12,
                            BLOCK_SIZE * 0.03,
                            BLOCK_SIZE * 0.35,
                        ));
                        let arrow_material = chunk_assets.materials.add(StandardMaterial {
                            base_color: Color::srgb(0.9, 0.9, 0.2),
                            ..default()
                        });
                        let belt_y = place_pos.y as f32 * BLOCK_SIZE + CONVEYOR_BELT_HEIGHT / 2.0;
                        commands
                            .spawn((
                                Mesh3d(conveyor_mesh),
                                MeshMaterial3d(material),
                                Transform::from_translation(Vec3::new(
                                    place_pos.x as f32 * BLOCK_SIZE + 0.5,
                                    belt_y,
                                    place_pos.z as f32 * BLOCK_SIZE + 0.5,
                                ))
                                .with_rotation(final_direction.to_rotation()),
                                Conveyor {
                                    position: place_pos,
                                    direction: final_direction,
                                    output_direction: final_direction,
                                    items: Vec::new(),
                                    last_output_index: 0,
                                    last_input_source: 0,
                                    shape: final_shape,
                                },
                                ConveyorVisual,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Mesh3d(arrow_mesh),
                                    MeshMaterial3d(arrow_material),
                                    Transform::from_translation(Vec3::new(
                                        0.0,
                                        CONVEYOR_BELT_HEIGHT / 2.0 + 0.02,
                                        -0.25,
                                    )),
                                ));
                            })
                            .id()
                    };
                // Send MachineSpawned event
                events.machine_spawned.send(MachineSpawned {
                    entity,
                    machine_type: BlockType::ConveyorBlock,
                    pos: place_pos,
                });
                rotation.offset = 0;
                // Tutorial event for conveyor placement
                events.tutorial.send(TutorialEvent::ConveyorPlaced {
                    position: place_pos,
                });
            }
            BlockType::CrusherBlock => {
                info!(
                    category = "MACHINE",
                    action = "place",
                    machine = "crusher",
                    ?place_pos,
                    "Crusher placed"
                );

                let entity = if let Some(model) = machine_models.crusher.clone() {
                    // VOX model has origin at bottom center, so Y offset is 0
                    let model_transform = Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    ));
                    commands
                        .spawn((
                            SceneRoot(model),
                            model_transform.with_rotation(player_facing.to_rotation()),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            Machine::new(&CRUSHER, place_pos, player_facing),
                        ))
                        .id()
                } else {
                    // Fallback cube mesh has center origin, so Y offset is +0.5
                    let cube_transform = Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE + 0.5,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    ));
                    let cube_mesh = chunk_assets
                        .meshes
                        .add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = chunk_assets.materials.add(StandardMaterial {
                        base_color: selected_type.color(),
                        ..default()
                    });
                    commands
                        .spawn((
                            Mesh3d(cube_mesh),
                            MeshMaterial3d(material),
                            cube_transform.with_rotation(player_facing.to_rotation()),
                            Machine::new(&CRUSHER, place_pos, player_facing),
                        ))
                        .id()
                };
                // Send MachineSpawned event
                events.machine_spawned.send(MachineSpawned {
                    entity,
                    machine_type: BlockType::CrusherBlock,
                    pos: place_pos,
                });
                // Tutorial event for crusher placement
                events
                    .tutorial
                    .send(TutorialEvent::MachinePlaced(BlockType::CrusherBlock));
            }
            BlockType::FurnaceBlock => {
                info!(
                    category = "MACHINE",
                    action = "place",
                    machine = "furnace",
                    ?place_pos,
                    "Furnace placed"
                );

                let entity = if let Some(model) = machine_models.furnace.clone() {
                    // VOX model has origin at bottom center, so Y offset is 0
                    let model_transform = Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    ));
                    commands
                        .spawn((
                            SceneRoot(model),
                            model_transform.with_rotation(player_facing.to_rotation()),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            Machine::new(&FURNACE, place_pos, player_facing),
                        ))
                        .id()
                } else {
                    // Fallback cube mesh has center origin, so Y offset is +0.5
                    let cube_transform = Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE + 0.5,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    ));
                    let cube_mesh = chunk_assets
                        .meshes
                        .add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = chunk_assets.materials.add(StandardMaterial {
                        base_color: selected_type.color(),
                        ..default()
                    });
                    commands
                        .spawn((
                            Mesh3d(cube_mesh),
                            MeshMaterial3d(material),
                            cube_transform.with_rotation(player_facing.to_rotation()),
                            Machine::new(&FURNACE, place_pos, player_facing),
                        ))
                        .id()
                };
                // Send MachineSpawned event
                events.machine_spawned.send(MachineSpawned {
                    entity,
                    machine_type: BlockType::FurnaceBlock,
                    pos: place_pos,
                });
                // Tutorial event for furnace placement
                events
                    .tutorial
                    .send(TutorialEvent::MachinePlaced(BlockType::FurnaceBlock));
            }
            _ => {
                info!(category = "BLOCK", action = "place", ?place_pos, block_type = ?selected_type, "Block placed");
                world_data.set_block(place_pos, selected_type);

                // Mark chunk and neighbors as dirty (mesh will be regenerated by process_dirty_chunks)
                let local_pos = WorldData::world_to_local(place_pos);
                dirty_chunks.mark_dirty(chunk_coord, local_pos);

                // Send block placed event
                let source = player_entity
                    .map(EventSource::Player)
                    .unwrap_or(EventSource::System);
                events.block_placed.send(BlockPlaced {
                    pos: place_pos,
                    block: selected_type,
                    source,
                });
            }
        }
    }
}
