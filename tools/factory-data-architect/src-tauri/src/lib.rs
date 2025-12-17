mod localization;
mod models;
mod recipe;

use localization::LocalizationManager;
use models::{AnimationType, AssetConfig, ItemData, LocalizationData, LocalizationEntry};
use recipe::{AssetCatalog, CatalogEntry, MachineCatalogEntry, MachineType, RecipeDef};
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

    // Default items if none found
    if catalog.items.is_empty() {
        catalog.items = vec![
            CatalogEntry { id: "iron_ore".into(), name: "Iron Ore".into(), icon_path: None },
            CatalogEntry { id: "iron_ingot".into(), name: "Iron Ingot".into(), icon_path: None },
            CatalogEntry { id: "iron_plate".into(), name: "Iron Plate".into(), icon_path: None },
            CatalogEntry { id: "copper_ore".into(), name: "Copper Ore".into(), icon_path: None },
            CatalogEntry { id: "copper_ingot".into(), name: "Copper Ingot".into(), icon_path: None },
            CatalogEntry { id: "gold_ore".into(), name: "Gold Ore".into(), icon_path: None },
            CatalogEntry { id: "coal".into(), name: "Coal".into(), icon_path: None },
        ];
    }

    // Default fluids
    catalog.fluids = vec![
        CatalogEntry { id: "water".into(), name: "Water".into(), icon_path: None },
        CatalogEntry { id: "lava".into(), name: "Lava".into(), icon_path: None },
        CatalogEntry { id: "oil".into(), name: "Oil".into(), icon_path: None },
        CatalogEntry { id: "steam".into(), name: "Steam".into(), icon_path: None },
    ];

    // Default machines
    catalog.machines = vec![
        MachineCatalogEntry { id: "assembler".into(), name: "Assembler".into(), machine_type: MachineType::Assembler, input_slots: 4, output_slots: 1 },
        MachineCatalogEntry { id: "mixer".into(), name: "Mixer".into(), machine_type: MachineType::Mixer, input_slots: 2, output_slots: 2 },
        MachineCatalogEntry { id: "press".into(), name: "Press".into(), machine_type: MachineType::Press, input_slots: 1, output_slots: 1 },
        MachineCatalogEntry { id: "furnace".into(), name: "Furnace".into(), machine_type: MachineType::Furnace, input_slots: 1, output_slots: 1 },
        MachineCatalogEntry { id: "crusher".into(), name: "Crusher".into(), machine_type: MachineType::Crusher, input_slots: 1, output_slots: 2 },
        MachineCatalogEntry { id: "centrifuge".into(), name: "Centrifuge".into(), machine_type: MachineType::Centrifuge, input_slots: 1, output_slots: 3 },
    ];

    // Default tags
    catalog.tags = vec!["forge:ores".into(), "forge:ingots".into(), "forge:plates".into(), "forge:dusts".into(), "forge:gems".into()];

    Ok(catalog)
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
