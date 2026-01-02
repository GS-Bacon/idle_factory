//! Chunk loading, unloading, and mesh generation systems

use crate::components::Player;
use crate::world::{ChunkData, ChunkMesh, ChunkMeshData, ChunkMeshTasks, WorldData};
use crate::VIEW_DISTANCE;
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::tasks::AsyncComputeTaskPool;
#[cfg(not(target_arch = "wasm32"))]
use futures_lite::future;
use std::collections::{HashMap, HashSet};

/// Generate chunk data synchronously (shared between native and WASM)
fn generate_chunk_sync(chunk_coord: IVec2) -> ChunkMeshData {
    let chunk_data = ChunkData::generate(chunk_coord);
    let mesh = chunk_data.generate_mesh(chunk_coord);

    // Convert local positions to world positions for the blocks map
    let mut world_blocks = HashMap::new();
    for (&local_pos, &block_type) in &chunk_data.blocks_map {
        let world_pos = WorldData::local_to_world(chunk_coord, local_pos);
        world_blocks.insert(world_pos, block_type);
    }

    ChunkMeshData {
        coord: chunk_coord,
        mesh,
        blocks: world_blocks,
    }
}

/// Spawn async tasks for chunk generation (Native: background threads, WASM: synchronous)
pub fn spawn_chunk_tasks(
    mut tasks: ResMut<ChunkMeshTasks>,
    world_data: Res<WorldData>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_world_pos = IVec3::new(
        player_transform.translation.x.floor() as i32,
        0,
        player_transform.translation.z.floor() as i32,
    );
    let player_chunk = WorldData::world_to_chunk(player_world_pos);

    // Limit chunks per frame: Native=4 (async), WASM=1 (sync, avoid freeze)
    #[cfg(not(target_arch = "wasm32"))]
    const MAX_SPAWN_PER_FRAME: i32 = 4;
    #[cfg(target_arch = "wasm32")]
    const MAX_SPAWN_PER_FRAME: i32 = 1;

    let mut spawned = 0;
    for dx in -VIEW_DISTANCE..=VIEW_DISTANCE {
        for dz in -VIEW_DISTANCE..=VIEW_DISTANCE {
            if spawned >= MAX_SPAWN_PER_FRAME {
                return;
            }

            let chunk_coord = IVec2::new(player_chunk.x + dx, player_chunk.y + dz);

            // Skip if already loaded or being generated
            if world_data.chunks.contains_key(&chunk_coord)
                || tasks.pending.contains_key(&chunk_coord)
            {
                continue;
            }

            // Native: spawn async task
            #[cfg(not(target_arch = "wasm32"))]
            {
                let task_pool = AsyncComputeTaskPool::get();
                let task = task_pool.spawn(async move { generate_chunk_sync(chunk_coord) });
                tasks.pending.insert(chunk_coord, PendingChunk::Task(task));
            }

            // WASM: generate synchronously and store result
            #[cfg(target_arch = "wasm32")]
            {
                let data = generate_chunk_sync(chunk_coord);
                tasks.pending.insert(chunk_coord, PendingChunk::Ready(data));
            }

            spawned += 1;
        }
    }
}

