//! Event subscription handlers for Mod API
//!
//! Handles event.subscribe and event.unsubscribe JSON-RPC methods.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::modding::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_PARAMS};

/// Supported event types that mods can subscribe to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Item was delivered to a platform (warehouse)
    #[serde(rename = "item.delivered")]
    ItemDelivered,
    /// Machine completed processing
    #[serde(rename = "machine.completed")]
    MachineCompleted,
    /// Block was placed in the world
    #[serde(rename = "block.placed")]
    BlockPlaced,
    /// Block was removed from the world
    #[serde(rename = "block.removed")]
    BlockRemoved,
}

impl EventType {
    /// Get all supported event types
    pub fn all() -> &'static [EventType] {
        &[
            EventType::ItemDelivered,
            EventType::MachineCompleted,
            EventType::BlockPlaced,
            EventType::BlockRemoved,
        ]
    }

    /// Parse event type from string
    pub fn parse(s: &str) -> Option<EventType> {
        match s {
            "item.delivered" => Some(EventType::ItemDelivered),
            "machine.completed" => Some(EventType::MachineCompleted),
            "block.placed" => Some(EventType::BlockPlaced),
            "block.removed" => Some(EventType::BlockRemoved),
            _ => None,
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::ItemDelivered => "item.delivered",
            EventType::MachineCompleted => "machine.completed",
            EventType::BlockPlaced => "block.placed",
            EventType::BlockRemoved => "block.removed",
        }
    }
}

/// A single event subscription
#[derive(Debug, Clone)]
pub struct Subscription {
    /// Unique subscription ID
    pub id: String,
    /// Connection ID that owns this subscription
    pub conn_id: u64,
    /// Event type subscribed to
    pub event_type: EventType,
    /// When the subscription was created (Unix timestamp)
    pub created_at: u64,
}

