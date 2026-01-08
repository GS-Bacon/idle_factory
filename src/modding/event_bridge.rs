//! Event bridge: forwards Bevy game events to subscribed Mod connections
//!
//! This module bridges Bevy's event system with the Mod API WebSocket server,
//! enabling mods to receive real-time notifications about game events.

use bevy::prelude::*;
use serde_json::json;

use crate::events::{BlockBreakEvent, BlockPlaceEvent, ItemTransferEvent};

use super::handlers::events::{EventSubscriptions, EventType};
use super::protocol::JsonRpcNotification;
use super::server::{ClientMessage, ModApiServer};

/// Plugin for event bridge system
pub struct EventBridgePlugin;

impl Plugin for EventBridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                bridge_block_place_events,
                bridge_block_break_events,
                bridge_item_transfer_events,
            )
                .run_if(resource_exists::<ModApiServer>),
        );
    }
}

/// Bridge BlockPlaceEvent → EventType::BlockPlaced
fn bridge_block_place_events(
    mut events: EventReader<BlockPlaceEvent>,
    subscriptions: Res<EventSubscriptions>,
    server: Res<ModApiServer>,
) {
    for event in events.read() {
        let conn_ids = subscriptions.get_subscriber_connections(EventType::BlockPlaced);
        if conn_ids.is_empty() {
            continue;
        }

        let notification = JsonRpcNotification::new(
            "event.block_placed",
            json!({
                "position": {
                    "x": event.position.x,
                    "y": event.position.y,
                    "z": event.position.z
                },
                "block_type": event.item_id.name().unwrap_or("unknown"),
                "player_id": event.player_id
            }),
        );

        for conn_id in conn_ids {
            let _ = server.tx.send(ClientMessage::Notify {
                conn_id,
                notification: notification.clone(),
            });
        }
    }
}

/// Bridge BlockBreakEvent → EventType::BlockRemoved
fn bridge_block_break_events(
    mut events: EventReader<BlockBreakEvent>,
    subscriptions: Res<EventSubscriptions>,
    server: Res<ModApiServer>,
) {
    for event in events.read() {
        let conn_ids = subscriptions.get_subscriber_connections(EventType::BlockRemoved);
        if conn_ids.is_empty() {
            continue;
        }

        let notification = JsonRpcNotification::new(
            "event.block_removed",
            json!({
                "position": {
                    "x": event.position.x,
                    "y": event.position.y,
                    "z": event.position.z
                },
                "player_id": event.player_id
            }),
        );

        for conn_id in conn_ids {
            let _ = server.tx.send(ClientMessage::Notify {
                conn_id,
                notification: notification.clone(),
            });
        }
    }
}

/// Bridge ItemTransferEvent → EventType::ItemDelivered
///
/// Note: Currently forwards ALL item transfers. In the future, this could
/// be filtered to only delivery events (transfers to platforms).
fn bridge_item_transfer_events(
    mut events: EventReader<ItemTransferEvent>,
    subscriptions: Res<EventSubscriptions>,
    server: Res<ModApiServer>,
) {
    for event in events.read() {
        let conn_ids = subscriptions.get_subscriber_connections(EventType::ItemDelivered);
        if conn_ids.is_empty() {
            continue;
        }

        let notification = JsonRpcNotification::new(
            "event.item_delivered",
            json!({
                "from": {
                    "x": event.from_pos.x,
                    "y": event.from_pos.y,
                    "z": event.from_pos.z
                },
                "to": {
                    "x": event.to_pos.x,
                    "y": event.to_pos.y,
                    "z": event.to_pos.z
                },
                "item": event.item.name().unwrap_or("unknown"),
                "count": event.count
            }),
        );

        for conn_id in conn_ids {
            let _ = server.tx.send(ClientMessage::Notify {
                conn_id,
                notification: notification.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bridge_plugin_builds() {
        // Just verify the plugin can be instantiated
        let _plugin = EventBridgePlugin;
    }

    #[test]
    fn test_json_notification_format() {
        let notification = JsonRpcNotification::new(
            "event.block_placed",
            json!({
                "position": {"x": 1, "y": 2, "z": 3},
                "block_type": "stone",
                "player_id": 0
            }),
        );

        assert_eq!(notification.method, "event.block_placed");
        assert!(notification.params.is_object());
    }
}
