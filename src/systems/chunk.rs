//! Chunk loading, unloading, and mesh generation systems

use crate::components::Player;
use crate::settings::GameSettings;
use crate::textures::{TextureRegistry, UVCache};
use crate::world::{ChunkData, ChunkLod, ChunkMesh, ChunkMeshData, ChunkMeshTasks, WorldData};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures_lite::future;
use std::collections::{HashMap, HashSet};

/// Cached material for chunk rendering with texture atlas
#[derive(Resource, Default)]
pub struct ChunkMaterialCache {
    /// Cached material handle (reused for all chunks)
    pub material: Option<Handle<StandardMaterial>>,
    /// Generation counter to detect texture atlas changes
    pub generation: u32,
}

/// Calculate LOD for a chunk based on distance from player
fn calculate_lod(chunk_coord: IVec2, player_chunk: IVec2) -> ChunkLod {
    let dx = (chunk_coord.x - player_chunk.x).abs();
    let dz = (chunk_coord.y - player_chunk.y).abs();
    let distance = dx.max(dz);
    ChunkLod::from_distance(distance)
}

/// Generate chunk data synchronously with texture UV support
fn generate_chunk_sync(chunk_coord: IVec2, uv_cache: UVCache) -> ChunkMeshData {
    let chunk_data = ChunkData::generate(chunk_coord);

    // Use textured mesh generation if UV cache has entries
    let mesh = if uv_cache.is_empty() {
        chunk_data.generate_mesh(chunk_coord)
    } else {
        chunk_data.generate_mesh_textured(chunk_coord, |_| false, ChunkLod::Full, &uv_cache)
    };

    // Convert flat array to world positions HashMap for ChunkMeshData
    let mut world_blocks = HashMap::new();
    for idx in 0..ChunkData::ARRAY_SIZE {
        if let Some(block_type) = chunk_data.blocks[idx] {
            let local_pos = ChunkData::index_to_pos(idx);
            let world_pos = WorldData::local_to_world(chunk_coord, local_pos);
            world_blocks.insert(world_pos, block_type);
        }
    }

    ChunkMeshData {
        coord: chunk_coord,
        mesh,
        blocks: world_blocks,
    }
}

/// Get or create the chunk material with texture atlas
fn get_chunk_material(
    cache: &mut ChunkMaterialCache,
    materials: &mut Assets<StandardMaterial>,
    texture_registry: &TextureRegistry,
) -> Handle<StandardMaterial> {
    // Check if we need to create or update the material
    if let Some(ref handle) = cache.material {
        // TODO: Check if texture atlas has changed and needs rebuild
        return handle.clone();
    }

    // Create new material with texture atlas
    let material = if let Some(atlas_image) = texture_registry.atlas_image() {
        StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(atlas_image),
            perceptual_roughness: 0.9,
            reflectance: 0.1,
            ..default()
        }
    } else {
        // Fallback to solid color material if no atlas
        StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.9,
            ..default()
        }
    };

    let handle = materials.add(material);
    cache.material = Some(handle.clone());
    cache.generation += 1;
    handle
}

/// Spawn async tasks for chunk generation using background threads
pub fn spawn_chunk_tasks(
    mut tasks: ResMut<ChunkMeshTasks>,
    world_data: Res<WorldData>,
    player_query: Query<&Transform, With<Player>>,
    settings: Res<GameSettings>,
    texture_registry: Res<TextureRegistry>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_grid = crate::world_to_grid(player_transform.translation);
    let player_world_pos = IVec3::new(player_grid.x, 0, player_grid.z);
    let player_chunk = WorldData::world_to_chunk(player_world_pos);

    // Limit chunks per frame for async generation
    const MAX_SPAWN_PER_FRAME: i32 = 4;

    // Export UV cache once for all tasks this frame
    let uv_cache = texture_registry.export_uv_cache();

    let view_distance = settings.view_distance;
    let mut spawned = 0;
    for dx in -view_distance..=view_distance {
        for dz in -view_distance..=view_distance {
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

            // Clone UV cache for this task
            let uv_cache_clone = uv_cache.clone();

            // Spawn async task with UV cache
            let task_pool = AsyncComputeTaskPool::get();
            let task =
                task_pool.spawn(async move { generate_chunk_sync(chunk_coord, uv_cache_clone) });
            tasks.pending.insert(chunk_coord, PendingChunk::Task(task));

            spawned += 1;
        }
    }
}

