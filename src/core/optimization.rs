// src/core/optimization.rs
//! 最適化システム
//! - 非同期チャンク生成
//! - LOD（Level of Detail）
//! - チャンクアンロード
//! - プレイヤー周囲の無限ワールド生成

use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use std::collections::{HashMap, HashSet};
use noise::{NoiseFn, Perlin};
use crate::ui::main_menu::AppState;
use crate::rendering::chunk::Chunk;
use crate::rendering::meshing::MeshDirty;

/// 最適化プラグイン
pub struct OptimizationPlugin;

impl Plugin for OptimizationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkLoadQueue>()
            .init_resource::<LodSettings>()
            .init_resource::<WorldChunkManager>()
            .add_systems(Update, (
                update_chunks_around_player,
                process_chunk_tasks,
                spawn_generated_chunks,
                update_chunk_lod,
                unload_distant_chunks,
            ).chain().run_if(in_state(AppState::InGame)));
    }
}

/// チャンク読み込みキュー
#[derive(Resource, Default)]
pub struct ChunkLoadQueue {
    pub pending: Vec<IVec3>,
    pub tasks: HashMap<IVec3, Task<ChunkData>>,
}

/// 生成されたチャンクデータ
pub struct ChunkData {
    pub position: IVec3,
    pub blocks: Vec<String>,
}

/// LOD設定
#[derive(Resource)]
pub struct LodSettings {
    /// LOD0（フル詳細）の距離
    pub lod0_distance: f32,
    /// LOD1（中程度）の距離
    pub lod1_distance: f32,
    /// LOD2（低詳細）の距離
    pub lod2_distance: f32,
    /// アンロード距離
    pub unload_distance: f32,
}

impl Default for LodSettings {
    fn default() -> Self {
        Self {
            lod0_distance: 64.0,
            lod1_distance: 128.0,
            lod2_distance: 256.0,
            unload_distance: 512.0,
        }
    }
}

/// ワールドチャンク管理
#[derive(Resource)]
pub struct WorldChunkManager {
    /// 生成済みチャンクの座標セット
    pub loaded_chunks: HashSet<IVec3>,
    /// チャンク生成半径（チャンク単位）
    pub render_distance: i32,
    /// ノイズシード
    pub seed: u32,
    /// 最後にチェックしたプレイヤーチャンク座標
    pub last_player_chunk: Option<IVec3>,
}

impl Default for WorldChunkManager {
    fn default() -> Self {
        Self {
            loaded_chunks: HashSet::new(),
            render_distance: 4, // 4チャンク = 128ブロック
            seed: 12345,
            last_player_chunk: None,
        }
    }
}

/// チャンクエンティティマーカー（座標追跡用）
#[derive(Component)]
pub struct ChunkCoord(pub IVec3);

/// チャンクのLODレベル
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ChunkLod {
    #[default]
    Full,    // LOD0: フル詳細
    Medium,  // LOD1: 中程度
    Low,     // LOD2: 低詳細
    Icon,    // LOD3: アイコン表示のみ
}

/// 非同期チャンク生成をキューに追加
pub fn queue_chunk_generation(queue: &mut ChunkLoadQueue, position: IVec3) {
    if !queue.pending.contains(&position) && !queue.tasks.contains_key(&position) {
        queue.pending.push(position);
    }
}

/// 非同期チャンク生成タスクを開始
fn start_chunk_task(position: IVec3, seed: u32) -> Task<ChunkData> {
    let thread_pool = AsyncComputeTaskPool::get();

    thread_pool.spawn(async move {
        // チャンク生成ロジック（テレイン生成など）
        let blocks = generate_chunk_blocks(position, seed);

        ChunkData { position, blocks }
    })
}

