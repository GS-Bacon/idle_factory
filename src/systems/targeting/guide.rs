//! Guide marker systems

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::meshes::create_wireframe_cube_mesh;
use crate::player::Inventory;
use crate::{BlockType, Conveyor, Crusher, Direction, Furnace, GuideMarker, GuideMarkers, Miner};

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
    let mut existing: HashSet<IVec3> = HashSet::new();

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
    let mut conveyor_positions: HashSet<IVec3> = HashSet::new();

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