/// Receive completed chunk meshes and spawn them
pub fn receive_chunk_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_data: ResMut<WorldData>,
    mut tasks: ResMut<ChunkMeshTasks>,
) {
    // Limit chunks processed per frame to avoid frame spikes
    #[cfg(not(target_arch = "wasm32"))]
    const MAX_CHUNKS_PER_FRAME: usize = 2;
    #[cfg(target_arch = "wasm32")]
    const MAX_CHUNKS_PER_FRAME: usize = 1;

    let mut completed: Vec<(IVec2, ChunkMeshData)> = Vec::new();

    // Collect completed chunks
    #[cfg(not(target_arch = "wasm32"))]
    {
        for (&coord, pending) in tasks.pending.iter_mut() {
            if completed.len() >= MAX_CHUNKS_PER_FRAME {
                break;
            }
            if let PendingChunk::Task(task) = pending {
                if let Some(data) = future::block_on(future::poll_once(task)) {
                    tracing::debug!("Task completed for chunk {:?}", coord);
                    completed.push((coord, data));
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        // WASM: all pending chunks are already Ready
        for (&coord, _) in tasks.pending.iter() {
            if completed.len() >= MAX_CHUNKS_PER_FRAME {
                break;
            }
            // Mark for extraction (we'll remove and extract below)
            completed.push((coord, ChunkMeshData::default()));
        }
    }

    // Collect coords that need neighbor mesh regeneration
    let mut coords_needing_neighbor_update: HashSet<IVec2> = HashSet::new();

    if !completed.is_empty() {
        tracing::debug!("Processing {} completed chunks", completed.len());
    }

    // Process completed chunks - use the data we already extracted
    for (coord, chunk_mesh_data) in completed {
        // Remove from pending (we already have the data)
        let _ = tasks.pending.remove(&coord);

        // Skip if chunk already exists (player may have modified it)
        if world_data.chunks.contains_key(&coord) {
            continue;
        }

        // Create chunk data from blocks
        let mut blocks = vec![None; ChunkData::ARRAY_SIZE];
        let mut blocks_map = HashMap::new();
        for (&world_pos, &block_type) in &chunk_mesh_data.blocks {
            let local_pos = WorldData::world_to_local(world_pos);
            let idx = ChunkData::pos_to_index(local_pos.x, local_pos.y, local_pos.z);
            blocks[idx] = Some(block_type);
            blocks_map.insert(local_pos, block_type);
        }

        // Apply player modifications (placed/destroyed blocks)
        for (&world_pos, &maybe_block) in &world_data.modified_blocks {
            // Only apply modifications for this chunk
            if WorldData::world_to_chunk(world_pos) != coord {
                continue;
            }
            let local_pos = WorldData::world_to_local(world_pos);
            let idx = ChunkData::pos_to_index(local_pos.x, local_pos.y, local_pos.z);
            match maybe_block {
                Some(block_type) => {
                    // Player placed a block
                    blocks[idx] = Some(block_type);
                    blocks_map.insert(local_pos, block_type);
                }
                None => {
                    // Player removed a block (air)
                    blocks[idx] = None;
                    blocks_map.remove(&local_pos);
                }
            }
        }

        let chunk_data = ChunkData { blocks, blocks_map };

        world_data.chunks.insert(coord, chunk_data);
        coords_needing_neighbor_update.insert(coord);
        tracing::debug!("Chunk {:?} loaded", coord);
    }

    // Now regenerate meshes for new chunks and their neighbors (with proper neighbor data)
    for coord in &coords_needing_neighbor_update {
        let coord = *coord;
        // Regenerate this chunk's mesh with neighbor awareness
        if let Some(new_mesh) = world_data.generate_chunk_mesh(coord) {
            let mesh_handle = meshes.add(new_mesh);
            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.9,
                ..default()
            });

            // Find and despawn old mesh entity if exists
            if let Some(entities) = world_data.chunk_entities.remove(&coord) {
                for entity in entities {
                    commands.entity(entity).try_despawn_recursive();
                }
            }

            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material),
                    Transform::IDENTITY,
                    ChunkMesh { coord },
                ))
                .id();

            world_data.chunk_entities.insert(coord, vec![entity]);
            tracing::trace!("Chunk {:?} mesh spawned", coord);
        }

        // Also regenerate neighboring chunks' meshes
        let neighbors = [
            IVec2::new(coord.x - 1, coord.y),
            IVec2::new(coord.x + 1, coord.y),
            IVec2::new(coord.x, coord.y - 1),
            IVec2::new(coord.x, coord.y + 1),
        ];

        for neighbor_coord in neighbors {
            if !world_data.chunks.contains_key(&neighbor_coord) {
                continue;
            }
            if coords_needing_neighbor_update.contains(&neighbor_coord) {
                continue;
            }

            if let Some(new_mesh) = world_data.generate_chunk_mesh(neighbor_coord) {
                let mesh_handle = meshes.add(new_mesh);
                let material = materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 0.9,
                    ..default()
                });

                if let Some(entities) = world_data.chunk_entities.remove(&neighbor_coord) {
                    for entity in entities {
                        commands.entity(entity).try_despawn_recursive();
                    }
                }

                let entity = commands
                    .spawn((
                        Mesh3d(mesh_handle),
                        MeshMaterial3d(material),
                        Transform::IDENTITY,
                        ChunkMesh {
                            coord: neighbor_coord,
                        },
                    ))
                    .id();

                world_data
                    .chunk_entities
                    .insert(neighbor_coord, vec![entity]);
            }
        }
    }
}

/// Unload distant chunks
pub fn unload_distant_chunks(
    mut commands: Commands,
    mut world_data: ResMut<WorldData>,
    mut tasks: ResMut<ChunkMeshTasks>,
    player_query: Query<&Transform, With<Player>>,
    chunk_mesh_query: Query<(Entity, &ChunkMesh)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_world_pos = IVec3::new(
        player_transform.translation.x.floor() as i32,
        0,
        player_transform.translation.z.floor() as i32,
    );
    let player_chunk = WorldData::world_to_chunk(player_world_pos);

    // Find chunks to unload
    let mut chunks_to_unload: Vec<IVec2> = Vec::new();
    for &chunk_coord in world_data.chunks.keys() {
        let dx = (chunk_coord.x - player_chunk.x).abs();
        let dz = (chunk_coord.y - player_chunk.y).abs();
        if dx > VIEW_DISTANCE + 1 || dz > VIEW_DISTANCE + 1 {
            chunks_to_unload.push(chunk_coord);
        }
    }

    // Unload chunks
    for chunk_coord in chunks_to_unload {
        for (entity, chunk_mesh) in chunk_mesh_query.iter() {
            if chunk_mesh.coord == chunk_coord {
                commands.entity(entity).despawn();
            }
        }

        world_data.chunks.remove(&chunk_coord);
        world_data.chunk_entities.remove(&chunk_coord);
        tasks.pending.remove(&chunk_coord);
    }
}

// Re-export PendingChunk from world module
pub use crate::world::PendingChunk;
