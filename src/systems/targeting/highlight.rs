//! Target block highlighting system

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;

use crate::meshes::{
    create_conveyor_mesh, create_conveyor_wireframe_mesh, create_wireframe_cube_mesh,
};
use crate::player::Inventory;
use crate::utils::{auto_conveyor_direction, yaw_to_direction};
use crate::{
    BlockType, Conveyor, ConveyorRotationOffset, ConveyorShape, Crusher, Direction, Furnace, Miner,
    PlaceHighlight, PlayerCamera, TargetBlock, TargetHighlight, BLOCK_SIZE,
};

/// Marker for conveyor preview arrow
#[derive(Component)]
pub struct ConveyorPreviewArrow;

/// Cached meshes for highlight wireframes (avoid recreation every frame)
#[derive(Resource)]
pub struct HighlightMeshCache {
    pub cube_mesh: Handle<Mesh>,
    pub conveyor_north: Handle<Mesh>,
    pub conveyor_south: Handle<Mesh>,
    pub conveyor_east: Handle<Mesh>,
    pub conveyor_west: Handle<Mesh>,
    // Solid conveyor preview meshes (semi-transparent)
    pub conveyor_solid: Handle<Mesh>,
    // Solid cube for machine preview
    pub machine_solid: Handle<Mesh>,
    // Arrow meshes for direction (3D solid arrows)
    pub arrow_north: Handle<Mesh>,
    pub arrow_south: Handle<Mesh>,
    pub arrow_east: Handle<Mesh>,
    pub arrow_west: Handle<Mesh>,
    pub red_material: Handle<StandardMaterial>,
    pub green_material: Handle<StandardMaterial>,
    // Semi-transparent green for conveyor preview
    pub conveyor_preview_material: Handle<StandardMaterial>,
    // Semi-transparent blue for machine preview
    pub machine_preview_material: Handle<StandardMaterial>,
    // Bright yellow for arrow visibility
    pub arrow_material: Handle<StandardMaterial>,
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

    pub fn get_arrow_mesh(&self, dir: Direction) -> Handle<Mesh> {
        match dir {
            Direction::North => self.arrow_north.clone(),
            Direction::South => self.arrow_south.clone(),
            Direction::East => self.arrow_east.clone(),
            Direction::West => self.arrow_west.clone(),
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
        // Solid conveyor mesh for preview
        conveyor_solid: meshes.add(create_conveyor_mesh(ConveyorShape::Straight)),
        // Solid cube for machine preview
        machine_solid: meshes.add(Cuboid::new(
            BLOCK_SIZE * 0.95,
            BLOCK_SIZE * 0.95,
            BLOCK_SIZE * 0.95,
        )),
        // Arrow meshes (3D solid)
        arrow_north: meshes.add(create_3d_arrow_mesh(Direction::North)),
        arrow_south: meshes.add(create_3d_arrow_mesh(Direction::South)),
        arrow_east: meshes.add(create_3d_arrow_mesh(Direction::East)),
        arrow_west: meshes.add(create_3d_arrow_mesh(Direction::West)),
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
        // Semi-transparent green for conveyor preview
        conveyor_preview_material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.8, 0.2, 0.5),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        // Semi-transparent blue for machine preview
        machine_preview_material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.3, 0.5, 1.0, 0.5),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        // Bright yellow for arrow visibility
        arrow_material: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.0),
            unlit: true,
            ..default()
        }),
    });
}

