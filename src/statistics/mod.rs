//! Statistics and analysis system

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use crate::block_type::BlockType;

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
    pub items_produced: HashMap<BlockType, TimeSeries>,
    /// アイテム消費数
    pub items_consumed: HashMap<BlockType, TimeSeries>,
    /// 総生産数（累計）
    pub total_produced: HashMap<BlockType, u64>,
    /// 総消費数（累計）
    pub total_consumed: HashMap<BlockType, u64>,
}

impl ProductionStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// 生産を記録
    pub fn record_production(&mut self, item: BlockType, count: u32, timestamp: f64) {
        let series = self
            .items_produced
            .entry(item)
            .or_insert_with(|| TimeSeries::new(1.0, 60)); // 1秒間隔、60サンプル
        series.add_sample(timestamp, count as f32);

        *self.total_produced.entry(item).or_insert(0) += count as u64;
    }

    /// 消費を記録
    pub fn record_consumption(&mut self, item: BlockType, count: u32, timestamp: f64) {
        let series = self
            .items_consumed
            .entry(item)
            .or_insert_with(|| TimeSeries::new(1.0, 60));
        series.add_sample(timestamp, count as f32);

        *self.total_consumed.entry(item).or_insert(0) += count as u64;
    }

    /// 総生産数を取得
    pub fn get_total_produced(&self, item: BlockType) -> u64 {
        self.total_produced.get(&item).copied().unwrap_or(0)
    }

    /// 総消費数を取得
    pub fn get_total_consumed(&self, item: BlockType) -> u64 {
        self.total_consumed.get(&item).copied().unwrap_or(0)
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

pub struct StatisticsPlugin;

impl Plugin for StatisticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProductionStats>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_production_stats() {
        let mut stats = ProductionStats::new();
        stats.record_production(BlockType::IronIngot, 10, 0.0);
        stats.record_production(BlockType::IronIngot, 5, 1.0);

        assert_eq!(stats.get_total_produced(BlockType::IronIngot), 15);
        assert_eq!(stats.get_total_produced(BlockType::Stone), 0);
    }

    #[test]
    fn test_bottleneck_default() {
        let analysis = BottleneckAnalysis::default();
        assert!(analysis.slow_machines.is_empty());
    }
}
