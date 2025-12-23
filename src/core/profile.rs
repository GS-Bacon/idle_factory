// src/core/profile.rs
//! プロファイルシステム
//!
//! ゲームコンテンツ（アイテム、レシピ、クエスト）とリソースパック設定を
//! プロファイル単位で管理する。
//!
//! ## 構造
//! ```text
//! profiles/
//! └── vanilla/
//!     ├── profile.yaml          # プロファイル設定
//!     ├── data/
//!     │   ├── items.yaml        # アイテム定義
//!     │   ├── recipes.yaml      # レシピツリー
//!     │   └── quests.yaml       # クエスト
//!     └── assets/
//!         ├── icons/
//!         └── models/
//! ```

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::resource_pack::{ResourcePackChangedEvent, ResourcePackManager};

/// プロファイルプラグイン
pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProfileManager>()
            .init_resource::<ActiveProfile>()
            .add_event::<ProfileChangedEvent>()
            .add_event::<LoadProfileEvent>()
            .add_systems(Startup, scan_profiles)
            .add_systems(
                Update,
                (handle_load_profile_event, apply_profile_resource_packs),
            );
    }
}

/// プロファイル設定（profile.yaml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// プロファイル名
    pub name: String,
    /// バージョン
    #[serde(default = "default_version")]
    pub version: String,
    /// 説明
    #[serde(default)]
    pub description: String,
    /// 作者
    #[serde(default)]
    pub author: String,
    /// 依存MOD
    #[serde(default)]
    pub mods: Vec<ModDependency>,
    /// 有効なリソースパック（ID順、後のものが優先）
    #[serde(default)]
    pub resource_packs: Vec<ResourcePackEntry>,
    /// サーバーオリジン（マルチプレイ用）
    #[serde(default)]
    pub server_origin: Option<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// MOD依存関係
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    /// MOD ID
    pub id: String,
    /// バージョン要件（SemVer）
    #[serde(default)]
    pub version: String,
}

/// リソースパックエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackEntry {
    /// リソースパックID
    pub id: String,
    /// 優先度オーバーライド（設定しない場合はパックのデフォルト優先度）
    #[serde(default)]
    pub priority_override: Option<i32>,
}

/// プロファイル
#[derive(Debug, Clone)]
pub struct Profile {
    /// プロファイルID（ディレクトリ名）
    pub id: String,
    /// プロファイル設定
    pub config: ProfileConfig,
    /// プロファイルのルートパス
    pub path: PathBuf,
    /// データディレクトリパス
    pub data_path: PathBuf,
    /// アセットディレクトリパス
    pub assets_path: PathBuf,
}

impl Profile {
    /// アイテム定義ファイルのパスを取得
    pub fn items_path(&self) -> PathBuf {
        self.data_path.join("items.yaml")
    }

    /// レシピ定義ファイルのパスを取得
    pub fn recipes_path(&self) -> PathBuf {
        self.data_path.join("recipes.yaml")
    }

    /// クエスト定義ファイルのパスを取得
    pub fn quests_path(&self) -> PathBuf {
        self.data_path.join("quests.yaml")
    }

    /// 実績定義ファイルのパスを取得
    pub fn achievements_path(&self) -> PathBuf {
        self.data_path.join("achievements.yaml")
    }
}

/// プロファイルマネージャー
#[derive(Resource, Default)]
pub struct ProfileManager {
    /// 検出された全プロファイル
    pub available_profiles: Vec<Profile>,
    /// プロファイルのルートディレクトリ
    pub profiles_directory: PathBuf,
}

impl ProfileManager {
    /// プロファイルをスキャン
    pub fn scan_profiles(&mut self) {
        self.available_profiles.clear();

        let dir = if self.profiles_directory.as_os_str().is_empty() {
            PathBuf::from("profiles")
        } else {
            self.profiles_directory.clone()
        };

        if !dir.exists() {
            if let Err(e) = fs::create_dir_all(&dir) {
                warn!("Failed to create profiles directory: {}", e);
                return;
            }
            info!("Created profiles directory");
        }

        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read profiles directory: {}", e);
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(profile) = self.load_profile(&path) {
                    info!(
                        "Found profile: {} ({})",
                        profile.config.name, profile.id
                    );
                    self.available_profiles.push(profile);
                }
            }
        }

        info!("Scanned {} profiles", self.available_profiles.len());
    }

    /// プロファイルを読み込む
    fn load_profile(&self, path: &Path) -> Option<Profile> {
        let config_path = path.join("profile.yaml");
        if !config_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&config_path).ok()?;
        let config: ProfileConfig = serde_yaml::from_str(&content).ok()?;

        let id = path.file_name()?.to_string_lossy().to_string();
        let data_path = path.join("data");
        let assets_path = path.join("assets");

        Some(Profile {
            id,
            config,
            path: path.to_path_buf(),
            data_path,
            assets_path,
        })
    }

    /// プロファイルをIDで検索
    pub fn get_profile(&self, id: &str) -> Option<&Profile> {
        self.available_profiles.iter().find(|p| p.id == id)
    }

    /// デフォルトプロファイル（vanilla）を取得
    pub fn get_default_profile(&self) -> Option<&Profile> {
        self.get_profile("vanilla")
            .or_else(|| self.available_profiles.first())
    }
}

/// 現在アクティブなプロファイル
#[derive(Resource, Default)]
pub struct ActiveProfile {
    /// アクティブなプロファイルID
    pub id: Option<String>,
    /// プロファイル設定のコピー
    pub config: Option<ProfileConfig>,
    /// プロファイルパス
    pub path: Option<PathBuf>,
}

impl ActiveProfile {
    /// プロファイルがアクティブかどうか
    pub fn is_active(&self) -> bool {
        self.id.is_some()
    }

