// src/gameplay/quest.rs
//! クエストシステム
//! - QuestData: クエスト定義
//! - QuestProgress: プレイヤーのクエスト進捗
//! - QuestManager: クエスト状態管理

use bevy::prelude::*;
use std::collections::HashMap;

/// 報酬タイプ
#[derive(Debug, Clone, PartialEq)]
pub enum RewardType {
    /// 納品ポートの解放（個数）
    PortUnlock(u32),
    /// アイテム報酬
    Item { item_id: String, amount: u32 },
}

/// クエスト要件
#[derive(Debug, Clone, PartialEq)]
pub struct QuestRequirement {
    pub item_id: String,
    pub amount: u32,
    pub item_type: RequirementType,
}

/// 要件タイプ
#[derive(Debug, Clone, PartialEq, Default)]
pub enum RequirementType {
    #[default]
    Item,
    Fluid,
    Power,
    Torque,
}

/// クエストタイプ
#[derive(Debug, Clone, PartialEq, Default)]
pub enum QuestType {
    #[default]
    Main,
    Sub,
}

/// クエスト定義
#[derive(Debug, Clone)]
pub struct QuestData {
    pub id: String,
    pub i18n_key: String,
    pub quest_type: QuestType,
    pub phase: u32,
    pub requirements: Vec<QuestRequirement>,
    pub rewards: Vec<RewardType>,
    pub prerequisites: Vec<String>, // 事前に完了が必要なクエストID
}

impl QuestData {
    pub fn new(id: impl Into<String>, quest_type: QuestType) -> Self {
        let id = id.into();
        Self {
            i18n_key: format!("quest.{}", &id),
            id,
            quest_type,
            phase: 1,
            requirements: Vec::new(),
            rewards: Vec::new(),
            prerequisites: Vec::new(),
        }
    }

    pub fn with_phase(mut self, phase: u32) -> Self {
        self.phase = phase;
        self
    }

    pub fn with_requirement(mut self, item_id: impl Into<String>, amount: u32) -> Self {
        self.requirements.push(QuestRequirement {
            item_id: item_id.into(),
            amount,
            item_type: RequirementType::Item,
        });
        self
    }

    pub fn with_reward(mut self, reward: RewardType) -> Self {
        self.rewards.push(reward);
        self
    }

    pub fn with_prerequisite(mut self, quest_id: impl Into<String>) -> Self {
        self.prerequisites.push(quest_id.into());
        self
    }
}

/// クエスト状態
#[derive(Debug, Clone, PartialEq, Default)]
pub enum QuestStatus {
    #[default]
    Locked,    // 前提クエスト未完了
    Available, // 受注可能
    Active,    // 進行中
    Completed, // 完了
}

/// クエスト進捗
#[derive(Debug, Clone)]
pub struct QuestProgress {
    pub quest_id: String,
    pub status: QuestStatus,
    pub delivered: HashMap<String, u32>, // アイテムID -> 納品済み数量
}

impl QuestProgress {
    pub fn new(quest_id: impl Into<String>) -> Self {
        Self {
            quest_id: quest_id.into(),
            status: QuestStatus::Locked,
            delivered: HashMap::new(),
        }
    }

    /// 納品済み数量を取得
    pub fn get_delivered(&self, item_id: &str) -> u32 {
        *self.delivered.get(item_id).unwrap_or(&0)
    }

    /// アイテムを納品
    pub fn deliver(&mut self, item_id: &str, amount: u32) {
        *self.delivered.entry(item_id.to_string()).or_insert(0) += amount;
    }
}

/// クエストレジストリ
#[derive(Resource, Default)]
pub struct QuestRegistry {
    pub quests: HashMap<String, QuestData>,
    pub main_quest_order: Vec<String>, // メインクエストの順序
    pub sub_quests_by_phase: HashMap<u32, Vec<String>>, // フェーズ -> サブクエストID
}

impl QuestRegistry {
    pub fn register(&mut self, quest: QuestData) {
        match quest.quest_type {
            QuestType::Main => {
                self.main_quest_order.push(quest.id.clone());
            }
            QuestType::Sub => {
                self.sub_quests_by_phase
                    .entry(quest.phase)
                    .or_default()
                    .push(quest.id.clone());
            }
        }
        self.quests.insert(quest.id.clone(), quest);
    }

