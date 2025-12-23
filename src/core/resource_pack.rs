// src/core/resource_pack.rs
//! リソースパックシステム
//!
//! Minecraft風のリソースパック機能を提供。
//! 一部のアセットだけを上書き・追加可能。
//!
//! ## 構造
//! ```
//! resource_packs/
//! └── my-pack/
//!     ├── pack.yaml           # メタ情報
//!     ├── textures/           # UI、ブロックテクスチャ
//!     ├── models/             # .voxモデル
//!     ├── sounds/             # サウンドファイル
//!     ├── fonts/              # カスタムフォント
//!     └── lang/               # 翻訳ファイル（ja.yaml, en.yaml）
//! ```

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// リソースパックプラグイン
pub struct ResourcePackPlugin;

impl Plugin for ResourcePackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourcePackManager>()
            .init_resource::<ActiveResourcePacks>()
            .init_resource::<TranslationManager>()
            .add_event::<ResourcePackChangedEvent>()
            .add_event::<ReloadResourcePacksEvent>()
            .add_systems(Startup, scan_resource_packs)
            .add_systems(Update, (handle_reload_event, apply_resource_pack_changes));
    }
}

/// リソースパックのメタ情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackManifest {
    /// パックID（ディレクトリ名）
    pub id: String,
    /// 表示名
    pub name: String,
    /// 説明
    pub description: String,
    /// バージョン
    pub version: String,
    /// 作者
    pub author: String,
    /// 対応ゲームバージョン
    pub game_version: String,
    /// アイコンパス（パック内相対パス）
    #[serde(default)]
    pub icon: Option<String>,
    /// 優先度（高いほど後に適用）
    #[serde(default)]
    pub priority: i32,
}

/// リソースパック
#[derive(Debug, Clone)]
pub struct ResourcePack {
    /// メタ情報
    pub manifest: ResourcePackManifest,
    /// パックのルートパス
    pub path: PathBuf,
    /// 有効かどうか
    pub enabled: bool,
    /// 含まれるリソース一覧
    pub resources: ResourcePackContents,
}

/// パックに含まれるリソースの一覧
#[derive(Debug, Clone, Default)]
pub struct ResourcePackContents {
    /// テクスチャファイル（相対パス → フルパス）
    pub textures: HashMap<String, PathBuf>,
    /// モデルファイル
    pub models: HashMap<String, PathBuf>,
    /// サウンドファイル
    pub sounds: HashMap<String, PathBuf>,
    /// フォントファイル
    pub fonts: HashMap<String, PathBuf>,
    /// 翻訳ファイル（言語コード → ファイルパス）
    pub translations: HashMap<String, PathBuf>,
}

/// リソースパックマネージャー
#[derive(Resource, Default)]
pub struct ResourcePackManager {
    /// 検出された全パック
    pub available_packs: Vec<ResourcePack>,
    /// パックのルートディレクトリ
    pub packs_directory: PathBuf,
}

