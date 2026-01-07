//! Achievement system

use crate::block_type::BlockType;
use bevy::prelude::*;

/// 実績の条件
#[derive(Debug, Clone, PartialEq)]
pub enum AchievementCondition {
    /// 特定アイテムを一定数生産
    ProduceItem { item: BlockType, count: u32 },
    /// 機械を一定数設置
    PlaceMachines { count: u32 },
    /// 特定クエスト完了
    CompleteQuest { quest_id: &'static str },
    /// プレイ時間（分）
    PlayTime { minutes: u32 },
    /// アイテム収集
    CollectItem { item: BlockType, count: u32 },
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

pub struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerAchievements>()
            .add_event::<AchievementUnlocked>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            item: BlockType::IronIngot,
            count: 100,
        };
        assert!(matches!(cond, AchievementCondition::ProduceItem { .. }));
    }
}
