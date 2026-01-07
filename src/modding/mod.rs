//! Modding system for extending game content
//!
//! ## Architecture
//! - `api`: Mod API server (WebSocket/JSON-RPC)
//! - `data`: Data-driven mod loading (TOML/JSON)
//! - `registry`: Mod content registration

pub mod api;
pub mod data;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mod情報
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModInfo {
    /// Mod ID（namespace形式: "author.modname"）
    pub id: String,
    /// 表示名
    pub name: String,
    /// バージョン
    pub version: String,
    /// 作者
    pub author: String,
    /// 説明
    pub description: String,
    /// 依存Mod（ID -> 最小バージョン）
    pub dependencies: HashMap<String, String>,
    /// 対応ゲームバージョン
    pub game_version: String,
}

impl ModInfo {
    /// 新しいMod情報を作成
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            author: String::new(),
            description: String::new(),
            dependencies: HashMap::new(),
            game_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// 作者を設定
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    /// 説明を設定
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// 依存を追加
    pub fn with_dependency(mut self, mod_id: &str, min_version: &str) -> Self {
        self.dependencies
            .insert(mod_id.to_string(), min_version.to_string());
        self
    }

    /// Namespace部分を取得
    pub fn namespace(&self) -> &str {
        self.id.split('.').next().unwrap_or(&self.id)
    }
}

/// Modの状態
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ModState {
    /// 未ロード
    #[default]
    Unloaded,
    /// ロード中
    Loading,
    /// ロード済み
    Loaded,
    /// エラー
    Error,
    /// 無効化
    Disabled,
}

/// ロード済みMod
#[derive(Clone, Debug)]
pub struct LoadedMod {
    /// Mod情報
    pub info: ModInfo,
    /// 状態
    pub state: ModState,
    /// エラーメッセージ（ある場合）
    pub error: Option<String>,
    /// ロード順序
    pub load_order: usize,
}

impl LoadedMod {
    /// 新しいロード済みModを作成
    pub fn new(info: ModInfo, load_order: usize) -> Self {
        Self {
            info,
            state: ModState::Unloaded,
            error: None,
            load_order,
        }
    }
}

/// Modマネージャー
#[derive(Resource, Default)]
pub struct ModManager {
    /// ロード済みMod（ID -> LoadedMod）
    mods: HashMap<String, LoadedMod>,
    /// ロード順序
    load_order: Vec<String>,
}

impl ModManager {
    /// 新しいModマネージャーを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// Modを登録
    pub fn register(&mut self, info: ModInfo) -> bool {
        if self.mods.contains_key(&info.id) {
            return false;
        }

        let order = self.load_order.len();
        let id = info.id.clone();
        self.mods.insert(id.clone(), LoadedMod::new(info, order));
        self.load_order.push(id);
        true
    }

    /// Modを取得
    pub fn get(&self, id: &str) -> Option<&LoadedMod> {
        self.mods.get(id)
    }

    /// Modを取得（変更可能）
    pub fn get_mut(&mut self, id: &str) -> Option<&mut LoadedMod> {
        self.mods.get_mut(id)
    }

    /// 全Modを取得（ロード順）
    pub fn all(&self) -> impl Iterator<Item = &LoadedMod> {
        self.load_order.iter().filter_map(|id| self.mods.get(id))
    }

    /// アクティブなModを取得
    pub fn active(&self) -> impl Iterator<Item = &LoadedMod> {
        self.all().filter(|m| m.state == ModState::Loaded)
    }

    /// Mod数を取得
    pub fn count(&self) -> usize {
        self.mods.len()
    }

    /// Modを有効化
    pub fn enable(&mut self, id: &str) -> bool {
        if let Some(m) = self.mods.get_mut(id) {
            if m.state == ModState::Disabled {
                m.state = ModState::Unloaded;
                return true;
            }
        }
        false
    }

    /// Modを無効化
    pub fn disable(&mut self, id: &str) -> bool {
        if let Some(m) = self.mods.get_mut(id) {
            m.state = ModState::Disabled;
            return true;
        }
        false
    }

