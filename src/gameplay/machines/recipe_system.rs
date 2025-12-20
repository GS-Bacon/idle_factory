// src/gameplay/machines/recipe_system.rs
//! レシピシステム
//!
//! 工作機械のレシピを管理するシステム。
//! - Recipe: レシピ定義（入力、出力、流体、加工時間、作業種別）
//! - RecipeManager: レシピ検索と管理

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// ========================================
// 作業種別
// ========================================

/// 機械の作業種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WorkType {
    /// 汎用加工（Assembler）
    #[default]
    Assembling,
    /// プレス加工（Mechanical Press）
    Pressing,
    /// 粉砕（Crusher）
    Crushing,
    /// 切断（Mechanical Saw）
    Cutting,
    /// 混合（Mixer）
    Mixing,
    /// 伸線（Wire Drawer）
    WireDrawing,
    /// 洗浄（Washer）
    Washing,
    /// 精錬（Smelter）
    Smelting,
}

impl WorkType {
    /// 文字列からWorkTypeを取得
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "assembling" => Some(Self::Assembling),
            "pressing" => Some(Self::Pressing),
            "crushing" => Some(Self::Crushing),
            "cutting" => Some(Self::Cutting),
            "mixing" => Some(Self::Mixing),
            "wiredrawing" | "wire_drawing" => Some(Self::WireDrawing),
            "washing" => Some(Self::Washing),
            "smelting" => Some(Self::Smelting),
            _ => None,
        }
    }
}

// ========================================
// レシピ材料
// ========================================

/// アイテム入出力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemIO {
    /// アイテムID
    pub item: String,
    /// 数量
    pub count: u32,
}

/// 流体入出力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidIO {
    /// 流体ID
    pub fluid: String,
    /// 量 (mB単位)
    pub amount: f32,
}

// ========================================
// レシピ定義
// ========================================

/// レシピ定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// レシピID
    pub id: String,
    /// 表示名
    pub name: String,
    /// 入力アイテムリスト
    #[serde(default)]
    pub inputs: Vec<ItemIO>,
    /// 入力流体（Option）
    #[serde(default)]
    pub input_fluid: Option<FluidIO>,
    /// 出力アイテムリスト
    #[serde(default)]
    pub outputs: Vec<ItemIO>,
    /// 副産物流体（Option）
    #[serde(default)]
    pub output_fluid: Option<FluidIO>,
    /// 加工時間（秒）
    pub craft_time: f32,
    /// 作業種別
    #[serde(default)]
    pub work_type: WorkType,
}

impl Recipe {
    /// 指定された入力アイテムがこのレシピを満たすか確認
    pub fn matches_inputs(&self, available_items: &HashMap<String, u32>) -> bool {
        for input in &self.inputs {
            let available = available_items.get(&input.item).copied().unwrap_or(0);
            if available < input.count {
                return false;
            }
        }
        true
    }

    /// 入力流体が必要か
    pub fn requires_fluid(&self) -> bool {
        self.input_fluid.is_some()
    }
}

// ========================================
// レシピマネージャー
// ========================================

/// レシピ管理リソース
#[derive(Resource, Default)]
pub struct RecipeManager {
    /// 全レシピ（ID → Recipe）
    pub recipes: HashMap<String, Recipe>,
    /// 作業種別ごとのレシピIDリスト
    pub by_work_type: HashMap<WorkType, Vec<String>>,
    /// 入力アイテムごとのレシピIDリスト（逆引き用）
    pub by_input_item: HashMap<String, Vec<String>>,
}

impl RecipeManager {
    /// レシピを追加
    pub fn add_recipe(&mut self, recipe: Recipe) {
        let id = recipe.id.clone();
        let work_type = recipe.work_type;

        // 入力アイテムでインデックス
        for input in &recipe.inputs {
            self.by_input_item
                .entry(input.item.clone())
                .or_default()
                .push(id.clone());
        }

        // 作業種別でインデックス
        self.by_work_type
            .entry(work_type)
            .or_default()
            .push(id.clone());

        self.recipes.insert(id, recipe);
    }

    /// IDでレシピを取得
    pub fn get(&self, id: &str) -> Option<&Recipe> {
        self.recipes.get(id)
    }

    /// 作業種別に対応するレシピを取得
    pub fn get_by_work_type(&self, work_type: WorkType) -> Vec<&Recipe> {
        self.by_work_type
            .get(&work_type)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// 入力アイテムから適合するレシピを検索
    pub fn find_matching_recipe(
        &self,
        work_type: WorkType,
        available_items: &HashMap<String, u32>,
    ) -> Option<&Recipe> {
        let recipes = self.get_by_work_type(work_type);
        recipes.into_iter().find(|r| r.matches_inputs(available_items))
    }

    /// 入力アイテムから適合するレシピを検索（流体も考慮）
    pub fn find_matching_recipe_with_fluid(
        &self,
        work_type: WorkType,
        available_items: &HashMap<String, u32>,
        available_fluid: Option<(&str, f32)>,
    ) -> Option<&Recipe> {
        let recipes = self.get_by_work_type(work_type);
        recipes.into_iter().find(|r| {
            // アイテムチェック
            if !r.matches_inputs(available_items) {
                return false;
            }
            // 流体チェック
            if let Some(required) = &r.input_fluid {
                match available_fluid {
                    Some((fluid_id, amount)) => {
                        if fluid_id != required.fluid || amount < required.amount {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            true
        })
    }

    /// 特定のアイテムが任意のレシピの入力に使えるか
    pub fn can_accept_item(&self, work_type: WorkType, item_id: &str) -> bool {
        self.get_by_work_type(work_type)
            .iter()
            .any(|r| r.inputs.iter().any(|i| i.item == item_id))
    }

    /// YAMLファイルからレシピを読み込む
    pub fn load_from_yaml(&mut self, path: &str) -> Result<usize, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;

        let recipes: Vec<Recipe> = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path, e))?;

        let count = recipes.len();
        for recipe in recipes {
            info!("Loaded recipe: {} ({})", recipe.name, recipe.id);
            self.add_recipe(recipe);
        }

        Ok(count)
    }
}

// ========================================
// プラグイン
// ========================================

/// レシピシステムプラグイン
pub struct RecipeSystemPlugin;

impl Plugin for RecipeSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RecipeManager>()
            .add_systems(Startup, load_kinetic_recipes);
    }
}

