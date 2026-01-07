//! Crafting system for player crafting and crafting stations

use bevy::prelude::*;
use std::collections::HashMap;

use crate::block_type::BlockType;
use crate::game_spec::recipes::{RecipeInput, RecipeOutput};

/// クラフト可能な場所
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum CraftingStation {
    /// 手持ちクラフト（常時利用可能）
    #[default]
    Hand,
    /// 作業台（より複雑なクラフト）
    Workbench,
    /// 鋳造台（金属加工）
    Forge,
}

/// クラフトレシピ
#[derive(Clone, Debug)]
pub struct CraftingRecipe {
    /// レシピ名
    pub name: &'static str,
    /// 入力アイテム
    pub inputs: Vec<RecipeInput>,
    /// 出力アイテム
    pub outputs: Vec<RecipeOutput>,
    /// クラフト時間（秒）
    pub craft_time: f32,
    /// 必要なクラフトステーション
    pub station: CraftingStation,
    /// アンロック条件（将来用）
    pub unlocked: bool,
}

impl CraftingRecipe {
    /// クラフトレシピビルダーを作成
    pub const fn builder(
        name: &'static str,
        station: CraftingStation,
        craft_time: f32,
    ) -> CraftingRecipeBuilder {
        CraftingRecipeBuilder {
            name,
            inputs: Vec::new(),
            outputs: Vec::new(),
            craft_time,
            station,
        }
    }

    /// 入力アイテムが足りているか確認
    pub fn can_craft(&self, inventory: &HashMap<BlockType, u32>) -> bool {
        for input in &self.inputs {
            let have = inventory.get(&input.item).copied().unwrap_or(0);
            if have < input.count {
                return false;
            }
        }
        true
    }

    /// 必要なアイテム一覧を取得
    pub fn required_items(&self) -> Vec<(BlockType, u32)> {
        self.inputs.iter().map(|i| (i.item, i.count)).collect()
    }
}

/// クラフトレシピビルダー
pub struct CraftingRecipeBuilder {
    name: &'static str,
    inputs: Vec<RecipeInput>,
    outputs: Vec<RecipeOutput>,
    craft_time: f32,
    station: CraftingStation,
}

impl CraftingRecipeBuilder {
    /// 入力アイテムを追加
    pub fn input(mut self, item: BlockType, count: u32) -> Self {
        self.inputs.push(RecipeInput::new(item, count, 0));
        self
    }

    /// 出力アイテムを追加
    pub fn output(mut self, item: BlockType, count: u32) -> Self {
        self.outputs.push(RecipeOutput::guaranteed(item, count));
        self
    }

    /// レシピを完成
    pub fn build(self) -> CraftingRecipe {
        CraftingRecipe {
            name: self.name,
            inputs: self.inputs,
            outputs: self.outputs,
            craft_time: self.craft_time,
            station: self.station,
            unlocked: true,
        }
    }
}

/// クラフトキュー内のアイテム
#[derive(Clone, Debug)]
pub struct CraftingJob {
    /// クラフト中のレシピ
    pub recipe_name: String,
    /// 残り時間
    pub remaining_time: f32,
    /// 総クラフト時間
    pub total_time: f32,
}

impl CraftingJob {
    /// 進捗率（0.0-1.0）
    pub fn progress(&self) -> f32 {
        if self.total_time <= 0.0 {
            return 1.0;
        }
        1.0 - (self.remaining_time / self.total_time)
    }

    /// 完了しているか
    pub fn is_complete(&self) -> bool {
        self.remaining_time <= 0.0
    }
}

/// プレイヤーのクラフト状態
#[derive(Component, Debug, Default)]
pub struct PlayerCrafting {
    /// 現在のクラフトジョブ
    pub current_job: Option<CraftingJob>,
    /// キュー（最大5個）
    pub queue: Vec<String>,
}

impl PlayerCrafting {
    /// クラフトキューに追加
    pub fn queue_craft(&mut self, recipe_name: &str) -> bool {
        if self.queue.len() >= 5 {
            return false;
        }
        self.queue.push(recipe_name.to_string());
        true
    }

    /// キューをキャンセル
    pub fn cancel_queue(&mut self, index: usize) -> bool {
        if index < self.queue.len() {
            self.queue.remove(index);
            true
        } else {
            false
        }
    }

    /// 現在のジョブをキャンセル
    pub fn cancel_current(&mut self) -> bool {
        if self.current_job.is_some() {
            self.current_job = None;
            true
        } else {
            false
        }
    }
}

