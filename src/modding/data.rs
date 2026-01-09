//! Data-driven mod loading from TOML/JSON files

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::ItemId;

/// Modデータファイル形式
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataFormat {
    /// TOML形式
    Toml,
    /// JSON形式
    Json,
}

impl DataFormat {
    /// 拡張子から形式を判定
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "toml" => Some(DataFormat::Toml),
            "json" => Some(DataFormat::Json),
            _ => None,
        }
    }
}

/// アイテム定義（データ駆動）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemDefinition {
    /// アイテムID
    pub id: String,
    /// 表示名
    pub name: String,
    /// 短縮名
    #[serde(default)]
    pub short_name: String,
    /// 説明
    #[serde(default)]
    pub description: String,
    /// スタックサイズ
    #[serde(default = "default_stack_size")]
    pub stack_size: u32,
    /// カテゴリ
    #[serde(default)]
    pub category: String,
    /// 設置可能か
    #[serde(default)]
    pub is_placeable: bool,
    /// 硬さ（採掘時間に影響）
    #[serde(default = "default_hardness")]
    pub hardness: f32,
    /// 色 [R, G, B]
    #[serde(default)]
    pub color: Option<[f32; 3]>,
    /// アイコンパス
    #[serde(default)]
    pub icon: String,
    /// モデルパス
    #[serde(default)]
    pub model: String,
    /// カスタムプロパティ
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
    /// タグ（Forge Ore Dictionary相当）
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_stack_size() -> u32 {
    64
}

fn default_hardness() -> f32 {
    1.0
}

impl ItemDefinition {
    /// 新しいアイテム定義を作成
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            short_name: String::new(),
            description: String::new(),
            stack_size: 64,
            category: String::new(),
            is_placeable: false,
            hardness: 1.0,
            color: None,
            icon: String::new(),
            model: String::new(),
            properties: HashMap::new(),
            tags: Vec::new(),
        }
    }
}

/// 機械定義（データ駆動）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MachineDefinition {
    /// 機械ID
    pub id: String,
    /// 表示名
    pub name: String,
    /// 処理時間（秒）
    #[serde(default = "default_process_time")]
    pub process_time: f32,
    /// 入力ポート数
    #[serde(default = "default_one")]
    pub input_ports: u8,
    /// 出力ポート数
    #[serde(default = "default_one")]
    pub output_ports: u8,
    /// 燃料が必要か
    #[serde(default)]
    pub requires_fuel: bool,
    /// 電力消費
    #[serde(default)]
    pub power_consumption: f32,
    /// モデルパス
    #[serde(default)]
    pub model: String,
}

fn default_process_time() -> f32 {
    2.0
}

fn default_one() -> u8 {
    1
}

impl MachineDefinition {
    /// 新しい機械定義を作成
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            process_time: 2.0,
            input_ports: 1,
            output_ports: 1,
            requires_fuel: false,
            power_consumption: 0.0,
            model: String::new(),
        }
    }
}

/// レシピ定義（データ駆動）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipeDefinition {
    /// レシピID
    pub id: String,
    /// 機械タイプ
    pub machine: String,
    /// 入力アイテム（ID -> 個数）
    pub inputs: HashMap<String, u32>,
    /// 出力アイテム（ID -> 個数）
    pub outputs: HashMap<String, u32>,
    /// 処理時間（秒、Noneの場合は機械のデフォルト）
    #[serde(default)]
    pub process_time: Option<f32>,
    /// 燃料消費（ID -> 個数）
    #[serde(default)]
    pub fuel: HashMap<String, u32>,
}

impl RecipeDefinition {
    /// 新しいレシピ定義を作成
    pub fn new(id: &str, machine: &str) -> Self {
        Self {
            id: id.to_string(),
            machine: machine.to_string(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            process_time: None,
            fuel: HashMap::new(),
        }
    }

    /// 入力を追加
    pub fn with_input(mut self, item_id: &str, count: u32) -> Self {
        self.inputs.insert(item_id.to_string(), count);
        self
    }

    /// 出力を追加
    pub fn with_output(mut self, item_id: &str, count: u32) -> Self {
        self.outputs.insert(item_id.to_string(), count);
        self
    }
}

/// Modデータパック
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModDataPack {
    /// アイテム定義
    #[serde(default)]
    pub items: Vec<ItemDefinition>,
    /// 機械定義
    #[serde(default)]
    pub machines: Vec<MachineDefinition>,
    /// レシピ定義
    #[serde(default)]
    pub recipes: Vec<RecipeDefinition>,
}

/// items.toml形式
#[derive(Debug, Deserialize)]
struct ItemsToml {
    #[serde(default, rename = "item")]
    items: Vec<ItemDefinition>,
}

/// machines.toml形式
#[derive(Debug, Deserialize)]
struct MachinesToml {
    #[serde(default, rename = "machine")]
    machines: Vec<MachineDefinition>,
}

/// recipes.toml形式
#[derive(Debug, Deserialize)]
struct RecipesToml {
    #[serde(default, rename = "recipe")]
    recipes: Vec<RecipeDefinition>,
}

impl ModDataPack {
    /// 新しいデータパックを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// JSONから読み込み
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// TOMLから読み込み（統合形式）
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// items.tomlから読み込み
    pub fn load_items_toml(toml_str: &str) -> Result<Vec<ItemDefinition>, toml::de::Error> {
        let parsed: ItemsToml = toml::from_str(toml_str)?;
        Ok(parsed.items)
    }

