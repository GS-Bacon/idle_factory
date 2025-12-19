use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// アニメーション種別
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(tag = "type", content = "params")]
pub enum AnimationType {
    #[default]
    None,
    /// 回転アニメーション
    Rotational {
        /// 回転軸 (x, y, z)
        axis: [f32; 3],
        /// 回転速度 (degrees/sec)
        speed: f32,
    },
    /// リニアアニメーション (往復運動)
    Linear {
        /// 移動方向 (x, y, z)
        direction: [f32; 3],
        /// 移動距離
        distance: f32,
        /// 移動速度 (units/sec)
        speed: f32,
    },
    /// スケルタルアニメーション
    Skeletal {
        /// アニメーションファイルパス
        animation_path: String,
        /// ループするか
        looping: bool,
    },
}

/// アセット設定 / ビジュアル設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetConfig {
    /// アイコンパス (assets/textures/icons/ 以下の相対パス)
    pub icon_path: Option<String>,
    /// モデルパス (.glb, .vox等)
    pub model_path: Option<String>,
    /// アニメーション設定
    pub animation: AnimationType,
}

/// ItemVisuals は AssetConfig のエイリアス
pub type ItemVisuals = AssetConfig;

/// ローカライズエントリ
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalizationEntry {
    pub name: String,
    pub description: String,
}

/// ローカライズデータ (言語コード -> エントリ)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalizationData {
    pub ja: LocalizationEntry,
    pub en: LocalizationEntry,
}

/// アイテムカテゴリ
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ItemCategory {
    #[default]
    Item,
    Machine,
    Multiblock,
}

/// アイテムデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemData {
    /// アイテムID
    pub id: String,
    /// ローカライズキー (例: "item.iron_ore")
    pub i18n_key: String,
    /// アセット設定
    pub asset: AssetConfig,
    /// カスタムプロパティ
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
    /// カテゴリ
    #[serde(default)]
    pub category: ItemCategory,
}

impl ItemData {
    pub fn new(id: String) -> Self {
        Self {
            i18n_key: format!("item.{}", id),
            id,
            asset: AssetConfig::default(),
            properties: HashMap::new(),
            category: ItemCategory::Item,
        }
    }
}

