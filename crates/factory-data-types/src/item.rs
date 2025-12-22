//! Item type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Item category
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum ItemCategory {
    #[default]
    Item,
    Machine,
    Multiblock,
}

/// Animation type for items/machines
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(tag = "type")]
pub enum AnimationType {
    #[default]
    None,
    Rotational {
        axis: [f32; 3],
        speed: f32,
    },
    Linear {
        direction: [f32; 3],
        distance: f32,
        speed: f32,
    },
    Skeletal {
        animation_path: String,
        looping: bool,
    },
}

/// Asset configuration
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct AssetConfig {
    pub icon_path: Option<String>,
    pub model_path: Option<String>,
    #[serde(default)]
    pub animation: AnimationType,
}

/// Localization entry for a single language
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct LocalizationEntry {
    pub name: String,
    pub description: String,
}

/// Complete item data definition (Editor format)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ItemData {
    pub id: String,
    pub i18n_key: String,
    #[serde(default)]
    pub asset: AssetConfig,
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub category: ItemCategory,
    #[serde(default)]
    pub subcategory: Option<String>,
}

impl ItemData {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            i18n_key: format!("item.{}", &id),
            id,
            asset: AssetConfig::default(),
            properties: HashMap::new(),
            category: ItemCategory::default(),
            subcategory: None,
        }
    }

    /// Generate i18n_key based on category
    pub fn update_i18n_key(&mut self) {
        let prefix = match self.category {
            ItemCategory::Item => "item",
            ItemCategory::Machine => "machine",
            ItemCategory::Multiblock => "multiblock",
        };
        self.i18n_key = format!("{}.{}", prefix, self.id);
    }
}

/// Game-compatible item definition (for YAML export)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct GameItemData {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default = "default_max_stack")]
    pub max_stack: u32,
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

fn default_max_stack() -> u32 {
    999
}

impl From<&ItemData> for GameItemData {
    fn from(item: &ItemData) -> Self {
        Self {
            id: item.id.clone(),
            name: item.i18n_key.split('.').next_back().unwrap_or(&item.id).to_string(),
            description: String::new(),
            icon: item.asset.icon_path.clone().unwrap_or_default(),
            max_stack: 999,
            properties: item
                .properties
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_data_new() {
        let item = ItemData::new("iron_ore");
        assert_eq!(item.id, "iron_ore");
        assert_eq!(item.i18n_key, "item.iron_ore");
        assert_eq!(item.category, ItemCategory::Item);
    }

    #[test]
    fn test_update_i18n_key() {
        let mut item = ItemData::new("assembler");
        item.category = ItemCategory::Machine;
        item.update_i18n_key();
        assert_eq!(item.i18n_key, "machine.assembler");
    }

    #[test]
    fn test_game_item_conversion() {
        let item = ItemData::new("test_item");
        let game_item = GameItemData::from(&item);
        assert_eq!(game_item.id, "test_item");
        assert_eq!(game_item.name, "test_item");
        assert_eq!(game_item.max_stack, 999);
    }
}
