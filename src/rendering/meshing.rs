use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::rendering::chunk::{Chunk, CHUNK_SIZE};
use crate::core::registry::BlockRegistry;
use crate::rendering::models::{MeshBuilder, MeshType, get_block_visual};

#[derive(Component)]
pub struct MeshDirty;

// ★追加: チャンク用マテリアルを保持するリソース
#[derive(Resource)]
pub struct ChunkMaterialHandle(pub Handle<StandardMaterial>);

#[derive(Clone, Copy)]
pub enum Direction { XPos, XNeg, YPos, YNeg, ZPos, ZNeg }

pub fn update_chunk_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>, // 削除: ここで毎回作らない
    chunk_material: Res<ChunkMaterialHandle>, // ★追加: 共有マテリアルを使用
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
                    let block_id = match chunk.get_block(x, y, z) {
                        Some(id) => id,
                        None => continue,
                    };

                    if block_id == "air" { continue; }
                    if !block_registry.map.contains_key(block_id) { continue; }

                    // マシン系は個別モデルなので除外
                    if block_id == "conveyor" || block_id == "miner" {
                        continue;
                    }

                    let bx = x as f32;
                    let by = y as f32;
                    let bz = z as f32;

                    let visual = get_block_visual(block_id);

                    match visual.mesh_type {
                        MeshType::VoxModel(_) => { continue; },
                        MeshType::Cube => {
                            let ix = x as i32;
                            let iy = y as i32;
                            let iz = z as i32;

                            let should_draw_face = |cx: i32, cy: i32, cz: i32| -> bool {
                                if cx < 0 || cy < 0 || cz < 0 || cx >= cs || cy >= cs || cz >= cs {
                                    return true;
                                }
                                chunk.get_block(cx as usize, cy as usize, cz as usize)
                                     .is_none_or(|neighbor_id| {
                                         let neighbor_visual = get_block_visual(neighbor_id);
                                         neighbor_visual.is_transparent
                                     })
                            };

                            if should_draw_face(ix, iy + 1, iz) { builder.push_face_by_dir(bx, by, bz, Direction::YPos, visual.color); }
                            if should_draw_face(ix, iy - 1, iz) { builder.push_face_by_dir(bx, by, bz, Direction::YNeg, visual.color); }
                            if should_draw_face(ix + 1, iy, iz) { builder.push_face_by_dir(bx, by, bz, Direction::XPos, visual.color); }
                            if should_draw_face(ix - 1, iy, iz) { builder.push_face_by_dir(bx, by, bz, Direction::XNeg, visual.color); }
                            if should_draw_face(ix, iy, iz + 1) { builder.push_face_by_dir(bx, by, bz, Direction::ZPos, visual.color); }
                            if should_draw_face(ix, iy, iz - 1) { builder.push_face_by_dir(bx, by, bz, Direction::ZNeg, visual.color); }
                        }
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
                // ★修正: 毎回生成せず、リソースからクローン(軽量コピー)する
                MeshMaterial3d(chunk_material.0.clone()), 
                Transform::from_translation(chunk.position.as_vec3() * CHUNK_SIZE as f32),
            ))
            .remove::<MeshDirty>();
    }
}