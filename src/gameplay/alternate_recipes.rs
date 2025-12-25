//! ハードドライブ・代替レシピシステム（Satisfactory MAM風）
//!
//! ## 概要
//! - ハードドライブを探索で発見
//! - MAM（Molecular Analysis Machine）で研究
//! - 3つの代替レシピから1つを選択してアンロック
//!
//! ## 代替レシピの特徴
//! - 異なる材料で同じ製品を作成
//! - より効率的な比率
//! - 副産物の有効活用

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 代替レシピ定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternateRecipe {
    /// レシピID
    pub id: String,
    /// 表示名
    pub name: String,
    /// 説明
    pub description: String,
    /// 入力アイテム
    pub inputs: Vec<RecipeInput>,
    /// 出力アイテム
    pub outputs: Vec<RecipeOutput>,
    /// 加工時間（秒）
    pub craft_time: f32,
    /// 対応する機械
    pub machine: String,
    /// 必要研究段階
    pub research_tier: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInput {
    pub item_id: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeOutput {
    pub item_id: String,
    pub count: u32,
}

/// ハードドライブコンポーネント
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct HardDrive {
    /// このハードドライブに含まれる候補レシピ（未開封時）
    pub candidate_recipes: Vec<String>,
    /// 既に使用済みかどうか
    pub is_used: bool,
}

impl Default for HardDrive {
    fn default() -> Self {
        Self {
            candidate_recipes: Vec::new(),
            is_used: false,
        }
    }
}

/// MAM（Molecular Analysis Machine）リソース
#[derive(Resource, Debug, Clone, Default)]
pub struct MamResearch {
    /// 現在研究中のハードドライブ
    pub current_research: Option<HardDriveResearch>,
    /// 研究履歴（解放済みレシピID）
    pub unlocked_recipes: HashSet<String>,
    /// 全代替レシピの定義
    pub all_alternate_recipes: HashMap<String, AlternateRecipe>,
    /// ハードドライブライブラリ（未使用のハードドライブ）
    pub hard_drive_library: u32,
}

/// ハードドライブ研究状態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardDriveResearch {
    /// 研究進捗（0.0-1.0）
    pub progress: f32,
    /// 選択肢として提示される3つのレシピ
    pub choices: [String; 3],
    /// 研究に必要な時間（秒）
    pub research_time: f32,
}

impl MamResearch {
    pub fn new() -> Self {
        let mut mam = Self::default();
        mam.initialize_default_recipes();
        mam
    }

    /// デフォルトの代替レシピを登録
    fn initialize_default_recipes(&mut self) {
        // Casted Screw: 鋼鉄インゴット -> ネジ（より効率的）
        self.add_recipe(AlternateRecipe {
            id: "alt_casted_screw".to_string(),
            name: "Casted Screw".to_string(),
            description: "Produces more screws from steel ingots".to_string(),
            inputs: vec![RecipeInput { item_id: "steel_ingot".to_string(), count: 1 }],
            outputs: vec![RecipeOutput { item_id: "screw".to_string(), count: 20 }],
            craft_time: 4.0,
            machine: "constructor".to_string(),
            research_tier: 1,
        });

        // Pure Iron Ingot: 鉄鉱石 + 水 -> 鉄インゴット（水で純度アップ）
        self.add_recipe(AlternateRecipe {
            id: "alt_pure_iron_ingot".to_string(),
            name: "Pure Iron Ingot".to_string(),
            description: "Uses water to produce purer iron ingots".to_string(),
            inputs: vec![
                RecipeInput { item_id: "iron_ore".to_string(), count: 2 },
                RecipeInput { item_id: "water_bucket".to_string(), count: 1 },
            ],
            outputs: vec![RecipeOutput { item_id: "iron_ingot".to_string(), count: 3 }],
            craft_time: 3.0,
            machine: "refinery".to_string(),
            research_tier: 2,
        });

        // Compacted Coal: 石炭 + 硫黄 -> 圧縮石炭（高効率燃料）
        self.add_recipe(AlternateRecipe {
            id: "alt_compacted_coal".to_string(),
            name: "Compacted Coal".to_string(),
            description: "Combines coal and sulfur for better fuel".to_string(),
            inputs: vec![
                RecipeInput { item_id: "coal".to_string(), count: 5 },
                RecipeInput { item_id: "sulfur".to_string(), count: 2 },
            ],
            outputs: vec![RecipeOutput { item_id: "compacted_coal".to_string(), count: 3 }],
            craft_time: 4.0,
            machine: "assembler".to_string(),
            research_tier: 2,
        });

        // Steel Beam: 鋼鉄インゴット -> 鋼鉄ビーム
        self.add_recipe(AlternateRecipe {
            id: "alt_steel_beam".to_string(),
            name: "Steel Beam".to_string(),
            description: "Efficient steel beam production".to_string(),
            inputs: vec![RecipeInput { item_id: "steel_ingot".to_string(), count: 2 }],
            outputs: vec![RecipeOutput { item_id: "steel_beam".to_string(), count: 3 }],
            craft_time: 6.0,
            machine: "constructor".to_string(),
            research_tier: 1,
        });

        // Recycled Plastic: プラスチック + 燃料 -> ゴム + プラスチック
        self.add_recipe(AlternateRecipe {
            id: "alt_recycled_plastic".to_string(),
            name: "Recycled Plastic".to_string(),
            description: "Recycles plastic with fuel to produce rubber".to_string(),
            inputs: vec![
                RecipeInput { item_id: "plastic".to_string(), count: 3 },
                RecipeInput { item_id: "fuel_bucket".to_string(), count: 1 },
            ],
            outputs: vec![
                RecipeOutput { item_id: "rubber".to_string(), count: 2 },
                RecipeOutput { item_id: "plastic".to_string(), count: 1 },
            ],
            craft_time: 8.0,
            machine: "refinery".to_string(),
            research_tier: 3,
        });
    }

    /// 代替レシピを追加
    pub fn add_recipe(&mut self, recipe: AlternateRecipe) {
        self.all_alternate_recipes.insert(recipe.id.clone(), recipe);
    }

    /// ハードドライブ研究を開始
    pub fn start_research(&mut self) -> Result<(), &'static str> {
        if self.current_research.is_some() {
            return Err("Research already in progress");
        }
        if self.hard_drive_library == 0 {
            return Err("No hard drives available");
        }

        // 未解放のレシピから3つ選択
        let available: Vec<_> = self.all_alternate_recipes.keys()
            .filter(|id| !self.unlocked_recipes.contains(*id))
            .cloned()
            .collect();

        if available.len() < 3 {
            return Err("Not enough recipes to research");
        }

        // シンプルなランダム選択（実際のゲームでは乱数を使用）
        let choices = [
            available[0].clone(),
            available[1].clone(),
            available[2].clone(),
        ];

        self.hard_drive_library -= 1;
        self.current_research = Some(HardDriveResearch {
            progress: 0.0,
            choices,
            research_time: 10.0, // 10秒
        });

        Ok(())
    }

    /// 研究進捗を更新
    pub fn update_research(&mut self, delta: f32) {
        if let Some(ref mut research) = self.current_research {
            research.progress += delta / research.research_time;
            research.progress = research.progress.min(1.0);
        }
    }

    /// 研究が完了しているか
    pub fn is_research_complete(&self) -> bool {
        self.current_research.as_ref()
            .map(|r| r.progress >= 1.0)
            .unwrap_or(false)
    }

    /// レシピを選択して解放
    pub fn select_recipe(&mut self, index: usize) -> Result<String, &'static str> {
        if !self.is_research_complete() {
            return Err("Research not complete");
        }

        let research = self.current_research.take()
            .ok_or("No research in progress")?;

        if index >= 3 {
            return Err("Invalid choice index");
        }

        let recipe_id = research.choices[index].clone();
        self.unlocked_recipes.insert(recipe_id.clone());

        Ok(recipe_id)
    }
}

// =====================================
// テスト
// =====================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mam_default_recipes() {
        let mam = MamResearch::new();
        assert!(mam.all_alternate_recipes.len() >= 5);
        assert!(mam.all_alternate_recipes.contains_key("alt_casted_screw"));
    }

    #[test]
    fn test_research_flow() {
        let mut mam = MamResearch::new();
        mam.hard_drive_library = 1;

        // 研究開始
        assert!(mam.start_research().is_ok());
        assert!(mam.current_research.is_some());
        assert_eq!(mam.hard_drive_library, 0);

        // 進捗更新
        mam.update_research(5.0); // 50%
        assert!(!mam.is_research_complete());

        mam.update_research(6.0); // 完了
        assert!(mam.is_research_complete());

        // レシピ選択
        let result = mam.select_recipe(0);
        assert!(result.is_ok());
        assert_eq!(mam.unlocked_recipes.len(), 1);
    }

    #[test]
    fn test_research_requires_hard_drive() {
        let mut mam = MamResearch::new();
        mam.hard_drive_library = 0;

        assert!(mam.start_research().is_err());
    }
}
