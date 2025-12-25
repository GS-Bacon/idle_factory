//! ホットリロード機能
//!
//! ゲームデータ（アイテム、レシピ等）を実行時に再読み込みする機能を提供
//! - F5キーで手動リロード
//! - ReloadDataEventでプログラムからリロードをトリガー

use bevy::prelude::*;
use std::fs;

use crate::gameplay::inventory::{ItemData, ItemDefinition, ItemRegistry};
use crate::gameplay::machines::recipe_system::RecipeManager;

/// データ再読み込みイベント
#[derive(Event)]
pub struct ReloadDataEvent;

/// ホットリロードプラグイン
pub struct HotReloadPlugin;

impl Plugin for HotReloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReloadDataEvent>()
            .add_systems(Update, (
                trigger_reload_on_f5,
                handle_reload_event,
            ));
    }
}

/// F5キーでリロードをトリガー
fn trigger_reload_on_f5(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut reload_events: EventWriter<ReloadDataEvent>,
) {
    if keyboard.just_pressed(KeyCode::F5) {
        debug!("F5 pressed - triggering data reload");
        reload_events.send(ReloadDataEvent);
    }
}

/// リロードイベントを処理
fn handle_reload_event(
    mut events: EventReader<ReloadDataEvent>,
    mut item_registry: ResMut<ItemRegistry>,
    mut recipe_manager: ResMut<RecipeManager>,
) {
    for _ in events.read() {
        debug!("Reloading game data...");

        // アイテムレジストリをリロード
        reload_items(&mut item_registry);

        // レシピマネージャをリロード
        reload_recipes(&mut recipe_manager);

        debug!("Data reload complete: {} items, {} recipes",
            item_registry.items.len(),
            recipe_manager.recipes.len()
        );
    }
}

/// アイテムを再読み込み
fn reload_items(registry: &mut ItemRegistry) {
    registry.clear();

    let path = "assets/data/items/core.yaml";
    if let Ok(content) = fs::read_to_string(path) {
        match serde_yaml::from_str::<Vec<ItemDefinition>>(&content) {
            Ok(defs) => {
                for def in defs {
                    debug!("Reloaded item: {}", def.id);
                    registry.register(def.into());
                }
            }
            Err(e) => {
                error!("Failed to parse items YAML: {}", e);
            }
        }
    } else {
        warn!("Items YAML not found, using fallback");
        register_fallback_items(registry);
    }
}

/// レシピを再読み込み
fn reload_recipes(manager: &mut RecipeManager) {
    manager.clear();

    let path = "assets/data/recipes/kinetic.yaml";
    match manager.load_from_yaml(path) {
        Ok(count) => debug!("Reloaded {} kinetic recipes", count),
        Err(e) => {
            warn!("Could not reload kinetic recipes: {}", e);
            add_default_recipes(manager);
        }
    }
}

/// フォールバック用アイテム
fn register_fallback_items(registry: &mut ItemRegistry) {
    registry.register(
        ItemData::new("raw_ore", "Raw Ore")
            .with_property("description", "Unprocessed ore from mining")
            .with_max_stack(999),
    );

    registry.register(
        ItemData::new("ingot", "Metal Ingot")
            .with_property("description", "Refined metal ingot")
            .with_max_stack(999),
    );
}

/// デフォルトレシピ
fn add_default_recipes(manager: &mut RecipeManager) {
    use crate::gameplay::machines::recipe_system::{Recipe, ItemIO, WorkType};

    manager.add_recipe(Recipe {
        id: "press_iron_plate".to_string(),
        name: "Iron Plate".to_string(),
        inputs: vec![ItemIO { item: "iron_ingot".to_string(), count: 1 }],
        input_fluid: None,
        outputs: vec![ItemIO { item: "iron_plate".to_string(), count: 1 }],
        output_fluid: None,
        craft_time: 1.0,
        work_type: WorkType::Pressing,
    });
}
