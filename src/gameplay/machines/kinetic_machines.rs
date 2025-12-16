// src/gameplay/machines/kinetic_machines.rs
//! 物理加工機械（Kinetic Machines）
//!
//! 回転動力を使用してアイテムを加工する機械群。
//! - MechanicalPress: プレス加工
//! - Crusher: 粉砕
//! - MechanicalSaw: 切断
//! - Mixer: 混合
//! - WireDrawer: 伸線
//!
//! 全ての機械は共通の `process_kinetic_machines` システムで処理される。

use bevy::prelude::*;
use std::collections::HashMap;

use super::machine_components::*;
use super::recipe_system::{RecipeManager, WorkType};
use crate::gameplay::power::{PowerConsumer, PowerNode, PowerNetworkGroups};

// ========================================
// 機械コンポーネント
// ========================================

/// 工作機械が処理する作業種別
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessingWorkType(pub WorkType);

/// 現在選択中のレシピID
#[derive(Component, Debug, Clone, Default)]
pub struct SelectedRecipe(pub Option<String>);

/// アニメーション状態
#[derive(Component, Debug, Clone, Default)]
pub struct MachineAnimation {
    /// 現在のアニメーションフレーム
    pub frame: u32,
    /// 最大フレーム数
    pub max_frames: u32,
    /// 経過時間
    pub timer: f32,
    /// 1フレームあたりの時間
    pub frame_duration: f32,
}

impl MachineAnimation {
    pub fn new(max_frames: u32, frame_duration: f32) -> Self {
        Self {
            frame: 0,
            max_frames,
            timer: 0.0,
            frame_duration,
        }
    }

    /// アニメーションを進める
    pub fn tick(&mut self, delta: f32) {
        self.timer += delta;
        // 複数フレーム分進む可能性があるのでwhileループ
        while self.timer >= self.frame_duration {
            self.timer -= self.frame_duration;
            self.frame = (self.frame + 1) % self.max_frames;
        }
    }

    /// アニメーションをリセット
    pub fn reset(&mut self) {
        self.frame = 0;
        self.timer = 0.0;
    }
}

// ========================================
// 機械タイプマーカー
// ========================================

/// プレス機マーカー
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MechanicalPress;

/// 粉砕機マーカー
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Crusher;

/// 自動ノコギリマーカー
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MechanicalSaw;

/// ミキサーマーカー
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Mixer;

/// 伸線機マーカー
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct WireDrawer;

// ========================================
// Bundles
// ========================================

/// プレス機Bundle
#[derive(Bundle)]
pub struct MechanicalPressBundle {
    pub marker: MechanicalPress,
    pub kinetic: KineticMachine,
    pub work_type: ProcessingWorkType,
    pub input: InputInventory,
    pub output: OutputInventory,
    pub state: MachineState,
    pub stress: StressImpact,
    pub recipe: SelectedRecipe,
    pub animation: MachineAnimation,
    pub power_consumer: PowerConsumer,
}

impl Default for MechanicalPressBundle {
    fn default() -> Self {
        Self {
            marker: MechanicalPress,
            kinetic: KineticMachine,
            work_type: ProcessingWorkType(WorkType::Pressing),
            input: InputInventory::new(1),
            output: OutputInventory::new(1),
            state: MachineState::Idle,
            stress: StressImpact::new(8.0),
            recipe: SelectedRecipe::default(),
            animation: MachineAnimation::new(8, 0.05), // 8フレーム、50ms/フレーム
            power_consumer: PowerConsumer {
                stress_impact: 8.0,
                is_active: false,
                current_speed_received: 0.0,
            },
        }
    }
}

/// 粉砕機Bundle
#[derive(Bundle)]
pub struct CrusherBundle {
    pub marker: Crusher,
    pub kinetic: KineticMachine,
    pub work_type: ProcessingWorkType,
    pub input: InputInventory,
    pub output: OutputInventory,
    pub state: MachineState,
    pub stress: StressImpact,
    pub recipe: SelectedRecipe,
    pub animation: MachineAnimation,
    pub power_consumer: PowerConsumer,
}