/// Receive completed chunk meshes and spawn them
#[allow(clippy::too_many_arguments)]
pub fn receive_chunk_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_data: ResMut<WorldData>,
    mut tasks: ResMut<ChunkMeshTasks>,
    player_query: Query<&Transform, With<Player>>,
    mut material_cache: ResMut<ChunkMaterialCache>,
    texture_registry: Res<TextureRegistry>,
) {
    // Get player chunk for LOD calculation
    let player_chunk = player_query
        .get_single()
        .map(|pt| {
            let player_grid = crate::world_to_grid(pt.translation);
            WorldData::world_to_chunk(IVec3::new(player_grid.x, 0, player_grid.z))
        })
        .unwrap_or(IVec2::ZERO);

    // Limit chunks processed per frame to avoid frame spikes
    const MAX_CHUNKS_PER_FRAME: usize = 2;

    let mut completed: Vec<(IVec2, ChunkMeshData)> = Vec::new();

    // Collect completed chunks from async tasks
    for (&coord, pending) in tasks.pending.iter_mut() {
        if completed.len() >= MAX_CHUNKS_PER_FRAME {
            break;
        }
        let PendingChunk::Task(task) = pending;
        if let Some(data) = future::block_on(future::poll_once(task)) {
            tracing::debug!("Task completed for chunk {:?}", coord);
            completed.push((coord, data));
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
        for (&world_pos, &block_type) in &chunk_mesh_data.blocks {
            let local_pos = WorldData::world_to_local(world_pos);
            let idx = ChunkData::pos_to_index(local_pos.x, local_pos.y, local_pos.z);
            blocks[idx] = Some(block_type);
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
                }
                None => {
                    // Player removed a block (air)
                    blocks[idx] = None;
                }
            }
        }

        let chunk_data = ChunkData { blocks };

        world_data.chunks.insert(coord, chunk_data);
        coords_needing_neighbor_update.insert(coord);
        tracing::debug!("Chunk {:?} loaded", coord);
    }

    // Export UV cache for textured mesh generation
    let uv_cache = texture_registry.export_uv_cache();

    // Now regenerate meshes for new chunks and their neighbors (with proper neighbor data)
    for coord in &coords_needing_neighbor_update {
        let coord = *coord;
        // Calculate LOD based on distance from player
        let lod = calculate_lod(coord, player_chunk);

        // Regenerate this chunk's mesh with neighbor awareness, LOD, and textures
        if let Some(new_mesh) = world_data.generate_chunk_mesh_textured(coord, lod, &uv_cache) {
            let mesh_handle = meshes.add(new_mesh);
            let material =
                get_chunk_material(&mut material_cache, &mut materials, &texture_registry);

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
                    ChunkMesh { coord, lod },
                ))
                .id();

            world_data.chunk_entities.insert(coord, vec![entity]);
            tracing::trace!("Chunk {:?} mesh spawned with LOD {:?}", coord, lod);
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

            let neighbor_lod = calculate_lod(neighbor_coord, player_chunk);
            if let Some(new_mesh) =
                world_data.generate_chunk_mesh_textured(neighbor_coord, neighbor_lod, &uv_cache)
            {
                let mesh_handle = meshes.add(new_mesh);
                let material =
                    get_chunk_material(&mut material_cache, &mut materials, &texture_registry);

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
                            lod: neighbor_lod,
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
    settings: Res<GameSettings>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_grid = crate::world_to_grid(player_transform.translation);
    let player_world_pos = IVec3::new(player_grid.x, 0, player_grid.z);
    let player_chunk = WorldData::world_to_chunk(player_world_pos);

    let view_distance = settings.view_distance;

    // Find chunks to unload
    let mut chunks_to_unload: Vec<IVec2> = Vec::new();
    for &chunk_coord in world_data.chunks.keys() {
        let dx = (chunk_coord.x - player_chunk.x).abs();
        let dz = (chunk_coord.y - player_chunk.y).abs();
        if dx > view_distance + 1 || dz > view_distance + 1 {
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
pub use crate::world::DirtyChunks;
pub use crate::world::PendingChunk;

/// Update LOD for chunks based on player distance
/// Regenerates mesh if LOD level should change
#[allow(clippy::too_many_arguments)]
pub fn update_chunk_lod(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_data: ResMut<WorldData>,
    player_query: Query<&Transform, With<Player>>,
    chunk_mesh_query: Query<(Entity, &ChunkMesh)>,
    mut material_cache: ResMut<ChunkMaterialCache>,
    texture_registry: Res<TextureRegistry>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_grid = crate::world_to_grid(player_transform.translation);
    let player_chunk = WorldData::world_to_chunk(IVec3::new(player_grid.x, 0, player_grid.z));

    // Limit LOD updates per frame to avoid frame spikes
    const MAX_LOD_UPDATES_PER_FRAME: usize = 2;
    let mut updates = 0;

    // Export UV cache for textured mesh generation
    let uv_cache = texture_registry.export_uv_cache();

    for (entity, chunk_mesh) in chunk_mesh_query.iter() {
        if updates >= MAX_LOD_UPDATES_PER_FRAME {
            break;
        }

        let new_lod = calculate_lod(chunk_mesh.coord, player_chunk);

        // Skip if LOD hasn't changed
        if new_lod == chunk_mesh.lod {
            continue;
        }

        // Check if chunk still exists
        if !world_data.chunks.contains_key(&chunk_mesh.coord) {
            continue;
        }

        // Regenerate mesh with new LOD and textures
        if let Some(new_mesh) =
            world_data.generate_chunk_mesh_textured(chunk_mesh.coord, new_lod, &uv_cache)
        {
            // Despawn old entity
            commands.entity(entity).despawn_recursive();

            // Remove from chunk_entities
            world_data.chunk_entities.remove(&chunk_mesh.coord);

            let mesh_handle = meshes.add(new_mesh);
            let material =
                get_chunk_material(&mut material_cache, &mut materials, &texture_registry);

            let new_entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material),
                    Transform::IDENTITY,
                    ChunkMesh {
                        coord: chunk_mesh.coord,
                        lod: new_lod,
                    },
                ))
                .id();

            world_data
                .chunk_entities
                .insert(chunk_mesh.coord, vec![new_entity]);
            tracing::debug!(
                "Chunk {:?} LOD updated: {:?} -> {:?}",
                chunk_mesh.coord,
                chunk_mesh.lod,
                new_lod
            );
            updates += 1;
        }
    }
}

