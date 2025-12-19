mod localization;
mod models;
mod recipe;

use localization::LocalizationManager;
use models::{AnimationType, AssetConfig, ItemData, LocalizationData, LocalizationEntry};
use recipe::{AssetCatalog, CatalogEntry, RecipeDef};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    assets_path: Mutex<Option<PathBuf>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn set_assets_path(path: String, state: State<AppState>) -> Result<(), String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("指定されたパスが存在しません".to_string());
    }
    *state.assets_path.lock().unwrap() = Some(path);
    Ok(())
}

#[tauri::command]
fn get_assets_path(state: State<AppState>) -> Option<String> {
    state.assets_path.lock().unwrap().as_ref().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn create_item(id: String) -> ItemData {
    ItemData::new(id)
}

#[tauri::command]
fn update_item_asset(mut item: ItemData, icon_path: Option<String>, model_path: Option<String>, animation: AnimationType) -> ItemData {
    item.asset = AssetConfig { icon_path, model_path, animation };
    item
}

#[tauri::command]
fn save_localization(i18n_key: String, localization: LocalizationData, state: State<AppState>) -> Result<(), String> {
    let assets_path = state.assets_path.lock().unwrap().clone().ok_or("アセットパスが設定されていません")?;
    let locales_path = assets_path.join("locales");
    let manager = LocalizationManager::new(locales_path);
    let mut entries = HashMap::new();
    entries.insert("ja".to_string(), localization.ja);
    entries.insert("en".to_string(), localization.en);
    manager.update_entries(&i18n_key, entries)
}

#[tauri::command]
fn load_localization(i18n_key: String, state: State<AppState>) -> Result<LocalizationData, String> {
    let assets_path = state.assets_path.lock().unwrap().clone().ok_or("アセットパスが設定されていません")?;
    let locales_path = assets_path.join("locales");
    let manager = LocalizationManager::new(locales_path);
    let ja = manager.get_entry("ja", &i18n_key)?.unwrap_or_default();
    let en = manager.get_entry("en", &i18n_key)?.unwrap_or_default();
    Ok(LocalizationData { ja, en })
}

#[tauri::command]
fn update_locale(lang: String, key: String, name: String, description: String, state: State<AppState>) -> Result<(), String> {
    let assets_path = state.assets_path.lock().unwrap().clone().ok_or("アセットパスが設定されていません")?;
    let locales_path = assets_path.join("locales");
    let manager = LocalizationManager::new(locales_path);
    let entry = LocalizationEntry { name, description };
    manager.update_entry(&lang, &key, entry)
}

#[tauri::command]
fn to_relative_path(absolute_path: String, state: State<AppState>) -> Result<String, String> {
    let assets_path = state.assets_path.lock().unwrap().clone().ok_or("アセットパスが設定されていません")?;
    let abs = PathBuf::from(&absolute_path);
    abs.strip_prefix(&assets_path).map(|p| p.to_string_lossy().to_string()).map_err(|_| "ファイルがアセットディレクトリ外にあります".to_string())
}

#[tauri::command]
fn save_item_data(item: ItemData, path: String) -> Result<(), String> {
    let content = ron::ser::to_string_pretty(&item, ron::ser::PrettyConfig::default()).map_err(|e| format!("シリアライズエラー: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("ファイル書き込みエラー: {}", e))
}

#[tauri::command]
fn load_item_data(path: String) -> Result<ItemData, String> {
    let content = std::fs::read_to_string(&path).map_err(|e| format!("ファイル読み込みエラー: {}", e))?;
    ron::from_str(&content).map_err(|e| format!("パースエラー: {}", e))
}

#[tauri::command]
fn save_recipe(recipe: RecipeDef, state: State<AppState>) -> Result<String, String> {
    let assets_path = state.assets_path.lock().unwrap().clone().ok_or("アセットパスが設定されていません")?;
    let recipes_path = assets_path.join("data").join("recipes");
    fs::create_dir_all(&recipes_path).map_err(|e| format!("ディレクトリ作成エラー: {}", e))?;
    let file_path = recipes_path.join(format!("{}.ron", recipe.id));
    let content = ron::ser::to_string_pretty(&recipe, ron::ser::PrettyConfig::default()).map_err(|e| format!("シリアライズエラー: {}", e))?;
    fs::write(&file_path, content).map_err(|e| format!("ファイル書き込みエラー: {}", e))?;
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
fn load_recipe(recipe_id: String, state: State<AppState>) -> Result<RecipeDef, String> {
    let assets_path = state.assets_path.lock().unwrap().clone().ok_or("アセットパスが設定されていません")?;
    let file_path = assets_path.join("data").join("recipes").join(format!("{}.ron", recipe_id));
    let content = fs::read_to_string(&file_path).map_err(|e| format!("ファイル読み込みエラー: {}", e))?;
    ron::from_str(&content).map_err(|e| format!("パースエラー: {}", e))
}

#[tauri::command]
fn list_recipes(state: State<AppState>) -> Result<Vec<String>, String> {
    let assets_path = state.assets_path.lock().unwrap().clone().ok_or("アセットパスが設定されていません")?;
    let recipes_path = assets_path.join("data").join("recipes");
    if !recipes_path.exists() { return Ok(Vec::new()); }
    let mut recipes = Vec::new();
    for entry in fs::read_dir(&recipes_path).map_err(|e| format!("読み込みエラー: {}", e))? {
        let entry = entry.map_err(|e| format!("エントリエラー: {}", e))?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "ron") {
            if let Some(stem) = path.file_stem() {
                recipes.push(stem.to_string_lossy().to_string());
            }
        }
    }
    Ok(recipes)
}

#[tauri::command]
fn get_assets_catalog(state: State<AppState>) -> Result<AssetCatalog, String> {
    let assets_path = state.assets_path.lock().unwrap().clone().ok_or("アセットパスが設定されていません")?;
    let mut catalog = AssetCatalog::default();

    // Load items from data/items/*.ron
    let items_path = assets_path.join("data").join("items");
    if items_path.exists() {
        if let Ok(entries) = fs::read_dir(&items_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "ron") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(item) = ron::from_str::<ItemData>(&content) {
                            catalog.items.push(CatalogEntry { id: item.id.clone(), name: item.id.clone(), icon_path: item.asset.icon_path });
                        }
                    }
                }
            }
        }
    }

    // No default items - use empty list if none found
    // Items should be defined in the Items tab of the editor

    // No default fluids - use empty list
    // Fluids should be defined in the Items tab of the editor

    // No default machines - use empty list
    // Machines should be defined in the Items tab of the editor

    // No default tags
    // Tags should be defined by the user

    Ok(catalog)
}

