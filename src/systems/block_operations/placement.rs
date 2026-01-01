//! Block placement system

use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::{
    BlockType, ChunkMesh, ContinuousActionTimer, Conveyor, ConveyorRotationOffset, ConveyorShape,
    ConveyorVisual, CreativeMode, Crusher, DeliveryPlatform, Direction, Furnace,
    InputStateResourcesWithCursor, Inventory, MachineModels, Miner, PlayerCamera, WorldData,
    BLOCK_SIZE, CHUNK_SIZE, CONVEYOR_BELT_HEIGHT, CONVEYOR_BELT_WIDTH, PLATFORM_SIZE,
    REACH_DISTANCE,
};
use crate::utils::{
    auto_conveyor_direction, ray_aabb_intersection, ray_aabb_intersection_with_normal,
    yaw_to_direction,
};

use super::MachinePlaceQueries;

#[allow(clippy::too_many_arguments)]
pub fn block_place(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &PlayerCamera)>,
    machines: MachinePlaceQueries,
    platform_query: Query<&Transform, With<DeliveryPlatform>>,
    mut world_data: ResMut<WorldData>,
    mut inventory: ResMut<Inventory>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Query<&Window>,
    creative_mode: Res<CreativeMode>,
    input_resources: InputStateResourcesWithCursor,
    mut action_timer: ResMut<ContinuousActionTimer>,
    mut rotation: ResMut<ConveyorRotationOffset>,
    machine_models: Res<MachineModels>,
) {
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

    // Check if looking at a furnace or crusher - if so, don't place
    for furnace_transform in machines.furnace.iter() {
        let furnace_pos = furnace_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            furnace_pos - Vec3::splat(half_size),
            furnace_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                return;
            }
        }
    }
    for (_, crusher_transform) in machines.crusher.iter() {
        let crusher_pos = crusher_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            crusher_pos - Vec3::splat(half_size),
            crusher_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                return;
            }
        }
    }

    // Find closest block intersection with hit normal using DDA
    let mut closest_hit: Option<(IVec3, Vec3, f32)> = None;

    {
        let mut current = IVec3::new(
            ray_origin.x.floor() as i32,
            ray_origin.y.floor() as i32,
            ray_origin.z.floor() as i32,
        );

        let step = IVec3::new(
            if ray_direction.x >= 0.0 { 1 } else { -1 },
            if ray_direction.y >= 0.0 { 1 } else { -1 },
            if ray_direction.z >= 0.0 { 1 } else { -1 },
        );

        let t_delta = Vec3::new(
            if ray_direction.x.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.x).abs() },
            if ray_direction.y.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.y).abs() },
            if ray_direction.z.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.z).abs() },
        );

        let mut t_max = Vec3::new(
            if ray_direction.x >= 0.0 {
                ((current.x + 1) as f32 - ray_origin.x) / ray_direction.x.abs().max(1e-8)
            } else {
                (ray_origin.x - current.x as f32) / ray_direction.x.abs().max(1e-8)
            },
            if ray_direction.y >= 0.0 {
                ((current.y + 1) as f32 - ray_origin.y) / ray_direction.y.abs().max(1e-8)
            } else {
                (ray_origin.y - current.y as f32) / ray_direction.y.abs().max(1e-8)
            },
            if ray_direction.z >= 0.0 {
                ((current.z + 1) as f32 - ray_origin.z) / ray_direction.z.abs().max(1e-8)
            } else {
                (ray_origin.z - current.z as f32) / ray_direction.z.abs().max(1e-8)
            },
        );

        let mut last_step_axis = 0;
        let max_steps = (REACH_DISTANCE * 2.0) as i32;

        for _ in 0..max_steps {
            if world_data.has_block(current) {
                let block_center = Vec3::new(
                    current.x as f32 + 0.5,
                    current.y as f32 + 0.5,
                    current.z as f32 + 0.5,
                );
                if let Some((hit_t, _normal)) = ray_aabb_intersection_with_normal(
                    ray_origin,
                    ray_direction,
                    block_center - Vec3::splat(half_size),
                    block_center + Vec3::splat(half_size),
                ) {
                    if hit_t > 0.0 && hit_t < REACH_DISTANCE {
                        let dda_normal = match last_step_axis {
                            0 => Vec3::new(-step.x as f32, 0.0, 0.0),
                            1 => Vec3::new(0.0, -step.y as f32, 0.0),
                            _ => Vec3::new(0.0, 0.0, -step.z as f32),
                        };
                        closest_hit = Some((current, dda_normal, hit_t));
                        break;
                    }
                }
            }

            if t_max.x < t_max.y && t_max.x < t_max.z {
                if t_max.x > REACH_DISTANCE { break; }
                current.x += step.x;
                t_max.x += t_delta.x;
                last_step_axis = 0;
            } else if t_max.y < t_max.z {
                if t_max.y > REACH_DISTANCE { break; }
                current.y += step.y;
                t_max.y += t_delta.y;
                last_step_axis = 1;
            } else {
                if t_max.z > REACH_DISTANCE { break; }
                current.z += step.z;
                t_max.z += t_delta.z;
                last_step_axis = 2;
            }
        }
    }

    // Also check DeliveryPlatform for raycast hit
    if let Ok(platform_transform) = platform_query.get_single() {
        let platform_center = platform_transform.translation;
        let platform_half_x = (PLATFORM_SIZE as f32 * BLOCK_SIZE) / 2.0;
        let platform_half_y = BLOCK_SIZE * 0.1;
        let platform_half_z = platform_half_x;

        let platform_min = platform_center - Vec3::new(platform_half_x, platform_half_y, platform_half_z);
        let platform_max = platform_center + Vec3::new(platform_half_x, platform_half_y, platform_half_z);

        if let Some((hit_t, normal)) = ray_aabb_intersection_with_normal(
            ray_origin,
            ray_direction,
            platform_min,
            platform_max,
        ) {
            if hit_t > 0.0 && hit_t < REACH_DISTANCE {
                let hit_point = ray_origin + ray_direction * hit_t;
                let hit_block_pos = IVec3::new(
                    hit_point.x.floor() as i32,
                    hit_point.y.floor() as i32,
                    hit_point.z.floor() as i32,
                );
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
        let place_pos = hit_pos + IVec3::new(
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
        for miner in machines.miner.iter() {
            if miner.position == place_pos {
                return;
            }
        }
        for (crusher, _) in machines.crusher.iter() {
            if crusher.position == place_pos {
                return;
            }
        }
        for furnace_transform in machines.furnace.iter() {
            let furnace_pos = IVec3::new(
                (furnace_transform.translation.x / BLOCK_SIZE).floor() as i32,
                (furnace_transform.translation.y / BLOCK_SIZE).floor() as i32,
                (furnace_transform.translation.z / BLOCK_SIZE).floor() as i32,
            );
            if furnace_pos == place_pos {
                return;
            }
        }

        // Consume from inventory (unless in creative mode)
        if !creative_mode.enabled {
            inventory.consume_selected();
        }

        let chunk_coord = WorldData::world_to_chunk(place_pos);
        let player_facing = yaw_to_direction(player_camera.yaw);

        let facing_direction = if selected_type == BlockType::ConveyorBlock {
            let conveyors: Vec<(IVec3, Direction)> = machines.conveyor
                .iter()
                .map(|c| (c.position, c.direction))
                .collect();

            let mut machine_positions: Vec<IVec3> = Vec::new();
            for miner in machines.miner.iter() {
                machine_positions.push(miner.position);
            }
            for (crusher, _) in machines.crusher.iter() {
                machine_positions.push(crusher.position);
            }
            for furnace_transform in machines.furnace.iter() {
                machine_positions.push(IVec3::new(
                    furnace_transform.translation.x.floor() as i32,
                    furnace_transform.translation.y.floor() as i32,
                    furnace_transform.translation.z.floor() as i32,
                ));
            }

            let mut dir = auto_conveyor_direction(place_pos, player_facing, &conveyors, &machine_positions);
            for _ in 0..rotation.offset {
                dir = dir.rotate_cw();
            }
            dir
        } else {
            player_facing
        };

        let regenerate_chunk = |coord: IVec2,
                                commands: &mut Commands,
                                world_data: &mut WorldData,
                                meshes: &mut Assets<Mesh>,
                                materials: &mut Assets<StandardMaterial>| {
            if let Some(old_entities) = world_data.chunk_entities.remove(&coord) {
                for entity in old_entities {
                    commands.entity(entity).try_despawn_recursive();
                }
            }

            if let Some(new_mesh) = world_data.generate_chunk_mesh(coord) {
                let mesh_handle = meshes.add(new_mesh);
                let material = materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 0.9,
                    ..default()
                });

                let entity = commands.spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material),
                    Transform::IDENTITY,
                    ChunkMesh { coord },
                )).id();

                world_data.chunk_entities.insert(coord, vec![entity]);
            }
        };

        match selected_type {
            BlockType::MinerBlock => {
                info!(category = "MACHINE", action = "place", machine = "miner", ?place_pos, "Miner placed");

                let transform = Transform::from_translation(Vec3::new(
                    place_pos.x as f32 * BLOCK_SIZE + 0.5,
                    place_pos.y as f32 * BLOCK_SIZE + 0.5,
                    place_pos.z as f32 * BLOCK_SIZE + 0.5,
                ));

                if let Some(model) = machine_models.miner.clone() {
                    commands.spawn((
                        SceneRoot(model),
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Miner {
                            position: place_pos,
                            ..default()
                        },
                    ));
                } else {
                    let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: selected_type.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(cube_mesh),
                        MeshMaterial3d(material),
                        transform,
                        Miner {
                            position: place_pos,
                            ..default()
                        },
                    ));
                }
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

                info!(category = "MACHINE", action = "place", machine = "conveyor", ?place_pos, ?final_direction, ?final_shape, "Conveyor placed");

                let conveyor_pos = Vec3::new(
                    place_pos.x as f32 * BLOCK_SIZE + 0.5,
                    place_pos.y as f32 * BLOCK_SIZE,
                    place_pos.z as f32 * BLOCK_SIZE + 0.5,
                );

                if let Some(model_handle) = machine_models.get_conveyor_model(final_shape) {
                    commands.spawn((
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
                            items: Vec::new(),
                            last_output_index: 0,
                            last_input_source: 0,
                            shape: final_shape,
                        },
                        ConveyorVisual,
                    ));
                } else {
                    let conveyor_mesh = meshes.add(Cuboid::new(
                        BLOCK_SIZE * CONVEYOR_BELT_WIDTH,
                        BLOCK_SIZE * CONVEYOR_BELT_HEIGHT,
                        BLOCK_SIZE
                    ));
                    let material = materials.add(StandardMaterial {
                        base_color: selected_type.color(),
                        ..default()
                    });
                    let arrow_mesh = meshes.add(Cuboid::new(BLOCK_SIZE * 0.12, BLOCK_SIZE * 0.03, BLOCK_SIZE * 0.35));
                    let arrow_material = materials.add(StandardMaterial {
                        base_color: Color::srgb(0.9, 0.9, 0.2),
                        ..default()
                    });
                    let belt_y = place_pos.y as f32 * BLOCK_SIZE + CONVEYOR_BELT_HEIGHT / 2.0;
                    commands.spawn((
                        Mesh3d(conveyor_mesh),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(
                            place_pos.x as f32 * BLOCK_SIZE + 0.5,
                            belt_y,
                            place_pos.z as f32 * BLOCK_SIZE + 0.5,
                        )).with_rotation(final_direction.to_rotation()),
                        Conveyor {
                            position: place_pos,
                            direction: final_direction,
                            items: Vec::new(),
                            last_output_index: 0,
                            last_input_source: 0,
                            shape: final_shape,
                        },
                        ConveyorVisual,
                    )).with_children(|parent| {
                        parent.spawn((
                            Mesh3d(arrow_mesh),
                            MeshMaterial3d(arrow_material),
                            Transform::from_translation(Vec3::new(0.0, CONVEYOR_BELT_HEIGHT / 2.0 + 0.02, -0.25)),
                        ));
                    });
                }
                rotation.offset = 0;
            }
            BlockType::CrusherBlock => {
                info!(category = "MACHINE", action = "place", machine = "crusher", ?place_pos, "Crusher placed");

                let transform = Transform::from_translation(Vec3::new(
                    place_pos.x as f32 * BLOCK_SIZE + 0.5,
                    place_pos.y as f32 * BLOCK_SIZE + 0.5,
                    place_pos.z as f32 * BLOCK_SIZE + 0.5,
                ));

                if let Some(model) = machine_models.crusher.clone() {
                    commands.spawn((
                        SceneRoot(model),
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Crusher {
                            position: place_pos,
                            input_type: None,
                            input_count: 0,
                            output_type: None,
                            output_count: 0,
                            progress: 0.0,
                        },
                    ));
                } else {
                    let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: selected_type.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(cube_mesh),
                        MeshMaterial3d(material),
                        transform,
                        Crusher {
                            position: place_pos,
                            input_type: None,
                            input_count: 0,
                            output_type: None,
                            output_count: 0,
                            progress: 0.0,
                        },
                    ));
                }
            }
            BlockType::FurnaceBlock => {
                info!(category = "MACHINE", action = "place", machine = "furnace", ?place_pos, "Furnace placed");

                let transform = Transform::from_translation(Vec3::new(
                    place_pos.x as f32 * BLOCK_SIZE + 0.5,
                    place_pos.y as f32 * BLOCK_SIZE + 0.5,
                    place_pos.z as f32 * BLOCK_SIZE + 0.5,
                ));

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
                    let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                    let material = materials.add(StandardMaterial {
                        base_color: selected_type.color(),
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(cube_mesh),
                        MeshMaterial3d(material),
                        transform,
                        Furnace::default(),
                    ));
                }
            }
            _ => {
                info!(category = "BLOCK", action = "place", ?place_pos, block_type = ?selected_type, "Block placed");
                world_data.set_block(place_pos, selected_type);
                regenerate_chunk(chunk_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);

                let local_pos = WorldData::world_to_local(place_pos);
                let neighbor_offsets: [(i32, i32, bool); 4] = [
                    (-1, 0, local_pos.x == 0),
                    (1, 0, local_pos.x == CHUNK_SIZE - 1),
                    (0, -1, local_pos.z == 0),
                    (0, 1, local_pos.z == CHUNK_SIZE - 1),
                ];

                for (dx, dz, at_boundary) in neighbor_offsets {
                    if at_boundary {
                        let neighbor_coord = IVec2::new(chunk_coord.x + dx, chunk_coord.y + dz);
                        if world_data.chunks.contains_key(&neighbor_coord) {
                            regenerate_chunk(neighbor_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);
                        }
                    }
                }
            }
        }
    }
}