impl Subscription {
    /// Create a new subscription
    pub fn new(id: String, conn_id: u64, event_type: EventType) -> Self {
        Self {
            id,
            conn_id,
            event_type,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

/// Manages event subscriptions for all connected mods
#[derive(Resource, Default)]
pub struct EventSubscriptions {
    /// All subscriptions (subscription_id -> Subscription)
    subscriptions: HashMap<String, Subscription>,
    /// Subscriptions by event type (for quick lookup when dispatching)
    by_event: HashMap<EventType, HashSet<String>>,
    /// Subscriptions by connection (for cleanup on disconnect)
    by_connection: HashMap<u64, HashSet<String>>,
    /// Counter for generating subscription IDs
    next_id: u64,
}

impl EventSubscriptions {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a new unique subscription ID
    fn generate_id(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("sub_{}", id)
    }

    /// Subscribe a connection to an event type
    ///
    /// Returns the subscription ID on success
    pub fn subscribe(&mut self, conn_id: u64, event_type: EventType) -> String {
        let sub_id = self.generate_id();
        let subscription = Subscription::new(sub_id.clone(), conn_id, event_type);

        // Add to main map
        self.subscriptions.insert(sub_id.clone(), subscription);

        // Add to event type index
        self.by_event
            .entry(event_type)
            .or_default()
            .insert(sub_id.clone());

        // Add to connection index
        self.by_connection
            .entry(conn_id)
            .or_default()
            .insert(sub_id.clone());

        sub_id
    }

    /// Unsubscribe by subscription ID
    ///
    /// Returns true if the subscription existed and was removed
    pub fn unsubscribe(&mut self, subscription_id: &str) -> bool {
        let Some(subscription) = self.subscriptions.remove(subscription_id) else {
            return false;
        };

        // Remove from event type index
        if let Some(set) = self.by_event.get_mut(&subscription.event_type) {
            set.remove(subscription_id);
            if set.is_empty() {
                self.by_event.remove(&subscription.event_type);
            }
        }

        // Remove from connection index
        if let Some(set) = self.by_connection.get_mut(&subscription.conn_id) {
            set.remove(subscription_id);
            if set.is_empty() {
                self.by_connection.remove(&subscription.conn_id);
            }
        }

        true
    }

    /// Remove all subscriptions for a connection (called on disconnect)
    pub fn remove_connection(&mut self, conn_id: u64) {
        let Some(sub_ids) = self.by_connection.remove(&conn_id) else {
            return;
        };

        for sub_id in sub_ids {
            if let Some(subscription) = self.subscriptions.remove(&sub_id) {
                if let Some(set) = self.by_event.get_mut(&subscription.event_type) {
                    set.remove(&sub_id);
                    if set.is_empty() {
                        self.by_event.remove(&subscription.event_type);
                    }
                }
            }
        }
    }

    /// Get all subscription IDs for an event type
    pub fn get_subscribers(&self, event_type: EventType) -> Vec<&Subscription> {
        self.by_event
            .get(&event_type)
            .map(|set| {
                set.iter()
                    .filter_map(|id| self.subscriptions.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all connection IDs subscribed to an event type
    pub fn get_subscriber_connections(&self, event_type: EventType) -> Vec<u64> {
        self.get_subscribers(event_type)
            .iter()
            .map(|s| s.conn_id)
            .collect()
    }

    /// Get a subscription by ID
    pub fn get(&self, subscription_id: &str) -> Option<&Subscription> {
        self.subscriptions.get(subscription_id)
    }

    /// Get all subscriptions for a connection
    pub fn get_by_connection(&self, conn_id: u64) -> Vec<&Subscription> {
        self.by_connection
            .get(&conn_id)
            .map(|set| {
                set.iter()
                    .filter_map(|id| self.subscriptions.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get total subscription count
    pub fn count(&self) -> usize {
        self.subscriptions.len()
    }

    /// Check if a connection is subscribed to an event type
    pub fn is_subscribed(&self, conn_id: u64, event_type: EventType) -> bool {
        self.get_by_connection(conn_id)
            .iter()
            .any(|s| s.event_type == event_type)
    }
}

/// Handle event.subscribe request
pub fn handle_event_subscribe(
    request: &JsonRpcRequest,
    conn_id: u64,
    subscriptions: &mut EventSubscriptions,
) -> JsonRpcResponse {
    // Extract event_type parameter
    let event_type_str = match request.params.get("event_type") {
        Some(serde_json::Value::String(s)) => s.as_str(),
        _ => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                "Missing required parameter: event_type",
            );
        }
    };

    // Parse event type
    let Some(event_type) = EventType::parse(event_type_str) else {
        let valid_types: Vec<&str> = EventType::all().iter().map(|e| e.as_str()).collect();
        return JsonRpcResponse::error_with_data(
            request.id,
            INVALID_PARAMS,
            format!("Unknown event type: {}", event_type_str),
            serde_json::json!({ "valid_types": valid_types }),
        );
    };

    // Create subscription
    let subscription_id = subscriptions.subscribe(conn_id, event_type);

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "success": true,
            "subscription_id": subscription_id
        }),
    )
}

/// Handle event.unsubscribe request
pub fn handle_event_unsubscribe(
    request: &JsonRpcRequest,
    subscriptions: &mut EventSubscriptions,
) -> JsonRpcResponse {
    // Extract subscription_id parameter
    let subscription_id = match request.params.get("subscription_id") {
        Some(serde_json::Value::String(s)) => s.as_str(),
        _ => {
            return JsonRpcResponse::error(
                request.id,
                INVALID_PARAMS,
                "Missing required parameter: subscription_id",
            );
        }
    };

    // Remove subscription
    let removed = subscriptions.unsubscribe(subscription_id);

    if removed {
        JsonRpcResponse::success(
            request.id,
            serde_json::json!({
                "success": true
            }),
        )
    } else {
        JsonRpcResponse::error(
            request.id,
            INVALID_PARAMS,
            format!("Subscription not found: {}", subscription_id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_parse() {
        assert_eq!(
            EventType::parse("item.delivered"),
            Some(EventType::ItemDelivered)
        );
        assert_eq!(
            EventType::parse("machine.completed"),
            Some(EventType::MachineCompleted)
        );
        assert_eq!(
            EventType::parse("block.placed"),
            Some(EventType::BlockPlaced)
        );
        assert_eq!(
            EventType::parse("block.removed"),
            Some(EventType::BlockRemoved)
        );
        assert_eq!(EventType::parse("unknown"), None);
    }

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::ItemDelivered.as_str(), "item.delivered");
        assert_eq!(EventType::MachineCompleted.as_str(), "machine.completed");
        assert_eq!(EventType::BlockPlaced.as_str(), "block.placed");
        assert_eq!(EventType::BlockRemoved.as_str(), "block.removed");
    }

    #[test]
    fn test_event_type_all() {
        let all = EventType::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&EventType::ItemDelivered));
        assert!(all.contains(&EventType::MachineCompleted));
        assert!(all.contains(&EventType::BlockPlaced));
        assert!(all.contains(&EventType::BlockRemoved));
    }

    #[test]
    fn test_subscription_new() {
        let sub = Subscription::new("sub_1".to_string(), 42, EventType::ItemDelivered);

        assert_eq!(sub.id, "sub_1");
        assert_eq!(sub.conn_id, 42);
        assert_eq!(sub.event_type, EventType::ItemDelivered);
        assert!(sub.created_at > 0);
    }

    #[test]
    fn test_event_subscriptions_subscribe() {
        let mut subs = EventSubscriptions::new();

        let sub_id = subs.subscribe(1, EventType::ItemDelivered);
        assert!(sub_id.starts_with("sub_"));

        let sub = subs.get(&sub_id).unwrap();
        assert_eq!(sub.conn_id, 1);
        assert_eq!(sub.event_type, EventType::ItemDelivered);
    }

    #[test]
    fn test_event_subscriptions_unsubscribe() {
        let mut subs = EventSubscriptions::new();

        let sub_id = subs.subscribe(1, EventType::ItemDelivered);
        assert_eq!(subs.count(), 1);

        // Unsubscribe should succeed
        assert!(subs.unsubscribe(&sub_id));
        assert_eq!(subs.count(), 0);

        // Second unsubscribe should fail
        assert!(!subs.unsubscribe(&sub_id));
    }

    #[test]
    fn test_event_subscriptions_remove_connection() {
        let mut subs = EventSubscriptions::new();

        // Connection 1 subscribes to two events
        subs.subscribe(1, EventType::ItemDelivered);
        subs.subscribe(1, EventType::MachineCompleted);

        // Connection 2 subscribes to one event
        subs.subscribe(2, EventType::ItemDelivered);

        assert_eq!(subs.count(), 3);

        // Remove connection 1
        subs.remove_connection(1);
        assert_eq!(subs.count(), 1);

        // Connection 2's subscription should remain
        assert_eq!(
            subs.get_subscriber_connections(EventType::ItemDelivered)
                .len(),
            1
        );
    }

    #[test]
    fn test_event_subscriptions_get_subscribers() {
        let mut subs = EventSubscriptions::new();

        subs.subscribe(1, EventType::ItemDelivered);
        subs.subscribe(2, EventType::ItemDelivered);
        subs.subscribe(3, EventType::MachineCompleted);

        let item_subs = subs.get_subscribers(EventType::ItemDelivered);
        assert_eq!(item_subs.len(), 2);

        let machine_subs = subs.get_subscribers(EventType::MachineCompleted);
        assert_eq!(machine_subs.len(), 1);

        let block_subs = subs.get_subscribers(EventType::BlockPlaced);
        assert_eq!(block_subs.len(), 0);
    }

    #[test]
    fn test_event_subscriptions_get_subscriber_connections() {
        let mut subs = EventSubscriptions::new();

        subs.subscribe(1, EventType::ItemDelivered);
        subs.subscribe(2, EventType::ItemDelivered);

        let conns = subs.get_subscriber_connections(EventType::ItemDelivered);
        assert!(conns.contains(&1));
        assert!(conns.contains(&2));
    }

    #[test]
    fn test_event_subscriptions_get_by_connection() {
        let mut subs = EventSubscriptions::new();

        subs.subscribe(1, EventType::ItemDelivered);
        subs.subscribe(1, EventType::MachineCompleted);

        let conn_subs = subs.get_by_connection(1);
        assert_eq!(conn_subs.len(), 2);

        let empty_subs = subs.get_by_connection(999);
        assert_eq!(empty_subs.len(), 0);
    }

    #[test]
    fn test_event_subscriptions_is_subscribed() {
        let mut subs = EventSubscriptions::new();

        subs.subscribe(1, EventType::ItemDelivered);

        assert!(subs.is_subscribed(1, EventType::ItemDelivered));
        assert!(!subs.is_subscribed(1, EventType::MachineCompleted));
        assert!(!subs.is_subscribed(2, EventType::ItemDelivered));
    }

    #[test]
    fn test_handle_event_subscribe_success() {
        let mut subs = EventSubscriptions::new();

        let request = JsonRpcRequest::new(
            1,
            "event.subscribe",
            serde_json::json!({ "event_type": "item.delivered" }),
        );

        let response = handle_event_subscribe(&request, 42, &mut subs);

        assert!(response.is_success());
        let result = response.result.unwrap();
        assert_eq!(result["success"], true);
        assert!(result["subscription_id"]
            .as_str()
            .unwrap()
            .starts_with("sub_"));
    }

    #[test]
    fn test_handle_event_subscribe_missing_param() {
        let mut subs = EventSubscriptions::new();

        let request = JsonRpcRequest::new(1, "event.subscribe", serde_json::json!({}));

        let response = handle_event_subscribe(&request, 42, &mut subs);

        assert!(response.is_error());
        assert!(response.error.unwrap().message.contains("event_type"));
    }

    #[test]
    fn test_handle_event_subscribe_invalid_event_type() {
        let mut subs = EventSubscriptions::new();

        let request = JsonRpcRequest::new(
            1,
            "event.subscribe",
            serde_json::json!({ "event_type": "invalid.event" }),
        );

        let response = handle_event_subscribe(&request, 42, &mut subs);

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert!(error.message.contains("Unknown event type"));
        // Should include valid types in data
        assert!(error.data.is_some());
    }

    #[test]
    fn test_handle_event_unsubscribe_success() {
        let mut subs = EventSubscriptions::new();
        let sub_id = subs.subscribe(42, EventType::ItemDelivered);

        let request = JsonRpcRequest::new(
            1,
            "event.unsubscribe",
            serde_json::json!({ "subscription_id": sub_id }),
        );

        let response = handle_event_unsubscribe(&request, &mut subs);

        assert!(response.is_success());
        assert_eq!(subs.count(), 0);
    }

    #[test]
    fn test_handle_event_unsubscribe_missing_param() {
        let mut subs = EventSubscriptions::new();

        let request = JsonRpcRequest::new(1, "event.unsubscribe", serde_json::json!({}));

        let response = handle_event_unsubscribe(&request, &mut subs);

        assert!(response.is_error());
        assert!(response.error.unwrap().message.contains("subscription_id"));
    }

    #[test]
    fn test_handle_event_unsubscribe_not_found() {
        let mut subs = EventSubscriptions::new();

        let request = JsonRpcRequest::new(
            1,
            "event.unsubscribe",
            serde_json::json!({ "subscription_id": "nonexistent" }),
        );

        let response = handle_event_unsubscribe(&request, &mut subs);

        assert!(response.is_error());
        assert!(response.error.unwrap().message.contains("not found"));
    }

    #[test]
    fn test_event_type_serialization() {
        // Test that event types serialize correctly
        let event = EventType::ItemDelivered;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "\"item.delivered\"");

        // Test deserialization
        let parsed: EventType = serde_json::from_str("\"machine.completed\"").unwrap();
        assert_eq!(parsed, EventType::MachineCompleted);
    }
}
