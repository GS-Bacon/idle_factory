//! Sound system foundation

use bevy::prelude::*;

/// サウンドカテゴリ
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundCategory {
    Master,
    Bgm,
    Sfx,
    Ambient,
}

/// サウンド仕様定義
#[derive(Debug, Clone)]
pub struct SoundSpec {
    pub id: &'static str,
    pub path: &'static str,
    pub category: SoundCategory,
    pub default_volume: f32,
    pub loop_: bool,
}

/// サウンドエミッターコンポーネント（3D空間サウンド用）
#[derive(Component, Debug, Clone)]
pub struct SoundEmitter {
    pub sound_id: &'static str,
    pub volume: f32,
    pub loop_: bool,
    pub playing: bool,
}

impl Default for SoundEmitter {
    fn default() -> Self {
        Self {
            sound_id: "",
            volume: 1.0,
            loop_: false,
            playing: false,
        }
    }
}

/// サウンドリソース（ロード済みサウンド管理）
/// Note: 将来 bevy_audio feature を有効にした際に Handle<AudioSource> を使用する
#[derive(Resource, Default)]
pub struct SoundAssets {
    /// 登録済みサウンドのパスマップ (id -> path)
    /// 実際のアセットハンドルは bevy_audio 有効化後に追加
    pub sound_paths: std::collections::HashMap<String, String>,
}

/// サウンド設定リソース
#[derive(Resource, Debug, Clone)]
pub struct SoundSettings {
    pub master_volume: f32,
    pub bgm_volume: f32,
    pub sfx_volume: f32,
    pub ambient_volume: f32,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            bgm_volume: 0.7,
            sfx_volume: 1.0,
            ambient_volume: 0.5,
        }
    }
}

impl SoundSettings {
    /// カテゴリ別の実効音量を取得
    pub fn effective_volume(&self, category: SoundCategory) -> f32 {
        let category_volume = match category {
            SoundCategory::Master => 1.0,
            SoundCategory::Bgm => self.bgm_volume,
            SoundCategory::Sfx => self.sfx_volume,
            SoundCategory::Ambient => self.ambient_volume,
        };
        self.master_volume * category_volume
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundAssets>()
            .init_resource::<SoundSettings>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_settings_default() {
        let settings = SoundSettings::default();
        assert_eq!(settings.master_volume, 1.0);
        assert_eq!(settings.bgm_volume, 0.7);
    }

    #[test]
    fn test_effective_volume() {
        let settings = SoundSettings {
            master_volume: 0.5,
            bgm_volume: 0.8,
            sfx_volume: 1.0,
            ambient_volume: 0.6,
        };
        assert!((settings.effective_volume(SoundCategory::Bgm) - 0.4).abs() < 0.001);
        assert!((settings.effective_volume(SoundCategory::Sfx) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_sound_emitter_default() {
        let emitter = SoundEmitter::default();
        assert!(!emitter.playing);
        assert_eq!(emitter.volume, 1.0);
    }

    #[test]
    fn test_sound_category_debug() {
        assert_eq!(format!("{:?}", SoundCategory::Bgm), "Bgm");
    }
}