/// クラフトレシピレジストリ
#[derive(Resource, Default)]
pub struct CraftingRegistry {
    /// レシピ（名前 -> レシピ）
    recipes: HashMap<String, CraftingRecipe>,
    /// ステーション別レシピリスト
    by_station: HashMap<CraftingStation, Vec<String>>,
}

impl CraftingRegistry {
    /// 新しいレジストリを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// レシピを登録
    pub fn register(&mut self, recipe: CraftingRecipe) {
        let name = recipe.name.to_string();
        let station = recipe.station;
        self.recipes.insert(name.clone(), recipe);

        self.by_station.entry(station).or_default().push(name);
    }

    /// レシピを取得
    pub fn get(&self, name: &str) -> Option<&CraftingRecipe> {
        self.recipes.get(name)
    }

    /// ステーション別レシピを取得
    pub fn get_by_station(&self, station: CraftingStation) -> Vec<&CraftingRecipe> {
        self.by_station
            .get(&station)
            .map(|names| names.iter().filter_map(|n| self.recipes.get(n)).collect())
            .unwrap_or_default()
    }

    /// 全レシピを取得
    pub fn all(&self) -> impl Iterator<Item = &CraftingRecipe> {
        self.recipes.values()
    }
}

/// クラフト開始イベント
#[derive(Event)]
pub struct StartCraftEvent {
    /// クラフターエンティティ
    pub crafter: Entity,
    /// レシピ名
    pub recipe_name: String,
}

/// クラフト完了イベント
#[derive(Event)]
pub struct CraftCompletedEvent {
    /// クラフターエンティティ
    pub crafter: Entity,
    /// レシピ名
    pub recipe_name: String,
    /// 出力アイテム
    pub outputs: Vec<(BlockType, u32)>,
}

/// クラフトプラグイン
pub struct CraftPlugin;

impl Plugin for CraftPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CraftingRegistry>()
            .add_event::<StartCraftEvent>()
            .add_event::<CraftCompletedEvent>()
            .add_systems(Startup, setup_default_recipes)
            .add_systems(Update, update_crafting);
    }
}

/// デフォルトレシピの登録
fn setup_default_recipes(mut registry: ResMut<CraftingRegistry>) {
    // 手持ちクラフト - 石のツール
    registry.register(
        CraftingRecipe::builder("stone_pickaxe", CraftingStation::Hand, 2.0)
            .input(BlockType::Stone, 3)
            .output(BlockType::Stone, 1) // TODO: ツールアイテム追加後に変更
            .build(),
    );

    // 手持ちクラフト - 松明（石炭使用）
    registry.register(
        CraftingRecipe::builder("torch", CraftingStation::Hand, 1.0)
            .input(BlockType::Coal, 1)
            .input(BlockType::Stone, 1)
            .output(BlockType::Coal, 4) // TODO: トーチアイテム追加後に変更
            .build(),
    );

    // 作業台クラフト - 鉄プレート
    registry.register(
        CraftingRecipe::builder("iron_plate", CraftingStation::Workbench, 3.0)
            .input(BlockType::IronIngot, 2)
            .output(BlockType::IronIngot, 1) // TODO: 鉄板アイテム追加後に変更
            .build(),
    );

    // 作業台クラフト - 銅線
    registry.register(
        CraftingRecipe::builder("copper_wire", CraftingStation::Workbench, 2.0)
            .input(BlockType::CopperIngot, 1)
            .output(BlockType::CopperIngot, 2) // TODO: 銅線アイテム追加後に変更
            .build(),
    );
}

