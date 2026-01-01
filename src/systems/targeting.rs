//! Target block and highlight systems
//!
//! Handles raycasting for block selection and visual highlighting

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use std::collections::HashSet;

use crate::components::*;
use crate::meshes::{create_conveyor_wireframe_mesh, create_wireframe_cube_mesh};
use crate::player::Inventory;
use crate::utils::{auto_conveyor_direction, yaw_to_direction};
use crate::world::WorldData;
use crate::{
    BlockType, Conveyor, ConveyorShape, Crusher, Direction, Furnace, Miner, MachineModels,
    ConveyorVisual, REACH_DISTANCE,
};
use crate::meshes::create_conveyor_mesh;

// === Target Block Highlight ===

/// Update target block based on player's view direction
pub fn update_target_block(
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    world_data: Res<WorldData>,
    windows: Query<&Window>,
    mut target: ResMut<TargetBlock>,
    interacting_furnace: Res<InteractingFurnace>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't update target while UI is open or paused
    if interacting_furnace.0.is_some() || cursor_state.paused {
        target.break_target = None;
        target.place_target = None;
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;
    if !cursor_locked {
        target.break_target = None;
        target.place_target = None;
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Use DDA (Digital Differential Analyzer) for precise voxel traversal
    // This ensures we check every voxel the ray passes through, in order

    // Current voxel position
    let mut current = IVec3::new(
        ray_origin.x.floor() as i32,
        ray_origin.y.floor() as i32,
        ray_origin.z.floor() as i32,
    );

    // Direction sign for stepping (+1 or -1 for each axis)
    let step = IVec3::new(
        if ray_direction.x >= 0.0 { 1 } else { -1 },
        if ray_direction.y >= 0.0 { 1 } else { -1 },
        if ray_direction.z >= 0.0 { 1 } else { -1 },
    );

    // How far along the ray we need to travel for one voxel step on each axis
    let t_delta = Vec3::new(
        if ray_direction.x.abs() < 1e-8 {
            f32::MAX
        } else {
            (1.0 / ray_direction.x).abs()
        },
        if ray_direction.y.abs() < 1e-8 {
            f32::MAX
        } else {
            (1.0 / ray_direction.y).abs()
        },
        if ray_direction.z.abs() < 1e-8 {
            f32::MAX
        } else {
            (1.0 / ray_direction.z).abs()
        },
    );

    // Distance to next voxel boundary for each axis
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

    // Track which axis we stepped on last (for face normal)
    let mut last_step_axis = 0; // 0=x, 1=y, 2=z

    // Maximum number of steps (prevent infinite loop)
    let max_steps = (REACH_DISTANCE * 2.0) as i32;

    for _ in 0..max_steps {
        // Check current voxel
        if world_data.has_block(current) {
            target.break_target = Some(current);

            // Calculate place position based on last step axis
            let normal = match last_step_axis {
                0 => IVec3::new(-step.x, 0, 0),
                1 => IVec3::new(0, -step.y, 0),
                _ => IVec3::new(0, 0, -step.z),
            };
            target.place_target = Some(current + normal);
            return;
        }

        // Step to next voxel
        if t_max.x < t_max.y && t_max.x < t_max.z {
            if t_max.x > REACH_DISTANCE {
                break;
            }
            current.x += step.x;
            t_max.x += t_delta.x;
            last_step_axis = 0;
        } else if t_max.y < t_max.z {
            if t_max.y > REACH_DISTANCE {
                break;
            }
            current.y += step.y;
            t_max.y += t_delta.y;
            last_step_axis = 1;
        } else {
            if t_max.z > REACH_DISTANCE {
                break;
            }
            current.z += step.z;
            t_max.z += t_delta.z;
            last_step_axis = 2;
        }
    }

    // No block found
    target.break_target = None;
    target.place_target = None;
}

/// Update target highlight entity position
#[allow(clippy::too_many_arguments)]
pub fn update_target_highlight(
    mut commands: Commands,
    mut target: ResMut<TargetBlock>,
    break_query: Query<Entity, (With<TargetHighlight>, Without<PlaceHighlight>)>,
    place_query: Query<Entity, (With<PlaceHighlight>, Without<TargetHighlight>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    inventory: Res<Inventory>,
    conveyor_query: Query<&Conveyor>,
    miner_query: Query<&Miner>,
    crusher_query: Query<&Crusher>,
    furnace_query: Query<&Transform, With<Furnace>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    rotation: Res<ConveyorRotationOffset>,
) {
    // Check if player has a placeable item selected
    let has_placeable_item = inventory.has_selected();

    // Check if selected item is a conveyor
    let selected_item = inventory.get_selected_type();
    let placing_conveyor = selected_item == Some(BlockType::ConveyorBlock);

    // Get player's facing direction as fallback
    let player_facing = camera_query.get_single().ok().map(|cam_transform| {
        let forward = cam_transform.forward().as_vec3();
        yaw_to_direction(-forward.x.atan2(-forward.z))
    });

    // Calculate place direction using auto_conveyor_direction (same logic as block_place)
    let place_direction = if placing_conveyor {
        if let (Some(place_pos), Some(fallback_dir)) = (target.place_target, player_facing) {
            // Collect conveyor positions and directions
            let conveyors: Vec<(IVec3, Direction)> = conveyor_query
                .iter()
                .map(|c| (c.position, c.direction))
                .collect();

            // Collect machine positions
            let mut machine_positions: Vec<IVec3> = Vec::new();
            for miner in miner_query.iter() {
                machine_positions.push(miner.position);
            }
            for crusher in crusher_query.iter() {
                machine_positions.push(crusher.position);
            }
            for furnace_transform in furnace_query.iter() {
                machine_positions.push(IVec3::new(
                    furnace_transform.translation.x.floor() as i32,
                    furnace_transform.translation.y.floor() as i32,
                    furnace_transform.translation.z.floor() as i32,
                ));
            }

            // Apply rotation offset (R key)
            let mut dir =
                auto_conveyor_direction(place_pos, fallback_dir, &conveyors, &machine_positions);
            for _ in 0..rotation.offset {
                dir = dir.rotate_cw();
            }
            Some(dir)
        } else {
            player_facing
        }
    } else {
        None
    };

    // Check if break target is a conveyor and get its direction
    let break_conveyor_dir = target
        .break_target
        .and_then(|pos| conveyor_query.iter().find(|c| c.position == pos).map(|c| c.direction));

    // === Break target (red wireframe) - always show when looking at a block ===
    if let Some(pos) = target.break_target {
        let center = Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5);

        // Always recreate mesh to handle conveyor direction changes
        // Despawn old entity if exists
        if let Some(entity) = target.break_highlight_entity.take() {
            if break_query.get(entity).is_ok() {
                commands.entity(entity).despawn();
            }
        }

        let mesh = if let Some(dir) = break_conveyor_dir {
            meshes.add(create_conveyor_wireframe_mesh(dir))
        } else {
            meshes.add(create_wireframe_cube_mesh())
        };
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.2),
            unlit: true,
            ..default()
        });
        let entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(center),
                TargetHighlight,
                NotShadowCaster,
            ))
            .id();
        target.break_highlight_entity = Some(entity);
    } else if let Some(entity) = target.break_highlight_entity.take() {
        if break_query.get(entity).is_ok() {
            commands.entity(entity).despawn();
        }
    }

    // === Place target (green wireframe) - only show if player has a placeable item ===
    if let Some(pos) = target.place_target.filter(|_| has_placeable_item) {
        let center = Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5);

        // Always recreate mesh to handle conveyor direction changes
        if let Some(entity) = target.place_highlight_entity.take() {
            if place_query.get(entity).is_ok() {
                commands.entity(entity).despawn();
            }
        }

        let mesh = if let Some(dir) = place_direction {
            meshes.add(create_conveyor_wireframe_mesh(dir))
        } else {
            meshes.add(create_wireframe_cube_mesh())
        };
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 1.0, 0.2),
            unlit: true,
            ..default()
        });
        let entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(center),
                PlaceHighlight,
                NotShadowCaster,
            ))
            .id();
        target.place_highlight_entity = Some(entity);
    } else if let Some(entity) = target.place_highlight_entity.take() {
        if place_query.get(entity).is_ok() {
            commands.entity(entity).despawn();
        }
    }
}

