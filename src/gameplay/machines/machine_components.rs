// src/gameplay/machines/machine_components.rs
//! 工作機械の共通コンポーネント
//!
//! 全ての工作機械が使用する基本的なコンポーネントを定義する。
//! - InputInventory / OutputInventory: アイテム入出力
//! - FluidTank: 流体タンク
//! - MachineState: 機械の動作状態
//! - StressImpact: 応力消費量

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ========================================
// インベントリスロット
// ========================================

/// インベントリの1スロット
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    /// アイテムID（空の場合はNone）
    pub item_id: Option<String>,
    /// スタック数
    pub count: u32,
    /// 最大スタック数
    pub max_stack: u32,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            item_id: None,
            count: 0,
            max_stack: 64,
        }
    }
}

impl Slot {
    /// 空のスロットを作成
    pub fn empty() -> Self {
        Self::default()
    }

    /// アイテムを持つスロットを作成
    pub fn new(item_id: &str, count: u32) -> Self {
        Self {
            item_id: Some(item_id.to_string()),
            count,
            max_stack: 64,
        }
    }

    /// スロットが空かどうか
    pub fn is_empty(&self) -> bool {
        self.item_id.is_none() || self.count == 0
    }

    /// アイテムを追加可能な数を返す
    pub fn can_add(&self, item_id: &str, amount: u32) -> u32 {
        if self.is_empty() {
            amount.min(self.max_stack)
        } else if self.item_id.as_deref() == Some(item_id) {
            (self.max_stack - self.count).min(amount)
        } else {
            0
        }
    }

    /// アイテムを追加し、追加できなかった数を返す
    pub fn add(&mut self, item_id: &str, amount: u32) -> u32 {
        let can_add = self.can_add(item_id, amount);
        if can_add > 0 {
            if self.is_empty() {
                self.item_id = Some(item_id.to_string());
            }
            self.count += can_add;
        }
        amount - can_add
    }

    /// アイテムを取り出し、取り出せた数を返す
    pub fn take(&mut self, amount: u32) -> u32 {
        let taken = self.count.min(amount);
        self.count -= taken;
        if self.count == 0 {
            self.item_id = None;
        }
        taken
    }
}

// ========================================
// インベントリコンポーネント
// ========================================

/// 入力インベントリ
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputInventory {
    pub slots: Vec<Slot>,
}

impl InputInventory {
    /// 指定スロット数で作成
    pub fn new(slot_count: usize) -> Self {
        Self {
            slots: vec![Slot::empty(); slot_count],
        }
    }

    /// アイテムを追加（最初の空きスロットに）
    pub fn add_item(&mut self, item_id: &str, mut amount: u32) -> u32 {
        // 既存スロットにスタック
        for slot in &mut self.slots {
            if amount == 0 { break; }
            if slot.item_id.as_deref() == Some(item_id) {
                amount = slot.add(item_id, amount);
            }
        }
        // 空きスロットに追加
        for slot in &mut self.slots {
            if amount == 0 { break; }
            if slot.is_empty() {
                amount = slot.add(item_id, amount);
            }
        }
        amount // 追加できなかった数
    }

    /// 指定アイテムの総数を取得
    pub fn count_item(&self, item_id: &str) -> u32 {
        self.slots.iter()
            .filter(|s| s.item_id.as_deref() == Some(item_id))
            .map(|s| s.count)
            .sum()
    }

    /// 指定アイテムを消費
    pub fn consume(&mut self, item_id: &str, mut amount: u32) -> bool {
        if self.count_item(item_id) < amount {
            return false;
        }
        for slot in &mut self.slots {
            if amount == 0 { break; }
            if slot.item_id.as_deref() == Some(item_id) {
                let taken = slot.take(amount);
                amount -= taken;
            }
        }
        true
    }
}

/// 出力インベントリ
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputInventory {
    pub slots: Vec<Slot>,
}

impl OutputInventory {
    /// 指定スロット数で作成
    pub fn new(slot_count: usize) -> Self {
        Self {
            slots: vec![Slot::empty(); slot_count],
        }
    }

