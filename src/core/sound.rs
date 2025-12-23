// src/core/sound.rs
//! サウンドシステム (S1-S4)
//!
//! ## S1. ミキシング階層
//! - Master > Music/SFX/Voice のカテゴリ別ボリューム
//!
//! ## S2. 反復疲労防止
//! - 頻出音に3+バリエーション
//! - ピッチ±10%ランダム化
//!
//! ## S3. 空間オーディオ
//! - 距離減衰
//! - 同時再生数制限
//!
//! ## S4. フィードバック
//! - 全プレイヤーアクションに音声確認

use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// サウンドシステムプラグイン
pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundSettings>()
            .init_resource::<SoundRegistry>()
            .init_resource::<ActiveSounds>()
            .add_event::<PlaySoundEvent>()
            .add_event::<StopSoundEvent>()
            .add_systems(
                Update,
                (
                    process_sound_events,
                    update_spatial_audio,
                    cleanup_finished_sounds,
                ),
            );
    }
}

/// サウンドカテゴリ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SoundCategory {
    /// マスターボリューム
    Master,
    /// 音楽
    Music,
    /// 効果音
    #[default]
    Sfx,
    /// ボイス/アナウンス
    Voice,
    /// アンビエント（環境音）
    Ambient,
    /// UI音
    Ui,
}

/// サウンド設定
#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct SoundSettings {
    /// カテゴリ別ボリューム（0.0 - 1.0）
    pub volumes: HashMap<SoundCategory, f32>,
    /// 最大同時再生数
    pub max_simultaneous_sounds: usize,
    /// 距離減衰の最大距離
    pub max_audio_distance: f32,
    /// ピッチランダム化の範囲（±%）
    pub pitch_variation: f32,
}

impl Default for SoundSettings {
    fn default() -> Self {
        let mut volumes = HashMap::new();
        volumes.insert(SoundCategory::Master, 1.0);
        volumes.insert(SoundCategory::Music, 0.7);
        volumes.insert(SoundCategory::Sfx, 1.0);
        volumes.insert(SoundCategory::Voice, 1.0);
        volumes.insert(SoundCategory::Ambient, 0.6);
        volumes.insert(SoundCategory::Ui, 0.8);

        Self {
            volumes,
            max_simultaneous_sounds: 32,
            max_audio_distance: 50.0,
            pitch_variation: 0.1, // ±10%
        }
    }
}

impl SoundSettings {
    /// カテゴリのボリュームを取得（マスター適用済み）
    pub fn get_volume(&self, category: SoundCategory) -> f32 {
        let master = self
            .volumes
            .get(&SoundCategory::Master)
            .copied()
            .unwrap_or(1.0);
        let category_vol = self.volumes.get(&category).copied().unwrap_or(1.0);
        master * category_vol
    }

    /// カテゴリのボリュームを設定
    pub fn set_volume(&mut self, category: SoundCategory, volume: f32) {
        self.volumes.insert(category, volume.clamp(0.0, 1.0));
    }
}

/// サウンド定義
#[derive(Clone)]
pub struct SoundDefinition {
    /// サウンドID
    pub id: String,
    /// カテゴリ
    pub category: SoundCategory,
    /// バリエーション（複数のオーディオソース）
    pub variants: Vec<Handle<AudioSource>>,
    /// ベースボリューム
    pub base_volume: f32,
    /// ピッチバリエーション有効
    pub pitch_variation_enabled: bool,
    /// 空間オーディオ有効
    pub spatial: bool,
    /// ループ再生
    pub looping: bool,
    /// 優先度（高いほど優先）
    pub priority: u8,
}

impl Default for SoundDefinition {
    fn default() -> Self {
        Self {
            id: String::new(),
            category: SoundCategory::Sfx,
            variants: Vec::new(),
            base_volume: 1.0,
            pitch_variation_enabled: true,
            spatial: true,
            looping: false,
            priority: 50,
        }
    }
}

/// サウンドレジストリ
#[derive(Resource, Default)]
pub struct SoundRegistry {
    /// 登録されたサウンド定義
    pub sounds: HashMap<String, SoundDefinition>,
}

impl SoundRegistry {
    /// サウンドを登録
    pub fn register(&mut self, definition: SoundDefinition) {
        self.sounds.insert(definition.id.clone(), definition);
    }

