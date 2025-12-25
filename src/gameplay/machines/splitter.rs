//! スマートスプリッター / プログラマブルスプリッター
//!
//! Satisfactory風の条件付きアイテム分配機構
//!
//! ## フィルタルール
//! - Any: 任意のアイテムを受け入れ
//! - None: このポートには出力しない
//! - Overflow: 他のポートが満杯の場合のみ出力
//! - ItemFilter: 特定アイテムのみ出力
//!
//! ## 仕様
//! - 3方向出力（左・正面・右）
//! - 各ポートに個別のフィルタルール設定
//! - 入力は背面から

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::gameplay::grid::{Direction, ItemSlot};

/// スプリッターのフィルタルール
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SplitterFilter {
    /// 任意のアイテムを受け入れ
    Any,
    /// このポートには出力しない
    None,
    /// 他のポートが満杯の場合のみ出力
    Overflow,
    /// 特定アイテムのみ出力
    ItemFilter(Vec<String>),
}

#[allow(clippy::derivable_impls)]
impl Default for SplitterFilter {
    fn default() -> Self {
        SplitterFilter::Any
    }
}

/// 出力ポートの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputPort {
    Left,
    Center,
    Right,
}

impl OutputPort {
    /// スプリッターの向きから出力方向を計算
    pub fn to_direction(&self, splitter_orientation: Direction) -> Direction {
        match splitter_orientation {
            Direction::North => match self {
                OutputPort::Left => Direction::West,
                OutputPort::Center => Direction::North,
                OutputPort::Right => Direction::East,
            },
            Direction::South => match self {
                OutputPort::Left => Direction::East,
                OutputPort::Center => Direction::South,
                OutputPort::Right => Direction::West,
            },
            Direction::East => match self {
                OutputPort::Left => Direction::North,
                OutputPort::Center => Direction::East,
                OutputPort::Right => Direction::South,
            },
            Direction::West => match self {
                OutputPort::Left => Direction::South,
                OutputPort::Center => Direction::West,
                OutputPort::Right => Direction::North,
            },
        }
    }

    /// 全ポートを順番に返す
    pub fn all() -> [OutputPort; 3] {
        [OutputPort::Left, OutputPort::Center, OutputPort::Right]
    }
}

/// スマートスプリッターコンポーネント
#[derive(Component, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SmartSplitter {
    /// 各出力ポートのフィルタ設定
    pub filters: [SplitterFilter; 3], // Left, Center, Right
    /// 入力バッファ（待機中のアイテム）
    pub input_buffer: Vec<ItemSlot>,
    /// 最後に使用したポート（ラウンドロビン用）
    pub last_output_port: usize,
}

impl SmartSplitter {
    /// 新しいスマートスプリッターを作成
    pub fn new() -> Self {
        Self {
            filters: [SplitterFilter::Any, SplitterFilter::Any, SplitterFilter::Any],
            input_buffer: Vec::new(),
            last_output_port: 0,
        }
    }

    /// フィルタを設定
    pub fn set_filter(&mut self, port: OutputPort, filter: SplitterFilter) {
        let index = match port {
            OutputPort::Left => 0,
            OutputPort::Center => 1,
            OutputPort::Right => 2,
        };
        self.filters[index] = filter;
    }

    /// フィルタを取得
    pub fn get_filter(&self, port: OutputPort) -> &SplitterFilter {
        let index = match port {
            OutputPort::Left => 0,
            OutputPort::Center => 1,
            OutputPort::Right => 2,
        };
        &self.filters[index]
    }

    /// アイテムがポートのフィルタを通過するかチェック
    pub fn matches_filter(&self, port: OutputPort, item_id: &str) -> bool {
        match self.get_filter(port) {
            SplitterFilter::Any => true,
            SplitterFilter::None => false,
            SplitterFilter::Overflow => false, // Overflowは特別処理が必要
            SplitterFilter::ItemFilter(items) => items.iter().any(|id| id == item_id),
        }
    }

    /// アイテムの出力先ポートを決定
    ///
    /// 優先順位:
    /// 1. ItemFilterが一致するポート
    /// 2. Anyポート（ラウンドロビン）
    /// 3. Overflowポート（他がブロックまたは不一致の場合）
    pub fn determine_output_port(&mut self, item_id: &str, blocked_ports: &[OutputPort]) -> Option<OutputPort> {
        let ports = OutputPort::all();

        // まずItemFilterが一致するポートを優先
        for i in 0..3 {
            let port_index = (self.last_output_port + 1 + i) % 3;
            let port = ports[port_index];

            if blocked_ports.contains(&port) {
                continue;
            }

            if let SplitterFilter::ItemFilter(items) = self.get_filter(port) {
                if items.iter().any(|id| id == item_id) {
                    self.last_output_port = port_index;
                    return Some(port);
                }
            }
        }

        // 次にAnyポートを試す
        for i in 0..3 {
            let port_index = (self.last_output_port + 1 + i) % 3;
            let port = ports[port_index];

            if blocked_ports.contains(&port) {
                continue;
            }

            if matches!(self.get_filter(port), SplitterFilter::Any) {
                self.last_output_port = port_index;
                return Some(port);
            }
        }

        // 最後にOverflowポートを試す
        // 条件: 他の非Overflowポートが全て「ブロック」「None」「フィルタ不一致」のいずれか
        let non_overflow_unavailable = ports.iter()
            .filter(|&&p| !matches!(self.get_filter(p), SplitterFilter::Overflow))
            .all(|&p| {
                if blocked_ports.contains(&p) {
                    return true;
                }
                match self.get_filter(p) {
                    SplitterFilter::None => true,
                    SplitterFilter::ItemFilter(items) => !items.iter().any(|id| id == item_id),
                    _ => false,
                }
            });

        if non_overflow_unavailable {
            for &port in &ports {
                if matches!(self.get_filter(port), SplitterFilter::Overflow) && !blocked_ports.contains(&port) {
                    return Some(port);
                }
            }
        }

        None
    }
}