/// ロケールファイル形式 (RON形式で保存)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocaleFile {
    pub entries: HashMap<String, LocalizationEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_type_serialization_json() {
        let rotational = AnimationType::Rotational {
            axis: [0.0, 1.0, 0.0],
            speed: 90.0,
        };
        let json = serde_json::to_string(&rotational).unwrap();
        let deserialized: AnimationType = serde_json::from_str(&json).unwrap();
        assert_eq!(rotational, deserialized);
    }

    #[test]
    fn test_animation_type_serialization_ron() {
        let rotational = AnimationType::Rotational {
            axis: [0.0, 1.0, 0.0],
            speed: 90.0,
        };
        let ron_str = ron::ser::to_string_pretty(&rotational, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: AnimationType = ron::from_str(&ron_str).unwrap();
        assert_eq!(rotational, deserialized);
    }

    #[test]
    fn test_animation_type_none_ron() {
        let none = AnimationType::None;
        let ron_str = ron::ser::to_string_pretty(&none, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: AnimationType = ron::from_str(&ron_str).unwrap();
        assert_eq!(none, deserialized);
    }

    #[test]
    fn test_animation_type_linear_ron() {
        let linear = AnimationType::Linear {
            direction: [1.0, 0.0, 0.0],
            distance: 2.0,
            speed: 1.5,
        };
        let ron_str = ron::ser::to_string_pretty(&linear, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: AnimationType = ron::from_str(&ron_str).unwrap();
        assert_eq!(linear, deserialized);
    }

    #[test]
    fn test_animation_type_skeletal_ron() {
        let skeletal = AnimationType::Skeletal {
            animation_path: "animations/walk.glb".to_string(),
            looping: true,
        };
        let ron_str = ron::ser::to_string_pretty(&skeletal, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: AnimationType = ron::from_str(&ron_str).unwrap();
        assert_eq!(skeletal, deserialized);
    }

    #[test]
    fn test_item_data_creation() {
        let item = ItemData::new("iron_ore".to_string());
        assert_eq!(item.id, "iron_ore");
        assert_eq!(item.i18n_key, "item.iron_ore");
        assert_eq!(item.category, ItemCategory::Item);
    }

    #[test]
    fn test_item_data_ron_serialization() {
        let item = ItemData::new("copper_ingot".to_string());
        let ron_str = ron::ser::to_string_pretty(&item, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: ItemData = ron::from_str(&ron_str).unwrap();
        assert_eq!(item.id, deserialized.id);
        assert_eq!(item.i18n_key, deserialized.i18n_key);
        assert_eq!(item.category, deserialized.category);
    }

    #[test]
    fn test_item_data_with_category_machine() {
        let mut item = ItemData::new("assembler".to_string());
        item.category = ItemCategory::Machine;
        item.i18n_key = "machine.assembler".to_string();

        let ron_str = ron::ser::to_string_pretty(&item, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: ItemData = ron::from_str(&ron_str).unwrap();
        assert_eq!(deserialized.category, ItemCategory::Machine);
        assert_eq!(deserialized.i18n_key, "machine.assembler");
    }

    #[test]
    fn test_item_data_with_category_multiblock() {
        let mut item = ItemData::new("furnace".to_string());
        item.category = ItemCategory::Multiblock;
        item.i18n_key = "multiblock.furnace".to_string();

        let ron_str = ron::ser::to_string_pretty(&item, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: ItemData = ron::from_str(&ron_str).unwrap();
        assert_eq!(deserialized.category, ItemCategory::Multiblock);
    }

    #[test]
    fn test_item_category_default() {
        let category: ItemCategory = Default::default();
        assert_eq!(category, ItemCategory::Item);
    }

    #[test]
    fn test_item_category_ron_lowercase() {
        // カテゴリがlowercaseでシリアライズされることを確認
        let ron_str = ron::ser::to_string(&ItemCategory::Machine).unwrap();
        assert!(ron_str.contains("machine") || ron_str == "machine");
    }

    #[test]
    fn test_item_data_with_asset_config() {
        let mut item = ItemData::new("test_item".to_string());
        item.asset = AssetConfig {
            icon_path: Some("textures/icons/test.png".to_string()),
            model_path: Some("models/test.glb".to_string()),
            animation: AnimationType::Rotational {
                axis: [0.0, 1.0, 0.0],
                speed: 45.0,
            },
        };

        let ron_str = ron::ser::to_string_pretty(&item, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: ItemData = ron::from_str(&ron_str).unwrap();
        assert_eq!(deserialized.asset.icon_path, Some("textures/icons/test.png".to_string()));
        assert_eq!(deserialized.asset.model_path, Some("models/test.glb".to_string()));
    }

    #[test]
    fn test_item_data_with_properties() {
        let mut item = ItemData::new("special_item".to_string());
        item.properties.insert("durability".to_string(), serde_json::json!(100));
        item.properties.insert("stackable".to_string(), serde_json::json!(true));

        let ron_str = ron::ser::to_string_pretty(&item, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: ItemData = ron::from_str(&ron_str).unwrap();
        assert_eq!(deserialized.properties.get("durability"), Some(&serde_json::json!(100)));
        assert_eq!(deserialized.properties.get("stackable"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn test_localization_entry_ron() {
        let entry = LocalizationEntry {
            name: "Iron Ore".to_string(),
            description: "Raw iron ore mined from the ground.".to_string(),
        };
        let ron_str = ron::ser::to_string_pretty(&entry, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: LocalizationEntry = ron::from_str(&ron_str).unwrap();
        assert_eq!(entry.name, deserialized.name);
        assert_eq!(entry.description, deserialized.description);
    }

    #[test]
    fn test_localization_data_ron() {
        let data = LocalizationData {
            ja: LocalizationEntry {
                name: "鉄鉱石".to_string(),
                description: "地中から採掘される生の鉄鉱石。".to_string(),
            },
            en: LocalizationEntry {
                name: "Iron Ore".to_string(),
                description: "Raw iron ore mined from the ground.".to_string(),
            },
        };
        let ron_str = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: LocalizationData = ron::from_str(&ron_str).unwrap();
        assert_eq!(data.ja.name, deserialized.ja.name);
        assert_eq!(data.en.name, deserialized.en.name);
    }

    #[test]
    fn test_locale_file_ron() {
        let mut file = LocaleFile::default();
        file.entries.insert("item.iron_ore".to_string(), LocalizationEntry {
            name: "Iron Ore".to_string(),
            description: "A piece of iron ore.".to_string(),
        });
        file.entries.insert("item.copper_ore".to_string(), LocalizationEntry {
            name: "Copper Ore".to_string(),
            description: "A piece of copper ore.".to_string(),
        });

        let ron_str = ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default()).unwrap();
        let deserialized: LocaleFile = ron::from_str(&ron_str).unwrap();
        assert_eq!(deserialized.entries.len(), 2);
        assert!(deserialized.entries.contains_key("item.iron_ore"));
        assert!(deserialized.entries.contains_key("item.copper_ore"));
    }

    #[test]
    fn test_asset_config_default() {
        let config: AssetConfig = Default::default();
        assert!(config.icon_path.is_none());
        assert!(config.model_path.is_none());
        assert_eq!(config.animation, AnimationType::None);
    }

    #[test]
    fn test_item_data_deserialize_without_category() {
        // category フィールドがないRONデータからデシリアライズできることを確認
        let ron_without_category = r#"(
            id: "old_item",
            i18n_key: "item.old_item",
            asset: (
                icon_path: None,
                model_path: None,
                animation: (type: None),
            ),
            properties: {},
        )"#;
        let item: ItemData = ron::from_str(ron_without_category).unwrap();
        assert_eq!(item.id, "old_item");
        assert_eq!(item.category, ItemCategory::Item); // デフォルト値
    }

    #[test]
    fn test_item_data_deserialize_with_category() {
        let ron_with_category = r#"(
            id: "machine_item",
            i18n_key: "machine.machine_item",
            asset: (
                icon_path: None,
                model_path: None,
                animation: (type: None),
            ),
            properties: {},
            category: machine,
        )"#;
        let item: ItemData = ron::from_str(ron_with_category).unwrap();
        assert_eq!(item.id, "machine_item");
        assert_eq!(item.category, ItemCategory::Machine);
    }
}
