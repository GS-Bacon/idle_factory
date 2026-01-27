//! Statistics and analysis system
//!
//! Internal storage uses `ItemId` for item identification.

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use crate::core::ItemId;
use crate::events::game_events::{ItemDelivered, MachineCompleted, MachineStarted};

/// 時系列データ
#[derive(Debug, Clone, Default)]
pub struct TimeSeries {
    /// (タイムスタンプ, 値) のサンプル
    pub samples: VecDeque<(f64, f32)>,
    /// サンプリング間隔（秒）
    pub resolution: f32,
    /// 最大サンプル数
    pub max_samples: usize,
}

impl TimeSeries {
    pub fn new(resolution: f32, max_samples: usize) -> Self {
        Self {
            samples: VecDeque::new(),
            resolution,
            max_samples,
        }
    }

    /// サンプルを追加
    pub fn add_sample(&mut self, timestamp: f64, value: f32) {
        self.samples.push_back((timestamp, value));
        while self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }

    /// 最新の値を取得
    pub fn latest(&self) -> Option<f32> {
        self.samples.back().map(|(_, v)| *v)
    }

    /// 平均を計算
    pub fn average(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.samples.iter().map(|(_, v)| v).sum();
        sum / self.samples.len() as f32
    }

    /// 最大値
    pub fn max(&self) -> f32 {
        self.samples.iter().map(|(_, v)| *v).fold(0.0f32, f32::max)
    }

    /// 最小値
    pub fn min(&self) -> f32 {
        self.samples
            .iter()
            .map(|(_, v)| *v)
            .fold(f32::MAX, f32::min)
    }
}

/// 生産統計リソース
#[derive(Resource, Debug, Default)]
pub struct ProductionStats {
    /// アイテム生産数
    pub items_produced: HashMap<ItemId, TimeSeries>,
    /// アイテム消費数
    pub items_consumed: HashMap<ItemId, TimeSeries>,
    /// 総生産数（累計）
    pub total_produced: HashMap<ItemId, u64>,
    /// 総消費数（累計）
    pub total_consumed: HashMap<ItemId, u64>,
}

impl ProductionStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// 生産を記録
    pub fn record_production_by_id(&mut self, item_id: ItemId, count: u32, timestamp: f64) {
        let series = self
            .items_produced
            .entry(item_id)
            .or_insert_with(|| TimeSeries::new(1.0, 60)); // 1秒間隔、60サンプル
        series.add_sample(timestamp, count as f32);

        *self.total_produced.entry(item_id).or_insert(0) += count as u64;
    }

    /// 消費を記録
    pub fn record_consumption_by_id(&mut self, item_id: ItemId, count: u32, timestamp: f64) {
        let series = self
            .items_consumed
            .entry(item_id)
            .or_insert_with(|| TimeSeries::new(1.0, 60));
        series.add_sample(timestamp, count as f32);

        *self.total_consumed.entry(item_id).or_insert(0) += count as u64;
    }

    /// 総生産数を取得
    pub fn get_total_produced_by_id(&self, item_id: ItemId) -> u64 {
        self.total_produced.get(&item_id).copied().unwrap_or(0)
    }

    /// 総消費数を取得
    pub fn get_total_consumed_by_id(&self, item_id: ItemId) -> u64 {
        self.total_consumed.get(&item_id).copied().unwrap_or(0)
    }

    /// 全生産統計を取得
    pub fn get_all_produced_by_id(&self) -> Vec<(ItemId, u64)> {
        self.total_produced
            .iter()
            .map(|(id, count)| (*id, *count))
            .collect()
    }

    /// 全消費統計を取得
    pub fn get_all_consumed_by_id(&self) -> Vec<(ItemId, u64)> {
        self.total_consumed
            .iter()
            .map(|(id, count)| (*id, *count))
            .collect()
    }
}

/// ボトルネック分析結果
#[derive(Debug, Clone, Default)]
pub struct BottleneckAnalysis {
    /// 稼働率が低い機械
    pub slow_machines: Vec<(Entity, f32)>,
    /// 出力が詰まっている機械
    pub blocked_outputs: Vec<Entity>,
    /// 入力待ちの機械
    pub waiting_inputs: Vec<Entity>,
}

/// 納品統計リソース
#[derive(Resource, Debug, Default)]
pub struct DeliveryStats {
    /// 納品数（累計）
    pub total_delivered: HashMap<ItemId, u64>,
}

impl DeliveryStats {
    /// 納品を記録
    pub fn record_delivery_by_id(&mut self, item_id: ItemId, count: u32) {
        *self.total_delivered.entry(item_id).or_insert(0) += count as u64;
    }

    /// 総納品数を取得
    pub fn get_total_delivered_by_id(&self, item_id: ItemId) -> u64 {
        self.total_delivered.get(&item_id).copied().unwrap_or(0)
    }

    /// 全アイテムの総納品数を取得
    pub fn get_grand_total(&self) -> u64 {
        self.total_delivered.values().sum()
    }