/// 起動時にレシピを読み込む
fn load_kinetic_recipes(mut manager: ResMut<RecipeManager>) {
    let path = "assets/data/recipes/kinetic.yaml";
    match manager.load_from_yaml(path) {
        Ok(count) => info!("Loaded {} kinetic recipes from {}", count, path),
        Err(e) => {
            warn!("Could not load kinetic recipes: {}", e);
            // デフォルトレシピを追加
            add_default_recipes(&mut manager);
        }
    }
}

/// デフォルトのレシピを追加（YAMLがない場合のフォールバック）
fn add_default_recipes(manager: &mut RecipeManager) {
    // プレス加工: インゴット → 板
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

    // 粉砕: 鉱石 → 粉×2
    manager.add_recipe(Recipe {
        id: "crush_iron_ore".to_string(),
        name: "Crushed Iron".to_string(),
        inputs: vec![ItemIO { item: "iron_ore".to_string(), count: 1 }],
        input_fluid: None,
        outputs: vec![ItemIO { item: "iron_dust".to_string(), count: 2 }],
        output_fluid: None,
        craft_time: 1.5,
        work_type: WorkType::Crushing,
    });

    // 切断: 原木 → 板材
    manager.add_recipe(Recipe {
        id: "cut_log".to_string(),
        name: "Wooden Planks".to_string(),
        inputs: vec![ItemIO { item: "log".to_string(), count: 1 }],
        input_fluid: None,
        outputs: vec![ItemIO { item: "plank".to_string(), count: 4 }],
        output_fluid: None,
        craft_time: 0.5,
        work_type: WorkType::Cutting,
    });

    // ミキシング: 材料混合
    manager.add_recipe(Recipe {
        id: "mix_alloy".to_string(),
        name: "Bronze Alloy".to_string(),
        inputs: vec![
            ItemIO { item: "copper_dust".to_string(), count: 3 },
            ItemIO { item: "tin_dust".to_string(), count: 1 },
        ],
        input_fluid: None,
        outputs: vec![ItemIO { item: "bronze_dust".to_string(), count: 4 }],
        output_fluid: None,
        craft_time: 2.0,
        work_type: WorkType::Mixing,
    });

    // 伸線: 板 → ワイヤー
    manager.add_recipe(Recipe {
        id: "draw_wire".to_string(),
        name: "Copper Wire".to_string(),
        inputs: vec![ItemIO { item: "copper_plate".to_string(), count: 1 }],
        input_fluid: None,
        outputs: vec![ItemIO { item: "copper_wire".to_string(), count: 2 }],
        output_fluid: None,
        craft_time: 1.0,
        work_type: WorkType::WireDrawing,
    });

    info!("Added {} default kinetic recipes", 5);
}

// ========================================
// テスト
// ========================================

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_manager() -> RecipeManager {
        let mut manager = RecipeManager::default();

        // テスト用レシピ
        manager.add_recipe(Recipe {
            id: "test_press".to_string(),
            name: "Test Press".to_string(),
            inputs: vec![ItemIO { item: "iron_ingot".to_string(), count: 1 }],
            input_fluid: None,
            outputs: vec![ItemIO { item: "iron_plate".to_string(), count: 1 }],
            output_fluid: None,
            craft_time: 1.0,
            work_type: WorkType::Pressing,
        });

        manager.add_recipe(Recipe {
            id: "test_crush".to_string(),
            name: "Test Crush".to_string(),
            inputs: vec![ItemIO { item: "iron_ore".to_string(), count: 1 }],
            input_fluid: None,
            outputs: vec![ItemIO { item: "iron_dust".to_string(), count: 2 }],
            output_fluid: None,
            craft_time: 1.5,
            work_type: WorkType::Crushing,
        });

        manager
    }

    #[test]
    fn test_recipe_manager_get() {
        let manager = setup_manager();

        let recipe = manager.get("test_press");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().name, "Test Press");
    }

    #[test]
    fn test_recipe_manager_by_work_type() {
        let manager = setup_manager();

        let pressing = manager.get_by_work_type(WorkType::Pressing);
        assert_eq!(pressing.len(), 1);
        assert_eq!(pressing[0].id, "test_press");

        let crushing = manager.get_by_work_type(WorkType::Crushing);
        assert_eq!(crushing.len(), 1);
    }

    #[test]
    fn test_find_matching_recipe() {
        let manager = setup_manager();

        let mut items = HashMap::new();
        items.insert("iron_ingot".to_string(), 5);

        let recipe = manager.find_matching_recipe(WorkType::Pressing, &items);
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().id, "test_press");

        // 材料不足
        items.clear();
        let recipe = manager.find_matching_recipe(WorkType::Pressing, &items);
        assert!(recipe.is_none());
    }

    #[test]
    fn test_can_accept_item() {
        let manager = setup_manager();

        assert!(manager.can_accept_item(WorkType::Pressing, "iron_ingot"));
        assert!(!manager.can_accept_item(WorkType::Pressing, "gold_ingot"));
    }
}
