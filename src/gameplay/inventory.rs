// src/gameplay/inventory.rs
//! プレイヤーインベントリシステム
//! - PlayerInventory: プレイヤーの所持アイテム管理
//! - ItemData: アイテムの定義情報（カスタムプロパティ対応）
//! - InventoryOperations: インベントリ操作のヘルパー関数

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

/// YAML用アイテム定義構造体
#[derive(Debug, Clone, Deserialize)]
pub struct ItemDefinition {
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

/// アイテムの定義情報
/// YAMLファイルから読み込まれ、カスタムプロパティを柔軟に保持
#[derive(Debug, Clone)]
pub struct ItemData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String, // アイコンのパス
    pub max_stack: u32, // スタック上限（デフォルト: 999）
    pub custom_properties: HashMap<String, String>, // 拡張プロパティ（応力、燃焼値など）
}

impl ItemData {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            icon: String::new(),
            max_stack: 999,
            custom_properties: HashMap::new(),
        }
    }

    /// カスタムプロパティを追加
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_properties.insert(key.into(), value.into());
        self
    }

    /// カスタムプロパティの取得
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.custom_properties.get(key)
    }

    /// スタック上限を設定
    pub fn with_max_stack(mut self, max_stack: u32) -> Self {
        self.max_stack = max_stack;
        self
    }
}

impl From<ItemDefinition> for ItemData {
    fn from(def: ItemDefinition) -> Self {
        Self {
            id: def.id,
            name: def.name,
            description: def.description,
            icon: def.icon,
            max_stack: def.max_stack,
            custom_properties: def.properties,
        }
    }
}

/// アイテムレジストリ
#[derive(Resource, Default)]
pub struct ItemRegistry {
    pub items: HashMap<String, ItemData>,
}

impl ItemRegistry {
    pub fn register(&mut self, item: ItemData) {
        self.items.insert(item.id.clone(), item);
    }

    pub fn get(&self, id: &str) -> Option<&ItemData> {
        self.items.get(id)
    }
}

/// インベントリスロット
#[derive(Debug, Clone, PartialEq)]
pub struct InventorySlot {
    pub item_id: Option<String>,
    pub count: u32,
}

impl InventorySlot {
    pub fn empty() -> Self {
        Self {
            item_id: None,
            count: 0,
        }
    }

    pub fn new(item_id: String, count: u32) -> Self {
        Self {
            item_id: Some(item_id),
            count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.item_id.is_none() || self.count == 0
    }

    pub fn clear(&mut self) {
        self.item_id = None;
        self.count = 0;
    }
}

/// プレイヤーインベントリ
#[derive(Resource)]
pub struct PlayerInventory {
    pub slots: Vec<InventorySlot>,
    pub size: usize,
    pub selected_hotbar_slot: usize, // ホットバーの選択スロット (50-59)
}

impl PlayerInventory {
    pub fn new(size: usize) -> Self {
        Self {
            slots: vec![InventorySlot::empty(); size],
            size,
            selected_hotbar_slot: 50, // デフォルトはホットバーの最初のスロット
        }
    }

    /// アイテムを追加（スタック処理含む）
    pub fn add_item(&mut self, item_id: String, count: u32, registry: &ItemRegistry) -> u32 {
        if count == 0 {
            return 0;
        }

        let max_stack = registry
            .get(&item_id)
            .map(|data| data.max_stack)
            .unwrap_or(999);

        let mut remaining = count;

        // 既存のスタックに追加
        for slot in &mut self.slots {
            if let Some(id) = &slot.item_id {
                if id == &item_id && slot.count < max_stack {
                    let space = max_stack - slot.count;
                    let add_count = remaining.min(space);
                    slot.count += add_count;
                    remaining -= add_count;

                    if remaining == 0 {
                        return 0;
                    }
                }
            }
        }

        // 空きスロットに追加
        for slot in &mut self.slots {
            if slot.is_empty() {
                let add_count = remaining.min(max_stack);
                slot.item_id = Some(item_id.clone());
                slot.count = add_count;
                remaining -= add_count;

                if remaining == 0 {
                    return 0;
                }
            }
        }

        remaining
    }

