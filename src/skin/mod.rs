//! Player skin customization system

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// スキンカテゴリ
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum SkinCategory {
    /// 頭装備
    Head,
    /// 体装備
    Body,
    /// 足装備
    Legs,
    /// アクセサリ
    Accessory,
    /// 背中装備
    Back,
}

/// スキンアイテム定義
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkinItem {
    /// スキンID
    pub id: String,
    /// 表示名
    pub name: String,
    /// カテゴリ
    pub category: SkinCategory,
    /// モデルパス
    pub model_path: String,
    /// テクスチャパス
    pub texture_path: Option<String>,
    /// アンロック条件（実績ID等）
    pub unlock_condition: Option<String>,
    /// レアリティ（0-4: Common, Uncommon, Rare, Epic, Legendary）
    pub rarity: u8,
}

impl SkinItem {
    /// 新しいスキンアイテムを作成
    pub fn new(id: &str, name: &str, category: SkinCategory, model_path: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category,
            model_path: model_path.to_string(),
            texture_path: None,
            unlock_condition: None,
            rarity: 0,
        }
    }

    /// テクスチャパスを設定
    pub fn with_texture(mut self, path: &str) -> Self {
        self.texture_path = Some(path.to_string());
        self
    }

    /// アンロック条件を設定
    pub fn with_unlock(mut self, condition: &str) -> Self {
        self.unlock_condition = Some(condition.to_string());
        self
    }

    /// レアリティを設定
    pub fn with_rarity(mut self, rarity: u8) -> Self {
        self.rarity = rarity.min(4);
        self
    }

    /// レアリティ名を取得
    pub fn rarity_name(&self) -> &'static str {
        match self.rarity {
            0 => "Common",
            1 => "Uncommon",
            2 => "Rare",
            3 => "Epic",
            4 => "Legendary",
            _ => "Unknown",
        }
    }
}

/// プレイヤーの装備中スキン
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct EquippedSkins {
    /// カテゴリごとの装備ID
    pub slots: HashMap<SkinCategory, String>,
}

impl EquippedSkins {
    /// スキンを装備
    pub fn equip(&mut self, category: SkinCategory, skin_id: &str) {
        self.slots.insert(category, skin_id.to_string());
    }

    /// スキンを外す
    pub fn unequip(&mut self, category: SkinCategory) -> Option<String> {
        self.slots.remove(&category)
    }

    /// 装備中のスキンIDを取得
    pub fn get(&self, category: SkinCategory) -> Option<&String> {
        self.slots.get(&category)
    }

    /// 装備数を取得
    pub fn count(&self) -> usize {
        self.slots.len()
    }
}

/// プレイヤーのアンロック済みスキン
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnlockedSkins {
    /// アンロック済みスキンID
    pub unlocked: Vec<String>,
}

impl UnlockedSkins {
    /// スキンをアンロック
    pub fn unlock(&mut self, skin_id: &str) -> bool {
        if self.is_unlocked(skin_id) {
            return false;
        }
        self.unlocked.push(skin_id.to_string());
        true
    }

    /// アンロック済みか確認
    pub fn is_unlocked(&self, skin_id: &str) -> bool {
        self.unlocked.iter().any(|id| id == skin_id)
    }

    /// アンロック数を取得
    pub fn count(&self) -> usize {
        self.unlocked.len()
    }
}

/// スキンレジストリ
#[derive(Resource, Default)]
pub struct SkinRegistry {
    /// スキンアイテム（ID -> SkinItem）
    skins: HashMap<String, SkinItem>,
    /// カテゴリ別スキンリスト
    by_category: HashMap<SkinCategory, Vec<String>>,
}

impl SkinRegistry {
    /// 新しいレジストリを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// スキンを登録
    pub fn register(&mut self, skin: SkinItem) {
        let id = skin.id.clone();
        let category = skin.category;
        self.skins.insert(id.clone(), skin);
        self.by_category.entry(category).or_default().push(id);
    }

    /// スキンを取得
    pub fn get(&self, id: &str) -> Option<&SkinItem> {
        self.skins.get(id)
    }

    /// カテゴリ別スキンを取得
    pub fn get_by_category(&self, category: SkinCategory) -> Vec<&SkinItem> {
        self.by_category
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.skins.get(id)).collect())
            .unwrap_or_default()
    }

    /// 全スキンを取得
    pub fn all(&self) -> impl Iterator<Item = &SkinItem> {
        self.skins.values()
    }

    /// スキン総数を取得
    pub fn count(&self) -> usize {
        self.skins.len()
    }
}

/// スキン変更イベント
#[derive(Event)]
pub struct SkinChangedEvent {
    /// プレイヤーエンティティ
    pub player: Entity,
    /// 変更されたカテゴリ
    pub category: SkinCategory,
    /// 新しいスキンID（Noneの場合は外した）
    pub new_skin: Option<String>,
}

/// スキンアンロックイベント
#[derive(Event)]
pub struct SkinUnlockedEvent {
    /// プレイヤーエンティティ
    pub player: Entity,
    /// アンロックされたスキンID
    pub skin_id: String,
}

/// スキンプラグイン
pub struct SkinPlugin;

impl Plugin for SkinPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkinRegistry>()
            .add_event::<SkinChangedEvent>()
            .add_event::<SkinUnlockedEvent>()
            .add_systems(Startup, setup_default_skins);
    }
}

