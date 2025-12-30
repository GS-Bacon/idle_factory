//! Game specification as code
//!
//! This file is the Single Source of Truth for game design.
//! If you change the spec, update this file. Tests will verify implementation matches.
//!
//! Reference: .specify/specs/first-30-minutes.md

use crate::block_type::BlockType;

/// Initial player equipment
/// Spec: first-30-minutes.md - "初期装備"
pub const INITIAL_EQUIPMENT: &[(BlockType, u32)] = &[
    // Current implementation (to be discussed with user)
    (BlockType::MinerBlock, 3),
    (BlockType::ConveyorBlock, 10),
    (BlockType::CrusherBlock, 2),
    (BlockType::FurnaceBlock, 2),
    (BlockType::IronOre, 5),
    (BlockType::Coal, 5),
];

/// Items dropped on ground at spawn
/// Spec: "スポーン地点に鉄鉱石×5、石炭×5が落ちている"
pub const SPAWN_GROUND_ITEMS: &[(BlockType, u32)] = &[
    // TODO: Implement dropped items
    // (BlockType::IronOre, 5),
    // (BlockType::Coal, 5),
];

/// Quest definitions
/// Spec: first-30-minutes.md
pub struct QuestSpec {
    pub description: &'static str,
    pub required_item: BlockType,
    pub required_amount: u32,
    pub rewards: &'static [(BlockType, u32)],
}

pub const QUESTS: &[QuestSpec] = &[
    // Quest 1: 手動フェーズ完了
    QuestSpec {
        description: "Deliver 3 Iron Ingots",
        required_item: BlockType::IronIngot,
        required_amount: 3,
        rewards: &[
            (BlockType::MinerBlock, 2),
            (BlockType::ConveyorBlock, 20),
        ],
    },
    // Quest 2: 自動化フェーズ
    // Spec: "鉄インゴットを100個納品せよ"
    // Current impl: 銅インゴット10個 (needs fix)
    QuestSpec {
        description: "Deliver 100 Iron Ingots",
        required_item: BlockType::IronIngot,
        required_amount: 100,
        rewards: &[
            (BlockType::MinerBlock, 2),
            (BlockType::ConveyorBlock, 40),
            (BlockType::FurnaceBlock, 2),
        ],
    },
    // Quest 3: スケールアップ
    QuestSpec {
        description: "Deliver 50 Copper Ingots",
        required_item: BlockType::CopperIngot,
        required_amount: 50,
        rewards: &[
            (BlockType::CrusherBlock, 2),
            (BlockType::FurnaceBlock, 2),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify quest progression makes sense
    #[test]
    fn test_quest_progression() {
        // Quest 1 should be easy (<=10 items)
        assert!(QUESTS[0].required_amount <= 10,
            "Quest 1 should be easy for manual phase");

        // Quest 2 should require automation (>=50 items)
        assert!(QUESTS[1].required_amount >= 50,
            "Quest 2 should require automation");
    }

    /// Verify rewards are reasonable
    #[test]
    fn test_quest_rewards_not_empty() {
        for (i, quest) in QUESTS.iter().enumerate() {
            assert!(!quest.rewards.is_empty(),
                "Quest {} should have rewards", i + 1);
        }
    }

    /// Verify initial equipment exists
    #[test]
    fn test_initial_equipment_not_empty() {
        assert!(!INITIAL_EQUIPMENT.is_empty(),
            "Player should start with some equipment");
    }
}
