//! Block placement and breaking systems
//!
//! This module contains the core block interaction systems:
//! - block_break: Breaking world blocks and machines
//! - block_place: Placing blocks and machines

mod breaking;
mod placement;

pub use breaking::block_break;
pub use placement::block_place;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::components::Machine;
use crate::core::ItemId;
use crate::events::game_events::InventoryChanged;
use crate::events::GuardedEventWriter;
use crate::player::{LocalPlayer, PlayerInventory};
use crate::{Conveyor, DeliveryPlatform};

/// Bundled local player inventory access (reduces parameter count)
#[derive(SystemParam)]
pub struct LocalPlayerInventory<'w, 's> {
    local_player: Option<Res<'w, LocalPlayer>>,
    inventories: Query<'w, 's, &'static mut PlayerInventory>,
    inventory_events: EventWriter<'w, InventoryChanged>,
}

impl LocalPlayerInventory<'_, '_> {
    /// Get mutable access to the local player's inventory
    pub fn get_mut(&mut self) -> Option<Mut<'_, PlayerInventory>> {
        let local_player = self.local_player.as_ref()?;
        self.inventories.get_mut(local_player.0).ok()
    }

    /// Get read-only access to the local player's inventory
    pub fn get(&self) -> Option<&PlayerInventory> {
        let local_player = self.local_player.as_ref()?;
        self.inventories.get(local_player.0).ok()
    }

    /// Get the local player's entity
    pub fn entity(&self) -> Option<Entity> {
        self.local_player.as_ref().map(|lp| lp.0)
    }

    /// Add item to inventory and send InventoryChanged event
    ///
    /// Returns the amount that couldn't be added (overflow).
    pub fn add_item(&mut self, item_id: ItemId, amount: u32) -> u32 {
        let entity = match self.entity() {
            Some(e) => e,
            None => return amount, // No player, can't add
        };

        let remaining = if let Some(mut inventory) = self.get_mut() {
            inventory.add_item_by_id(item_id, amount)
        } else {
            return amount;
        };

        let added = amount - remaining;
        if added > 0 {
            self.inventory_events.send(InventoryChanged {
                entity,
                item_id,
                delta: added as i32,
            });
        }

        remaining
    }

    /// Consume item from inventory and send InventoryChanged event
    ///
    /// Returns true if successful, false if not enough items.
    pub fn consume_item(&mut self, item_id: ItemId, amount: u32) -> bool {
        let entity = match self.entity() {
            Some(e) => e,
            None => return false,
        };

        let success = if let Some(mut inventory) = self.get_mut() {
            inventory.consume_item_by_id(item_id, amount)
        } else {
            return false;
        };

        if success {
            self.inventory_events.send(InventoryChanged {
                entity,
                item_id,
                delta: -(amount as i32),
            });
        }

        success
    }
}

/// Bundled machine queries for block_break system (reduces parameter count)
#[derive(SystemParam)]
pub struct MachineBreakQueries<'w, 's> {
    pub conveyor: Query<'w, 's, (Entity, &'static Conveyor, &'static GlobalTransform)>,
    pub machine: Query<'w, 's, (Entity, &'static Machine, &'static GlobalTransform)>,
    pub platform: Query<'w, 's, &'static Transform, With<DeliveryPlatform>>,
}

/// Bundled machine queries for block_place system (reduces parameter count)
#[derive(SystemParam)]
pub struct MachinePlaceQueries<'w, 's> {
    pub conveyor: Query<'w, 's, &'static Conveyor>,
    pub machine: Query<'w, 's, (&'static Machine, &'static Transform)>,
}

/// Bundled chunk render assets (reduces parameter count)
#[derive(SystemParam)]
pub struct ChunkAssets<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
}

/// Bundled block break events (reduces parameter count)
#[derive(SystemParam)]
pub struct BlockBreakEvents<'w> {
    pub tutorial: EventWriter<'w, crate::systems::TutorialEvent>,
    pub block_broken: GuardedEventWriter<'w, crate::events::game_events::BlockBroken>,
}