impl Default for CrusherBundle {
    fn default() -> Self {
        Self {
            marker: Crusher,
            kinetic: KineticMachine,
            work_type: ProcessingWorkType(WorkType::Crushing),
            input: InputInventory::new(1),
            output: OutputInventory::new(2),
            state: MachineState::Idle,
            stress: StressImpact::new(12.0),
            recipe: SelectedRecipe::default(),
            animation: MachineAnimation::new(12, 0.04),
            power_consumer: PowerConsumer {
                stress_impact: 12.0,
                is_active: false,
                current_speed_received: 0.0,
            },
        }
    }
}

/// 自動ノコギリBundle
#[derive(Bundle)]
pub struct MechanicalSawBundle {
    pub marker: MechanicalSaw,
    pub kinetic: KineticMachine,
    pub work_type: ProcessingWorkType,
    pub input: InputInventory,
    pub output: OutputInventory,
    pub state: MachineState,
    pub stress: StressImpact,
    pub recipe: SelectedRecipe,
    pub animation: MachineAnimation,
    pub power_consumer: PowerConsumer,
}

impl Default for MechanicalSawBundle {
    fn default() -> Self {
        Self {
            marker: MechanicalSaw,
            kinetic: KineticMachine,
            work_type: ProcessingWorkType(WorkType::Cutting),
            input: InputInventory::new(1),
            output: OutputInventory::new(4),
            state: MachineState::Idle,
            stress: StressImpact::new(4.0),
            recipe: SelectedRecipe::default(),
            animation: MachineAnimation::new(6, 0.03),
            power_consumer: PowerConsumer {
                stress_impact: 4.0,
                is_active: false,
                current_speed_received: 0.0,
            },
        }
    }
}

/// ミキサーBundle
#[derive(Bundle)]
pub struct MixerBundle {
    pub marker: Mixer,
    pub kinetic: KineticMachine,
    pub work_type: ProcessingWorkType,
    pub input: InputInventory,
    pub output: OutputInventory,
    pub fluid_input: FluidTank,
    pub fluid_output: FluidTank,
    pub state: MachineState,
    pub stress: StressImpact,
    pub recipe: SelectedRecipe,
    pub animation: MachineAnimation,
    pub power_consumer: PowerConsumer,
}

impl Default for MixerBundle {
    fn default() -> Self {
        Self {
            marker: Mixer,
            kinetic: KineticMachine,
            work_type: ProcessingWorkType(WorkType::Mixing),
            input: InputInventory::new(4),
            output: OutputInventory::new(2),
            fluid_input: FluidTank::new(4000.0),
            fluid_output: FluidTank::new(4000.0),
            state: MachineState::Idle,
            stress: StressImpact::new(16.0),
            recipe: SelectedRecipe::default(),
            animation: MachineAnimation::new(16, 0.06),
            power_consumer: PowerConsumer {
                stress_impact: 16.0,
                is_active: false,
                current_speed_received: 0.0,
            },
        }
    }
}

/// 伸線機Bundle
#[derive(Bundle)]
pub struct WireDrawerBundle {
    pub marker: WireDrawer,
    pub kinetic: KineticMachine,
    pub work_type: ProcessingWorkType,
    pub input: InputInventory,
    pub output: OutputInventory,
    pub state: MachineState,
    pub stress: StressImpact,
    pub recipe: SelectedRecipe,
    pub animation: MachineAnimation,
    pub power_consumer: PowerConsumer,
}