    /// サウンドを取得
    pub fn get(&self, id: &str) -> Option<&SoundDefinition> {
        self.sounds.get(id)
    }
}

/// アクティブなサウンドインスタンス
#[derive(Component)]
pub struct ActiveSound {
    /// サウンドID
    pub sound_id: String,
    /// カテゴリ
    pub category: SoundCategory,
    /// 開始時刻
    pub started_at: f64,
    /// 再生位置（空間オーディオ用）
    pub position: Option<Vec3>,
    /// 現在のボリューム
    pub volume: f32,
    /// 現在のピッチ
    pub pitch: f32,
}

/// アクティブなサウンドの追跡
#[derive(Resource, Default)]
pub struct ActiveSounds {
    /// アクティブなサウンド数（カテゴリ別）
    pub count_by_category: HashMap<SoundCategory, usize>,
    /// 総アクティブ数
    pub total_count: usize,
}

/// サウンド再生イベント
#[derive(Event)]
pub struct PlaySoundEvent {
    /// サウンドID
    pub sound_id: String,
    /// 再生位置（空間オーディオ用、Noneなら2D）
    pub position: Option<Vec3>,
    /// ボリュームオーバーライド（Noneなら定義のbase_volume）
    pub volume: Option<f32>,
    /// ピッチオーバーライド（Noneなら自動）
    pub pitch: Option<f32>,
}

impl PlaySoundEvent {
    /// 2Dサウンド再生
    pub fn play_2d(sound_id: impl Into<String>) -> Self {
        Self {
            sound_id: sound_id.into(),
            position: None,
            volume: None,
            pitch: None,
        }
    }

    /// 3D空間サウンド再生
    pub fn play_3d(sound_id: impl Into<String>, position: Vec3) -> Self {
        Self {
            sound_id: sound_id.into(),
            position: Some(position),
            volume: None,
            pitch: None,
        }
    }
}

/// サウンド停止イベント
#[derive(Event)]
pub struct StopSoundEvent {
    /// 停止するサウンドID（Noneなら全て）
    pub sound_id: Option<String>,
    /// フェードアウト時間（秒）
    pub fade_out: f32,
}

/// サウンドイベントを処理
fn process_sound_events(
    mut commands: Commands,
    mut play_events: EventReader<PlaySoundEvent>,
    settings: Res<SoundSettings>,
    registry: Res<SoundRegistry>,
    mut active_sounds: ResMut<ActiveSounds>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();

    for event in play_events.read() {
        // サウンド定義を取得
        let Some(definition) = registry.get(&event.sound_id) else {
            warn!("Sound not found: {}", event.sound_id);
            continue;
        };

        // 最大同時再生数チェック
        if active_sounds.total_count >= settings.max_simultaneous_sounds {
            // 優先度の低いサウンドをスキップ
            if definition.priority < 80 {
                continue;
            }
        }

        // バリエーションからランダムに選択
        if definition.variants.is_empty() {
            warn!("Sound has no variants: {}", event.sound_id);
            continue;
        }
        let _variant_idx = rng.gen_range(0..definition.variants.len());

        // ピッチ計算
        let pitch = event.pitch.unwrap_or_else(|| {
            if definition.pitch_variation_enabled {
                let variation = settings.pitch_variation;
                1.0 + rng.gen_range(-variation..variation)
            } else {
                1.0
            }
        });

        // ボリューム計算
        let category_volume = settings.get_volume(definition.category);
        let base_volume = event.volume.unwrap_or(definition.base_volume);
        let volume = base_volume * category_volume;

        // サウンドエンティティを生成
        commands.spawn(ActiveSound {
            sound_id: event.sound_id.clone(),
            category: definition.category,
            started_at: time.elapsed_secs_f64(),
            position: event.position,
            volume,
            pitch,
        });

        // カウント更新
        *active_sounds
            .count_by_category
            .entry(definition.category)
            .or_insert(0) += 1;
        active_sounds.total_count += 1;

        info!(
            "Playing sound: {} (vol={:.2}, pitch={:.2})",
            event.sound_id, volume, pitch
        );
    }
}

