//! Event notification system for WebSocket subscribers
//!
//! Listens for game events and sends notifications to subscribed connections.

use bevy::prelude::*;

use crate::events::{BlockBroken, BlockPlaced, ItemDelivered, MachineCompleted};

use super::handlers::events::{EventSubscriptions, EventType};
use super::protocol::JsonRpcNotification;
use super::server::{ClientMessage, ModApiServer};

/// Plugin for event notification system
pub struct EventNotifierPlugin;

impl Plugin for EventNotifierPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                notify_item_delivered,
                notify_machine_completed,
                notify_block_placed,
                notify_block_broken,
            )
                .run_if(resource_exists::<ModApiServer>),
        );
    }
}

/// Notify subscribers when items are delivered
fn notify_item_delivered(
    server: Res<ModApiServer>,
    subscriptions: Res<EventSubscriptions>,
    mut events: EventReader<ItemDelivered>,
) {
    for event in events.read() {
        let connections = subscriptions.get_subscriber_connections(EventType::ItemDelivered);
        if connections.is_empty() {
            continue;
        }

        let item_name = event.item.name().unwrap_or("unknown");
        let notification = JsonRpcNotification::new(
            "event.item_delivered",
            serde_json::json!({
                "event_type": "item.delivered",
                "item_id": item_name,
                "count": event.count,
            }),
        );

        for conn_id in connections {
            let _ = server.tx.send(ClientMessage::Notify {
                conn_id,
                notification: notification.clone(),
            });
        }
    }
}

/// Notify subscribers when machines complete processing
fn notify_machine_completed(
    server: Res<ModApiServer>,
    subscriptions: Res<EventSubscriptions>,
    mut events: EventReader<MachineCompleted>,
) {
    for event in events.read() {
        let connections = subscriptions.get_subscriber_connections(EventType::MachineCompleted);
        if connections.is_empty() {
            continue;
        }

        let outputs: Vec<_> = event
            .outputs
            .iter()
            .map(|(id, count)| {
                serde_json::json!({
                    "item_id": id.name().unwrap_or("unknown"),
                    "count": count,
                })
            })
            .collect();

        let notification = JsonRpcNotification::new(
            "event.machine_completed",
            serde_json::json!({
                "event_type": "machine.completed",
                "entity": event.entity.index(),
                "outputs": outputs,
            }),
        );

        for conn_id in connections {
            let _ = server.tx.send(ClientMessage::Notify {
                conn_id,
                notification: notification.clone(),
            });
        }
    }
}

/// Notify subscribers when blocks are placed
fn notify_block_placed(
    server: Res<ModApiServer>,
    subscriptions: Res<EventSubscriptions>,
    mut events: EventReader<BlockPlaced>,
) {
    for event in events.read() {
        let connections = subscriptions.get_subscriber_connections(EventType::BlockPlaced);
        if connections.is_empty() {
            continue;
        }

        let notification = JsonRpcNotification::new(
            "event.block_placed",
            serde_json::json!({
                "event_type": "block.placed",
                "position": [event.pos.x, event.pos.y, event.pos.z],
                "block_id": event.block.name().unwrap_or("unknown"),
            }),
        );

        for conn_id in connections {
            let _ = server.tx.send(ClientMessage::Notify {
                conn_id,
                notification: notification.clone(),
            });
        }
    }
}

/// Notify subscribers when blocks are removed
fn notify_block_broken(
    server: Res<ModApiServer>,
    subscriptions: Res<EventSubscriptions>,
    mut events: EventReader<BlockBroken>,
) {
    for event in events.read() {
        let connections = subscriptions.get_subscriber_connections(EventType::BlockRemoved);
        if connections.is_empty() {
            continue;
        }

        let notification = JsonRpcNotification::new(
            "event.block_removed",
            serde_json::json!({
                "event_type": "block.removed",
                "position": [event.pos.x, event.pos.y, event.pos.z],
                "block_id": event.block.name().unwrap_or("unknown"),
            }),
        );

        for conn_id in connections {
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
    fn test_event_notifier_plugin_creation() {
        // Just verify the plugin can be created
        let _plugin = EventNotifierPlugin;
    }
}