/// Create a 3D solid arrow mesh for better visibility
fn create_3d_arrow_mesh(direction: Direction) -> Mesh {
    use bevy::render::mesh::{Indices, PrimitiveTopology};
    use bevy::render::render_asset::RenderAssetUsages;

    let arrow_y = 0.55; // Height above conveyor/machine
    let shaft_length = 0.35;
    let shaft_width = 0.08;
    let shaft_height = 0.06;
    let head_length = 0.2;
    let head_width = 0.2;

    // Build arrow pointing in +Z direction, then rotate
    let half_sw = shaft_width / 2.0;
    let half_sh = shaft_height / 2.0;
    let half_hw = head_width / 2.0;

    // Shaft vertices (8 vertices for cuboid)
    let shaft_front = shaft_length / 2.0;
    let shaft_back = -shaft_length / 2.0;

    // Head vertices (5 vertices for pyramid)
    let head_tip = shaft_front + head_length;
    let head_base = shaft_front;

    let mut positions = vec![
        // Shaft (box) - 8 vertices
        [-half_sw, arrow_y - half_sh, shaft_back],  // 0
        [half_sw, arrow_y - half_sh, shaft_back],   // 1
        [half_sw, arrow_y + half_sh, shaft_back],   // 2
        [-half_sw, arrow_y + half_sh, shaft_back],  // 3
        [-half_sw, arrow_y - half_sh, shaft_front], // 4
        [half_sw, arrow_y - half_sh, shaft_front],  // 5
        [half_sw, arrow_y + half_sh, shaft_front],  // 6
        [-half_sw, arrow_y + half_sh, shaft_front], // 7
        // Head (pyramid base + tip) - 5 vertices
        [-half_hw, arrow_y - half_sh, head_base], // 8
        [half_hw, arrow_y - half_sh, head_base],  // 9
        [half_hw, arrow_y + half_sh, head_base],  // 10
        [-half_hw, arrow_y + half_sh, head_base], // 11
        [0.0, arrow_y, head_tip],                 // 12 (tip)
    ];

    // Rotate positions based on direction
    let rotation = match direction {
        Direction::North => std::f32::consts::PI,        // -Z
        Direction::South => 0.0,                         // +Z
        Direction::East => -std::f32::consts::FRAC_PI_2, // +X
        Direction::West => std::f32::consts::FRAC_PI_2,  // -X
    };

    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    for pos in &mut positions {
        let x = pos[0];
        let z = pos[2];
        pos[0] = x * cos_r - z * sin_r;
        pos[2] = x * sin_r + z * cos_r;
    }

    // Indices
    let indices = vec![
        // Shaft faces
        0, 1, 2, 0, 2, 3, // back
        4, 6, 5, 4, 7, 6, // front
        0, 4, 5, 0, 5, 1, // bottom
        3, 2, 6, 3, 6, 7, // top
        0, 3, 7, 0, 7, 4, // left
        1, 5, 6, 1, 6, 2, // right
        // Head faces (pyramid)
        8, 12, 9, // bottom
        9, 12, 10, // right
        10, 12, 11, // top
        11, 12, 8, // left
        8, 9, 10, 8, 10, 11, // base
    ];

    // Simple normals (up direction for visibility)
    let normals: Vec<[f32; 3]> = positions.iter().map(|_| [0.0, 1.0, 0.0]).collect();

    let uvs: Vec<[f32; 2]> = positions.iter().map(|_| [0.0, 0.0]).collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
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

    // Check what item is selected
    let selected_item = inventory.get_selected_type();
    let placing_conveyor = selected_item == Some(BlockType::ConveyorBlock);
    let placing_machine = matches!(
        selected_item,
        Some(BlockType::MinerBlock | BlockType::FurnaceBlock | BlockType::CrusherBlock)
    );

    // Get player's facing direction as fallback
    let player_facing = camera_query.get_single().ok().map(|cam_transform| {
        let forward = cam_transform.forward().as_vec3();
        yaw_to_direction(-forward.x.atan2(-forward.z))
    });

    // Calculate place direction using auto_conveyor_direction (same logic as block_place)
    let place_direction = if placing_conveyor || placing_machine {
        if let (Some(place_pos), Some(fallback_dir)) = (target.place_target, player_facing) {
            if placing_conveyor {
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
                    machine_positions.push(crate::world_to_grid(furnace_transform.translation));
                }

                // Apply rotation offset (R key)
                let mut dir = auto_conveyor_direction(
                    place_pos,
                    fallback_dir,
                    &conveyors,
                    &machine_positions,
                );
                for _ in 0..rotation.offset {
                    dir = dir.rotate_cw();
                }
                Some(dir)
            } else {
                // Machine: use player facing with rotation offset
                let mut dir = fallback_dir;
                for _ in 0..rotation.offset {
                    dir = dir.rotate_cw();
                }
                Some(dir)
            }
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

    // === Place target - only show if player has a placeable item ===
    if let Some(pos) = target.place_target.filter(|_| has_placeable_item) {
        // Despawn old entity if exists (recursively to remove arrow child)
        if let Some(entity) = target.place_highlight_entity.take() {
            if place_query.get(entity).is_ok() {
                commands.entity(entity).despawn_recursive();
            }
        }

        // Calculate position based on item type
        let entity = if placing_conveyor {
            // Conveyor: position at ground level (Y = 0 for Y=0 block)
            let conveyor_center = Vec3::new(
                pos.x as f32 + 0.5,
                pos.y as f32, // Ground level, not center
                pos.z as f32 + 0.5,
            );
            let dir = place_direction.unwrap_or(Direction::North);
            let rotation = dir.to_rotation();
            commands
                .spawn((
                    Mesh3d(cache.conveyor_solid.clone()),
                    MeshMaterial3d(cache.conveyor_preview_material.clone()),
                    Transform::from_translation(conveyor_center).with_rotation(rotation),
                    PlaceHighlight,
                    NotShadowCaster,
                ))
                .with_children(|parent| {
                    // Arrow on top (in local space, so no rotation needed)
                    parent.spawn((
                        Mesh3d(cache.get_arrow_mesh(dir)),
                        MeshMaterial3d(cache.arrow_material.clone()),
                        // Arrow is defined in world space, so reverse rotation for local
                        Transform::from_rotation(rotation.inverse()),
                        ConveyorPreviewArrow,
                        NotShadowCaster,
                    ));
                })
                .id()
        } else if placing_machine {
            // Machine: semi-transparent cube + direction arrow
            let machine_center =
                Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5);
            let dir = place_direction.unwrap_or(Direction::North);
            let rotation = dir.to_rotation();
            commands
                .spawn((
                    Mesh3d(cache.machine_solid.clone()),
                    MeshMaterial3d(cache.machine_preview_material.clone()),
                    Transform::from_translation(machine_center).with_rotation(rotation),
                    PlaceHighlight,
                    NotShadowCaster,
                ))
                .with_children(|parent| {
                    // Arrow on top
                    parent.spawn((
                        Mesh3d(cache.get_arrow_mesh(dir)),
                        MeshMaterial3d(cache.arrow_material.clone()),
                        Transform::from_rotation(rotation.inverse()),
                        ConveyorPreviewArrow,
                        NotShadowCaster,
                    ));
                })
                .id()
        } else {
            // Other items: green wireframe at block center
            let center = Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5);
            commands
                .spawn((
                    Mesh3d(cache.cube_mesh.clone()),
                    MeshMaterial3d(cache.green_material.clone()),
                    Transform::from_translation(center),
                    PlaceHighlight,
                    NotShadowCaster,
                ))
                .id()
        };
        target.place_highlight_entity = Some(entity);
    } else if let Some(entity) = target.place_highlight_entity.take() {
        if place_query.get(entity).is_ok() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