    /// アイテムを削除
    pub fn remove_item(&mut self, item_id: &str, count: u32) -> u32 {
        let mut remaining = count;

        for slot in &mut self.slots {
            if let Some(id) = &slot.item_id {
                if id == item_id {
                    let remove_count = remaining.min(slot.count);
                    slot.count -= remove_count;
                    remaining -= remove_count;

                    if slot.count == 0 {
                        slot.clear();
                    }

                    if remaining == 0 {
                        return 0;
                    }
                }
            }
        }

        remaining
    }

    /// アイテムの所持数を取得
    pub fn count_item(&self, item_id: &str) -> u32 {
        self.slots
            .iter()
            .filter_map(|slot| {
                if let Some(id) = &slot.item_id {
                    if id == item_id {
                        return Some(slot.count);
                    }
                }
                None
            })
            .sum()
    }

    /// インベントリをID順にソート
    pub fn sort(&mut self) {
        // 非空スロットを抽出
        let mut non_empty: Vec<_> = self.slots.iter().filter(|s| !s.is_empty()).cloned().collect();

        // ID順にソート
        non_empty.sort_by(|a, b| {
            a.item_id.as_ref().cmp(&b.item_id.as_ref())
        });

        // スロットをクリアして再配置
        for slot in &mut self.slots {
            slot.clear();
        }

        for (i, slot) in non_empty.into_iter().enumerate() {
            if i < self.size {
                self.slots[i] = slot;
            }
        }
    }
}

/// 装備スロット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlotType {
    Head,
    Chest,
    Legs,
    Feet,
    Tool,
}

/// 装備管理
#[derive(Resource)]
pub struct EquipmentSlots {
    pub slots: HashMap<EquipmentSlotType, InventorySlot>,
}

impl EquipmentSlots {
    pub fn new() -> Self {
        let mut slots = HashMap::new();
        slots.insert(EquipmentSlotType::Head, InventorySlot::empty());
        slots.insert(EquipmentSlotType::Chest, InventorySlot::empty());
        slots.insert(EquipmentSlotType::Legs, InventorySlot::empty());
        slots.insert(EquipmentSlotType::Feet, InventorySlot::empty());
        slots.insert(EquipmentSlotType::Tool, InventorySlot::empty());
        Self { slots }
    }

    pub fn get(&self, slot_type: EquipmentSlotType) -> &InventorySlot {
        &self.slots[&slot_type]
    }

    pub fn get_mut(&mut self, slot_type: EquipmentSlotType) -> &mut InventorySlot {
        self.slots.get_mut(&slot_type).unwrap()
    }
}

impl Default for EquipmentSlots {
    fn default() -> Self {
        Self::new()
    }
}

/// インベントリプラグイン
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            .insert_resource(PlayerInventory::new(60)) // 60スロット (メイン5x10 + ホットバー1x10)
            .init_resource::<EquipmentSlots>()
            .add_systems(Startup, load_items);
    }
}

/// アイテム定義をロード（YAMLファイルから読み込み + フォールバック）
fn load_items(mut registry: ResMut<ItemRegistry>) {
    let path = "assets/data/items/core.yaml";
    let mut loaded_from_yaml = false;

    if let Ok(content) = fs::read_to_string(path) {
        match serde_yaml::from_str::<Vec<ItemDefinition>>(&content) {
            Ok(defs) => {
                for def in defs {
                    info!("Loaded item from YAML: {}", def.id);
                    registry.register(def.into());
                }
                loaded_from_yaml = true;
            }
            Err(e) => {
                error!("Failed to parse items YAML: {}", e);
            }
        }
    }

    // YAMLからロードできなかった場合はフォールバック
    if !loaded_from_yaml {
        info!("Using fallback item definitions");
        register_fallback_items(&mut registry);
    }

    info!("Loaded {} items", registry.items.len());
}