/// Handle R key to rotate conveyor placement direction
pub fn rotate_conveyor_placement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut rotation: ResMut<ConveyorRotationOffset>,
    inventory: Res<Inventory>,
    input_resources: InputStateResourcesWithCursor,
) {
    // Only active when placing conveyors
    let selected = inventory.get_selected_type();
    if selected != Some(BlockType::ConveyorBlock) {
        // Reset rotation when not placing conveyor
        rotation.offset = 0;
        return;
    }

    // Check input state allows this action
    let input_state = input_resources.get_state();
    if !input_state.allows_block_actions() {
        return;
    }

    // R key rotates 90 degrees clockwise
    if keyboard.just_pressed(KeyCode::KeyR) {
        rotation.offset = (rotation.offset + 1) % 4;
    }
}

/// Update conveyor shapes based on adjacent conveyor connections
/// Adds visual extensions for side inputs (L-shape, T-shape)
/// Detects splitter mode when multiple outputs are available
#[allow(clippy::type_complexity)]
pub fn update_conveyor_shapes(
    mut commands: Commands,
    mut conveyors: Query<(
        Entity,
        &mut Conveyor,
        Option<&mut Mesh3d>,
        Option<&SceneRoot>,
        &Transform,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    machine_models: Res<MachineModels>,
    furnace_query: Query<&Transform, (With<Furnace>, Without<Conveyor>)>,
    crusher_query: Query<&Crusher>,
) {
    // Collect all conveyor positions and directions first (read-only pass)
    let conveyor_data: Vec<(IVec3, Direction)> = conveyors
        .iter()
        .map(|(_, c, _, _, _)| (c.position, c.direction))
        .collect();

    // Collect positions that can accept items (conveyors, furnaces, crushers)
    let conveyor_positions: HashSet<IVec3> = conveyor_data.iter().map(|(p, _)| *p).collect();
    let furnace_positions: HashSet<IVec3> = furnace_query
        .iter()
        .map(|t| {
            IVec3::new(
                t.translation.x.floor() as i32,
                t.translation.y.floor() as i32,
                t.translation.z.floor() as i32,
            )
        })
        .collect();
    let crusher_positions: HashSet<IVec3> = crusher_query.iter().map(|c| c.position).collect();

    for (entity, mut conveyor, mesh3d_opt, scene_root_opt, transform) in conveyors.iter_mut() {
        // Calculate shape using the new auto-connect logic (2026-01-01)
        // 1. Check inputs: which neighbors output to this conveyor
        // 2. Check "waiting": which neighbors can receive input from this conveyor
        // 3. Determine shape based on input count and waiting count

        let back_pos = conveyor.position - conveyor.direction.to_ivec3();
        let left_pos = conveyor.position + conveyor.direction.left().to_ivec3();
        let right_pos = conveyor.position + conveyor.direction.right().to_ivec3();
        let front_pos = conveyor.position + conveyor.direction.to_ivec3();

        // Check inputs: which neighbors output to this conveyor
        let mut has_back_input = false;
        let mut has_left_input = false;
        let mut has_right_input = false;
        let mut has_front_input = false;

        for (pos, dir) in &conveyor_data {
            // Check if this conveyor outputs to our position
            let outputs_to_us = *pos + dir.to_ivec3() == conveyor.position;
            if !outputs_to_us {
                continue;
            }

            if *pos == back_pos {
                has_back_input = true;
            } else if *pos == left_pos {
                has_left_input = true;
            } else if *pos == right_pos {
                has_right_input = true;
            } else if *pos == front_pos {
                has_front_input = true;
            }
        }

        // Check "waiting": which neighbors can receive input from this conveyor
        // A neighbor is "waiting" if it can receive from our position (back, left, or right)
        // and is not already outputting to us
        let can_receive_from = |neighbor_pos: IVec3, from_pos: IVec3| -> bool {
            for (pos, dir) in &conveyor_data {
                if *pos == neighbor_pos {
                    // A conveyor can receive from back, left, or right (not front)
                    let nb_back = neighbor_pos - dir.to_ivec3();
                    let nb_left = neighbor_pos + dir.left().to_ivec3();
                    let nb_right = neighbor_pos + dir.right().to_ivec3();
                    return from_pos == nb_back || from_pos == nb_left || from_pos == nb_right;
                }
            }
            // Also check if furnace or crusher at this position (always accepts)
            furnace_positions.contains(&neighbor_pos) || crusher_positions.contains(&neighbor_pos)
        };

        let left_waiting = !has_left_input
            && conveyor_positions.contains(&left_pos)
            && can_receive_from(left_pos, conveyor.position);
        let right_waiting = !has_right_input
            && conveyor_positions.contains(&right_pos)
            && can_receive_from(right_pos, conveyor.position);
        let front_waiting = !has_front_input
            && (conveyor_positions.contains(&front_pos) && can_receive_from(front_pos, conveyor.position)
                || furnace_positions.contains(&front_pos)
                || crusher_positions.contains(&front_pos));

        let input_count = [has_back_input, has_left_input, has_right_input, has_front_input]
            .iter()
            .filter(|&&b| b)
            .count();
        let wait_count = [left_waiting, right_waiting, front_waiting]
            .iter()
            .filter(|&&b| b)
            .count();

        // Determine new shape using the auto-connect logic
        let new_shape = if input_count >= 2 {
            // Input 2+: TJunction (merge)
            ConveyorShape::TJunction
        } else if input_count == 1 {
            if has_back_input {
                // Back input
                if wait_count >= 2 {
                    ConveyorShape::Splitter
                } else if right_waiting && !front_waiting {
                    ConveyorShape::CornerRight
                } else if left_waiting && !front_waiting {
                    ConveyorShape::CornerLeft
                } else {
                    ConveyorShape::Straight
                }
            } else if has_left_input {
                // Left input
                if front_waiting && right_waiting {
                    ConveyorShape::Splitter
                } else if right_waiting && !front_waiting {
                    ConveyorShape::CornerRight // left in, right out
                } else {
                    ConveyorShape::CornerLeft // left in, front out
                }
            } else if has_right_input {
                // Right input
                if front_waiting && left_waiting {
                    ConveyorShape::Splitter
                } else if left_waiting && !front_waiting {
                    ConveyorShape::CornerLeft // right in, left out
                } else {
                    ConveyorShape::CornerRight // right in, front out
                }
            } else {
                // Front input (head-on)
                ConveyorShape::Straight
            }
        } else {
            // Input 0: Straight
            ConveyorShape::Straight
        };

        // Only update if shape changed
        if conveyor.shape != new_shape {
            conveyor.shape = new_shape;

            // Check if using glTF model (has SceneRoot component)
            let uses_gltf = scene_root_opt.is_some();

            if uses_gltf {
                // Using glTF models - need to despawn and respawn with new model
                if let Some(new_model) = machine_models.get_conveyor_model(new_shape) {
                    // Store conveyor data before despawn
                    let conv_data = Conveyor {
                        position: conveyor.position,
                        direction: conveyor.direction,
                        items: std::mem::take(&mut conveyor.items),
                        last_output_index: conveyor.last_output_index,
                        last_input_source: conveyor.last_input_source,
                        shape: new_shape,
                    };
                    let conv_transform = *transform;

                    // Despawn old entity
                    commands.entity(entity).despawn_recursive();

                    // Spawn new entity with new model
                    // Note: GlobalTransform and Visibility are required for rendering
                    commands.spawn((
                        SceneRoot(new_model),
                        conv_transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        conv_data,
                        ConveyorVisual,
                    ));
                }
            } else if let Some(mut mesh3d) = mesh3d_opt {
                // Using procedural mesh - just swap the mesh
                let new_mesh = create_conveyor_mesh(new_shape);
                *mesh3d = Mesh3d(meshes.add(new_mesh));
            }
        }
    }
}

// === Guide Markers ===

/// Update guide markers based on selected item
/// Shows recommended placement positions for machines
#[allow(clippy::too_many_arguments)]
pub fn update_guide_markers(
    mut commands: Commands,
    mut guide_markers: ResMut<GuideMarkers>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    inventory: Res<Inventory>,
    time: Res<Time>,
    miner_query: Query<&Miner>,
    conveyor_query: Query<&Conveyor>,
    furnace_query: Query<&Transform, (With<Furnace>, Without<GuideMarker>)>,
    crusher_query: Query<&Transform, (With<Crusher>, Without<GuideMarker>)>,
) {
    let selected = inventory.get_selected_type();

    // Clear markers if selection changed or nothing selected
    if selected != guide_markers.last_selected {
        for entity in guide_markers.entities.drain(..) {
            commands.entity(entity).despawn_recursive();
        }
        guide_markers.last_selected = selected;
    }

    // No markers if nothing is selected or non-machine item
    let Some(block_type) = selected else {
        return;
    };

    // Only show guides for placeable machines
    if !matches!(
        block_type,
        BlockType::MinerBlock
            | BlockType::ConveyorBlock
            | BlockType::FurnaceBlock
            | BlockType::CrusherBlock
    ) {
        return;
    }

    // Calculate pulse effect (0.3 to 0.7 alpha)
    let pulse = (time.elapsed_secs() * 3.0).sin() * 0.2 + 0.5;

    // Generate guide positions based on selected item
    let guide_positions = match block_type {
        BlockType::MinerBlock => {
            // Show positions outside delivery platform edges
            generate_miner_guide_positions()
        }
        BlockType::ConveyorBlock => {
            // Show positions extending from existing machines
            generate_conveyor_guide_positions(
                &miner_query,
                &conveyor_query,
                &furnace_query,
                &crusher_query,
            )
        }
        BlockType::FurnaceBlock | BlockType::CrusherBlock => {
            // Show positions along conveyor paths
            generate_processor_guide_positions(&conveyor_query)
        }
        _ => vec![],
    };

    // Only update if we need to spawn new markers
    if guide_markers.entities.is_empty() && !guide_positions.is_empty() {
        let mesh = meshes.add(create_wireframe_cube_mesh());

        for pos in guide_positions {
            let material = materials.add(StandardMaterial {
                base_color: Color::srgba(0.3, 0.6, 1.0, pulse),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            let entity = commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        pos.x as f32 + 0.5,
                        pos.y as f32 + 0.5,
                        pos.z as f32 + 0.5,
                    )),
                    GuideMarker,
                    NotShadowCaster,
                ))
                .id();

            guide_markers.entities.push(entity);
        }
    }
    // Note: pulse effect would require material recreation each frame - skipped for performance
}