/// チャンクブロックを生成（Perlinノイズによるテレイン）
fn generate_chunk_blocks(chunk_pos: IVec3, seed: u32) -> Vec<String> {
    const CHUNK_SIZE: usize = 32;
    let size = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
    let mut blocks = vec!["air".to_string(); size];

    // Perlinノイズ生成器
    let perlin = Perlin::new(seed);

    // ワールド座標オフセット
    let world_x_offset = chunk_pos.x * CHUNK_SIZE as i32;
    let world_y_offset = chunk_pos.y * CHUNK_SIZE as i32;
    let world_z_offset = chunk_pos.z * CHUNK_SIZE as i32;

    for local_y in 0..CHUNK_SIZE {
        for local_z in 0..CHUNK_SIZE {
            for local_x in 0..CHUNK_SIZE {
                let world_x = world_x_offset + local_x as i32;
                let world_y = world_y_offset + local_y as i32;
                let world_z = world_z_offset + local_z as i32;

                // ノイズベースの地形高さを計算
                let noise_scale = 0.02;
                let noise_val = perlin.get([
                    world_x as f64 * noise_scale,
                    world_z as f64 * noise_scale,
                ]);

                // 地形の高さ（-8〜+8ブロックの範囲で変動）
                let terrain_height = (noise_val * 8.0) as i32;

                // ブロック決定
                let block_id = if world_y < terrain_height - 4 {
                    "stone"
                } else if world_y < terrain_height {
                    "dirt"
                } else if world_y == terrain_height {
                    "dirt" // 草ブロックがあれば "grass"
                } else {
                    "air"
                };

                let idx = (local_y * CHUNK_SIZE * CHUNK_SIZE) + (local_z * CHUNK_SIZE) + local_x;
                blocks[idx] = block_id.to_string();
            }
        }
    }

    blocks
}

/// プレイヤー周囲のチャンクを更新
fn update_chunks_around_player(
    mut manager: ResMut<WorldChunkManager>,
    mut queue: ResMut<ChunkLoadQueue>,
    player_query: Query<&Transform, With<crate::gameplay::player::Player>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    // プレイヤーのチャンク座標を計算
    let player_pos = player_transform.translation;
    let chunk_x = (player_pos.x / 32.0).floor() as i32;
    let chunk_z = (player_pos.z / 32.0).floor() as i32;
    let player_chunk = IVec3::new(chunk_x, 0, chunk_z);

    // 前回と同じチャンクなら何もしない（負荷軽減）
    if manager.last_player_chunk == Some(player_chunk) {
        return;
    }
    manager.last_player_chunk = Some(player_chunk);

    // プレイヤー周囲のチャンクを生成キューに追加
    let render_distance = manager.render_distance;
    for dx in -render_distance..=render_distance {
        for dz in -render_distance..=render_distance {
            // Y方向は-1〜1の3層（地下、地表、空）
            for dy in -1..=1 {
                let chunk_coord = IVec3::new(chunk_x + dx, dy, chunk_z + dz);

                // 既に生成済みなら追加
                if manager.loaded_chunks.contains(&chunk_coord) {
                    continue;
                }

                // 生成キューに追加
                if !queue.pending.contains(&chunk_coord) && !queue.tasks.contains_key(&chunk_coord) {
                    queue.pending.push(chunk_coord);
                    manager.loaded_chunks.insert(chunk_coord);
                }
            }
        }
    }
}

/// 非同期タスクの完了を処理
fn process_chunk_tasks(
    mut queue: ResMut<ChunkLoadQueue>,
    manager: Res<WorldChunkManager>,
) {
    // 1フレームあたりの最大タスク開始数（パフォーマンス制限）
    const MAX_TASKS_PER_FRAME: usize = 4;

    // ペンディングキューからタスクを開始
    let drain_count = queue.pending.len().min(MAX_TASKS_PER_FRAME);
    let pending: Vec<IVec3> = queue.pending.drain(..drain_count).collect();
    for position in pending {
        let task = start_chunk_task(position, manager.seed);
        queue.tasks.insert(position, task);
    }
}

/// 生成されたチャンクをスポーン
fn spawn_generated_chunks(
    mut queue: ResMut<ChunkLoadQueue>,
    mut commands: Commands,
) {
    // 完了したタスクを処理してスポーン
    // タスクを順番にチェックし、完了していれば削除してスポーン
    let positions: Vec<IVec3> = queue.tasks.keys().cloned().collect();

    // 1フレームあたりの最大スポーン数
    const MAX_SPAWNS_PER_FRAME: usize = 4;
    let mut spawned_count = 0;

    for position in positions {
        if spawned_count >= MAX_SPAWNS_PER_FRAME {
            break;
        }

        // タスクを取り出してポーリング
        if let Some(mut task) = queue.tasks.remove(&position) {
            match block_on(poll_once(&mut task)) {
                Some(chunk_data) => {
                    // タスク完了 - チャンクをスポーン
                    let mut chunk = Chunk::new(chunk_data.position);
                    chunk.blocks = chunk_data.blocks;

                    commands.spawn((
                        chunk,
                        MeshDirty,
                        Transform::from_translation(Vec3::new(
                            (chunk_data.position.x * 32) as f32,
                            (chunk_data.position.y * 32) as f32,
                            (chunk_data.position.z * 32) as f32,
                        )),
                        Visibility::default(),
                        ChunkLod::Full,
                        ChunkCoord(chunk_data.position),
                    ));

                    info!("Chunk spawned at {:?}", position);
                    spawned_count += 1;
                }
                None => {
                    // まだ完了していない - タスクを戻す
                    queue.tasks.insert(position, task);
                }
            }
        }
    }
}