/// 空間オーディオの更新
fn update_spatial_audio(
    mut sounds: Query<(&mut ActiveSound, &Transform), With<ActiveSound>>,
    listener: Query<&Transform, With<Camera3d>>,
    settings: Res<SoundSettings>,
) {
    let Ok(listener_transform) = listener.get_single() else {
        return;
    };

    for (mut sound, sound_transform) in sounds.iter_mut() {
        if let Some(position) = sound.position {
            // リスナーとの距離を計算
            let distance = listener_transform.translation.distance(position);

            // 距離減衰を適用
            let attenuation = if distance >= settings.max_audio_distance {
                0.0
            } else {
                1.0 - (distance / settings.max_audio_distance)
            };

            sound.volume *= attenuation;

            // Transformを更新
            let _ = sound_transform; // 位置は既に設定済み
        }
    }
}

/// 終了したサウンドをクリーンアップ
fn cleanup_finished_sounds(
    mut commands: Commands,
    sounds: Query<(Entity, &ActiveSound)>,
    mut active_sounds: ResMut<ActiveSounds>,
    time: Res<Time>,
) {
    // 仮の実装：3秒後に自動削除（実際はオーディオ終了を検出）
    let current_time = time.elapsed_secs_f64();

    for (entity, sound) in sounds.iter() {
        if current_time - sound.started_at > 3.0 {
            commands.entity(entity).despawn();

            // カウント更新
            if let Some(count) = active_sounds.count_by_category.get_mut(&sound.category) {
                *count = count.saturating_sub(1);
            }
            active_sounds.total_count = active_sounds.total_count.saturating_sub(1);
        }
    }
}

/// 工場ゲーム用の定義済みサウンド
pub mod factory_sounds {
    /// 機械関連
    pub const MACHINE_PLACE: &str = "machine_place";
    pub const MACHINE_REMOVE: &str = "machine_remove";
    pub const MACHINE_START: &str = "machine_start";
    pub const MACHINE_STOP: &str = "machine_stop";
    pub const MACHINE_RUNNING: &str = "machine_running";
    pub const MACHINE_ERROR: &str = "machine_error";

    /// コンベア関連
    pub const CONVEYOR_LOOP: &str = "conveyor_loop";
    pub const ITEM_DROP: &str = "item_drop";
    pub const ITEM_PICKUP: &str = "item_pickup";

    /// 電力関連
    pub const POWER_ON: &str = "power_on";
    pub const POWER_OFF: &str = "power_off";
    pub const OVERSTRESS_WARNING: &str = "overstress_warning";

    /// UI関連
    pub const UI_CLICK: &str = "ui_click";
    pub const UI_HOVER: &str = "ui_hover";
    pub const UI_OPEN: &str = "ui_open";
    pub const UI_CLOSE: &str = "ui_close";
    pub const UI_ERROR: &str = "ui_error";
    pub const UI_SUCCESS: &str = "ui_success";

    /// 達成関連
    pub const ACHIEVEMENT_UNLOCK: &str = "achievement_unlock";
    pub const QUEST_COMPLETE: &str = "quest_complete";
    pub const MILESTONE_REACHED: &str = "milestone_reached";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_settings_volume() {
        let settings = SoundSettings::default();

        // マスターは1.0、Musicは0.7なので、Music全体は0.7
        assert!((settings.get_volume(SoundCategory::Music) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_sound_settings_set_volume() {
        let mut settings = SoundSettings::default();
        settings.set_volume(SoundCategory::Master, 0.5);

        // マスターが0.5になったので、他のカテゴリも半減
        assert!((settings.get_volume(SoundCategory::Sfx) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_play_sound_event() {
        let event_2d = PlaySoundEvent::play_2d("test");
        assert!(event_2d.position.is_none());

        let event_3d = PlaySoundEvent::play_3d("test", Vec3::new(1.0, 2.0, 3.0));
        assert!(event_3d.position.is_some());
    }

    #[test]
    fn test_sound_registry() {
        let mut registry = SoundRegistry::default();

        let definition = SoundDefinition {
            id: "test_sound".to_string(),
            category: SoundCategory::Sfx,
            base_volume: 0.8,
            ..Default::default()
        };

        registry.register(definition);

        let retrieved = registry.get("test_sound").unwrap();
        assert_eq!(retrieved.id, "test_sound");
        assert!((retrieved.base_volume - 0.8).abs() < 0.001);
    }
}