/// デフォルトスキンの登録
fn setup_default_skins(mut registry: ResMut<SkinRegistry>) {
    // 基本頭装備
    registry.register(SkinItem::new(
        "helmet_basic",
        "Basic Helmet",
        SkinCategory::Head,
        "models/skins/helmet_basic.glb",
    ));

    registry.register(
        SkinItem::new(
            "helmet_iron",
            "Iron Helmet",
            SkinCategory::Head,
            "models/skins/helmet_iron.glb",
        )
        .with_rarity(1)
        .with_unlock("craft_iron_ingot"),
    );

    // 基本体装備
    registry.register(SkinItem::new(
        "armor_basic",
        "Basic Armor",
        SkinCategory::Body,
        "models/skins/armor_basic.glb",
    ));

    registry.register(
        SkinItem::new(
            "armor_iron",
            "Iron Armor",
            SkinCategory::Body,
            "models/skins/armor_iron.glb",
        )
        .with_rarity(1)
        .with_unlock("craft_iron_ingot"),
    );

    // アクセサリ
    registry.register(
        SkinItem::new(
            "goggles",
            "Engineer Goggles",
            SkinCategory::Accessory,
            "models/skins/goggles.glb",
        )
        .with_rarity(2)
        .with_unlock("build_100_machines"),
    );

    // 背中装備
    registry.register(
        SkinItem::new(
            "backpack",
            "Storage Backpack",
            SkinCategory::Back,
            "models/skins/backpack.glb",
        )
        .with_rarity(1),
    );

    registry.register(
        SkinItem::new(
            "jetpack",
            "Jetpack",
            SkinCategory::Back,
            "models/skins/jetpack.glb",
        )
        .with_rarity(3)
        .with_unlock("unlock_all_machines"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_item_new() {
        let skin = SkinItem::new("test", "Test Skin", SkinCategory::Head, "models/test.glb");

        assert_eq!(skin.id, "test");
        assert_eq!(skin.name, "Test Skin");
        assert_eq!(skin.category, SkinCategory::Head);
        assert_eq!(skin.rarity, 0);
        assert!(skin.texture_path.is_none());
        assert!(skin.unlock_condition.is_none());
    }

    #[test]
    fn test_skin_item_builder() {
        let skin = SkinItem::new("test", "Test", SkinCategory::Body, "model.glb")
            .with_texture("texture.png")
            .with_unlock("achievement_1")
            .with_rarity(3);

        assert_eq!(skin.texture_path, Some("texture.png".to_string()));
        assert_eq!(skin.unlock_condition, Some("achievement_1".to_string()));
        assert_eq!(skin.rarity, 3);
    }

    #[test]
    fn test_skin_rarity_name() {
        let skin = SkinItem::new("test", "Test", SkinCategory::Head, "model.glb");
        assert_eq!(skin.rarity_name(), "Common");

        let skin = skin.with_rarity(4);
        assert_eq!(skin.rarity_name(), "Legendary");
    }

    #[test]
    fn test_skin_rarity_clamp() {
        let skin = SkinItem::new("test", "Test", SkinCategory::Head, "model.glb").with_rarity(10);
        assert_eq!(skin.rarity, 4); // 最大4にクランプ
    }

    #[test]
    fn test_equipped_skins() {
        let mut equipped = EquippedSkins::default();

        equipped.equip(SkinCategory::Head, "helmet_1");
        equipped.equip(SkinCategory::Body, "armor_1");

        assert_eq!(
            equipped.get(SkinCategory::Head),
            Some(&"helmet_1".to_string())
        );
        assert_eq!(equipped.count(), 2);

        let removed = equipped.unequip(SkinCategory::Head);
        assert_eq!(removed, Some("helmet_1".to_string()));
        assert!(equipped.get(SkinCategory::Head).is_none());
    }

    #[test]
    fn test_unlocked_skins() {
        let mut unlocked = UnlockedSkins::default();

        assert!(!unlocked.is_unlocked("skin_1"));

        assert!(unlocked.unlock("skin_1"));
        assert!(unlocked.is_unlocked("skin_1"));

        // 重複アンロックは失敗
        assert!(!unlocked.unlock("skin_1"));
        assert_eq!(unlocked.count(), 1);
    }

    #[test]
    fn test_skin_registry() {
        let mut registry = SkinRegistry::new();

        let skin1 = SkinItem::new("head_1", "Head 1", SkinCategory::Head, "head.glb");
        let skin2 = SkinItem::new("body_1", "Body 1", SkinCategory::Body, "body.glb");
        let skin3 = SkinItem::new("head_2", "Head 2", SkinCategory::Head, "head2.glb");

        registry.register(skin1);
        registry.register(skin2);
        registry.register(skin3);

        assert_eq!(registry.count(), 3);
        assert!(registry.get("head_1").is_some());

        let head_skins = registry.get_by_category(SkinCategory::Head);
        assert_eq!(head_skins.len(), 2);

        let body_skins = registry.get_by_category(SkinCategory::Body);
        assert_eq!(body_skins.len(), 1);
    }

    #[test]
    fn test_skin_category_values() {
        // 全カテゴリをカバー
        let categories = [
            SkinCategory::Head,
            SkinCategory::Body,
            SkinCategory::Legs,
            SkinCategory::Accessory,
            SkinCategory::Back,
        ];

        for cat in categories {
            let skin = SkinItem::new("test", "Test", cat, "model.glb");
            assert_eq!(skin.category, cat);
        }
    }
}