/// LODを更新
fn update_chunk_lod(
    mut chunks: Query<(&Transform, &mut ChunkLod)>,
    camera: Query<&Transform, With<Camera3d>>,
    settings: Res<LodSettings>,
) {
    let Ok(camera_transform) = camera.get_single() else {
        return;
    };

    let camera_pos = camera_transform.translation;

    for (chunk_transform, mut lod) in &mut chunks {
        let distance = camera_pos.distance(chunk_transform.translation);

        let new_lod = if distance < settings.lod0_distance {
            ChunkLod::Full
        } else if distance < settings.lod1_distance {
            ChunkLod::Medium
        } else if distance < settings.lod2_distance {
            ChunkLod::Low
        } else {
            ChunkLod::Icon
        };

        if *lod != new_lod {
            *lod = new_lod;
        }
    }
}

/// 遠いチャンクをアンロード
fn unload_distant_chunks(
    mut commands: Commands,
    mut manager: ResMut<WorldChunkManager>,
    chunks: Query<(Entity, &Transform, Option<&ChunkCoord>), With<ChunkLod>>,
    camera: Query<&Transform, With<Camera3d>>,
    settings: Res<LodSettings>,
) {
    let Ok(camera_transform) = camera.get_single() else {
        return;
    };

    let camera_pos = camera_transform.translation;

    for (entity, chunk_transform, chunk_coord) in &chunks {
        let distance = camera_pos.distance(chunk_transform.translation);

        if distance > settings.unload_distance {
            // loaded_chunksから削除
            if let Some(coord) = chunk_coord {
                manager.loaded_chunks.remove(&coord.0);
            }
            commands.entity(entity).despawn_recursive();
            info!("Unloaded distant chunk at {:?}", chunk_transform.translation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_chunk_blocks_with_noise() {
        let seed = 12345;

        // 地下チャンク（Y=-1）は主にstone
        let blocks = generate_chunk_blocks(IVec3::new(0, -1, 0), seed);
        let stone_count = blocks.iter().filter(|b| *b == "stone").count();
        // 地下チャンクはほとんどstoneのはず
        assert!(stone_count > 32 * 32 * 16, "Underground should be mostly stone");

        // 地表チャンク（Y=0）はdirtとairの混合
        let blocks = generate_chunk_blocks(IVec3::new(0, 0, 0), seed);
        let dirt_count = blocks.iter().filter(|b| *b == "dirt").count();
        let air_count = blocks.iter().filter(|b| *b == "air").count();
        // dirtとairが両方存在するはず
        assert!(dirt_count > 0, "Surface chunk should have dirt");
        assert!(air_count > 0, "Surface chunk should have air");

        // 空中チャンク（Y=1）は全てair
        let blocks = generate_chunk_blocks(IVec3::new(0, 1, 0), seed);
        assert!(blocks.iter().all(|b| b == "air"), "Sky chunk should be all air");
    }

    #[test]
    fn test_noise_consistency() {
        let seed = 12345;
        // 同じシードで同じチャンクを生成すると同じ結果になる
        let blocks1 = generate_chunk_blocks(IVec3::new(5, 0, 5), seed);
        let blocks2 = generate_chunk_blocks(IVec3::new(5, 0, 5), seed);
        assert_eq!(blocks1, blocks2, "Same seed should produce same terrain");
    }

    #[test]
    fn test_lod_settings() {
        let settings = LodSettings::default();
        assert!(settings.lod0_distance < settings.lod1_distance);
        assert!(settings.lod1_distance < settings.lod2_distance);
        assert!(settings.lod2_distance < settings.unload_distance);
    }

    #[test]
    fn test_chunk_lod() {
        let lod = ChunkLod::default();
        assert_eq!(lod, ChunkLod::Full);
    }

    #[test]
    fn test_world_chunk_manager_default() {
        let manager = WorldChunkManager::default();
        assert!(manager.loaded_chunks.is_empty());
        assert_eq!(manager.render_distance, 4);
        assert!(manager.last_player_chunk.is_none());
    }
}