/// フォールバック用のハードコードアイテム
fn register_fallback_items(registry: &mut ItemRegistry) {
    registry.register(
        ItemData::new("raw_ore", "Raw Ore")
            .with_property("description", "Unprocessed ore from mining")
            .with_max_stack(999),
    );

    registry.register(
        ItemData::new("ingot", "Metal Ingot")
            .with_property("description", "Refined metal ingot")
            .with_property("stress_impact", "2.0")
            .with_max_stack(999),
    );

    registry.register(
        ItemData::new("iron_ingot", "Iron Ingot")
            .with_property("description", "Refined iron ingot")
            .with_property("stress_impact", "1.5")
            .with_max_stack(999),
    );

    registry.register(
        ItemData::new("coal", "Coal")
            .with_property("description", "Fuel for furnaces")
            .with_property("burn_time", "80")
            .with_max_stack(999),
    );

    registry.register(
        ItemData::new("miner", "Mining Drill")
            .with_property("description", "Automated mining machine")
            .with_property("placeable", "true")
            .with_max_stack(64),
    );

    registry.register(
        ItemData::new("conveyor", "Conveyor Belt")
            .with_property("description", "Transports items automatically")
            .with_property("placeable", "true")
            .with_max_stack(64),
    );

    registry.register(
        ItemData::new("assembler", "Assembler")
            .with_property("description", "Crafts items from recipes")
            .with_property("placeable", "true")
            .with_max_stack(64),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> ItemRegistry {
        let mut registry = ItemRegistry::default();
        registry.register(ItemData::new("test_item", "Test Item").with_max_stack(100));
        registry
    }

    #[test]
    fn test_add_item() {
        let mut inventory = PlayerInventory::new(10);
        let registry = test_registry();

        let remaining = inventory.add_item("test_item".to_string(), 50, &registry);
        assert_eq!(remaining, 0);
        assert_eq!(inventory.count_item("test_item"), 50);
    }

    #[test]
    fn test_add_item_with_stacking() {
        let mut inventory = PlayerInventory::new(10);
        let registry = test_registry();

        inventory.add_item("test_item".to_string(), 80, &registry);
        inventory.add_item("test_item".to_string(), 50, &registry);

        assert_eq!(inventory.count_item("test_item"), 130);
        // 80 + 20 = 100 (slot 0), 30 (slot 1)
        assert_eq!(inventory.slots[0].count, 100);
        assert_eq!(inventory.slots[1].count, 30);
    }

    #[test]
    fn test_remove_item() {
        let mut inventory = PlayerInventory::new(10);
        let registry = test_registry();

        inventory.add_item("test_item".to_string(), 50, &registry);
        let remaining = inventory.remove_item("test_item", 30);

        assert_eq!(remaining, 0);
        assert_eq!(inventory.count_item("test_item"), 20);
    }

    #[test]
    fn test_sort() {
        let mut inventory = PlayerInventory::new(10);
        inventory.slots[5] = InventorySlot::new("item_c".to_string(), 10);
        inventory.slots[2] = InventorySlot::new("item_a".to_string(), 5);
        inventory.slots[8] = InventorySlot::new("item_b".to_string(), 15);

        inventory.sort();

        assert_eq!(inventory.slots[0].item_id.as_deref(), Some("item_a"));
        assert_eq!(inventory.slots[1].item_id.as_deref(), Some("item_b"));
        assert_eq!(inventory.slots[2].item_id.as_deref(), Some("item_c"));
    }

    #[test]
    fn test_item_definition_conversion() {
        let def = ItemDefinition {
            id: "test".to_string(),
            name: "Test Item".to_string(),
            description: "A test item".to_string(),
            icon: "icons/test.png".to_string(),
            max_stack: 64,
            properties: HashMap::from([("custom_prop".to_string(), "value".to_string())]),
        };

        let item: ItemData = def.into();
        assert_eq!(item.id, "test");
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.description, "A test item");
        assert_eq!(item.max_stack, 64);
        assert_eq!(item.get_property("custom_prop"), Some(&"value".to_string()));
    }
}
