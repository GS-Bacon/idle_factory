//! Achievement system

use crate::core::{items, ItemId};
use crate::events::game_events::{BlockPlaced, ItemDelivered, MachineCompleted, MachineSpawned};
use crate::events::GuardedEventWriter;
use bevy::prelude::*;
use std::sync::LazyLock;

/// 実績の条件
#[derive(Debug, Clone, PartialEq)]
pub enum AchievementCondition {
    /// 特定アイテムを一定数生産
    ProduceItem { item: ItemId, count: u32 },
    /// 機械を一定数設置
    PlaceMachines { count: u32 },
    /// 特定クエスト完了
    CompleteQuest { quest_id: &'static str },
    /// プレイ時間（分）
    PlayTime { minutes: u32 },
    /// アイテム収集
    CollectItem { item: ItemId, count: u32 },
    /// 初回イベント
    FirstTime { event: &'static str },
}

/// 実績定義
#[derive(Debug, Clone)]
pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub condition: AchievementCondition,
    pub icon: Option<&'static str>,
    pub hidden: bool,
}

/// 実績の進捗状態
#[derive(Debug, Clone, Default)]
pub struct AchievementProgress {
    pub current: u32,
    pub target: u32,
    pub unlocked: bool,
    pub unlock_time: Option<f64>,
}

/// プレイヤーの実績状態
#[derive(Resource, Debug, Default)]
pub struct PlayerAchievements {
    pub progress: std::collections::HashMap<String, AchievementProgress>,
    pub total_unlocked: u32,
}

impl PlayerAchievements {
    /// 実績をアンロック
    pub fn unlock(&mut self, id: &str, time: f64) {
        if let Some(progress) = self.progress.get_mut(id) {
            if !progress.unlocked {
                progress.unlocked = true;
                progress.unlock_time = Some(time);
                self.total_unlocked += 1;
            }
        }
    }

    /// 進捗を更新
    pub fn update_progress(&mut self, id: &str, current: u32) {
        if let Some(progress) = self.progress.get_mut(id) {
            if !progress.unlocked {
                progress.current = current;
            }
        }
    }