/// Process dirty chunks - regenerate meshes for chunks that had block changes
/// Limits regeneration to MAX_DIRTY_PER_FRAME to avoid frame spikes
#[allow(clippy::too_many_arguments)]
pub fn process_dirty_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_data: ResMut<WorldData>,
    mut dirty_chunks: ResMut<DirtyChunks>,
    player_query: Query<&Transform, With<Player>>,
    mut material_cache: ResMut<ChunkMaterialCache>,
    texture_registry: Res<TextureRegistry>,
) {
    if dirty_chunks.is_empty() {
        return;
    }

    // Get player chunk for LOD calculation
    let player_chunk = player_query
        .get_single()
        .map(|pt| {
            let player_grid = crate::world_to_grid(pt.translation);
            WorldData::world_to_chunk(IVec3::new(player_grid.x, 0, player_grid.z))
        })
        .unwrap_or(IVec2::ZERO);

    // Limit chunks processed per frame to avoid frame spikes
    const MAX_DIRTY_PER_FRAME: usize = 4;

    // Export UV cache for textured mesh generation
    let uv_cache = texture_registry.export_uv_cache();

    let all_dirty = dirty_chunks.take_all();
    let mut processed_count = 0;

    for coord in all_dirty.into_iter() {
        // Skip if chunk doesn't exist (unloaded)
        if !world_data.chunks.contains_key(&coord) {
            continue;
        }

        // Calculate LOD for this chunk
        let lod = calculate_lod(coord, player_chunk);

        // Regenerate mesh with LOD and textures
        if let Some(new_mesh) = world_data.generate_chunk_mesh_textured(coord, lod, &uv_cache) {
            // Remove old mesh entity
            if let Some(old_entities) = world_data.chunk_entities.remove(&coord) {
                for entity in old_entities {
                    commands.entity(entity).try_despawn_recursive();
                }
            }

            let mesh_handle = meshes.add(new_mesh);
            let material =
                get_chunk_material(&mut material_cache, &mut materials, &texture_registry);

            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material),
                    Transform::IDENTITY,
                    ChunkMesh { coord, lod },
                ))
                .id();

            world_data.chunk_entities.insert(coord, vec![entity]);
            tracing::trace!(
                "Dirty chunk {:?} mesh regenerated with LOD {:?}",
                coord,
                lod
            );
        }

        processed_count += 1;
        if processed_count >= MAX_DIRTY_PER_FRAME {
            // Re-add remaining dirty chunks for next frame
            // (Not needed since we took all - remaining are processed next frame)
            break;
        }
    }

    if processed_count > 0 {
        tracing::debug!("Processed {} dirty chunks this frame", processed_count);
    }
}