    pub fn get(&self, id: &str) -> Option<&QuestData> {
        self.quests.get(id)
    }
}

/// プレイヤーのクエスト進捗管理
#[derive(Resource, Default)]
pub struct QuestManager {
    pub progress: HashMap<String, QuestProgress>,
    pub current_phase: u32, // 現在のフェーズ
    pub unlocked_ports: u32, // 解放済みポート数
}

impl QuestManager {
    pub fn new() -> Self {
        Self {
            progress: HashMap::new(),
            current_phase: 1,
            unlocked_ports: 3, // 初期ポート数
        }
    }

    /// クエスト進捗を取得または作成
    pub fn get_or_create(&mut self, quest_id: &str) -> &mut QuestProgress {
        if !self.progress.contains_key(quest_id) {
            self.progress
                .insert(quest_id.to_string(), QuestProgress::new(quest_id));
        }
        self.progress.get_mut(quest_id).unwrap()
    }

    /// クエストが完了しているかチェック
    pub fn is_completed(&self, quest_id: &str) -> bool {
        self.progress
            .get(quest_id)
            .is_some_and(|p| p.status == QuestStatus::Completed)
    }

    /// 前提条件を満たしているかチェック
    pub fn can_unlock(&self, quest: &QuestData) -> bool {
        quest
            .prerequisites
            .iter()
            .all(|prereq| self.is_completed(prereq))
    }

    /// クエスト要件を全て満たしているかチェック
    pub fn check_requirements_met(&self, quest: &QuestData) -> bool {
        if let Some(progress) = self.progress.get(&quest.id) {
            quest.requirements.iter().all(|req| {
                progress.get_delivered(&req.item_id) >= req.amount
            })
        } else {
            false
        }
    }
}

/// クエスト完了イベント
#[derive(Event)]
pub struct QuestCompletedEvent {
    pub quest_id: String,
    pub rewards: Vec<RewardType>,
}

/// クエスト開始イベント
#[derive(Event)]
pub struct QuestStartedEvent {
    pub quest_id: String,
}

/// クエストプラグイン
pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuestRegistry>()
            .insert_resource(QuestManager::new())
            .add_event::<QuestCompletedEvent>()
            .add_event::<QuestStartedEvent>()
            .add_systems(Startup, load_quests)
            .add_systems(Update, (
                update_quest_availability,
                check_quest_completion,
                apply_quest_rewards,
            ));
    }
}

/// クエスト定義をロード
fn load_quests(mut registry: ResMut<QuestRegistry>) {
    // Phase 1: 基本生産
    registry.register(
        QuestData::new("main_1_iron_ingots", QuestType::Main)
            .with_phase(1)
            .with_requirement("iron_ingot", 100)
            .with_reward(RewardType::PortUnlock(2)),
    );

    registry.register(
        QuestData::new("main_2_copper_wires", QuestType::Main)
            .with_phase(1)
            .with_requirement("copper_wire", 200)
            .with_prerequisite("main_1_iron_ingots")
            .with_reward(RewardType::PortUnlock(2)),
    );

    // Phase 1 サブクエスト
    registry.register(
        QuestData::new("sub_coal_delivery", QuestType::Sub)
            .with_phase(1)
            .with_requirement("coal", 500)
            .with_reward(RewardType::Item {
                item_id: "speed_upgrade".to_string(),
                amount: 5,
            }),
    );

    registry.register(
        QuestData::new("sub_stone_delivery", QuestType::Sub)
            .with_phase(1)
            .with_requirement("stone", 1000)
            .with_reward(RewardType::Item {
                item_id: "storage_upgrade".to_string(),
                amount: 3,
            }),
    );

    info!(
        "Loaded {} quests ({} main, {} sub phases)",
        registry.quests.len(),
        registry.main_quest_order.len(),
        registry.sub_quests_by_phase.len()
    );
}

