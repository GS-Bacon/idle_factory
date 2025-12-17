use serde::{Deserialize, Serialize};

/// 材料の種類（アイテムまたはタグ）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum IngredientType {
    /// 特定のアイテムID
    Item(String),
    /// 鉱石辞書タグ（複数アイテムを許容）
    Tag(String),
}

/// レシピの材料
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    /// アイテムまたはタグ
    pub ingredient_type: IngredientType,
    /// 必要数量
    pub amount: u32,
}

/// 生成物の種類
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ProductType {
    /// アイテム
    Item(String),
    /// 流体
    Fluid(String),
}

/// レシピの生成物
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    /// アイテムまたは流体
    pub product_type: ProductType,
    /// 生成数量
    pub amount: u32,
    /// 生成確率 (0.0 - 1.0)
    #[serde(default = "default_chance")]
    pub chance: f32,
}

fn default_chance() -> f32 {
    1.0
}

/// 機械タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MachineType {
    Assembler,
    Mixer,
    Press,
    Furnace,
    Crusher,
    Centrifuge,
    ChemicalReactor,
    Packager,
    Custom(String),
}

impl Default for MachineType {
    fn default() -> Self {
        MachineType::Assembler
    }
}

/// レシピ定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDef {
    /// レシピID
    pub id: String,
    /// 使用する機械タイプ
    pub machine_type: MachineType,
    /// 材料リスト
    pub ingredients: Vec<Ingredient>,
    /// 生成物リスト
    pub results: Vec<Product>,
    /// 加工時間（秒）
    pub process_time: f32,
    /// 消費応力
    #[serde(default)]
    pub stress_impact: f32,
    /// 説明用i18nキー
    #[serde(default)]
    pub i18n_key: String,
}

impl RecipeDef {
    pub fn new(id: String) -> Self {
        Self {
            i18n_key: format!("recipe.{}", id),
            id,
            machine_type: MachineType::default(),
            ingredients: Vec::new(),
            results: Vec::new(),
            process_time: 1.0,
            stress_impact: 0.0,
        }
    }
}

/// アセットツリーのノード
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssetNode {
    /// ディレクトリ
    Directory {
        name: String,
        children: Vec<AssetNode>,
    },
    /// アイテム定義
    Item {
        id: String,
        name: String,
        path: String,
    },
    /// 流体定義
    Fluid {
        id: String,
        name: String,
    },
    /// 機械定義
    Machine {
        id: String,
        name: String,
        machine_type: MachineType,
    },
}

/// パレット用のアセットカタログ
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetCatalog {
    pub items: Vec<CatalogEntry>,
    pub fluids: Vec<CatalogEntry>,
    pub machines: Vec<MachineCatalogEntry>,
    pub tags: Vec<String>,
}

/// カタログエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub icon_path: Option<String>,
}

/// 機械カタログエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineCatalogEntry {
    pub id: String,
    pub name: String,
    pub machine_type: MachineType,
    pub input_slots: u32,
    pub output_slots: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_serialization() {
        let recipe = RecipeDef {
            id: "iron_plate".to_string(),
            machine_type: MachineType::Press,
            ingredients: vec![Ingredient {
                ingredient_type: IngredientType::Item("iron_ingot".to_string()),
                amount: 1,
            }],
            results: vec![Product {
                product_type: ProductType::Item("iron_plate".to_string()),
                amount: 1,
                chance: 1.0,
            }],
            process_time: 2.0,
            stress_impact: 4.0,
            i18n_key: "recipe.iron_plate".to_string(),
        };

        let json = serde_json::to_string_pretty(&recipe).unwrap();
        let deserialized: RecipeDef = serde_json::from_str(&json).unwrap();
        assert_eq!(recipe.id, deserialized.id);
    }
}