    /// machines.tomlから読み込み
    pub fn load_machines_toml(toml_str: &str) -> Result<Vec<MachineDefinition>, toml::de::Error> {
        let parsed: MachinesToml = toml::from_str(toml_str)?;
        Ok(parsed.machines)
    }

    /// recipes.tomlから読み込み
    pub fn load_recipes_toml(toml_str: &str) -> Result<Vec<RecipeDefinition>, toml::de::Error> {
        let parsed: RecipesToml = toml::from_str(toml_str)?;
        Ok(parsed.recipes)
    }

    /// JSONに書き出し
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// アイテムを追加
    pub fn add_item(&mut self, item: ItemDefinition) {
        self.items.push(item);
    }

    /// 機械を追加
    pub fn add_machine(&mut self, machine: MachineDefinition) {
        self.machines.push(machine);
    }

    /// レシピを追加
    pub fn add_recipe(&mut self, recipe: RecipeDefinition) {
        self.recipes.push(recipe);
    }

    /// アイテム数を取得
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// 機械数を取得
    pub fn machine_count(&self) -> usize {
        self.machines.len()
    }

    /// レシピ数を取得
    pub fn recipe_count(&self) -> usize {
        self.recipes.len()
    }

    /// Modディレクトリからデータパックを読み込み
    pub fn load_from_directory(mod_path: &std::path::Path) -> Result<Self, ModLoadError> {
        let mut pack = ModDataPack::new();

        // items.toml
        let items_path = mod_path.join("items.toml");
        if items_path.exists() {
            let content = std::fs::read_to_string(&items_path)
                .map_err(|e| ModLoadError::IoError(items_path.clone(), e.to_string()))?;
            let items = Self::load_items_toml(&content)
                .map_err(|e| ModLoadError::ParseError(items_path.clone(), e.to_string()))?;
            pack.items = items;
        }

        // machines.toml
        let machines_path = mod_path.join("machines.toml");
        if machines_path.exists() {
            let content = std::fs::read_to_string(&machines_path)
                .map_err(|e| ModLoadError::IoError(machines_path.clone(), e.to_string()))?;
            let machines = Self::load_machines_toml(&content)
                .map_err(|e| ModLoadError::ParseError(machines_path.clone(), e.to_string()))?;
            pack.machines = machines;
        }

        // recipes.toml
        let recipes_path = mod_path.join("recipes.toml");
        if recipes_path.exists() {
            let content = std::fs::read_to_string(&recipes_path)
                .map_err(|e| ModLoadError::IoError(recipes_path.clone(), e.to_string()))?;
            let recipes = Self::load_recipes_toml(&content)
                .map_err(|e| ModLoadError::ParseError(recipes_path.clone(), e.to_string()))?;
            pack.recipes = recipes;
        }

        Ok(pack)
    }
}

/// Modロードエラー
#[derive(Debug)]
pub enum ModLoadError {
    /// IOエラー
    IoError(PathBuf, String),
    /// パースエラー
    ParseError(PathBuf, String),
    /// Mod情報が見つからない
    ModInfoNotFound(PathBuf),
}

/// データローダー
pub struct DataLoader {
    /// ベースパス
    base_path: PathBuf,
}

impl DataLoader {
    /// 新しいローダーを作成
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Mod IDからパスを構築
    pub fn mod_path(&self, mod_id: &str) -> PathBuf {
        self.base_path.join("mods").join(mod_id)
    }

