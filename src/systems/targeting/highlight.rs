//! Target block highlighting system

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;

use crate::meshes::{create_conveyor_wireframe_mesh, create_wireframe_cube_mesh};
use crate::player::Inventory;
use crate::utils::{auto_conveyor_direction, yaw_to_direction};
use crate::{
    BlockType, Conveyor, ConveyorRotationOffset, Crusher, Direction, Furnace, Miner,
    PlaceHighlight, PlayerCamera, TargetBlock, TargetHighlight,
};

/// Cached meshes for highlight wireframes (avoid recreation every frame)
#[derive(Resource)]
pub struct HighlightMeshCache {
    pub cube_mesh: Handle<Mesh>,
    pub conveyor_north: Handle<Mesh>,
    pub conveyor_south: Handle<Mesh>,
    pub conveyor_east: Handle<Mesh>,
    pub conveyor_west: Handle<Mesh>,
    pub red_material: Handle<StandardMaterial>,
    pub green_material: Handle<StandardMaterial>,
}

impl HighlightMeshCache {
    pub fn get_conveyor_mesh(&self, dir: Direction) -> Handle<Mesh> {
        match dir {
            Direction::North => self.conveyor_north.clone(),
            Direction::South => self.conveyor_south.clone(),
            Direction::East => self.conveyor_east.clone(),
            Direction::West => self.conveyor_west.clone(),
        }
    }
}

/// Setup highlight mesh cache (run once at startup)
pub fn setup_highlight_cache(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(HighlightMeshCache {
        cube_mesh: meshes.add(create_wireframe_cube_mesh()),
        conveyor_north: meshes.add(create_conveyor_wireframe_mesh(Direction::North)),
        conveyor_south: meshes.add(create_conveyor_wireframe_mesh(Direction::South)),
        conveyor_east: meshes.add(create_conveyor_wireframe_mesh(Direction::East)),
        conveyor_west: meshes.add(create_conveyor_wireframe_mesh(Direction::West)),
        red_material: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.2),
            unlit: true,
            ..default()
        }),
        green_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 1.0, 0.2),
            unlit: true,
            ..default()
        }),
    });
}

/// Update target highlight entity position
#[allow(clippy::too_many_arguments)]
pub fn update_target_highlight(
    mut commands: Commands,
    mut target: ResMut<TargetBlock>,
    break_query: Query<Entity, (With<TargetHighlight>, Without<PlaceHighlight>)>,
    place_query: Query<Entity, (With<PlaceHighlight>, Without<TargetHighlight>)>,
    cache: Res<HighlightMeshCache>,
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
            let conveyors: Vec<(IVec3, crate::Direction)> = conveyor_query
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
    let break_conveyor_dir = target.break_target.and_then(|pos| {
        conveyor_query
            .iter()
            .find(|c| c.position == pos)
            .map(|c| c.direction)
    });

    // === Break target (red wireframe) - always show when looking at a block ===
    if let Some(pos) = target.break_target {
        let center = Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5);

        // Despawn old entity if exists
        if let Some(entity) = target.break_highlight_entity.take() {
            if break_query.get(entity).is_ok() {
                commands.entity(entity).despawn();
            }
        }

        // Use cached mesh (no recreation every frame)
        let mesh = if let Some(dir) = break_conveyor_dir {
            cache.get_conveyor_mesh(dir)
        } else {
            cache.cube_mesh.clone()
        };
        let entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(cache.red_material.clone()),
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

        // Despawn old entity if exists
        if let Some(entity) = target.place_highlight_entity.take() {
            if place_query.get(entity).is_ok() {
                commands.entity(entity).despawn();
            }
        }

        // Use cached mesh (no recreation every frame)
        let mesh = if let Some(dir) = place_direction {
            cache.get_conveyor_mesh(dir)
        } else {
            cache.cube_mesh.clone()
        };
        let entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(cache.green_material.clone()),
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
