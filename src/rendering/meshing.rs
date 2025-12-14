use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::rendering::chunk::{Chunk, CHUNK_SIZE};
use crate::core::registry::BlockRegistry;
// ★修正: modelsから MeshType, get_block_visual をインポート
use crate::rendering::models::{MeshBuilder, MeshType, get_block_visual};

#[derive(Component)]
pub struct MeshDirty;

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
                    // ブロックID取得
                    let block_id = match chunk.get_block(x, y, z) {
                        Some(id) => id,
                        None => continue,
                    };

                    // Airと無効なブロックはスキップ
                    if block_id == "air" { continue; }
                    if block_registry.map.get(block_id).is_none() { continue; }

                    let bx = x as f32;
                    let by = y as f32;
                    let bz = z as f32;

                    // ★変更点: ここで一括取得！
                    let visual = get_block_visual(block_id);

                    // ★変更点: Visual情報に基づいて分岐（IDによるハードコーディング排除）
                    match visual.mesh_type {
                        MeshType::Custom(mesh_fn) => {
                            // カスタム形状なら関数を実行
                            mesh_fn(&mut builder, bx, by, bz, visual.color);
                        },
                        MeshType::Cube => {
                            // 通常ブロックなら面カリング処理
                            let ix = x as i32;
                            let iy = y as i32;
                            let iz = z as i32;

                            // 隣が「透明（透過設定あり）」または「範囲外」なら面を描く
                            let should_draw_face = |cx: i32, cy: i32, cz: i32| -> bool {
                                if cx < 0 || cy < 0 || cz < 0 || cx >= cs || cy >= cs || cz >= cs { 
                                    return true; 
                                }
                                // 隣のブロックを取得して視覚情報を確認
                                chunk.get_block(cx as usize, cy as usize, cz as usize)
                                     .map_or(true, |neighbor_id| {
                                         // 隣がAir、または「is_transparent = true」なブロックなら描画する
                                         get_block_visual(neighbor_id).is_transparent
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
        
        // メッシュ生成・エンティティ更新処理 (変更なし)
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