use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::rendering::chunk::{Chunk, CHUNK_SIZE};
use crate::core::registry::BlockRegistry;
use crate::rendering::models::{MeshBuilder, mesh_conveyor};

// ★復活: これが消えていました
#[derive(Component)]
pub struct MeshDirty;

// Directionもここで公開しておきます
#[derive(Clone, Copy)]
pub enum Direction { XPos, XNeg, YPos, YNeg, ZPos, ZNeg }

pub fn update_chunk_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_registry: Res<BlockRegistry>,
    query: Query<(Entity, &Chunk), With<MeshDirty>>,
) {
    for (entity, chunk) in query.iter() {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        let mut colors = Vec::new();
        let mut idx_counter = 0;
        let cs = CHUNK_SIZE as i32;

        let mut builder = MeshBuilder {
            positions: &mut positions,
            normals: &mut normals,
            indices: &mut indices,
            colors: &mut colors,
            idx_counter: &mut idx_counter,
        };

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    // ★修正: 型推論エラーを防ぐため、一度変数で受けます
                    let maybe_block = chunk.get_block(x, y, z);
                    let block_id = if let Some(id) = maybe_block {
                        id
                    } else {
                        continue;
                    };

                    if block_id == "air" { continue; }
                    if block_registry.map.get(block_id).is_none() { continue; }

                    let bx = x as f32;
                    let by = y as f32;
                    let bz = z as f32;

                    let color = match block_id.as_str() {
                        "conveyor" => [0.2, 0.2, 0.2, 1.0],
                        "dirt" => [0.4, 0.25, 0.1, 1.0],
                        "stone" => [0.5, 0.5, 0.5, 1.0],
                        _ => [1.0, 0.0, 1.0, 1.0],
                    };

                    if block_id == "conveyor" {
                        mesh_conveyor(&mut builder, bx, by, bz, color);
                    } else {
                        let ix = x as i32;
                        let iy = y as i32;
                        let iz = z as i32;

                        let is_air_or_transparent = |cx: i32, cy: i32, cz: i32| -> bool {
                            if cx < 0 || cy < 0 || cz < 0 || cx >= cs || cy >= cs || cz >= cs { return true; }
                            chunk.get_block(cx as usize, cy as usize, cz as usize)
                                 .map_or(true, |id| id == "air" || id == "conveyor") 
                        };

                        if is_air_or_transparent(ix, iy + 1, iz) { builder.push_face_by_dir(bx, by, bz, Direction::YPos, color); }
                        if is_air_or_transparent(ix, iy - 1, iz) { builder.push_face_by_dir(bx, by, bz, Direction::YNeg, color); }
                        if is_air_or_transparent(ix + 1, iy, iz) { builder.push_face_by_dir(bx, by, bz, Direction::XPos, color); }
                        if is_air_or_transparent(ix - 1, iy, iz) { builder.push_face_by_dir(bx, by, bz, Direction::XNeg, color); }
                        if is_air_or_transparent(ix, iy, iz + 1) { builder.push_face_by_dir(bx, by, bz, Direction::ZPos, color); }
                        if is_air_or_transparent(ix, iy, iz - 1) { builder.push_face_by_dir(bx, by, bz, Direction::ZNeg, color); }
                    }
                }
            }
        }
        
        if positions.is_empty() {
            commands.entity(entity).remove::<MeshDirty>();
            continue;
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));

        commands.entity(entity)
            .insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 0.6,
                    ..default()
                })),
                Transform::from_translation(chunk.position.as_vec3() * CHUNK_SIZE as f32),
            ))
            .remove::<MeshDirty>();
    }
}