    /// アイテムを追加（最初の空きスロットに）
    pub fn add_item(&mut self, item_id: &str, mut amount: u32) -> u32 {
        // 既存スロットにスタック
        for slot in &mut self.slots {
            if amount == 0 { break; }
            if slot.item_id.as_deref() == Some(item_id) {
                amount = slot.add(item_id, amount);
            }
        }
        // 空きスロットに追加
        for slot in &mut self.slots {
            if amount == 0 { break; }
            if slot.is_empty() {
                amount = slot.add(item_id, amount);
            }
        }
        amount // 追加できなかった数
    }

    /// 出力が満杯かどうか
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| !s.is_empty() && s.count >= s.max_stack)
    }

    /// 最初のアイテムを1つ取り出す
    pub fn take_first(&mut self) -> Option<(String, u32)> {
        for slot in &mut self.slots {
            if !slot.is_empty() {
                let item_id = slot.item_id.clone()?;
                let taken = slot.take(1);
                if taken > 0 {
                    return Some((item_id, taken));
                }
            }
        }
        None
    }
}

// ========================================
// 流体タンク
// ========================================

/// 流体ID
pub type FluidId = String;

/// 流体タンクコンポーネント
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct FluidTank {
    /// 現在の流体ID（空の場合はNone）
    pub fluid_id: Option<FluidId>,
    /// 現在の量 (mB単位)
    pub amount: f32,
    /// 最大容量 (mB単位)
    pub capacity: f32,
}

impl FluidTank {
    /// 新しいタンクを作成
    pub fn new(capacity: f32) -> Self {
        Self {
            fluid_id: None,
            amount: 0.0,
            capacity,
        }
    }

    /// タンクが空かどうか
    pub fn is_empty(&self) -> bool {
        self.fluid_id.is_none() || self.amount <= 0.0
    }

    /// タンクが満杯かどうか
    pub fn is_full(&self) -> bool {
        self.amount >= self.capacity
    }

    /// 流体を追加し、追加できなかった量を返す
    pub fn fill(&mut self, fluid_id: &str, amount: f32) -> f32 {
        if self.is_empty() {
            self.fluid_id = Some(fluid_id.to_string());
        }

        if self.fluid_id.as_deref() != Some(fluid_id) {
            return amount; // 異なる流体は追加できない
        }

        let space = self.capacity - self.amount;
        let to_add = amount.min(space);
        self.amount += to_add;
        amount - to_add
    }

    /// 流体を取り出し、取り出せた量を返す
    pub fn drain(&mut self, amount: f32) -> f32 {
        let to_drain = amount.min(self.amount);
        self.amount -= to_drain;
        if self.amount <= 0.0 {
            self.fluid_id = None;
            self.amount = 0.0;
        }
        to_drain
    }
}

// ========================================
// 機械の状態
// ========================================

/// 機械の動作状態
#[derive(Component, Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum MachineState {
    /// 待機中（材料待ち）
    #[default]
    Idle,
    /// 加工中（タイマー付き）
    Processing {
        /// 経過時間
        elapsed: f32,
        /// 必要時間
        total: f32,
    },
    /// 詰まり状態（出力が満杯）
    Jammed,
    /// 動力不足
    NoPower,
}

impl MachineState {
    /// 加工中かどうか
    pub fn is_processing(&self) -> bool {
        matches!(self, MachineState::Processing { .. })
    }

    /// 動作可能かどうか（Idle以外は動作不可）
    pub fn can_start(&self) -> bool {
        matches!(self, MachineState::Idle)
    }

    /// 加工を開始
    pub fn start_processing(&mut self, total_time: f32) {
        *self = MachineState::Processing {
            elapsed: 0.0,
            total: total_time,
        };
    }

    /// 加工を進める（完了したらtrueを返す）
    pub fn tick(&mut self, delta: f32) -> bool {
        if let MachineState::Processing { elapsed, total } = self {
            *elapsed += delta;
            if *elapsed >= *total {
                *self = MachineState::Idle;
                return true;
            }
        }
        false
    }

