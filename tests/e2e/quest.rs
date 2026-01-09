//! Quest and delivery platform tests

use super::common::*;
use idle_factory::core::items;
use idle_factory::ItemId;
use std::collections::HashMap;

// ============================================================================
// Quest Types
// ============================================================================

#[derive(Clone)]
struct QuestDef {
    target_item: ItemId,
    required_count: u32,
    reward_items: Vec<(ItemId, u32)>,
}

struct CurrentQuest {
    #[allow(dead_code)]
    index: usize,
    progress: u32,
    completed: bool,
    rewards_claimed: bool,
}

impl CurrentQuest {
    fn new(index: usize) -> Self {
        Self {
            index,
            progress: 0,
            completed: false,
            rewards_claimed: false,
        }
    }

    fn add_progress(&mut self, quest: &QuestDef, amount: u32) {
        if self.completed {
            return;
        }
        self.progress += amount;
        if self.progress >= quest.required_count {
            self.completed = true;
        }
    }

    fn claim_rewards(&mut self, quest: &QuestDef, inventory: &mut SlotInventory) -> bool {
        if !self.completed || self.rewards_claimed {
            return false;
        }
        for (item, count) in &quest.reward_items {
            inventory.add_item(*item, *count);
        }
        self.rewards_claimed = true;
        true
    }
}

struct DeliveryPlatform {
    delivered: HashMap<ItemId, u32>,
}

impl DeliveryPlatform {
    fn new() -> Self {
        Self {
            delivered: HashMap::new(),
        }
    }

    fn deliver(&mut self, item: ItemId) {
        *self.delivered.entry(item).or_insert(0) += 1;
    }

    fn get_delivered(&self, item: ItemId) -> u32 {
        *self.delivered.get(&item).unwrap_or(&0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_quest_progress() {
    let quest = QuestDef {
        target_item: items::stone(),
        required_count: 3,
        reward_items: vec![(items::grass(), 10)],
    };

    let mut current = CurrentQuest::new(0);

    current.add_progress(&quest, 1);
    assert_eq!(current.progress, 1);
    assert!(!current.completed);

    current.add_progress(&quest, 2);
    assert_eq!(current.progress, 3);
    assert!(current.completed);
}

#[test]
fn test_quest_rewards() {
    let quest = QuestDef {
        target_item: items::stone(),
        required_count: 1,
        reward_items: vec![(items::grass(), 5), (items::stone(), 3)],
    };

    let mut current = CurrentQuest::new(0);
    let mut inventory = SlotInventory::default();

    assert!(!current.claim_rewards(&quest, &mut inventory));

    current.add_progress(&quest, 1);
    assert!(current.completed);

    assert!(current.claim_rewards(&quest, &mut inventory));
    assert_eq!(inventory.get_slot_count(0), 5);
    assert_eq!(inventory.get_slot_count(1), 3);

    assert!(!current.claim_rewards(&quest, &mut inventory));
}

#[test]
fn test_delivery_platform() {
    let mut platform = DeliveryPlatform::new();

    platform.deliver(items::stone());
    platform.deliver(items::stone());
    platform.deliver(items::grass());

    assert_eq!(platform.get_delivered(items::stone()), 2);
    assert_eq!(platform.get_delivered(items::grass()), 1);
}

#[test]
fn test_delivery_updates_quest() {
    let quest = QuestDef {
        target_item: items::stone(),
        required_count: 5,
        reward_items: vec![],
    };

    let mut current = CurrentQuest::new(0);
    let mut platform = DeliveryPlatform::new();

    for _ in 0..5 {
        platform.deliver(items::stone());
        current.add_progress(&quest, 1);
    }

    assert_eq!(platform.get_delivered(items::stone()), 5);
    assert!(current.completed);
}
