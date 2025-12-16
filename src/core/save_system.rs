// src/core/save_system.rs
//! セーブデータ管理システム
//! - SaveMetadata: ワールド名、シード値、プレイ時間、最終プレイ日時
//! - SaveSlotData: 全スロットのメタデータを保持
//! - JSON形式での永続化

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::PathBuf;

/// セーブシステムプラグイン
pub struct SaveSystemPlugin;

impl Plugin for SaveSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveSlotData>()
            .init_resource::<WorldGenerationParams>()
            .add_systems(Startup, load_save_slots);
    }
}

/// セーブメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    /// ファイル名（スロット番号）
    pub file_name: String,
    /// ワールド名
    pub world_name: String,
    /// シード値
    pub seed: u64,
    /// プレイ時間（秒）
    pub play_time: f64,
    /// 最終プレイ日時
    pub last_played_date: DateTime<Utc>,
}

impl SaveMetadata {
    pub fn new(slot_index: usize, world_name: &str, seed: u64) -> Self {
        Self {
            file_name: format!("slot_{}", slot_index),
            world_name: world_name.to_string(),
            seed,
            play_time: 0.0,
            last_played_date: Utc::now(),
        }
    }

    /// 最終プレイ日時のフォーマット済み文字列
    pub fn formatted_date(&self) -> String {
        self.last_played_date.format("%Y-%m-%d %H:%M").to_string()
    }

    /// プレイ時間のフォーマット済み文字列
    pub fn formatted_play_time(&self) -> String {
        let hours = self.play_time / 3600.0;
        if hours >= 1.0 {
            format!("{:.1}h", hours)
        } else {
            let minutes = self.play_time / 60.0;
            format!("{:.0}m", minutes)
        }
    }
}

/// セーブスロットデータ（全スロット）
#[derive(Resource, Default)]
pub struct SaveSlotData {
    /// スロット（最大8個、Noneは空）
    pub slots: [Option<SaveMetadata>; 8],
}

impl SaveSlotData {
    /// スロットを取得
    pub fn get(&self, index: usize) -> Option<&SaveMetadata> {
        self.slots.get(index).and_then(|s| s.as_ref())
    }

    /// スロットを設定
    pub fn set(&mut self, index: usize, meta: SaveMetadata) {
        if index < self.slots.len() {
            self.slots[index] = Some(meta);
        }
    }

    /// スロットを削除
    pub fn clear(&mut self, index: usize) {
        if index < self.slots.len() {
            self.slots[index] = None;
        }
    }

    /// 使用中のスロット数
    pub fn used_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }
}

/// ワールド生成パラメータ（シーン間でデータを渡す）
#[derive(Resource, Default)]
pub struct WorldGenerationParams {
    pub world_name: String,
    pub seed: u64,
    pub slot_index: Option<usize>,
    pub is_new_world: bool,
}

/// セーブディレクトリのパス
fn save_dir() -> PathBuf {
    PathBuf::from("saves")
}

/// メタデータファイルのパス
fn metadata_path(slot_index: usize) -> PathBuf {
    save_dir().join(format!("slot_{}", slot_index)).join("metadata.json")
}

/// 起動時にセーブスロットを読み込む
fn load_save_slots(mut slot_data: ResMut<SaveSlotData>) {
    let dir = save_dir();

    // セーブディレクトリを作成
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            warn!("Failed to create saves directory: {}", e);
            return;
        }
    }

    // 各スロットを走査
    for i in 0..8 {
        let path = metadata_path(i);
        if path.exists() {
            match load_metadata(&path) {
                Ok(meta) => {
                    info!("Loaded save slot {}: {}", i, meta.world_name);
                    slot_data.slots[i] = Some(meta);
                }
                Err(e) => {
                    warn!("Failed to load slot {}: {}", i, e);
                }
            }
        }
    }

    info!("Loaded {} save slots", slot_data.used_count());
}

/// メタデータを読み込む
fn load_metadata(path: &PathBuf) -> Result<SaveMetadata, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Read error: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Parse error: {}", e))
}

/// メタデータを保存
pub fn save_metadata(meta: &SaveMetadata, slot_index: usize) -> Result<(), String> {
    let dir = save_dir().join(format!("slot_{}", slot_index));

    // ディレクトリを作成
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let path = dir.join("metadata.json");
    let content = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("Serialize error: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Write error: {}", e))?;

    info!("Saved metadata to {:?}", path);
    Ok(())
}

/// セーブデータを削除
pub fn delete_save(slot_index: usize) -> Result<(), String> {
    let dir = save_dir().join(format!("slot_{}", slot_index));

    if dir.exists() {
        fs::remove_dir_all(&dir)
            .map_err(|e| format!("Failed to delete: {}", e))?;
        info!("Deleted save slot {}", slot_index);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_metadata_new() {
        let meta = SaveMetadata::new(0, "Test World", 12345);
        assert_eq!(meta.file_name, "slot_0");
        assert_eq!(meta.world_name, "Test World");
        assert_eq!(meta.seed, 12345);
        assert_eq!(meta.play_time, 0.0);
    }

    #[test]
    fn test_save_metadata_formatted_time() {
        let mut meta = SaveMetadata::new(0, "Test", 0);

        meta.play_time = 30.0;
        assert!(meta.formatted_play_time().contains("m"));

        meta.play_time = 7200.0;
        assert!(meta.formatted_play_time().contains("h"));
    }

    #[test]
    fn test_save_slot_data() {
        let mut data = SaveSlotData::default();
        assert_eq!(data.used_count(), 0);

        data.set(0, SaveMetadata::new(0, "World 1", 100));
        assert_eq!(data.used_count(), 1);
        assert!(data.get(0).is_some());

        data.clear(0);
        assert_eq!(data.used_count(), 0);
    }

    #[test]
    fn test_metadata_serialization() {
        let meta = SaveMetadata::new(0, "Test World", 12345);
        let json = serde_json::to_string(&meta).unwrap();
        let restored: SaveMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(meta.world_name, restored.world_name);
        assert_eq!(meta.seed, restored.seed);
    }
}