    /// 進捗率（0.0〜1.0）
    pub fn progress(&self) -> f32 {
        match self {
            MachineState::Processing { elapsed, total } => {
                if *total > 0.0 { (*elapsed / *total).clamp(0.0, 1.0) } else { 0.0 }
            }
            _ => 0.0,
        }
    }
}

// ========================================
// 応力消費
// ========================================

/// 機械が消費する応力値
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StressImpact(pub f32);

impl StressImpact {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}

// ========================================
// オーバークロック/アンダークロック
// ========================================

/// オーバークロック設定（Satisfactory風）
///
/// クロック速度を1%-250%で調整可能
/// 電力消費は非線形: power * (clock_speed ^ 1.6)
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Overclock {
    /// クロック速度（1.0 = 100%）
    /// 範囲: 0.01 - 2.5 (1% - 250%)
    pub clock_speed: f32,
    /// 装着されているパワーシャードの数（0-3）
    /// 150%以上にはシャードが必要
    pub power_shards: u8,
}

impl Default for Overclock {
    fn default() -> Self {
        Self {
            clock_speed: 1.0,
            power_shards: 0,
        }
    }
}

impl Overclock {
    /// 新しいオーバークロック設定を作成
    pub fn new(clock_speed: f32) -> Self {
        Self {
            clock_speed: clock_speed.clamp(0.01, 2.5),
            power_shards: 0,
        }
    }

    /// クロック速度を設定（パワーシャードに応じた上限）
    pub fn set_clock_speed(&mut self, speed: f32) {
        let max_speed = self.max_clock_speed();
        self.clock_speed = speed.clamp(0.01, max_speed);
    }

    /// パワーシャードに応じた最大クロック速度
    pub fn max_clock_speed(&self) -> f32 {
        match self.power_shards {
            0 => 1.0,   // 100%まで（パワーシャードなし）
            1 => 1.5,   // 150%まで
            2 => 2.0,   // 200%まで
            _ => 2.5,   // 250%まで（3個以上）
        }
    }

    /// パワーシャードを追加
    pub fn add_power_shard(&mut self) -> bool {
        if self.power_shards < 3 {
            self.power_shards += 1;
            true
        } else {
            false
        }
    }

    /// 電力乗数を計算（非線形: speed ^ 1.6）
    pub fn power_multiplier(&self) -> f32 {
        self.clock_speed.powf(1.6)
    }

    /// 実際の処理速度乗数
    pub fn speed_multiplier(&self) -> f32 {
        self.clock_speed
    }
}

// ========================================
// 品質システム（Factorio 2.0 Space Age風）
// ========================================

/// アイテムの品質ティア
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ItemQuality {
    #[default]
    Normal,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ItemQuality {
    /// 品質によるボーナス乗数
    pub fn bonus_multiplier(&self) -> f32 {
        match self {
            ItemQuality::Normal => 1.0,
            ItemQuality::Uncommon => 1.3,  // +30%
            ItemQuality::Rare => 1.6,      // +60%
            ItemQuality::Epic => 1.9,      // +90%
            ItemQuality::Legendary => 2.5, // +150%
        }
    }

    /// 品質の表示名
    pub fn display_name(&self) -> &'static str {
        match self {
            ItemQuality::Normal => "Normal",
            ItemQuality::Uncommon => "Uncommon",
            ItemQuality::Rare => "Rare",
            ItemQuality::Epic => "Epic",
            ItemQuality::Legendary => "Legendary",
        }
    }

    /// 品質の色（RGB）
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            ItemQuality::Normal => (200, 200, 200),    // Gray
            ItemQuality::Uncommon => (30, 200, 30),    // Green
            ItemQuality::Rare => (30, 100, 255),       // Blue
            ItemQuality::Epic => (160, 30, 240),       // Purple
            ItemQuality::Legendary => (255, 180, 30),  // Orange/Gold
        }
    }

    /// 次の品質ティア
    pub fn next(&self) -> Option<ItemQuality> {
        match self {
            ItemQuality::Normal => Some(ItemQuality::Uncommon),
            ItemQuality::Uncommon => Some(ItemQuality::Rare),
            ItemQuality::Rare => Some(ItemQuality::Epic),
            ItemQuality::Epic => Some(ItemQuality::Legendary),
            ItemQuality::Legendary => None,
        }
    }
}