impl ResourcePackManager {
    /// パックディレクトリを設定
    pub fn with_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.packs_directory = path.into();
        self
    }

    /// 利用可能なパックをスキャン
    pub fn scan_packs(&mut self) {
        self.available_packs.clear();

        let dir = if self.packs_directory.as_os_str().is_empty() {
            PathBuf::from("resource_packs")
        } else {
            self.packs_directory.clone()
        };

        if !dir.exists() {
            if let Err(e) = fs::create_dir_all(&dir) {
                warn!("Failed to create resource_packs directory: {}", e);
                return;
            }
            info!("Created resource_packs directory");
        }

        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read resource_packs directory: {}", e);
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(pack) = self.load_pack(&path) {
                    info!(
                        "Found resource pack: {} ({})",
                        pack.manifest.name, pack.manifest.id
                    );
                    self.available_packs.push(pack);
                }
            }
        }

        // 優先度順にソート
        self.available_packs.sort_by_key(|p| p.manifest.priority);

        info!("Scanned {} resource packs", self.available_packs.len());
    }

    /// パックを読み込む
    fn load_pack(&self, path: &Path) -> Option<ResourcePack> {
        let manifest_path = path.join("pack.yaml");
        if !manifest_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&manifest_path).ok()?;
        let mut manifest: ResourcePackManifest = serde_yaml::from_str(&content).ok()?;

        // IDが未設定ならディレクトリ名を使用
        if manifest.id.is_empty() {
            manifest.id = path.file_name()?.to_string_lossy().to_string();
        }

        let resources = self.scan_pack_contents(path);

        Some(ResourcePack {
            manifest,
            path: path.to_path_buf(),
            enabled: false,
            resources,
        })
    }

    /// パック内のリソースをスキャン
    fn scan_pack_contents(&self, pack_path: &Path) -> ResourcePackContents {
        let mut contents = ResourcePackContents::default();

        // テクスチャ
        Self::scan_directory(
            &pack_path.join("textures"),
            &["png", "jpg", "jpeg"],
            &mut contents.textures,
        );

        // モデル
        Self::scan_directory(&pack_path.join("models"), &["vox"], &mut contents.models);

        // サウンド
        Self::scan_directory(
            &pack_path.join("sounds"),
            &["ogg", "wav", "mp3"],
            &mut contents.sounds,
        );

        // フォント
        Self::scan_directory(
            &pack_path.join("fonts"),
            &["ttf", "otf"],
            &mut contents.fonts,
        );

        // 翻訳
        let lang_dir = pack_path.join("lang");
        if lang_dir.exists() {
            if let Ok(entries) = fs::read_dir(&lang_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "yaml" || ext == "yml" {
                            if let Some(stem) = path.file_stem() {
                                let lang_code = stem.to_string_lossy().to_string();
                                contents.translations.insert(lang_code, path);
                            }
                        }
                    }
                }
            }
        }

        contents
    }

    /// ディレクトリを再帰的にスキャン
    fn scan_directory(dir: &Path, extensions: &[&str], result: &mut HashMap<String, PathBuf>) {
        if !dir.exists() {
            return;
        }

        Self::scan_directory_recursive(dir, dir, extensions, result);
    }

    fn scan_directory_recursive(
        base: &Path,
        current: &Path,
        extensions: &[&str],
        result: &mut HashMap<String, PathBuf>,
    ) {
        let entries = match fs::read_dir(current) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_directory_recursive(base, &path, extensions, result);
            } else if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if extensions.iter().any(|e| *e == ext_str) {
                    // 相対パスをキーとして使用
                    if let Ok(relative) = path.strip_prefix(base) {
                        let key = relative.to_string_lossy().replace('\\', "/");
                        result.insert(key, path);
                    }
                }
            }
        }
    }

    /// パックを有効化
    pub fn enable_pack(&mut self, pack_id: &str) -> bool {
        if let Some(pack) = self
            .available_packs
            .iter_mut()
            .find(|p| p.manifest.id == pack_id)
        {
            pack.enabled = true;
            true
        } else {
            false
        }
    }

    /// パックを無効化
    pub fn disable_pack(&mut self, pack_id: &str) -> bool {
        if let Some(pack) = self
            .available_packs
            .iter_mut()
            .find(|p| p.manifest.id == pack_id)
        {
            pack.enabled = false;
            true
        } else {
            false
        }
    }

    /// 有効なパックを優先度順に取得
    pub fn get_enabled_packs(&self) -> Vec<&ResourcePack> {
        self.available_packs.iter().filter(|p| p.enabled).collect()
    }
}

/// アクティブなリソースパックの解決済みパス
#[derive(Resource, Default)]
pub struct ActiveResourcePacks {
    /// 解決済みテクスチャパス（リソースキー → 実際のパス）
    pub textures: HashMap<String, PathBuf>,
    /// 解決済みモデルパス
    pub models: HashMap<String, PathBuf>,
    /// 解決済みサウンドパス
    pub sounds: HashMap<String, PathBuf>,
    /// 解決済みフォントパス
    pub fonts: HashMap<String, PathBuf>,
}