    /// アクティブなプロファイルIDを取得
    pub fn get_id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}

/// プロファイル変更イベント
#[derive(Event)]
pub struct ProfileChangedEvent {
    /// 新しいプロファイルID
    pub profile_id: String,
}

/// プロファイル読み込みイベント
#[derive(Event)]
pub struct LoadProfileEvent {
    /// 読み込むプロファイルID
    pub profile_id: String,
}

/// 起動時にプロファイルをスキャン
fn scan_profiles(mut manager: ResMut<ProfileManager>) {
    manager.profiles_directory = PathBuf::from("profiles");
    manager.scan_profiles();
}

/// プロファイル読み込みイベントを処理
fn handle_load_profile_event(
    mut events: EventReader<LoadProfileEvent>,
    manager: Res<ProfileManager>,
    mut active: ResMut<ActiveProfile>,
    mut changed_event: EventWriter<ProfileChangedEvent>,
) {
    for event in events.read() {
        if let Some(profile) = manager.get_profile(&event.profile_id) {
            active.id = Some(profile.id.clone());
            active.config = Some(profile.config.clone());
            active.path = Some(profile.path.clone());

            changed_event.send(ProfileChangedEvent {
                profile_id: profile.id.clone(),
            });

            info!("Loaded profile: {} ({})", profile.config.name, profile.id);
        } else {
            warn!("Profile not found: {}", event.profile_id);
        }
    }
}

/// プロファイル変更時にリソースパックを適用
fn apply_profile_resource_packs(
    mut events: EventReader<ProfileChangedEvent>,
    manager: Res<ProfileManager>,
    mut pack_manager: ResMut<ResourcePackManager>,
    mut pack_changed_event: EventWriter<ResourcePackChangedEvent>,
) {
    for event in events.read() {
        if let Some(profile) = manager.get_profile(&event.profile_id) {
            // 全パックを一旦無効化
            for pack in &mut pack_manager.available_packs {
                pack.enabled = false;
            }

            // プロファイルで指定されたパックを有効化
            let mut enabled_count = 0;
            for pack_entry in &profile.config.resource_packs {
                if pack_manager.enable_pack(&pack_entry.id) {
                    enabled_count += 1;
                    info!("Enabled resource pack: {}", pack_entry.id);

                    // 優先度オーバーライドがある場合は適用
                    if let Some(priority) = pack_entry.priority_override {
                        if let Some(pack) = pack_manager
                            .available_packs
                            .iter_mut()
                            .find(|p| p.manifest.id == pack_entry.id)
                        {
                            pack.manifest.priority = priority;
                        }
                    }
                } else {
                    warn!(
                        "Resource pack not found: {} (profile: {})",
                        pack_entry.id, profile.id
                    );
                }
            }

            // 優先度順に再ソート
            pack_manager
                .available_packs
                .sort_by_key(|p| p.manifest.priority);

            // リソースパック変更イベントを発火
            pack_changed_event.send(ResourcePackChangedEvent);

            info!(
                "Profile '{}' applied {} resource packs",
                profile.id, enabled_count
            );
        }
    }
}

/// アクティブプロファイル設定ファイル（config/active_profile.yaml）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActiveProfileConfig {
    /// アクティブなプロファイルID
    pub profile: String,
}

impl ActiveProfileConfig {
    /// 設定ファイルを読み込む
    pub fn load(path: impl AsRef<Path>) -> Option<Self> {
        let content = fs::read_to_string(path.as_ref()).ok()?;
        serde_yaml::from_str(&content).ok()
    }

    /// 設定ファイルを保存
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let content = serde_yaml::to_string(self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        // 親ディレクトリを作成
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_config_serialization() {
        let config = ProfileConfig {
            name: "Test Profile".to_string(),
            version: "1.0.0".to_string(),
            description: "A test profile".to_string(),
            author: "Test Author".to_string(),
            mods: vec![],
            resource_packs: vec![
                ResourcePackEntry {
                    id: "hd-textures".to_string(),
                    priority_override: Some(100),
                },
                ResourcePackEntry {
                    id: "japanese-translation".to_string(),
                    priority_override: None,
                },
            ],
            server_origin: None,
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let restored: ProfileConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(config.name, restored.name);
        assert_eq!(config.resource_packs.len(), 2);
        assert_eq!(restored.resource_packs[0].id, "hd-textures");
        assert_eq!(restored.resource_packs[0].priority_override, Some(100));
    }

    #[test]
    fn test_active_profile_config() {
        let config = ActiveProfileConfig {
            profile: "vanilla".to_string(),
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let restored: ActiveProfileConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(config.profile, restored.profile);
    }

    #[test]
    fn test_profile_paths() {
        let profile = Profile {
            id: "test".to_string(),
            config: ProfileConfig {
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
                author: String::new(),
                mods: vec![],
                resource_packs: vec![],
                server_origin: None,
            },
            path: PathBuf::from("profiles/test"),
            data_path: PathBuf::from("profiles/test/data"),
            assets_path: PathBuf::from("profiles/test/assets"),
        };

        assert_eq!(profile.items_path(), PathBuf::from("profiles/test/data/items.yaml"));
        assert_eq!(profile.recipes_path(), PathBuf::from("profiles/test/data/recipes.yaml"));
        assert_eq!(profile.quests_path(), PathBuf::from("profiles/test/data/quests.yaml"));
    }

    #[test]
    fn test_mod_dependency_serialization() {
        let dep = ModDependency {
            id: "steel_age".to_string(),
            version: ">=1.0.0".to_string(),
        };

        let yaml = serde_yaml::to_string(&dep).unwrap();
        assert!(yaml.contains("steel_age"));
        assert!(yaml.contains(">=1.0.0"));
    }
}