/// 品質モジュールが装着された機械
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct QualityModuleSlots {
    /// 装着されているモジュール（最大4スロット）
    pub modules: Vec<QualityModule>,
}

/// 品質モジュール
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityModule {
    /// モジュールのティア（1-3）
    pub tier: u8,
    /// 品質アップグレード確率（%）
    pub quality_bonus: f32,
    /// 速度ペナルティ（%、負の値）
    pub speed_penalty: f32,
}

impl QualityModule {
    pub fn tier1() -> Self {
        Self { tier: 1, quality_bonus: 2.5, speed_penalty: -5.0 }
    }
    pub fn tier2() -> Self {
        Self { tier: 2, quality_bonus: 5.0, speed_penalty: -10.0 }
    }
    pub fn tier3() -> Self {
        Self { tier: 3, quality_bonus: 10.0, speed_penalty: -15.0 }
    }
}

impl QualityModuleSlots {
    /// 合計品質ボーナスを計算
    pub fn total_quality_bonus(&self) -> f32 {
        self.modules.iter().map(|m| m.quality_bonus).sum()
    }

    /// 合計速度ペナルティを計算
    pub fn total_speed_penalty(&self) -> f32 {
        self.modules.iter().map(|m| m.speed_penalty).sum()
    }

    /// 速度乗数（1.0 - ペナルティ/100）
    pub fn speed_multiplier(&self) -> f32 {
        1.0 + self.total_speed_penalty() / 100.0
    }
}

// ========================================
// 機械マーカー
// ========================================

/// 工作機械であることを示すマーカー
#[derive(Component, Debug, Clone, Copy)]
pub struct KineticMachine;

// ========================================
// テスト
// ========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_operations() {
        let mut slot = Slot::empty();
        assert!(slot.is_empty());

        // 追加
        let remaining = slot.add("iron_ingot", 10);
        assert_eq!(remaining, 0);
        assert_eq!(slot.count, 10);
        assert_eq!(slot.item_id, Some("iron_ingot".to_string()));

        // 取り出し
        let taken = slot.take(3);
        assert_eq!(taken, 3);
        assert_eq!(slot.count, 7);

        // 全部取り出し
        let taken = slot.take(10);
        assert_eq!(taken, 7);
        assert!(slot.is_empty());
    }

    #[test]
    fn test_input_inventory() {
        let mut inv = InputInventory::new(3);

        // 追加（3スロットあるので64*3=192個まで入る）
        let remaining = inv.add_item("iron_ore", 100);
        assert_eq!(remaining, 0); // 全部入る（64+36）
        assert_eq!(inv.count_item("iron_ore"), 100);

        // 消費
        assert!(inv.consume("iron_ore", 10));
        assert_eq!(inv.count_item("iron_ore"), 90);

        // 不足時は消費失敗
        assert!(!inv.consume("iron_ore", 100));
    }

    #[test]
    fn test_fluid_tank() {
        let mut tank = FluidTank::new(1000.0);
        assert!(tank.is_empty());

        // 充填
        let remaining = tank.fill("water", 500.0);
        assert_eq!(remaining, 0.0);
        assert_eq!(tank.amount, 500.0);

        // 異なる流体は追加不可
        let remaining = tank.fill("lava", 100.0);
        assert_eq!(remaining, 100.0);

        // 排出
        let drained = tank.drain(200.0);
        assert_eq!(drained, 200.0);
        assert_eq!(tank.amount, 300.0);
    }

    #[test]
    fn test_machine_state() {
        let mut state = MachineState::Idle;
        assert!(state.can_start());

        state.start_processing(1.0);
        assert!(state.is_processing());
        assert!(!state.can_start());

        // 進捗
        let completed = state.tick(0.5);
        assert!(!completed);
        assert!((state.progress() - 0.5).abs() < 0.01);

        // 完了
        let completed = state.tick(0.6);
        assert!(completed);
        assert_eq!(state, MachineState::Idle);
    }
}