    /// 依存関係を検証
    pub fn validate_dependencies(&self, id: &str) -> Result<(), Vec<String>> {
        let Some(loaded_mod) = self.mods.get(id) else {
            return Err(vec![format!("Mod not found: {}", id)]);
        };

        let mut errors = Vec::new();

        for (dep_id, min_version) in &loaded_mod.info.dependencies {
            match self.mods.get(dep_id) {
                None => {
                    errors.push(format!("Missing dependency: {} >= {}", dep_id, min_version));
                }
                Some(dep) => {
                    // TODO: 実際のバージョン比較
                    if dep.state != ModState::Loaded {
                        errors.push(format!("Dependency not loaded: {}", dep_id));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Modイベント: ロード完了
#[derive(Event)]
pub struct ModLoadedEvent {
    /// Mod ID
    pub mod_id: String,
}

/// Modイベント: アンロード
#[derive(Event)]
pub struct ModUnloadedEvent {
    /// Mod ID
    pub mod_id: String,
}

/// Modイベント: エラー
#[derive(Event)]
pub struct ModErrorEvent {
    /// Mod ID
    pub mod_id: String,
    /// エラーメッセージ
    pub error: String,
}

/// Moddingプラグイン
pub struct ModdingPlugin;

impl Plugin for ModdingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModManager>()
            .add_event::<ModLoadedEvent>()
            .add_event::<ModUnloadedEvent>()
            .add_event::<ModErrorEvent>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_info_new() {
        let info = ModInfo::new("test.mymod", "My Mod", "1.0.0");

        assert_eq!(info.id, "test.mymod");
        assert_eq!(info.name, "My Mod");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.namespace(), "test");
    }

    #[test]
    fn test_mod_info_builder() {
        let info = ModInfo::new("author.mod", "Test Mod", "1.0.0")
            .with_author("Test Author")
            .with_description("A test mod")
            .with_dependency("base.core", "0.1.0");

        assert_eq!(info.author, "Test Author");
        assert_eq!(info.description, "A test mod");
        assert!(info.dependencies.contains_key("base.core"));
    }

    #[test]
    fn test_mod_manager_register() {
        let mut manager = ModManager::new();

        let info = ModInfo::new("test.mod1", "Mod 1", "1.0.0");
        assert!(manager.register(info));

        // 重複登録は失敗
        let info2 = ModInfo::new("test.mod1", "Mod 1 Duplicate", "2.0.0");
        assert!(!manager.register(info2));

        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_mod_manager_get() {
        let mut manager = ModManager::new();
        manager.register(ModInfo::new("test.mod", "Test", "1.0.0"));

        let loaded = manager.get("test.mod");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().info.name, "Test");

        assert!(manager.get("nonexistent").is_none());
    }

    #[test]
    fn test_mod_manager_enable_disable() {
        let mut manager = ModManager::new();
        manager.register(ModInfo::new("test.mod", "Test", "1.0.0"));

        // 初期状態はUnloaded
        assert_eq!(manager.get("test.mod").unwrap().state, ModState::Unloaded);

        // 無効化
        assert!(manager.disable("test.mod"));
        assert_eq!(manager.get("test.mod").unwrap().state, ModState::Disabled);

        // 有効化
        assert!(manager.enable("test.mod"));
        assert_eq!(manager.get("test.mod").unwrap().state, ModState::Unloaded);
    }

    #[test]
    fn test_mod_manager_all() {
        let mut manager = ModManager::new();
        manager.register(ModInfo::new("test.mod1", "Mod 1", "1.0.0"));
        manager.register(ModInfo::new("test.mod2", "Mod 2", "1.0.0"));
        manager.register(ModInfo::new("test.mod3", "Mod 3", "1.0.0"));

        let all: Vec<_> = manager.all().collect();
        assert_eq!(all.len(), 3);

        // ロード順序を確認
        assert_eq!(all[0].info.id, "test.mod1");
        assert_eq!(all[1].info.id, "test.mod2");
        assert_eq!(all[2].info.id, "test.mod3");
    }

    #[test]
    fn test_mod_state_values() {
        let states = [
            ModState::Unloaded,
            ModState::Loading,
            ModState::Loaded,
            ModState::Error,
            ModState::Disabled,
        ];

        for state in states {
            let mut loaded = LoadedMod::new(ModInfo::new("test", "Test", "1.0.0"), 0);
            loaded.state = state;
            assert_eq!(loaded.state, state);
        }
    }
}
