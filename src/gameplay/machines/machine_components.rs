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