    /// 全納品統計を取得
    pub fn get_all_delivered_by_id(&self) -> Vec<(ItemId, u64)> {
        self.total_delivered
            .iter()
            .map(|(id, count)| (*id, *count))
            .collect()
    }
}

/// 機械完了イベントを購読して生産統計を記録
fn handle_machine_completed(
    mut events: MessageReader<MachineCompleted>,
    mut stats: ResMut<ProductionStats>,
    time: Res<Time>,
) {
    let timestamp = time.elapsed_secs_f64();
    for event in events.read() {
        for (item_id, count) in &event.outputs {
            stats.record_production_by_id(*item_id, *count, timestamp);
        }
    }
}

/// 機械開始イベントを購読して消費統計を記録
fn handle_machine_started(
    mut events: MessageReader<MachineStarted>,
    mut stats: ResMut<ProductionStats>,
    time: Res<Time>,
) {
    let timestamp = time.elapsed_secs_f64();
    for event in events.read() {
        for (item_id, count) in &event.inputs {
            stats.record_consumption_by_id(*item_id, *count, timestamp);
        }
    }
}

/// 納品イベントを購読して納品統計を記録
fn handle_item_delivered(
    mut events: MessageReader<ItemDelivered>,
    mut stats: ResMut<DeliveryStats>,
) {
    for event in events.read() {
        stats.record_delivery_by_id(event.item, event.count);
    }
}

pub struct StatisticsPlugin;

impl Plugin for StatisticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProductionStats>()
            .init_resource::<DeliveryStats>()
            .add_systems(
                Update,
                (
                    handle_machine_completed,
                    handle_machine_started,
                    handle_item_delivered,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

    // =========================================================================
    // TimeSeries tests (no deprecation)
    // =========================================================================

    #[test]
    fn test_time_series_basic() {
        let mut ts = TimeSeries::new(1.0, 10);
        ts.add_sample(0.0, 5.0);
        ts.add_sample(1.0, 10.0);
        ts.add_sample(2.0, 15.0);

        assert_eq!(ts.latest(), Some(15.0));
        assert_eq!(ts.average(), 10.0);
        assert_eq!(ts.max(), 15.0);
        assert_eq!(ts.min(), 5.0);
    }

    #[test]
    fn test_time_series_max_samples() {
        let mut ts = TimeSeries::new(1.0, 3);
        for i in 0..5 {
            ts.add_sample(i as f64, i as f32);
        }
        assert_eq!(ts.samples.len(), 3);
        assert_eq!(ts.latest(), Some(4.0));
    }

    #[test]
    fn test_bottleneck_default() {
        let analysis = BottleneckAnalysis::default();
        assert!(analysis.slow_machines.is_empty());
    }

    #[test]
    fn test_production_stats_by_id() {
        let mut stats = ProductionStats::new();
        stats.record_production_by_id(items::iron_ingot(), 10, 0.0);
        stats.record_production_by_id(items::iron_ingot(), 5, 1.0);

        assert_eq!(stats.get_total_produced_by_id(items::iron_ingot()), 15);
        assert_eq!(stats.get_total_produced_by_id(items::stone()), 0);
    }

    #[test]
    fn test_consumption_stats_by_id() {
        let mut stats = ProductionStats::new();
        stats.record_consumption_by_id(items::coal(), 5, 0.0);
        stats.record_consumption_by_id(items::coal(), 3, 1.0);

        assert_eq!(stats.get_total_consumed_by_id(items::coal()), 8);
        assert_eq!(stats.get_total_consumed_by_id(items::stone()), 0);
    }

    #[test]
    fn test_delivery_stats_by_id() {
        let mut stats = DeliveryStats::default();
        stats.record_delivery_by_id(items::iron_ingot(), 10);
        stats.record_delivery_by_id(items::iron_ingot(), 5);
        stats.record_delivery_by_id(items::stone(), 3);

        assert_eq!(stats.get_total_delivered_by_id(items::iron_ingot()), 15);
        assert_eq!(stats.get_total_delivered_by_id(items::stone()), 3);
        assert_eq!(stats.get_grand_total(), 18);
    }

    #[test]
    fn test_get_all_by_id() {
        let mut stats = ProductionStats::new();
        stats.record_production_by_id(items::iron_ingot(), 10, 0.0);
        stats.record_production_by_id(items::copper_ingot(), 5, 1.0);

        let all_produced = stats.get_all_produced_by_id();
        assert_eq!(all_produced.len(), 2);

        // Check totals (order not guaranteed in HashMap)
        let iron_total: u64 = all_produced
            .iter()
            .filter(|(id, _)| *id == items::iron_ingot())
            .map(|(_, c)| *c)
            .sum();
        let copper_total: u64 = all_produced
            .iter()
            .filter(|(id, _)| *id == items::copper_ingot())
            .map(|(_, c)| *c)
            .sum();
        assert_eq!(iron_total, 10);
        assert_eq!(copper_total, 5);
    }
}