/// Bundled block place events (reduces parameter count)
#[derive(SystemParam)]
pub struct BlockPlaceEvents<'w> {
    pub tutorial: EventWriter<'w, crate::systems::TutorialEvent>,
    pub block_placed: GuardedEventWriter<'w, crate::events::game_events::BlockPlaced>,
    pub machine_spawned: GuardedEventWriter<'w, crate::events::game_events::MachineSpawned>,
    pub network_block_placed: EventWriter<'w, crate::logistics::network::NetworkBlockPlaced>,
    pub network_types: Res<'w, crate::logistics::network::NetworkTypeRegistry>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

    /// Test that InventoryChanged event is sent when adding items
    #[test]
    fn test_inventory_changed_event_add() {
        use bevy::prelude::*;

        // Setup minimal app
        let mut app = App::new();
        app.add_event::<InventoryChanged>();

        // Create a player entity with inventory
        let player_entity = app.world_mut().spawn(PlayerInventory::default()).id();
        app.world_mut().insert_resource(LocalPlayer(player_entity));

        // Run a system that adds items
        app.add_systems(Update, |mut player_inv: LocalPlayerInventory| {
            let remaining = player_inv.add_item(items::stone(), 10);
            assert_eq!(remaining, 0);
        });
        app.update();

        // Check event was sent
        let events = app.world().resource::<Events<InventoryChanged>>();
        let mut reader = events.get_cursor();
        let received: Vec<_> = reader.read(events).collect();

        assert_eq!(received.len(), 1);
        assert_eq!(received[0].entity, player_entity);
        assert_eq!(received[0].item_id, items::stone());
        assert_eq!(received[0].delta, 10);
    }

    /// Test that InventoryChanged event is sent when consuming items
    #[test]
    fn test_inventory_changed_event_consume() {
        use bevy::prelude::*;

        // Setup minimal app
        let mut app = App::new();
        app.add_event::<InventoryChanged>();

        // Create a player entity with inventory containing items
        let mut inv = PlayerInventory::default();
        inv.add_item_by_id(items::stone(), 20);
        let player_entity = app.world_mut().spawn(inv).id();
        app.world_mut().insert_resource(LocalPlayer(player_entity));

        // Run a system that consumes items
        app.add_systems(Update, |mut player_inv: LocalPlayerInventory| {
            let success = player_inv.consume_item(items::stone(), 5);
            assert!(success);
        });
        app.update();

        // Check event was sent
        let events = app.world().resource::<Events<InventoryChanged>>();
        let mut reader = events.get_cursor();
        let received: Vec<_> = reader.read(events).collect();

        assert_eq!(received.len(), 1);
        assert_eq!(received[0].entity, player_entity);
        assert_eq!(received[0].item_id, items::stone());
        assert_eq!(received[0].delta, -5);
    }

    /// Test that no event is sent when consume fails
    #[test]
    fn test_inventory_changed_event_no_event_on_failed_consume() {
        use bevy::prelude::*;

        // Setup minimal app
        let mut app = App::new();
        app.add_event::<InventoryChanged>();

        // Create a player entity with inventory containing NO items
        let player_entity = app.world_mut().spawn(PlayerInventory::default()).id();
        app.world_mut().insert_resource(LocalPlayer(player_entity));

        // Run a system that tries to consume items (should fail)
        app.add_systems(Update, |mut player_inv: LocalPlayerInventory| {
            let success = player_inv.consume_item(items::stone(), 5);
            assert!(!success);
        });
        app.update();

        // Check NO event was sent
        let events = app.world().resource::<Events<InventoryChanged>>();
        let mut reader = events.get_cursor();
        let received: Vec<_> = reader.read(events).collect();

        assert_eq!(received.len(), 0);
    }

    /// Test that no event is sent when add amount is 0
    #[test]
    fn test_inventory_changed_event_no_event_on_zero_add() {
        use bevy::prelude::*;

        // Setup minimal app
        let mut app = App::new();
        app.add_event::<InventoryChanged>();

        // Create a player entity with inventory
        let player_entity = app.world_mut().spawn(PlayerInventory::default()).id();
        app.world_mut().insert_resource(LocalPlayer(player_entity));

        // Run a system that adds 0 items
        app.add_systems(Update, |mut player_inv: LocalPlayerInventory| {
            let remaining = player_inv.add_item(items::stone(), 0);
            assert_eq!(remaining, 0);
        });
        app.update();

        // Check NO event was sent
        let events = app.world().resource::<Events<InventoryChanged>>();
        let mut reader = events.get_cursor();
        let received: Vec<_> = reader.read(events).collect();

        assert_eq!(received.len(), 0);
    }
}