/// クラフト進行システム
fn update_crafting(
    time: Res<Time>,
    mut crafting_query: Query<(Entity, &mut PlayerCrafting)>,
    registry: Res<CraftingRegistry>,
    mut completed_events: EventWriter<CraftCompletedEvent>,
) {
    for (entity, mut crafting) in crafting_query.iter_mut() {
        // 現在のジョブを更新
        if let Some(ref mut job) = crafting.current_job {
            job.remaining_time -= time.delta_secs();

            if job.is_complete() {
                // 完了イベントを発火
                if let Some(recipe) = registry.get(&job.recipe_name) {
                    let outputs: Vec<_> =
                        recipe.outputs.iter().map(|o| (o.item, o.count)).collect();

                    completed_events.send(CraftCompletedEvent {
                        crafter: entity,
                        recipe_name: job.recipe_name.clone(),
                        outputs,
                    });
                }
                crafting.current_job = None;
            }
        }

        // キューから次のジョブを開始
        if crafting.current_job.is_none() && !crafting.queue.is_empty() {
            let next_recipe_name = crafting.queue.remove(0);
            if let Some(recipe) = registry.get(&next_recipe_name) {
                crafting.current_job = Some(CraftingJob {
                    recipe_name: next_recipe_name,
                    remaining_time: recipe.craft_time,
                    total_time: recipe.craft_time,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crafting_station_default() {
        let station = CraftingStation::default();
        assert_eq!(station, CraftingStation::Hand);
    }

    #[test]
    fn test_crafting_recipe_builder() {
        let recipe = CraftingRecipe::builder("test", CraftingStation::Hand, 1.0)
            .input(BlockType::Stone, 2)
            .output(BlockType::IronOre, 1)
            .build();

        assert_eq!(recipe.name, "test");
        assert_eq!(recipe.inputs.len(), 1);
        assert_eq!(recipe.outputs.len(), 1);
        assert_eq!(recipe.craft_time, 1.0);
    }

    #[test]
    fn test_crafting_recipe_can_craft() {
        let recipe = CraftingRecipe::builder("test", CraftingStation::Hand, 1.0)
            .input(BlockType::Stone, 5)
            .input(BlockType::Coal, 2)
            .output(BlockType::IronOre, 1)
            .build();

        let mut inventory = HashMap::new();
        inventory.insert(BlockType::Stone, 10);
        inventory.insert(BlockType::Coal, 3);

        assert!(recipe.can_craft(&inventory));

        inventory.insert(BlockType::Coal, 1);
        assert!(!recipe.can_craft(&inventory));
    }

    #[test]
    fn test_crafting_job_progress() {
        let job = CraftingJob {
            recipe_name: "test".to_string(),
            remaining_time: 2.5,
            total_time: 5.0,
        };

        assert!((job.progress() - 0.5).abs() < 0.01);
        assert!(!job.is_complete());

        let complete_job = CraftingJob {
            recipe_name: "test".to_string(),
            remaining_time: 0.0,
            total_time: 5.0,
        };

        assert!(complete_job.is_complete());
    }

    #[test]
    fn test_player_crafting_queue() {
        let mut crafting = PlayerCrafting::default();

        assert!(crafting.queue_craft("recipe1"));
        assert!(crafting.queue_craft("recipe2"));
        assert_eq!(crafting.queue.len(), 2);

        assert!(crafting.cancel_queue(0));
        assert_eq!(crafting.queue.len(), 1);
        assert_eq!(crafting.queue[0], "recipe2");
    }

    #[test]
    fn test_player_crafting_queue_limit() {
        let mut crafting = PlayerCrafting::default();

        for i in 0..5 {
            assert!(crafting.queue_craft(&format!("recipe{}", i)));
        }

        // 6番目は追加できない
        assert!(!crafting.queue_craft("recipe5"));
        assert_eq!(crafting.queue.len(), 5);
    }

    #[test]
    fn test_crafting_registry() {
        let mut registry = CraftingRegistry::new();

        let recipe1 = CraftingRecipe::builder("hand_craft", CraftingStation::Hand, 1.0)
            .input(BlockType::Stone, 1)
            .output(BlockType::IronOre, 1)
            .build();

        let recipe2 = CraftingRecipe::builder("workbench_craft", CraftingStation::Workbench, 2.0)
            .input(BlockType::IronIngot, 2)
            .output(BlockType::Coal, 1)
            .build();

        registry.register(recipe1);
        registry.register(recipe2);

        assert!(registry.get("hand_craft").is_some());
        assert!(registry.get("workbench_craft").is_some());

        let hand_recipes = registry.get_by_station(CraftingStation::Hand);
        assert_eq!(hand_recipes.len(), 1);

        let workbench_recipes = registry.get_by_station(CraftingStation::Workbench);
        assert_eq!(workbench_recipes.len(), 1);
    }

    #[test]
    fn test_crafting_recipe_required_items() {
        let recipe = CraftingRecipe::builder("test", CraftingStation::Hand, 1.0)
            .input(BlockType::Stone, 5)
            .input(BlockType::Coal, 2)
            .output(BlockType::IronOre, 1)
            .build();

        let required = recipe.required_items();
        assert_eq!(required.len(), 2);
        assert!(required.contains(&(BlockType::Stone, 5)));
        assert!(required.contains(&(BlockType::Coal, 2)));
    }
}