    /// データパックのパスを構築
    pub fn data_pack_path(&self, mod_id: &str) -> PathBuf {
        self.mod_path(mod_id).join("data.json")
    }
}

/// アイテムID変換ヘルパー
pub fn parse_item_id(id: &str) -> Option<ItemId> {
    crate::core::items::by_name(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_format_from_extension() {
        assert_eq!(DataFormat::from_extension("toml"), Some(DataFormat::Toml));
        assert_eq!(DataFormat::from_extension("json"), Some(DataFormat::Json));
        assert_eq!(DataFormat::from_extension("TOML"), Some(DataFormat::Toml));
        assert_eq!(DataFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_item_definition_new() {
        let item = ItemDefinition::new("custom:item", "Custom Item");

        assert_eq!(item.id, "custom:item");
        assert_eq!(item.name, "Custom Item");
        assert_eq!(item.stack_size, 64);
    }

    #[test]
    fn test_machine_definition_new() {
        let machine = MachineDefinition::new("custom:machine", "Custom Machine");

        assert_eq!(machine.id, "custom:machine");
        assert_eq!(machine.process_time, 2.0);
        assert_eq!(machine.input_ports, 1);
        assert_eq!(machine.output_ports, 1);
    }

    #[test]
    fn test_recipe_definition_builder() {
        let recipe = RecipeDefinition::new("custom:recipe", "furnace")
            .with_input("iron_ore", 1)
            .with_output("iron_ingot", 1);

        assert_eq!(recipe.id, "custom:recipe");
        assert_eq!(recipe.inputs.get("iron_ore"), Some(&1));
        assert_eq!(recipe.outputs.get("iron_ingot"), Some(&1));
    }

    #[test]
    fn test_mod_data_pack() {
        let mut pack = ModDataPack::new();

        pack.add_item(ItemDefinition::new("item1", "Item 1"));
        pack.add_item(ItemDefinition::new("item2", "Item 2"));
        pack.add_machine(MachineDefinition::new("machine1", "Machine 1"));
        pack.add_recipe(RecipeDefinition::new("recipe1", "furnace"));

        assert_eq!(pack.item_count(), 2);
        assert_eq!(pack.machine_count(), 1);
        assert_eq!(pack.recipe_count(), 1);
    }

    #[test]
    fn test_mod_data_pack_json() {
        let mut pack = ModDataPack::new();
        pack.add_item(ItemDefinition::new("test:item", "Test Item"));

        let json = pack.to_json().unwrap();
        let loaded = ModDataPack::from_json(&json).unwrap();

        assert_eq!(loaded.item_count(), 1);
        assert_eq!(loaded.items[0].id, "test:item");
    }

    #[test]
    fn test_data_loader_paths() {
        let loader = DataLoader::new(PathBuf::from("/game"));

        assert_eq!(
            loader.mod_path("test.mod"),
            PathBuf::from("/game/mods/test.mod")
        );
        assert_eq!(
            loader.data_pack_path("test.mod"),
            PathBuf::from("/game/mods/test.mod/data.json")
        );
    }

    #[test]
    fn test_item_definition_json_roundtrip() {
        let item = ItemDefinition::new("test:item", "Test Item");
        let json = serde_json::to_string(&item).unwrap();
        let loaded: ItemDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.id, item.id);
        assert_eq!(loaded.name, item.name);
    }

    #[test]
    fn test_load_items_toml() {
        let toml_str = r#"
[[item]]
id = "stone"
name = "Stone"
short_name = "Stn"
description = "Basic building material"
stack_size = 999
category = "terrain"
is_placeable = true
hardness = 1.0
color = [0.5, 0.5, 0.5]

[[item]]
id = "iron_ore"
name = "Iron Ore"
stack_size = 999
category = "ore"
"#;

        let items = ModDataPack::load_items_toml(toml_str).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "stone");
        assert_eq!(items[0].short_name, "Stn");
        assert!(items[0].is_placeable);
        assert_eq!(items[0].hardness, 1.0);
        assert_eq!(items[0].color, Some([0.5, 0.5, 0.5]));
        assert_eq!(items[1].id, "iron_ore");
    }

    #[test]
    fn test_load_machines_toml() {
        let toml_str = r#"
[[machine]]
id = "miner"
name = "Miner"
process_time = 1.5
input_ports = 0
output_ports = 1
"#;

        let machines = ModDataPack::load_machines_toml(toml_str).unwrap();
        assert_eq!(machines.len(), 1);
        assert_eq!(machines[0].id, "miner");
        assert_eq!(machines[0].process_time, 1.5);
        assert_eq!(machines[0].output_ports, 1);
    }

    #[test]
    fn test_load_recipes_toml() {
        let toml_str = r#"
[[recipe]]
id = "iron_smelting"
machine = "furnace"
process_time = 2.0

[recipe.inputs]
iron_ore = 1

[recipe.outputs]
iron_ingot = 1

[recipe.fuel]
coal = 1
"#;

        let recipes = ModDataPack::load_recipes_toml(toml_str).unwrap();
        assert_eq!(recipes.len(), 1);
        assert_eq!(recipes[0].id, "iron_smelting");
        assert_eq!(recipes[0].machine, "furnace");
        assert_eq!(recipes[0].inputs.get("iron_ore"), Some(&1));
        assert_eq!(recipes[0].outputs.get("iron_ingot"), Some(&1));
        assert_eq!(recipes[0].fuel.get("coal"), Some(&1));
    }

    #[test]
    fn test_load_base_mod_from_directory() {
        // Test loading from the actual mods/base directory if it exists
        let base_path = std::path::PathBuf::from("mods/base");
        if base_path.exists() {
            let pack = ModDataPack::load_from_directory(&base_path).unwrap();
            assert!(pack.item_count() > 0, "Base mod should have items");
            assert!(pack.machine_count() > 0, "Base mod should have machines");
            assert!(pack.recipe_count() > 0, "Base mod should have recipes");
        }
    }
}
