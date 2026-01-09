//! Event system tests

use bevy::prelude::*;
use idle_factory::core::items;
use idle_factory::events::{BlockBreakEvent, GameEventsPlugin, QuestProgressEvent};
use idle_factory::player::PlayerInventory;
use idle_factory::ItemId;
use std::collections::HashMap;

// ============================================================================
// Helper Resources
// ============================================================================

#[derive(Resource, Default)]
struct EventCounter {
    block_breaks: u32,
}

fn count_block_break_events(
    mut events: EventReader<BlockBreakEvent>,
    mut counter: ResMut<EventCounter>,
) {
    for _event in events.read() {
        counter.block_breaks += 1;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_custom_system_event_handling() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GameEventsPlugin);
    app.init_resource::<EventCounter>();
    app.add_systems(Update, count_block_break_events);

    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(0, 0, 0),
        player_id: 1,
    });
    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(1, 1, 1),
        player_id: 1,
    });
    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(2, 2, 2),
        player_id: 1,
    });

    app.update();

    let counter = app.world().resource::<EventCounter>();
    assert_eq!(counter.block_breaks, 3);
}

#[test]
fn test_inventory_slot_selection() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut inv = PlayerInventory::default();
    inv.add_item_by_id(items::grass(), 10);
    inv.add_item_by_id(items::coal(), 20);
    inv.add_item_by_id(items::stone(), 30);

    let player_entity = app.world_mut().spawn(inv).id();
    app.update();

    {
        let inventory = app.world().get::<PlayerInventory>(player_entity).unwrap();
        assert_eq!(inventory.selected_slot, 0);
        assert_eq!(inventory.selected_item_id(), Some(items::grass()));
    }

    {
        let mut inventory = app
            .world_mut()
            .get_mut::<PlayerInventory>(player_entity)
            .unwrap();
        inventory.selected_slot = 2;
    }

    app.update();

    {
        let inventory = app.world().get::<PlayerInventory>(player_entity).unwrap();
        assert_eq!(inventory.selected_slot, 2);
        assert_eq!(inventory.selected_item_id(), Some(items::stone()));
    }
}

#[test]
fn test_quest_progress_event_chain() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GameEventsPlugin);

    #[derive(Resource, Default)]
    struct QuestTracker {
        items_collected: HashMap<ItemId, u32>,
    }

    fn track_quest_progress(
        mut events: EventReader<QuestProgressEvent>,
        mut tracker: ResMut<QuestTracker>,
    ) {
        for event in events.read() {
            *tracker.items_collected.entry(event.item_id).or_default() += event.amount;
        }
    }

    app.init_resource::<QuestTracker>();
    app.add_systems(Update, track_quest_progress);

    app.world_mut().send_event(QuestProgressEvent {
        item_id: items::iron_ore(),
        amount: 5,
    });
    app.world_mut().send_event(QuestProgressEvent {
        item_id: items::coal(),
        amount: 10,
    });
    app.world_mut().send_event(QuestProgressEvent {
        item_id: items::iron_ore(),
        amount: 3,
    });

    app.update();

    let tracker = app.world().resource::<QuestTracker>();
    assert_eq!(tracker.items_collected.get(&items::iron_ore()), Some(&8));
    assert_eq!(tracker.items_collected.get(&items::coal()), Some(&10));
}

#[test]
fn test_events_consumed_after_read() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GameEventsPlugin);
    app.init_resource::<EventCounter>();
    app.add_systems(Update, count_block_break_events);

    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(0, 0, 0),
        player_id: 1,
    });

    app.update();
    assert_eq!(app.world().resource::<EventCounter>().block_breaks, 1);

    app.update();
    assert_eq!(app.world().resource::<EventCounter>().block_breaks, 1);

    app.world_mut().send_event(BlockBreakEvent {
        position: IVec3::new(1, 1, 1),
        player_id: 1,
    });

    app.update();
    assert_eq!(app.world().resource::<EventCounter>().block_breaks, 2);
}

#[test]
fn test_inventory_item_stacking_via_bevy() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let mut inv = PlayerInventory::default();
    inv.add_item_by_id(items::coal(), 100);

    let player_entity = app.world_mut().spawn(inv).id();

    for _ in 0..2 {
        let mut inventory = app
            .world_mut()
            .get_mut::<PlayerInventory>(player_entity)
            .unwrap();
        inventory.add_item_by_id(items::coal(), 100);
    }

    app.update();

    let inventory = app.world().get::<PlayerInventory>(player_entity).unwrap();
    let total_coal = inventory.get_total_count_by_id(items::coal());

    assert_eq!(total_coal, 300);
}
