//! ロジスティクスロボットシステム（Factorio風）
//!
//! ## 概要
//! - ロボポート: 充電ステーション兼コマンドセンター
//! - ロジスティクスロボット: アイテム輸送用ドローン
//! - 建設ロボット: 建築・修理用ドローン
//! - ロジスティクスチェスト: ネットワークへのアイテム供給/要求
//!
//! ## チェストタイプ
//! - Provider (黄): ネットワークにアイテムを提供
//! - Requester (青): ネットワークからアイテムを要求
//! - Storage (茶): パッシブストレージ
//! - Buffer (緑): Provider + Requester

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// ロボポートコンポーネント
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Roboport {
    /// ロジスティクス範囲（ブロック）
    pub logistics_range: f32,
    /// 建設範囲（ブロック）
    pub construction_range: f32,
    /// 格納可能なロボット数
    pub robot_capacity: u32,
    /// 充電スロット数
    pub charging_slots: u32,
    /// 現在格納されているロボット数
    pub stored_robots: u32,
    /// 充電中のロボットID
    pub charging_robots: Vec<Entity>,
}

impl Default for Roboport {
    fn default() -> Self {
        Self {
            logistics_range: 50.0,
            construction_range: 110.0,
            robot_capacity: 7,
            charging_slots: 4,
            stored_robots: 0,
            charging_robots: Vec::new(),
        }
    }
}

/// ロジスティクスロボットコンポーネント
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct LogisticsRobot {
    /// 移動速度（ブロック/秒）
    pub speed: f32,
    /// 積載容量（スタック数）
    pub cargo_capacity: u32,
    /// 現在のバッテリー残量（0.0-1.0）
    pub battery: f32,
    /// 最大バッテリー容量
    pub max_battery: f32,
    /// 現在運んでいるアイテム
    pub cargo: Option<(String, u32)>,
    /// 現在のタスク
    pub current_task: Option<RobotTask>,
    /// 所属するロボポートのEntity
    pub home_roboport: Option<Entity>,
}

impl Default for LogisticsRobot {
    fn default() -> Self {
        Self {
            speed: 5.0,
            cargo_capacity: 1,
            battery: 1.0,
            max_battery: 1500.0,
            cargo: None,
            current_task: None,
            home_roboport: None,
        }
    }
}

/// 建設ロボットコンポーネント
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionRobot {
    /// 基本のロジスティクス機能
    pub base: LogisticsRobot,
    /// 建設能力
    pub can_build: bool,
    /// 修理能力
    pub can_repair: bool,
}

impl Default for ConstructionRobot {
    fn default() -> Self {
        Self {
            base: LogisticsRobot::default(),
            can_build: true,
            can_repair: true,
        }
    }
}

/// ロボットのタスク
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RobotTask {
    /// アイテムを取りに行く
    PickUp {
        source: Entity,
        item_id: String,
        count: u32,
    },
    /// アイテムを届ける
    Deliver {
        target: Entity,
        item_id: String,
        count: u32,
    },
    /// 充電しに帰る
    ReturnToCharge {
        roboport: Entity,
    },
    /// ブロックを建設
    Build {
        position: IVec3,
        block_id: String,
    },
    /// 構造物を修理
    Repair {
        target: Entity,
    },
}

/// ロジスティクスチェストのタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogisticsChestType {
    /// アイテムを提供（黄色）
    Provider,
    /// アイテムを要求（青色）
    Requester,
    /// パッシブストレージ（茶色）
    Storage,
    /// Provider + Requester（緑色）
    Buffer,
}

/// ロジスティクスチェストコンポーネント
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct LogisticsChest {
    /// チェストタイプ
    pub chest_type: LogisticsChestType,
    /// 要求フィルタ（Requester/Bufferのみ）
    pub request_filters: Vec<(String, u32)>, // (item_id, count)
    /// スロット数
    pub slot_count: u32,
    /// 現在のアイテム
    pub items: HashMap<String, u32>,
}

impl LogisticsChest {
    pub fn new(chest_type: LogisticsChestType) -> Self {
        Self {
            chest_type,
            request_filters: Vec::new(),
            slot_count: 48,
            items: HashMap::new(),
        }
    }

    /// フィルタを追加
    pub fn add_filter(&mut self, item_id: &str, count: u32) {
        self.request_filters.push((item_id.to_string(), count));
    }

    /// アイテムを追加
    pub fn add_item(&mut self, item_id: &str, count: u32) {
        *self.items.entry(item_id.to_string()).or_insert(0) += count;
    }

    /// アイテムを取り出す
    pub fn take_item(&mut self, item_id: &str, count: u32) -> u32 {
        if let Some(current) = self.items.get_mut(item_id) {
            let taken = (*current).min(count);
            *current -= taken;
            if *current == 0 {
                self.items.remove(item_id);
            }
            taken
        } else {
            0
        }
    }
}

/// ロジスティクスネットワークリソース
#[derive(Resource, Default)]
pub struct LogisticsNetwork {
    /// 全ロボポートのEntity
    pub roboports: HashSet<Entity>,
    /// 全ロジスティクスチェストのEntity
    pub chests: HashSet<Entity>,
    /// 保留中のリクエスト
    pub pending_requests: Vec<LogisticsRequest>,
}

/// ロジスティクスリクエスト
#[derive(Debug, Clone)]
pub struct LogisticsRequest {
    pub requester: Entity,
    pub item_id: String,
    pub count: u32,
    pub priority: u32,
}

impl LogisticsNetwork {
    /// ロボポートを登録
    pub fn register_roboport(&mut self, entity: Entity) {
        self.roboports.insert(entity);
    }

    /// チェストを登録
    pub fn register_chest(&mut self, entity: Entity) {
        self.chests.insert(entity);
    }

    /// リクエストを追加
    pub fn add_request(&mut self, request: LogisticsRequest) {
        self.pending_requests.push(request);
        // 優先度でソート
        self.pending_requests.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
}

// =====================================
// テスト
// =====================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logistics_chest_operations() {
        let mut chest = LogisticsChest::new(LogisticsChestType::Provider);

        // アイテム追加
        chest.add_item("iron_ore", 100);
        assert_eq!(chest.items.get("iron_ore"), Some(&100));

        // アイテム取り出し
        let taken = chest.take_item("iron_ore", 30);
        assert_eq!(taken, 30);
        assert_eq!(chest.items.get("iron_ore"), Some(&70));

        // 全部取り出し
        let taken = chest.take_item("iron_ore", 100);
        assert_eq!(taken, 70);
        assert!(chest.items.get("iron_ore").is_none());
    }

    #[test]
    fn test_roboport_default() {
        let port = Roboport::default();
        assert_eq!(port.logistics_range, 50.0);
        assert_eq!(port.construction_range, 110.0);
        assert_eq!(port.robot_capacity, 7);
    }

    #[test]
    fn test_robot_task() {
        let robot = LogisticsRobot::default();
        assert!(robot.current_task.is_none());
        assert_eq!(robot.battery, 1.0);
        assert_eq!(robot.speed, 5.0);
    }
}