impl Default for WireDrawerBundle {
    fn default() -> Self {
        Self {
            marker: WireDrawer,
            kinetic: KineticMachine,
            work_type: ProcessingWorkType(WorkType::WireDrawing),
            input: InputInventory::new(1),
            output: OutputInventory::new(2),
            state: MachineState::Idle,
            stress: StressImpact::new(6.0),
            recipe: SelectedRecipe::default(),
            animation: MachineAnimation::new(10, 0.05),
            power_consumer: PowerConsumer {
                stress_impact: 6.0,
                is_active: false,
                current_speed_received: 0.0,
            },
        }
    }
}

// ========================================
// 汎用加工システム
// ========================================

/// 工作機械の汎用処理システム
///
/// 全てのKineticMachineを処理する。
/// 1. 動力チェック（回転速度が0、または応力過多なら停止）
/// 2. 材料チェック（レシピに必要な材料があるか）
/// 3. 加工進行（タイマー更新）
/// 4. 完了処理（入力消費、出力生成）
pub fn process_kinetic_machines(
    mut query: Query<(
        Entity,
        &ProcessingWorkType,
        &mut InputInventory,
        &mut OutputInventory,
        &mut MachineState,
        &mut SelectedRecipe,
        &mut MachineAnimation,
        &PowerConsumer,
        Option<&PowerNode>,
    ), With<KineticMachine>>,
    recipe_manager: Res<RecipeManager>,
    power_groups: Res<PowerNetworkGroups>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (
        _entity,
        work_type,
        mut input,
        mut output,
        mut state,
        mut selected_recipe,
        mut animation,
        power_consumer,
        power_node,
    ) in &mut query {
        // --- 動力チェック ---
        let has_power = check_power(power_consumer, power_node, &power_groups);
        if !has_power {
            if *state != MachineState::NoPower {
                *state = MachineState::NoPower;
                animation.reset();
            }
            continue;
        }

        // NoPowerから復帰
        if *state == MachineState::NoPower {
            *state = MachineState::Idle;
        }

        // --- 出力満杯チェック ---
        if output.is_full() {
            if *state != MachineState::Jammed {
                *state = MachineState::Jammed;
                animation.reset();
            }
            continue;
        }

        // Jammedから復帰
        if *state == MachineState::Jammed {
            *state = MachineState::Idle;
        }

        // --- 加工中の処理 ---
        if let MachineState::Processing { elapsed, total } = &mut *state {
            *elapsed += dt;
            animation.tick(dt);

            if *elapsed >= *total {
                // 加工完了
                if let Some(recipe) = selected_recipe.0.as_ref()
                    .and_then(|id| recipe_manager.get(id))
                {
                    // 入力消費
                    for input_item in &recipe.inputs {
                        input.consume(&input_item.item, input_item.count);
                    }
                    // 出力生成
                    for output_item in &recipe.outputs {
                        output.add_item(&output_item.item, output_item.count);
                    }
                    info!(
                        "[KineticMachine] Crafted {} (recipe: {})",
                        recipe.name,
                        recipe.id
                    );
                }
                *state = MachineState::Idle;
                animation.reset();
            }
            continue;
        }

        // --- Idle時: 新しいレシピを探す ---
        if *state == MachineState::Idle {
            // 入力アイテムを集計
            let mut available_items: HashMap<String, u32> = HashMap::new();
            for slot in &input.slots {
                if let Some(item_id) = &slot.item_id {
                    *available_items.entry(item_id.clone()).or_insert(0) += slot.count;
                }
            }

            // レシピ検索
            if let Some(recipe) = recipe_manager.find_matching_recipe(work_type.0, &available_items) {
                selected_recipe.0 = Some(recipe.id.clone());
                state.start_processing(recipe.craft_time);
                info!(
                    "[KineticMachine] Starting recipe: {} (time: {}s)",
                    recipe.name,
                    recipe.craft_time
                );
            }
        }
    }
}

