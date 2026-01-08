//! Conveyor rotation and shape update systems

use bevy::prelude::*;
use std::collections::HashSet;

use crate::components::Machine;
use crate::core::{items, ItemId};
use crate::meshes::create_conveyor_mesh;
use crate::player::{LocalPlayer, PlayerInventory};
use crate::{
    Conveyor, ConveyorRotationOffset, ConveyorShape, ConveyorVisual, Direction,
    InputStateResourcesWithCursor, MachineModels,
};

/// Handle R key to rotate conveyor/machine placement direction
pub fn rotate_conveyor_placement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut rotation: ResMut<ConveyorRotationOffset>,
    local_player: Option<Res<LocalPlayer>>,
    inventories: Query<&PlayerInventory>,
    input_resources: InputStateResourcesWithCursor,
) {
    // Only active when placing conveyors or machines
    let Some(local_player) = local_player else {
        return;
    };
    let Ok(inventory) = inventories.get(local_player.0) else {
        return;
    };
    let selected_item_id: Option<ItemId> = inventory.get_selected_item_id();
    // Check if the selected item is rotatable (conveyor or machine)
    let is_rotatable =
        selected_item_id.is_some_and(|id| id == items::conveyor_block() || id.is_machine());
    if !is_rotatable {
        // Reset rotation when not placing rotatable block
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
    machine_query: Query<&Machine, Without<Conveyor>>,
) {
    // Collect all conveyor positions and directions first (read-only pass)
    let conveyor_data: Vec<(IVec3, Direction)> = conveyors
        .iter()
        .map(|(_, c, _, _, _)| (c.position, c.direction))
        .collect();

    // Collect positions that can accept items (conveyors, furnaces, crushers)
    let conveyor_positions: HashSet<IVec3> = conveyor_data.iter().map(|(p, _)| *p).collect();
    let mut furnace_positions: HashSet<IVec3> = HashSet::new();
    let mut crusher_positions: HashSet<IVec3> = HashSet::new();

    for machine in machine_query.iter() {
        let machine_id = machine.spec.item_id();
        if machine_id == items::furnace_block() {
            furnace_positions.insert(machine.position);
        } else if machine_id == items::crusher_block() {
            crusher_positions.insert(machine.position);
        }
    }

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
            && (conveyor_positions.contains(&front_pos)
                && can_receive_from(front_pos, conveyor.position)
                || furnace_positions.contains(&front_pos)
                || crusher_positions.contains(&front_pos));

        let input_count = [
            has_back_input,
            has_left_input,
            has_right_input,
            has_front_input,
        ]
        .iter()
        .filter(|&&b| b)
        .count();
        let wait_count = [left_waiting, right_waiting, front_waiting]
            .iter()
            .filter(|&&b| b)
            .count();

        // Determine new shape and output direction using the auto-connect logic
        let (new_shape, new_output_dir) = if input_count >= 2 {
            // Input 2+: TJunction (merge) - output is always forward
            (ConveyorShape::TJunction, conveyor.direction)
        } else if input_count == 1 {
            if has_back_input {
                // Back input
                if wait_count >= 2 {
                    (ConveyorShape::Splitter, conveyor.direction)
                } else if right_waiting && !front_waiting {
                    // Back in, right out
                    (ConveyorShape::CornerRight, conveyor.direction.right())
                } else if left_waiting && !front_waiting {
                    // Back in, left out
                    (ConveyorShape::CornerLeft, conveyor.direction.left())
                } else {
                    (ConveyorShape::Straight, conveyor.direction)
                }
            } else if has_left_input {
                // Left input
                if front_waiting && right_waiting {
                    (ConveyorShape::Splitter, conveyor.direction)
                } else if right_waiting && !front_waiting {
                    // Left in, right out (U-turn)
                    (ConveyorShape::CornerRight, conveyor.direction.right())
                } else {
                    // Left in, front out: item turns RIGHT
                    (ConveyorShape::CornerRight, conveyor.direction)
                }
            } else if has_right_input {
                // Right input
                if front_waiting && left_waiting {
                    (ConveyorShape::Splitter, conveyor.direction)
                } else if left_waiting && !front_waiting {
                    // Right in, left out (U-turn)
                    (ConveyorShape::CornerLeft, conveyor.direction.left())
                } else {
                    // Right in, front out: item turns LEFT
                    (ConveyorShape::CornerLeft, conveyor.direction)
                }
            } else {
                // Front input (head-on)
                (ConveyorShape::Straight, conveyor.direction)
            }
        } else {
            // Input 0: Straight
            (ConveyorShape::Straight, conveyor.direction)
        };

        // Update output direction
        conveyor.output_direction = new_output_dir;

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
                        output_direction: new_output_dir,
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