/// クエストの利用可能状態を更新
fn update_quest_availability(
    registry: Res<QuestRegistry>,
    mut manager: ResMut<QuestManager>,
) {
    // まずアンロック可能なクエストを収集
    let quests_to_unlock: Vec<String> = registry
        .quests
        .iter()
        .filter_map(|(quest_id, quest)| {
            let is_locked = manager
                .progress
                .get(quest_id)
                .map(|p| p.status == QuestStatus::Locked)
                .unwrap_or(true);

            if is_locked && manager.can_unlock(quest) {
                Some(quest_id.clone())
            } else {
                None
            }
        })
        .collect();

    // 収集したクエストをアンロック
    for quest_id in quests_to_unlock {
        let progress = manager.get_or_create(&quest_id);
        progress.status = QuestStatus::Available;
    }
}

/// クエスト完了チェック
fn check_quest_completion(
    registry: Res<QuestRegistry>,
    mut manager: ResMut<QuestManager>,
    mut ev_completed: EventWriter<QuestCompletedEvent>,
) {
    for (quest_id, quest) in registry.quests.iter() {
        if let Some(progress) = manager.progress.get(quest_id) {
            if progress.status == QuestStatus::Active && manager.check_requirements_met(quest) {
                // 完了処理
                if let Some(progress) = manager.progress.get_mut(quest_id) {
                    progress.status = QuestStatus::Completed;
                }

                ev_completed.send(QuestCompletedEvent {
                    quest_id: quest_id.clone(),
                    rewards: quest.rewards.clone(),
                });

                info!("Quest completed: {}", quest_id);
            }
        }
    }
}

/// クエスト報酬を適用
fn apply_quest_rewards(
    mut ev_completed: EventReader<QuestCompletedEvent>,
    mut manager: ResMut<QuestManager>,
    mut inventory: ResMut<super::inventory::PlayerInventory>,
    registry: Res<super::inventory::ItemRegistry>,
) {
    for event in ev_completed.read() {
        for reward in &event.rewards {
            match reward {
                RewardType::PortUnlock(count) => {
                    manager.unlocked_ports += count;
                    info!("Unlocked {} new ports (total: {})", count, manager.unlocked_ports);
                }
                RewardType::Item { item_id, amount } => {
                    let remaining = inventory.add_item(item_id.clone(), *amount, &registry);
                    if remaining > 0 {
                        warn!("Could not add {} items to inventory (full)", remaining);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_creation() {
        let quest = QuestData::new("test_quest", QuestType::Main)
            .with_phase(1)
            .with_requirement("iron_ingot", 100)
            .with_reward(RewardType::PortUnlock(2));

        assert_eq!(quest.id, "test_quest");
        assert_eq!(quest.phase, 1);
        assert_eq!(quest.requirements.len(), 1);
        assert_eq!(quest.rewards.len(), 1);
    }

    #[test]
    fn test_quest_progress() {
        let mut progress = QuestProgress::new("test_quest");
        assert_eq!(progress.get_delivered("iron_ingot"), 0);

        progress.deliver("iron_ingot", 50);
        assert_eq!(progress.get_delivered("iron_ingot"), 50);

        progress.deliver("iron_ingot", 30);
        assert_eq!(progress.get_delivered("iron_ingot"), 80);
    }

    #[test]
    fn test_quest_manager_prerequisites() {
        let mut manager = QuestManager::new();

        let _quest1 = QuestData::new("quest1", QuestType::Main);
        let quest2 = QuestData::new("quest2", QuestType::Main).with_prerequisite("quest1");

        // quest1が完了していない場合、quest2はロック
        assert!(!manager.can_unlock(&quest2));

        // quest1を完了
        manager.get_or_create("quest1").status = QuestStatus::Completed;

        // quest2がアンロック可能に
        assert!(manager.can_unlock(&quest2));
    }

    #[test]
    fn test_requirements_check() {
        let mut manager = QuestManager::new();

        let quest = QuestData::new("test_quest", QuestType::Main)
            .with_requirement("iron_ingot", 100);

        // 進捗なし
        assert!(!manager.check_requirements_met(&quest));

        // 進捗追加（不足）
        let progress = manager.get_or_create("test_quest");
        progress.status = QuestStatus::Active;
        progress.deliver("iron_ingot", 50);
        assert!(!manager.check_requirements_met(&quest));

        // 進捗追加（達成）
        let progress = manager.get_or_create("test_quest");
        progress.deliver("iron_ingot", 50);
        assert!(manager.check_requirements_met(&quest));
    }
}