/// Generate guide positions for miners (outside delivery platform edges)
fn generate_miner_guide_positions() -> Vec<IVec3> {
    let mut positions = Vec::new();

    // Delivery platform: origin (20, 8, 10), size 12x12
    // Show positions 2-3 blocks outside each edge at Y=8

    // North of platform (z = 8, 9)
    for x in (20..32).step_by(3) {
        positions.push(IVec3::new(x, 8, 8));
    }

    // South of platform (z = 23, 24)
    for x in (20..32).step_by(3) {
        positions.push(IVec3::new(x, 8, 23));
    }

    // West of platform (x = 18)
    for z in (10..22).step_by(3) {
        positions.push(IVec3::new(18, 8, z));
    }

    // East of platform (x = 33)
    for z in (10..22).step_by(3) {
        positions.push(IVec3::new(33, 8, z));
    }

    positions
}

/// Generate guide positions for conveyors (extending from existing machines)
fn generate_conveyor_guide_positions(
    miner_query: &Query<&Miner>,
    conveyor_query: &Query<&Conveyor>,
    furnace_query: &Query<&Transform, (With<Furnace>, Without<GuideMarker>)>,
    crusher_query: &Query<&Transform, (With<Crusher>, Without<GuideMarker>)>,
) -> Vec<IVec3> {
    let mut positions = Vec::new();
    let mut existing: std::collections::HashSet<IVec3> = std::collections::HashSet::new();

    // Collect existing machine positions
    for miner in miner_query.iter() {
        existing.insert(miner.position);
    }
    for conveyor in conveyor_query.iter() {
        existing.insert(conveyor.position);
    }
    for transform in furnace_query.iter() {
        let pos = IVec3::new(
            transform.translation.x.floor() as i32,
            transform.translation.y.floor() as i32,
            transform.translation.z.floor() as i32,
        );
        existing.insert(pos);
    }
    for transform in crusher_query.iter() {
        let pos = IVec3::new(
            transform.translation.x.floor() as i32,
            transform.translation.y.floor() as i32,
            transform.translation.z.floor() as i32,
        );
        existing.insert(pos);
    }

    // Show positions adjacent to conveyor ends
    for conveyor in conveyor_query.iter() {
        let next_pos = match conveyor.direction {
            Direction::North => conveyor.position + IVec3::new(0, 0, -1),
            Direction::South => conveyor.position + IVec3::new(0, 0, 1),
            Direction::East => conveyor.position + IVec3::new(1, 0, 0),
            Direction::West => conveyor.position + IVec3::new(-1, 0, 0),
        };

        if !existing.contains(&next_pos) && !positions.contains(&next_pos) {
            positions.push(next_pos);
        }
    }

    // Show positions adjacent to miners (output side)
    for miner in miner_query.iter() {
        for dir in [IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z] {
            let adj = miner.position + dir;
            if !existing.contains(&adj) && !positions.contains(&adj) {
                positions.push(adj);
                break; // Only one suggestion per miner
            }
        }
    }

    // Limit to 8 suggestions to avoid clutter
    positions.truncate(8);
    positions
}

/// Generate guide positions for processors (along conveyor paths)
fn generate_processor_guide_positions(conveyor_query: &Query<&Conveyor>) -> Vec<IVec3> {
    let mut positions = Vec::new();
    let mut conveyor_positions: std::collections::HashSet<IVec3> =
        std::collections::HashSet::new();

    for conveyor in conveyor_query.iter() {
        conveyor_positions.insert(conveyor.position);
    }

    // Show positions adjacent to conveyors (as inline processors)
    for conveyor in conveyor_query.iter() {
        // Position perpendicular to conveyor direction
        let perpendicular = match conveyor.direction {
            Direction::North | Direction::South => [IVec3::X, IVec3::NEG_X],
            Direction::East | Direction::West => [IVec3::Z, IVec3::NEG_Z],
        };

        for dir in perpendicular {
            let adj = conveyor.position + dir;
            if !conveyor_positions.contains(&adj) && !positions.contains(&adj) {
                positions.push(adj);
            }
        }
    }

    // Limit to 6 suggestions
    positions.truncate(6);
    positions
}