impl ActiveResourcePacks {
    /// 有効なパックからリソースを解決
    pub fn resolve_from_packs(&mut self, packs: &[&ResourcePack], default_assets: &Path) {
        self.textures.clear();
        self.models.clear();
        self.sounds.clear();
        self.fonts.clear();

        // デフォルトアセットを先にロード
        Self::scan_default_assets(
            default_assets,
            &mut self.textures,
            "textures",
            &["png", "jpg"],
        );
        Self::scan_default_assets(default_assets, &mut self.models, "models", &["vox"]);
        Self::scan_default_assets(default_assets, &mut self.sounds, "sounds", &["ogg", "wav"]);
        Self::scan_default_assets(default_assets, &mut self.fonts, "fonts", &["ttf", "otf"]);

        // パックで上書き（優先度順）
        for pack in packs {
            for (key, path) in &pack.resources.textures {
                self.textures.insert(key.clone(), path.clone());
            }
            for (key, path) in &pack.resources.models {
                self.models.insert(key.clone(), path.clone());
            }
            for (key, path) in &pack.resources.sounds {
                self.sounds.insert(key.clone(), path.clone());
            }
            for (key, path) in &pack.resources.fonts {
                self.fonts.insert(key.clone(), path.clone());
            }
        }
    }

    fn scan_default_assets(
        base: &Path,
        result: &mut HashMap<String, PathBuf>,
        subdir: &str,
        extensions: &[&str],
    ) {
        let dir = base.join(subdir);
        if dir.exists() {
            ResourcePackManager::scan_directory(&dir, extensions, result);
        }
    }

    /// テクスチャパスを取得
    pub fn get_texture(&self, key: &str) -> Option<&PathBuf> {
        self.textures.get(key)
    }

    /// モデルパスを取得
    pub fn get_model(&self, key: &str) -> Option<&PathBuf> {
        self.models.get(key)
    }

    /// サウンドパスを取得
    pub fn get_sound(&self, key: &str) -> Option<&PathBuf> {
        self.sounds.get(key)
    }

    /// フォントパスを取得
    pub fn get_font(&self, key: &str) -> Option<&PathBuf> {
        self.fonts.get(key)
    }
}

/// 翻訳マネージャー
#[derive(Resource, Default)]
pub struct TranslationManager {
    /// 現在の言語コード
    pub current_language: String,
    /// フォールバック言語コード
    pub fallback_language: String,
    /// 翻訳データ（キー → テキスト）
    pub translations: HashMap<String, String>,
    /// フォールバック翻訳データ
    pub fallback_translations: HashMap<String, String>,
}

impl TranslationManager {
    /// 新しい翻訳マネージャーを作成
    pub fn new() -> Self {
        Self {
            current_language: "ja".to_string(),
            fallback_language: "en".to_string(),
            translations: HashMap::new(),
            fallback_translations: HashMap::new(),
        }
    }

    /// 言語を設定
    pub fn set_language(&mut self, language: &str) {
        self.current_language = language.to_string();
    }

    /// 翻訳ファイルを読み込む
    pub fn load_translations(&mut self, packs: &[&ResourcePack], default_lang_dir: &Path) {
        self.translations.clear();
        self.fallback_translations.clear();

        // デフォルト翻訳を読み込む
        self.load_language_file(
            &default_lang_dir.join(format!("{}.yaml", self.fallback_language)),
            &mut self.fallback_translations,
        );
        self.load_language_file(
            &default_lang_dir.join(format!("{}.yaml", self.current_language)),
            &mut self.translations,
        );

        // パックからの翻訳で上書き
        for pack in packs {
            // フォールバック言語
            if let Some(path) = pack.resources.translations.get(&self.fallback_language) {
                self.load_language_file(path, &mut self.fallback_translations);
            }
            // 現在の言語
            if let Some(path) = pack.resources.translations.get(&self.current_language) {
                self.load_language_file(path, &mut self.translations);
            }
        }

        info!(
            "Loaded {} translations ({} fallback)",
            self.translations.len(),
            self.fallback_translations.len()
        );
    }

    /// 言語ファイルを読み込む
    fn load_language_file(&mut self, path: &Path, target: &mut HashMap<String, String>) {
        if !path.exists() {
            return;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read language file {:?}: {}", path, e);
                return;
            }
        };

        let data: HashMap<String, serde_yaml::Value> = match serde_yaml::from_str(&content) {
            Ok(d) => d,
            Err(e) => {
                warn!("Failed to parse language file {:?}: {}", path, e);
                return;
            }
        };

