//! Quest API handlers for E2E testing
//!
//! Provides APIs to:
//! - List all quests
//! - Get quest details
//! - Check quest progress

use super::super::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};
use serde::{Deserialize, Serialize};

// Error codes
pub const QUEST_NOT_FOUND: i32 = -32120;

/// Quest info for API responses
#[derive(Serialize, Clone)]
pub struct QuestInfo {
    pub id: String,
    pub quest_type: String,
    pub description: String,
    pub required_items: Vec<RequiredItemInfo>,
    pub rewards: Vec<RewardInfo>,
    pub unlocks: Vec<String>,
}

/// Required item info
#[derive(Serialize, Clone)]
pub struct RequiredItemInfo {
    pub item_id: String,
    pub required: u32,
    pub current: u32,
}

/// Reward info
#[derive(Serialize, Clone)]
pub struct RewardInfo {
    pub item_id: String,
    pub amount: u32,
}

/// Quest state info for E2E testing
#[derive(Default, Clone)]
pub struct QuestStateInfo {
    /// All quests with their current progress
    pub quests: Vec<QuestProgressInfo>,
}

/// Quest progress info
#[derive(Clone, Serialize)]
pub struct QuestProgressInfo {
    pub id: String,
    pub quest_type: String,
    pub description: String,
    pub complete: bool,
    pub progress: Vec<ItemProgressInfo>,
}

/// Item progress info
#[derive(Clone, Serialize)]
pub struct ItemProgressInfo {
    pub item_id: String,
    pub required: u32,
    pub current: u32,
}

// === quest.list ===

/// Handle quest.list request
///
/// Returns all quests with their progress.
///
/// # Response
/// ```json
/// {
///   "quests": [
///     {
///       "id": "main_1",
///       "quest_type": "Main",
///       "description": "...",
///       "complete": false,
///       "progress": [{ "item_id": "base:iron_ingot", "required": 10, "current": 5 }]
///     }
///   ]
/// }
/// ```
pub fn handle_quest_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    use crate::game_spec::{main_quests, sub_quests};

    let mut quests = Vec::new();

    // Add main quests
    for quest in main_quests() {
        let progress: Vec<ItemProgressInfo> = quest
            .required_items
            .iter()
            .map(|(item_id, required)| ItemProgressInfo {
                item_id: item_id.name().unwrap_or("unknown").to_string(),
                required: *required,
                current: 0, // TODO: Get actual progress from player inventory/delivered items
            })
            .collect();

        quests.push(QuestProgressInfo {
            id: quest.id.to_string(),
            quest_type: "Main".to_string(),
            description: quest.description.to_string(),
            complete: false, // TODO: Check actual completion status
            progress,
        });
    }

    // Add sub quests
    for quest in sub_quests() {
        let progress: Vec<ItemProgressInfo> = quest
            .required_items
            .iter()
            .map(|(item_id, required)| ItemProgressInfo {
                item_id: item_id.name().unwrap_or("unknown").to_string(),
                required: *required,
                current: 0,
            })
            .collect();

        quests.push(QuestProgressInfo {
            id: quest.id.to_string(),
            quest_type: "Sub".to_string(),
            description: quest.description.to_string(),
            complete: false,
            progress,
        });
    }

    JsonRpcResponse::success(request.id, serde_json::json!({ "quests": quests }))
}

// === quest.get ===

#[derive(Deserialize)]
pub struct GetQuestParams {
    pub id: String,
}

/// Handle quest.get request
///
/// Returns details for a specific quest.
///
/// # Request
/// ```json
/// { "id": "main_1" }
/// ```
///
/// # Response
/// ```json
/// {
///   "id": "main_1",
///   "quest_type": "Main",
///   "description": "...",
///   "required_items": [...],
///   "rewards": [...],
///   "unlocks": [...]
/// }
/// ```
pub fn handle_quest_get(request: &JsonRpcRequest) -> JsonRpcResponse {
    use crate::game_spec::{main_quests, sub_quests};

    let params: GetQuestParams = match serde_json::from_value(request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                format!("Invalid params: {}", e),
            );
        }
    };

    // Find quest in main or sub quests
    let quest = main_quests()
        .iter()
        .chain(sub_quests().iter())
        .find(|q| q.id == params.id);

    match quest {
        Some(q) => {
            let required_items: Vec<RequiredItemInfo> = q
                .required_items
                .iter()
                .map(|(item_id, required)| RequiredItemInfo {
                    item_id: item_id.name().unwrap_or("unknown").to_string(),
                    required: *required,
                    current: 0, // TODO: Get actual progress
                })
                .collect();

            let rewards: Vec<RewardInfo> = q
                .rewards
                .iter()
                .map(|(item_id, amount)| RewardInfo {
                    item_id: item_id.name().unwrap_or("unknown").to_string(),
                    amount: *amount,
                })
                .collect();

            let unlocks: Vec<String> = q
                .unlocks
                .iter()
                .filter_map(|item_id| item_id.name().map(|s| s.to_string()))
                .collect();

            let info = QuestInfo {
                id: q.id.to_string(),
                quest_type: format!("{:?}", q.quest_type),
                description: q.description.to_string(),
                required_items,
                rewards,
                unlocks,
            };

            JsonRpcResponse::success(request.id, serde_json::to_value(info).unwrap())
        }
        None => JsonRpcResponse::error(
            request.id,
            QUEST_NOT_FOUND,
            format!("Quest not found: {}", params.id),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::protocol::JsonRpcRequest;

    #[test]
    fn test_handle_quest_list() {
        let request = JsonRpcRequest::new(1, "quest.list", serde_json::Value::Null);
        let response = handle_quest_list(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        let quests = result["quests"].as_array().unwrap();
        // Should have at least main quests
        assert!(!quests.is_empty());
    }

    #[test]
    fn test_handle_quest_get() {
        let request = JsonRpcRequest::new(1, "quest.get", serde_json::json!({ "id": "main_1" }));
        let response = handle_quest_get(&request);
        assert!(response.is_success());

        let result = response.result.unwrap();
        assert_eq!(result["id"], "main_1");
        assert_eq!(result["quest_type"], "Main");
    }

    #[test]
    fn test_handle_quest_get_not_found() {
        let request =
            JsonRpcRequest::new(1, "quest.get", serde_json::json!({ "id": "nonexistent" }));
        let response = handle_quest_get(&request);
        assert!(response.is_error());
        assert_eq!(response.error.unwrap().code, QUEST_NOT_FOUND);
    }
}
