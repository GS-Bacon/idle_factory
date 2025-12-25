// src/core/optimization.rs
//! 最適化システム
//! - 非同期チャンク生成
//! - LOD（Level of Detail）
//! - チャンクアンロード

use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use std::collections::HashMap;

use super::worldgen::{create_generator, WorldGenConfig};

/// 最適化プラグイン
pub struct OptimizationPlugin;

impl Plugin for OptimizationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkLoadQueue>()
            .init_resource::<LodSettings>()
            .add_systems(Update, (
                process_chunk_tasks,
                update_chunk_lod,
                unload_distant_chunks,
            ));
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
fn start_chunk_task(position: IVec3, config: WorldGenConfig) -> Task<ChunkData> {
    let thread_pool = AsyncComputeTaskPool::get();

    thread_pool.spawn(async move {
        // 新しいワールド生成システムを使用
        let generator = create_generator(config.world_type);
        let blocks = generator.generate(position, &config);

        ChunkData { position, blocks }
    })
}

/// 非同期タスクの完了を処理
fn process_chunk_tasks(
    mut queue: ResMut<ChunkLoadQueue>,
    mut commands: Commands,
    config: Res<WorldGenConfig>,
) {
    // ペンディングキューからタスクを開始
    let pending: Vec<IVec3> = queue.pending.drain(..).collect();
    for position in pending {
        let task = start_chunk_task(position, config.clone());
        queue.tasks.insert(position, task);
    }

    // 完了したタスクを処理
    let mut completed = Vec::new();
    for (position, task) in queue.tasks.iter_mut() {
        if let Some(chunk_data) = block_on(poll_once(task)) {
            completed.push((*position, chunk_data));
        }
    }

    for (position, chunk_data) in completed {
        queue.tasks.remove(&position);

        // チャンクエンティティを生成
        commands.spawn((
            Transform::from_translation(Vec3::new(
                (chunk_data.position.x * 32) as f32,
                (chunk_data.position.y * 32) as f32,
                (chunk_data.position.z * 32) as f32,
            )),
            ChunkLod::Full,
            AsyncChunkMarker { blocks: chunk_data.blocks },
        ));

        info!("Async chunk generated at {:?}", position);
    }
}

/// 非同期生成されたチャンクのマーカー
#[derive(Component)]
pub struct AsyncChunkMarker {
    pub blocks: Vec<String>,
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
    chunks: Query<(Entity, &Transform), With<ChunkLod>>,
    camera: Query<&Transform, With<Camera3d>>,
    settings: Res<LodSettings>,
) {
    let Ok(camera_transform) = camera.get_single() else {
        return;
    };

    let camera_pos = camera_transform.translation;

    for (entity, chunk_transform) in &chunks {
        let distance = camera_pos.distance(chunk_transform.translation);

        if distance > settings.unload_distance {
            commands.entity(entity).despawn_recursive();
            info!("Unloaded distant chunk at {:?}", chunk_transform.translation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::worldgen::WorldType;
    use crate::rendering::chunk::CHUNK_SIZE;

    #[test]
    fn test_worldgen_integration() {
        // ワールド生成システムとの統合テスト
        let config = WorldGenConfig {
            seed: 12345,
            world_type: WorldType::Normal,
            ..Default::default()
        };

        let generator = create_generator(config.world_type);
        let blocks = generator.generate(IVec3::new(0, 2, 0), &config);

        // 正しいサイズのブロック配列が生成される
        assert_eq!(blocks.len(), CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
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
}
