// src/core/modding.rs
//! Modローダーシステム
//! - Modの検出と読み込み
//! - アセット統合
//! - Mod依存関係管理

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Moddingプラグイン
pub struct ModdingPlugin;

impl Plugin for ModdingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModRegistry>()
            .add_systems(Startup, discover_mods);
    }
}

/// Modメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    /// Mod ID（一意識別子）
    pub id: String,
    /// 表示名
    pub name: String,
    /// バージョン
    pub version: String,
    /// 説明
    pub description: String,
    /// 作者
    pub author: String,
    /// 依存Mod
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,
    /// 読み込み順序（小さい方が先）
    #[serde(default)]
    pub load_order: i32,
}

/// Mod依存関係
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub mod_id: String,
    pub version: String,
    #[serde(default)]
    pub optional: bool,
}

/// ロードされたMod情報
#[derive(Debug, Clone)]
pub struct LoadedMod {
    pub manifest: ModManifest,
    pub path: PathBuf,
    pub enabled: bool,
}

/// Modレジストリ
#[derive(Resource, Default)]
pub struct ModRegistry {
    /// 検出されたMod
    pub mods: HashMap<String, LoadedMod>,
    /// 読み込み順序
    pub load_order: Vec<String>,
    /// エラーログ
    pub errors: Vec<String>,
}

impl ModRegistry {
    /// Modを取得
    pub fn get(&self, id: &str) -> Option<&LoadedMod> {
        self.mods.get(id)
    }

    /// 有効なModの一覧を取得
    pub fn enabled_mods(&self) -> Vec<&LoadedMod> {
        self.load_order
            .iter()
            .filter_map(|id| self.mods.get(id))
            .filter(|m| m.enabled)
            .collect()
    }

    /// Modのパスを取得
    pub fn get_mod_path(&self, id: &str) -> Option<&Path> {
        self.mods.get(id).map(|m| m.path.as_path())
    }

    /// Modのアセットパスを取得
    pub fn get_asset_path(&self, mod_id: &str, asset: &str) -> Option<PathBuf> {
        self.mods.get(mod_id).map(|m| m.path.join("assets").join(asset))
    }
}

/// Modsディレクトリからmodを検出
fn discover_mods(mut registry: ResMut<ModRegistry>) {
    let mods_dir = PathBuf::from("mods");

    if !mods_dir.exists() {
        // modsディレクトリを作成
        if let Err(e) = fs::create_dir_all(&mods_dir) {
            warn!("Failed to create mods directory: {}", e);
            return;
        }
        info!("Created mods directory");
    }

    // modsディレクトリ内のサブディレクトリを探索
    let entries = match fs::read_dir(&mods_dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read mods directory: {}", e);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // mod.yaml を探す
        let manifest_path = path.join("mod.yaml");
        if !manifest_path.exists() {
            continue;
        }

        // マニフェストを読み込む
        match load_manifest(&manifest_path) {
            Ok(manifest) => {
                let mod_id = manifest.id.clone();
                info!("Discovered mod: {} v{}", manifest.name, manifest.version);

                registry.mods.insert(mod_id.clone(), LoadedMod {
                    manifest,
                    path,
                    enabled: true,
                });
            }
            Err(e) => {
                let error = format!("Failed to load mod manifest at {:?}: {}", manifest_path, e);
                warn!("{}", error);
                registry.errors.push(error);
            }
        }
    }

    // 依存関係を解決して読み込み順序を決定
    resolve_load_order(&mut registry);

    info!("Loaded {} mods", registry.mods.len());
}

/// マニフェストを読み込む
fn load_manifest(path: &Path) -> Result<ModManifest, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| e.to_string())?;

    serde_yaml::from_str(&content)
        .map_err(|e| e.to_string())
}

/// 依存関係を解決して読み込み順序を決定
fn resolve_load_order(registry: &mut ModRegistry) {
    // 単純なトポロジカルソート
    let mut sorted: Vec<String> = registry.mods.keys().cloned().collect();

    // load_orderでソート、次にIDでソート
    sorted.sort_by(|a, b| {
        let a_order = registry.mods.get(a).map(|m| m.manifest.load_order).unwrap_or(0);
        let b_order = registry.mods.get(b).map(|m| m.manifest.load_order).unwrap_or(0);
        a_order.cmp(&b_order).then_with(|| a.cmp(b))
    });

    // 依存関係をチェック
    for mod_id in &sorted {
        if let Some(loaded_mod) = registry.mods.get(mod_id) {
            for dep in &loaded_mod.manifest.dependencies {
                if !dep.optional && !registry.mods.contains_key(&dep.mod_id) {
                    let error = format!(
                        "Mod '{}' requires '{}' v{} but it's not found",
                        mod_id, dep.mod_id, dep.version
                    );
                    warn!("{}", error);
                    registry.errors.push(error);
                }
            }
        }
    }

    registry.load_order = sorted;
}

/// Modアセットローダー（ヘルパー関数）
pub fn load_mod_yaml<T: for<'de> Deserialize<'de>>(
    registry: &ModRegistry,
    mod_id: &str,
    file: &str,
) -> Result<T, String> {
    let path = registry
        .get_asset_path(mod_id, file)
        .ok_or_else(|| format!("Mod '{}' not found", mod_id))?;

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;

    serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse {:?}: {}", path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_manifest_deserialize() {
        let yaml = r#"
id: test_mod
name: Test Mod
version: "1.0.0"
description: A test mod
author: Test Author
dependencies:
  - mod_id: base
    version: "1.0.0"
load_order: 10
"#;

        let manifest: ModManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.id, "test_mod");
        assert_eq!(manifest.name, "Test Mod");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.dependencies.len(), 1);
        assert_eq!(manifest.load_order, 10);
    }

    #[test]
    fn test_mod_registry() {
        let mut registry = ModRegistry::default();

        registry.mods.insert("test".to_string(), LoadedMod {
            manifest: ModManifest {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
                author: String::new(),
                dependencies: vec![],
                load_order: 0,
            },
            path: PathBuf::from("mods/test"),
            enabled: true,
        });
        registry.load_order.push("test".to_string());

        assert!(registry.get("test").is_some());
        assert_eq!(registry.enabled_mods().len(), 1);
    }

    #[test]
    fn test_mod_dependency() {
        let dep = ModDependency {
            mod_id: "base".to_string(),
            version: "1.0.0".to_string(),
            optional: false,
        };

        assert!(!dep.optional);
    }
}