/// プログラマブルスプリッター（Lua対応版）
#[derive(Component, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProgrammableSplitter {
    /// ベースのスマートスプリッター機能
    pub base: SmartSplitter,
    /// Luaスクリプト名（オプション）
    pub script_name: Option<String>,
    /// カスタム設定（Luaから設定可能）
    pub custom_config: std::collections::HashMap<String, String>,
}

impl ProgrammableSplitter {
    pub fn new() -> Self {
        Self {
            base: SmartSplitter::new(),
            script_name: None,
            custom_config: std::collections::HashMap::new(),
        }
    }
}

// =====================================
// テスト
// =====================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_splitter_default() {
        let splitter = SmartSplitter::new();
        assert_eq!(splitter.filters[0], SplitterFilter::Any);
        assert_eq!(splitter.filters[1], SplitterFilter::Any);
        assert_eq!(splitter.filters[2], SplitterFilter::Any);
    }

    #[test]
    fn test_output_port_direction() {
        // North向きのスプリッター
        assert_eq!(OutputPort::Left.to_direction(Direction::North), Direction::West);
        assert_eq!(OutputPort::Center.to_direction(Direction::North), Direction::North);
        assert_eq!(OutputPort::Right.to_direction(Direction::North), Direction::East);

        // South向きのスプリッター
        assert_eq!(OutputPort::Left.to_direction(Direction::South), Direction::East);
        assert_eq!(OutputPort::Center.to_direction(Direction::South), Direction::South);
        assert_eq!(OutputPort::Right.to_direction(Direction::South), Direction::West);
    }

    #[test]
    fn test_filter_matching() {
        let mut splitter = SmartSplitter::new();

        // Anyは全て通す
        assert!(splitter.matches_filter(OutputPort::Left, "iron_ore"));

        // Noneは全て拒否
        splitter.set_filter(OutputPort::Left, SplitterFilter::None);
        assert!(!splitter.matches_filter(OutputPort::Left, "iron_ore"));

        // ItemFilterは指定アイテムのみ
        splitter.set_filter(OutputPort::Center, SplitterFilter::ItemFilter(vec!["iron_ore".to_string()]));
        assert!(splitter.matches_filter(OutputPort::Center, "iron_ore"));
        assert!(!splitter.matches_filter(OutputPort::Center, "copper_ore"));
    }

    #[test]
    fn test_determine_output_port() {
        let mut splitter = SmartSplitter::new();

        // Anyフィルタでラウンドロビン
        let port1 = splitter.determine_output_port("iron_ore", &[]);
        assert!(port1.is_some());

        let port2 = splitter.determine_output_port("iron_ore", &[]);
        assert!(port2.is_some());

        // 2つのポートはブロック、残りの1つのみ使用可能
        splitter.set_filter(OutputPort::Left, SplitterFilter::None);
        splitter.set_filter(OutputPort::Right, SplitterFilter::None);
        let port3 = splitter.determine_output_port("iron_ore", &[]);
        assert_eq!(port3, Some(OutputPort::Center));
    }

    #[test]
    fn test_overflow_behavior() {
        let mut splitter = SmartSplitter::new();
        splitter.set_filter(OutputPort::Left, SplitterFilter::ItemFilter(vec!["iron_ore".to_string()]));
        splitter.set_filter(OutputPort::Center, SplitterFilter::None);
        splitter.set_filter(OutputPort::Right, SplitterFilter::Overflow);

        // iron_oreは左ポートへ
        let port = splitter.determine_output_port("iron_ore", &[]);
        assert_eq!(port, Some(OutputPort::Left));

        // iron_oreで左がブロックされている場合、Overflowへ
        let port = splitter.determine_output_port("iron_ore", &[OutputPort::Left]);
        assert_eq!(port, Some(OutputPort::Right));

        // copper_oreはOverflowへ（Left=iron_only, Center=None）
        let port = splitter.determine_output_port("copper_ore", &[]);
        assert_eq!(port, Some(OutputPort::Right));
    }
}