        // フラット化して読み込む
        Self::flatten_translations("", &data, target);
    }

    /// ネストされた翻訳データをフラット化
    fn flatten_translations(
        prefix: &str,
        data: &HashMap<String, serde_yaml::Value>,
        target: &mut HashMap<String, String>,
    ) {
        for (key, value) in data {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            match value {
                serde_yaml::Value::String(s) => {
                    target.insert(full_key, s.clone());
                }
                serde_yaml::Value::Mapping(map) => {
                    let nested: HashMap<String, serde_yaml::Value> = map
                        .iter()
                        .filter_map(|(k, v)| k.as_str().map(|s| (s.to_string(), v.clone())))
                        .collect();
                    Self::flatten_translations(&full_key, &nested, target);
                }
                _ => {}
            }
        }
    }

    /// 翻訳を取得
    pub fn get(&self, key: &str) -> &str {
        self.translations
            .get(key)
            .or_else(|| self.fallback_translations.get(key))
            .map(|s| s.as_str())
            .unwrap_or(key)
    }

    /// 翻訳を取得（プレースホルダー置換付き）
    pub fn get_with_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut result = self.get(key).to_string();
        for (placeholder, value) in args {
            result = result.replace(&format!("{{{}}}", placeholder), value);
        }
        result
    }
}

/// リソースパック変更イベント
#[derive(Event)]
pub struct ResourcePackChangedEvent;

/// リソースパック再読み込みイベント
#[derive(Event)]
pub struct ReloadResourcePacksEvent;

/// 起動時にリソースパックをスキャン
fn scan_resource_packs(mut manager: ResMut<ResourcePackManager>) {
    manager.packs_directory = PathBuf::from("resource_packs");
    manager.scan_packs();
}

/// 再読み込みイベントを処理
fn handle_reload_event(
    mut events: EventReader<ReloadResourcePacksEvent>,
    mut manager: ResMut<ResourcePackManager>,
    mut changed_event: EventWriter<ResourcePackChangedEvent>,
) {
    for _ in events.read() {
        manager.scan_packs();
        changed_event.send(ResourcePackChangedEvent);
        info!("Resource packs reloaded");
    }
}

/// リソースパック変更を適用
fn apply_resource_pack_changes(
    mut events: EventReader<ResourcePackChangedEvent>,
    manager: Res<ResourcePackManager>,
    mut active: ResMut<ActiveResourcePacks>,
    mut translations: ResMut<TranslationManager>,
) {
    for _ in events.read() {
        let enabled_packs = manager.get_enabled_packs();
        let default_assets = PathBuf::from("assets");

        active.resolve_from_packs(&enabled_packs, &default_assets);
        translations.load_translations(&enabled_packs, &default_assets.join("lang"));

        info!(
            "Applied {} resource packs: {} textures, {} models, {} sounds",
            enabled_packs.len(),
            active.textures.len(),
            active.models.len(),
            active.sounds.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_manager() {
        let mut manager = TranslationManager::new();
        manager
            .translations
            .insert("test.key".to_string(), "Hello".to_string());

        assert_eq!(manager.get("test.key"), "Hello");
        assert_eq!(manager.get("unknown.key"), "unknown.key");
    }

    #[test]
    fn test_translation_with_args() {
        let mut manager = TranslationManager::new();
        manager.translations.insert(
            "greeting".to_string(),
            "Hello, {name}! You have {count} items.".to_string(),
        );

        let result = manager.get_with_args("greeting", &[("name", "Alice"), ("count", "5")]);
        assert_eq!(result, "Hello, Alice! You have 5 items.");
    }

    #[test]
    fn test_resource_pack_contents_default() {
        let contents = ResourcePackContents::default();
        assert!(contents.textures.is_empty());
        assert!(contents.models.is_empty());
        assert!(contents.sounds.is_empty());
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = ResourcePackManifest {
            id: "test-pack".to_string(),
            name: "Test Pack".to_string(),
            description: "A test resource pack".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            game_version: "0.1.0".to_string(),
            icon: None,
            priority: 100,
        };

        let yaml = serde_yaml::to_string(&manifest).unwrap();
        let restored: ResourcePackManifest = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(manifest.id, restored.id);
        assert_eq!(manifest.priority, restored.priority);
    }
}