/// 動力チェック
fn check_power(
    consumer: &PowerConsumer,
    power_node: Option<&PowerNode>,
    power_groups: &PowerNetworkGroups,
) -> bool {
    // PowerNodeがない場合は動力不要として扱う（テスト用）
    let Some(node) = power_node else {
        return true;
    };

    // グループIDがない場合は未接続
    let Some(group_id) = node.group_id else {
        return false;
    };

    // グループの状態をチェック
    if let Some(group) = power_groups.groups.get(&group_id) {
        // 応力過多でないこと
        if group.is_overstressed {
            return false;
        }
        // 回転速度があること
        if consumer.current_speed_received <= 0.0 {
            return false;
        }
        return true;
    }

    false
}

/// アニメーション更新システム（加工中のみ）
pub fn update_machine_animations(
    mut query: Query<(&MachineState, &mut MachineAnimation), With<KineticMachine>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (state, mut animation) in &mut query {
        if state.is_processing() {
            animation.tick(dt);
        }
    }
}

// ========================================
// プラグイン
// ========================================

/// 工作機械プラグイン
pub struct KineticMachinesPlugin;

impl Plugin for KineticMachinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, process_kinetic_machines)
            .add_systems(Update, update_machine_animations);
    }
}

// ========================================
// テスト
// ========================================

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<RecipeManager>();
        app.init_resource::<PowerNetworkGroups>();

        // テスト用レシピを追加
        let mut manager = app.world_mut().resource_mut::<RecipeManager>();
        use super::super::recipe_system::{Recipe, ItemIO};
        manager.add_recipe(Recipe {
            id: "test_press".to_string(),
            name: "Test Press".to_string(),
            inputs: vec![ItemIO { item: "iron_ingot".to_string(), count: 1 }],
            input_fluid: None,
            outputs: vec![ItemIO { item: "iron_plate".to_string(), count: 1 }],
            output_fluid: None,
            craft_time: 0.1,
            work_type: WorkType::Pressing,
        });

        app
    }

    #[test]
    fn test_mechanical_press_bundle() {
        let mut app = setup_test_app();
        let entity = app.world_mut().spawn(MechanicalPressBundle::default()).id();

        // コンポーネントが正しく追加されているか
        let world = app.world();
        assert!(world.get::<MechanicalPress>(entity).is_some());
        assert!(world.get::<KineticMachine>(entity).is_some());
        assert!(world.get::<InputInventory>(entity).is_some());
        assert!(world.get::<OutputInventory>(entity).is_some());
        assert!(world.get::<MachineState>(entity).is_some());

        let work_type = world.get::<ProcessingWorkType>(entity).unwrap();
        assert_eq!(work_type.0, WorkType::Pressing);
    }

    #[test]
    fn test_machine_animation() {
        let mut anim = MachineAnimation::new(4, 0.1);
        assert_eq!(anim.frame, 0);

        anim.tick(0.05);
        assert_eq!(anim.frame, 0); // まだ切り替わらない

        anim.tick(0.06);
        assert_eq!(anim.frame, 1); // 切り替わった（timer = 0.01残り）

        // 0.3秒後（0.01 + 0.3 = 0.31秒、3フレーム進む）
        // whileループで: frame 1→2 (timer=0.21)→3 (timer=0.11)→0 (timer=0.01)
        anim.tick(0.3);
        assert_eq!(anim.frame, 0); // ループして戻る
    }

    #[test]
    fn test_kinetic_processing_no_power_node() {
        let mut app = setup_test_app();

        // PowerNodeなしで機械を生成（テストモード）
        let entity = app.world_mut().spawn(MechanicalPressBundle::default()).id();

        // 入力アイテムを追加
        {
            let mut input = app.world_mut().get_mut::<InputInventory>(entity).unwrap();
            input.add_item("iron_ingot", 1);
        }

        // システムを手動実行
        app.add_systems(Update, process_kinetic_machines);
        app.update();

        // 加工が開始されているはず
        let state = app.world().get::<MachineState>(entity).unwrap();
        assert!(state.is_processing(), "Should be processing without PowerNode (test mode)");
    }
}