// Internal functions for testing (not Tauri commands)
fn internal_save_item_data(item: &ItemData, path: &std::path::Path) -> Result<(), String> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("ディレクトリ作成エラー: {}", e))?;
    }
    let content = ron::ser::to_string_pretty(item, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("シリアライズエラー: {}", e))?;
    fs::write(path, content).map_err(|e| format!("ファイル書き込みエラー: {}", e))
}

fn internal_load_item_data(path: &std::path::Path) -> Result<ItemData, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("ファイル読み込みエラー: {}", e))?;
    ron::from_str(&content).map_err(|e| format!("パースエラー: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState { assets_path: Mutex::new(None) })
        .invoke_handler(tauri::generate_handler![
            greet, set_assets_path, get_assets_path, create_item, update_item_asset,
            save_localization, load_localization, update_locale, to_relative_path,
            save_item_data, load_item_data, save_recipe, load_recipe, list_recipes, get_assets_catalog,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::ItemCategory;
    use tempfile::TempDir;

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert_eq!(result, "Hello, World! You've been greeted from Rust!");
    }

    #[test]
    fn test_create_item() {
        let item = create_item("test_item".to_string());
        assert_eq!(item.id, "test_item");
        assert_eq!(item.i18n_key, "item.test_item");
        assert_eq!(item.category, ItemCategory::Item);
    }

    #[test]
    fn test_update_item_asset() {
        let item = create_item("test".to_string());
        let updated = update_item_asset(
            item,
            Some("icons/test.png".to_string()),
            Some("models/test.glb".to_string()),
            AnimationType::Rotational { axis: [0.0, 1.0, 0.0], speed: 90.0 },
        );
        assert_eq!(updated.asset.icon_path, Some("icons/test.png".to_string()));
        assert_eq!(updated.asset.model_path, Some("models/test.glb".to_string()));
    }

    #[test]
    fn test_save_and_load_item_data() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_item.ron");

        // Create item
        let mut item = ItemData::new("test_item".to_string());
        item.category = ItemCategory::Machine;
        item.i18n_key = "machine.test_item".to_string();
        item.asset.icon_path = Some("icons/machine.png".to_string());

        // Save
        internal_save_item_data(&item, &file_path).unwrap();
        assert!(file_path.exists());

        // Load
        let loaded = internal_load_item_data(&file_path).unwrap();
        assert_eq!(loaded.id, "test_item");
        assert_eq!(loaded.category, ItemCategory::Machine);
        assert_eq!(loaded.i18n_key, "machine.test_item");
        assert_eq!(loaded.asset.icon_path, Some("icons/machine.png".to_string()));
    }

    #[test]
    fn test_save_and_load_item_with_all_categories() {
        let temp_dir = TempDir::new().unwrap();

        // Test Item category
        let item1 = ItemData::new("iron_ore".to_string());
        let path1 = temp_dir.path().join("iron_ore.ron");
        internal_save_item_data(&item1, &path1).unwrap();
        let loaded1 = internal_load_item_data(&path1).unwrap();
        assert_eq!(loaded1.category, ItemCategory::Item);

        // Test Machine category
        let mut item2 = ItemData::new("assembler".to_string());
        item2.category = ItemCategory::Machine;
        item2.i18n_key = "machine.assembler".to_string();
        let path2 = temp_dir.path().join("assembler.ron");
        internal_save_item_data(&item2, &path2).unwrap();
        let loaded2 = internal_load_item_data(&path2).unwrap();
        assert_eq!(loaded2.category, ItemCategory::Machine);

        // Test Multiblock category
        let mut item3 = ItemData::new("furnace".to_string());
        item3.category = ItemCategory::Multiblock;
        item3.i18n_key = "multiblock.furnace".to_string();
        let path3 = temp_dir.path().join("furnace.ron");
        internal_save_item_data(&item3, &path3).unwrap();
        let loaded3 = internal_load_item_data(&path3).unwrap();
        assert_eq!(loaded3.category, ItemCategory::Multiblock);
    }

    #[test]
    fn test_save_and_load_item_with_properties() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("special_item.ron");

        let mut item = ItemData::new("special_item".to_string());
        item.properties.insert("durability".to_string(), serde_json::json!(100));
        item.properties.insert("stackable".to_string(), serde_json::json!(true));
        item.properties.insert("rarity".to_string(), serde_json::json!("rare"));

        internal_save_item_data(&item, &file_path).unwrap();
        let loaded = internal_load_item_data(&file_path).unwrap();

        assert_eq!(loaded.properties.get("durability"), Some(&serde_json::json!(100)));
        assert_eq!(loaded.properties.get("stackable"), Some(&serde_json::json!(true)));
        assert_eq!(loaded.properties.get("rarity"), Some(&serde_json::json!("rare")));
    }

    #[test]
    fn test_save_and_load_item_with_animations() {
        let temp_dir = TempDir::new().unwrap();

        // Rotational animation
        let mut item1 = ItemData::new("rotating_item".to_string());
        item1.asset.animation = AnimationType::Rotational {
            axis: [0.0, 1.0, 0.0],
            speed: 45.0,
        };
        let path1 = temp_dir.path().join("rotating.ron");
        internal_save_item_data(&item1, &path1).unwrap();
        let loaded1 = internal_load_item_data(&path1).unwrap();
        assert!(matches!(loaded1.asset.animation, AnimationType::Rotational { .. }));

        // Linear animation
        let mut item2 = ItemData::new("moving_item".to_string());
        item2.asset.animation = AnimationType::Linear {
            direction: [1.0, 0.0, 0.0],
            distance: 2.0,
            speed: 1.0,
        };
        let path2 = temp_dir.path().join("moving.ron");
        internal_save_item_data(&item2, &path2).unwrap();
        let loaded2 = internal_load_item_data(&path2).unwrap();
        assert!(matches!(loaded2.asset.animation, AnimationType::Linear { .. }));

        // Skeletal animation
        let mut item3 = ItemData::new("animated_item".to_string());
        item3.asset.animation = AnimationType::Skeletal {
            animation_path: "anims/idle.glb".to_string(),
            looping: true,
        };
        let path3 = temp_dir.path().join("animated.ron");
        internal_save_item_data(&item3, &path3).unwrap();
        let loaded3 = internal_load_item_data(&path3).unwrap();
        assert!(matches!(loaded3.asset.animation, AnimationType::Skeletal { .. }));
    }

    #[test]
    fn test_load_nonexistent_file_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.ron");

        let result = internal_load_item_data(&nonexistent_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ファイル読み込みエラー"));
    }

    #[test]
    fn test_load_invalid_ron_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("invalid.ron");

        // Write invalid RON content
        fs::write(&invalid_path, "this is not valid RON").unwrap();

        let result = internal_load_item_data(&invalid_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("パースエラー"));
    }

    #[test]
    fn test_save_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("deep").join("nested").join("dir").join("item.ron");

        let item = ItemData::new("nested_item".to_string());
        internal_save_item_data(&item, &nested_path).unwrap();

        assert!(nested_path.exists());
        let loaded = internal_load_item_data(&nested_path).unwrap();
        assert_eq!(loaded.id, "nested_item");
    }
}
