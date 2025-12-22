//! Quest type definitions

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Quest type
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum QuestType {
    #[default]
    Main,
    Sub,
}

impl QuestType {
    /// Parse a string into a QuestType
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sub" => Self::Sub,
            _ => Self::Main,
        }
    }
}

/// Requirement type
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum RequirementType {
    #[default]
    Item,
    Fluid,
    Power,
    Torque,
}

impl RequirementType {
    /// Parse a string into a RequirementType
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fluid" => Self::Fluid,
            "power" => Self::Power,
            "torque" => Self::Torque,
            _ => Self::Item,
        }
    }
}

/// Quest requirement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct QuestRequirement {
    pub item_id: String,
    pub amount: u32,
    #[serde(default)]
    pub item_type: RequirementType,
}

/// Reward type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(tag = "type")]
pub enum RewardType {
    PortUnlock { count: u32 },
    Item { item_id: String, amount: u32 },
}

/// Quest definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct QuestData {
    pub id: String,
    pub i18n_key: String,
    #[serde(default)]
    pub quest_type: QuestType,
    #[serde(default = "default_phase")]
    pub phase: u32,
    #[serde(default)]
    pub requirements: Vec<QuestRequirement>,
    #[serde(default)]
    pub rewards: Vec<RewardType>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
}

fn default_phase() -> u32 {
    1
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_creation() {
        let quest = QuestData::new("test_quest", QuestType::Main)
            .with_phase(1)
            .with_requirement("iron_ingot", 100)
            .with_reward(RewardType::PortUnlock { count: 2 });

        assert_eq!(quest.id, "test_quest");
        assert_eq!(quest.i18n_key, "quest.test_quest");
        assert_eq!(quest.phase, 1);
        assert_eq!(quest.requirements.len(), 1);
        assert_eq!(quest.rewards.len(), 1);
    }

    #[test]
    fn test_quest_with_prerequisites() {
        let quest = QuestData::new("quest2", QuestType::Main)
            .with_prerequisite("quest1");

        assert_eq!(quest.prerequisites.len(), 1);
        assert_eq!(quest.prerequisites[0], "quest1");
    }
}