    /// 実績がアンロック済みか確認
    pub fn is_unlocked(&self, id: &str) -> bool {
        self.progress.get(id).map(|p| p.unlocked).unwrap_or(false)
    }
}

/// 実績アンロックイベント
#[derive(Event, Debug)]
pub struct AchievementUnlocked {
    pub id: String,
    pub name: String,
}

/// 実績追跡用カウンター
#[derive(Resource, Debug, Default)]
pub struct AchievementCounters {
    /// 設置した機械数
    pub machines_placed: u32,
    /// 設置したブロック数
    pub blocks_placed: u32,
    /// 生産したアイテム数（種類別）
    pub items_produced: std::collections::HashMap<ItemId, u32>,
    /// 納品したアイテム数（種類別）
    pub items_delivered: std::collections::HashMap<ItemId, u32>,
    /// 総納品数
    pub total_delivered: u32,
}

/// 基本実績の定義
pub static ACHIEVEMENTS: LazyLock<Vec<Achievement>> = LazyLock::new(|| {
    vec![
        Achievement {
            id: "first_machine",
            name: "工場長のはじまり",
            description: "最初の機械を設置する",
            condition: AchievementCondition::PlaceMachines { count: 1 },
            icon: None,
            hidden: false,
        },
        Achievement {
            id: "mass_production",
            name: "量産体制",
            description: "機械を10台設置する",
            condition: AchievementCondition::PlaceMachines { count: 10 },
            icon: None,
            hidden: false,
        },
        Achievement {
            id: "first_delivery",
            name: "初出荷",
            description: "アイテムを初めて納品する",
            condition: AchievementCondition::FirstTime {
                event: "item_delivered",
            },
            icon: None,
            hidden: false,
        },
        Achievement {
            id: "iron_producer",
            name: "鉄鋼生産者",
            description: "鉄インゴットを100個生産する",
            condition: AchievementCondition::ProduceItem {
                item: items::iron_ingot(),
                count: 100,
            },
            icon: None,
            hidden: false,
        },
    ]
});

/// 機械設置イベントを購読してカウンターを更新
fn handle_machine_spawned(
    mut events: EventReader<MachineSpawned>,
    mut counters: ResMut<AchievementCounters>,
) {
    for _event in events.read() {
        counters.machines_placed += 1;
    }
}

/// ブロック設置イベントを購読してカウンターを更新
fn handle_block_placed(
    mut events: EventReader<BlockPlaced>,
    mut counters: ResMut<AchievementCounters>,
) {
    for _event in events.read() {
        counters.blocks_placed += 1;
    }
}

/// 機械完了イベントを購読して生産カウンターを更新
fn handle_machine_completed_for_achievements(
    mut events: EventReader<MachineCompleted>,
    mut counters: ResMut<AchievementCounters>,
) {
    for event in events.read() {
        for (item_id, count) in &event.outputs {
            *counters.items_produced.entry(*item_id).or_insert(0) += count;
        }
    }
}

/// 納品イベントを購読してカウンターを更新
fn handle_item_delivered_for_achievements(
    mut events: EventReader<ItemDelivered>,
    mut counters: ResMut<AchievementCounters>,
) {
    for event in events.read() {
        *counters.items_delivered.entry(event.item).or_insert(0) += event.count;
        counters.total_delivered += event.count;
    }
}

/// 実績の進捗をチェックして必要に応じてアンロック
fn check_achievements(
    counters: Res<AchievementCounters>,
    mut achievements: ResMut<PlayerAchievements>,
    mut unlock_events: GuardedEventWriter<AchievementUnlocked>,
    time: Res<Time>,
) {
    // カウンターが変更されていない場合はスキップ
    if !counters.is_changed() {
        return;
    }

    let timestamp = time.elapsed_secs_f64();

    for achievement in ACHIEVEMENTS.iter() {
        // 既にアンロック済みならスキップ
        if achievements.is_unlocked(achievement.id) {
            continue;
        }

        let (current, _target, unlocked) = match &achievement.condition {
            AchievementCondition::PlaceMachines { count } => {
                let current = counters.machines_placed;
                (current, count, current >= *count)
            }
            AchievementCondition::ProduceItem { item, count } => {
                let current = counters.items_produced.get(item).copied().unwrap_or(0);
                (current, count, current >= *count)
            }
            AchievementCondition::FirstTime { event } => {
                let unlocked = match *event {
                    "item_delivered" => counters.total_delivered > 0,
                    _ => false,
                };
                (if unlocked { 1 } else { 0 }, &1u32, unlocked)
            }
            AchievementCondition::CollectItem { item, count } => {
                let current = counters.items_delivered.get(item).copied().unwrap_or(0);
                (current, count, current >= *count)
            }
            _ => continue, // 他の条件は別途処理
        };

        // 進捗を更新
        achievements.update_progress(achievement.id, current);

        // アンロック判定
        if unlocked {
            achievements.unlock(achievement.id, timestamp);
            let _ = unlock_events.send(AchievementUnlocked {
                id: achievement.id.to_string(),
                name: achievement.name.to_string(),
            });
        }
    }
}

/// 実績の初期進捗を設定
fn setup_achievement_progress(mut achievements: ResMut<PlayerAchievements>) {
    for achievement in ACHIEVEMENTS.iter() {
        let target = match &achievement.condition {
            AchievementCondition::PlaceMachines { count } => *count,
            AchievementCondition::ProduceItem { count, .. } => *count,
            AchievementCondition::CollectItem { count, .. } => *count,
            AchievementCondition::PlayTime { minutes } => *minutes,
            AchievementCondition::FirstTime { .. } => 1,
            AchievementCondition::CompleteQuest { .. } => 1,
        };

        achievements.progress.insert(
            achievement.id.to_string(),
            AchievementProgress {
                current: 0,
                target,
                unlocked: false,
                unlock_time: None,
            },
        );
    }
}

pub struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerAchievements>()
            .init_resource::<AchievementCounters>()
            .add_event::<AchievementUnlocked>()
            .add_systems(Startup, setup_achievement_progress)
            .add_systems(
                Update,
                (
                    handle_machine_spawned,
                    handle_block_placed,
                    handle_machine_completed_for_achievements,
                    handle_item_delivered_for_achievements,
                    check_achievements,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

    #[test]
    fn test_achievement_progress_default() {
        let progress = AchievementProgress::default();
        assert!(!progress.unlocked);
        assert_eq!(progress.current, 0);
    }

    #[test]
    fn test_player_achievements_unlock() {
        let mut achievements = PlayerAchievements::default();
        achievements.progress.insert(
            "test".to_string(),
            AchievementProgress {
                current: 0,
                target: 10,
                unlocked: false,
                unlock_time: None,
            },
        );

        achievements.unlock("test", 123.0);

        assert!(achievements.is_unlocked("test"));
        assert_eq!(achievements.total_unlocked, 1);
    }

    #[test]
    fn test_update_progress() {
        let mut achievements = PlayerAchievements::default();
        achievements.progress.insert(
            "test".to_string(),
            AchievementProgress {
                current: 0,
                target: 10,
                unlocked: false,
                unlock_time: None,
            },
        );

        achievements.update_progress("test", 5);

        assert_eq!(achievements.progress.get("test").unwrap().current, 5);
    }

    #[test]
    fn test_condition_variants() {
        let cond = AchievementCondition::ProduceItem {
            item: items::iron_ingot(),
            count: 100,
        };
        assert!(matches!(cond, AchievementCondition::ProduceItem { .. }));
    }

    #[test]
    fn test_achievement_counters() {
        let mut counters = AchievementCounters::default();

        counters.machines_placed = 5;
        counters.blocks_placed = 10;
        counters.items_produced.insert(items::iron_ingot(), 50);
        counters.items_delivered.insert(items::iron_ore(), 20);
        counters.total_delivered = 20;

        assert_eq!(counters.machines_placed, 5);
        assert_eq!(counters.blocks_placed, 10);
        assert_eq!(counters.items_produced.get(&items::iron_ingot()), Some(&50));
        assert_eq!(counters.total_delivered, 20);
    }

    #[test]
    fn test_achievements_list() {
        // 基本実績が定義されていることを確認
        assert!(ACHIEVEMENTS.len() >= 4);

        // first_machine実績が存在することを確認
        let first_machine = ACHIEVEMENTS.iter().find(|a| a.id == "first_machine");
        assert!(first_machine.is_some());
    }
}
