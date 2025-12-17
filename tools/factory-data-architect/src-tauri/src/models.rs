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
}

impl ItemData {
    pub fn new(id: String) -> Self {
        Self {
            i18n_key: format!("item.{}", id),
            id,
            asset: AssetConfig::default(),
            properties: HashMap::new(),
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
    fn test_animation_type_serialization() {
        let rotational = AnimationType::Rotational {
            axis: [0.0, 1.0, 0.0],
            speed: 90.0,
        };
        let json = serde_json::to_string(&rotational).unwrap();
        let deserialized: AnimationType = serde_json::from_str(&json).unwrap();
        assert_eq!(rotational, deserialized);
    }

    #[test]
    fn test_item_data_creation() {
        let item = ItemData::new("iron_ore".to_string());
        assert_eq!(item.id, "iron_ore");
        assert_eq!(item.i18n_key, "item.iron_ore");
    }
}
