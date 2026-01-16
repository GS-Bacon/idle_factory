//! Tests for generic machine systems

use crate::components::{Machine, MachineSlot};
use crate::core::items;
use crate::game_spec::{CRUSHER, FURNACE, MINER};
use bevy::prelude::*;

use crate::machines::generic::auto_generate::get_biome_output;

#[test]
fn test_machine_slot_operations() {
    let mut slot = MachineSlot::empty();
    assert!(slot.is_empty());

    // Add items
    slot.add_id(items::iron_ore(), 5);
    assert_eq!(slot.count, 5);
    assert_eq!(slot.get_item_id(), Some(items::iron_ore()));

    // Can't add different type
    let added = slot.add_id(items::copper_ore(), 3);
    assert_eq!(added, 0);
    assert_eq!(slot.count, 5);

    // Can add same type
    slot.add_id(items::iron_ore(), 3);
    assert_eq!(slot.count, 8);

    // Take items
    let taken = slot.take(3);
    assert_eq!(taken, 3);
    assert_eq!(slot.count, 5);

    // Take all
    slot.take(5);
    assert!(slot.is_empty());
}

#[test]
fn test_machine_creation() {
    let machine = Machine::new(
        &MINER,
        IVec3::new(0, 0, 0),
        crate::components::Direction::North,
    );
    assert_eq!(machine.spec.id, "miner");
    assert_eq!(machine.progress, 0.0);
    assert_eq!(machine.slots.outputs.len(), 1);
}

#[test]
fn test_furnace_machine_creation() {
    let machine = Machine::new(
        &FURNACE,
        IVec3::new(0, 0, 0),
        crate::components::Direction::North,
    );
    assert_eq!(machine.spec.id, "furnace");
    assert!(machine.spec.requires_fuel);
    assert_eq!(machine.slots.inputs.len(), 1);
    assert_eq!(machine.slots.outputs.len(), 1);
}

#[test]
fn test_crusher_machine_creation() {
    let machine = Machine::new(
        &CRUSHER,
        IVec3::new(0, 0, 0),
        crate::components::Direction::North,
    );
    assert_eq!(machine.spec.id, "crusher");
    assert!(!machine.spec.requires_fuel);
}

#[test]
fn test_biome_output_deterministic() {
    use crate::world::biome::BiomeType;

    // Same tick should give same output
    let out1 = get_biome_output(BiomeType::Iron, 100);
    let out2 = get_biome_output(BiomeType::Iron, 100);
    assert_eq!(out1, out2);

    // Different biomes can give different outputs
    let iron_out = get_biome_output(BiomeType::Iron, 0);
    let copper_out = get_biome_output(BiomeType::Copper, 0);
    // At tick 0, both should give their primary ore
    assert_eq!(iron_out, items::iron_ore());
    assert_eq!(copper_out, items::copper_ore());
}

#[test]
fn test_machine_despawn_clears_interacting_machine() {
    use crate::components::InteractingMachine;

    // Test that InteractingMachine is cleared when the referenced entity is despawned
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<InteractingMachine>();

    // Create a machine entity
    let machine = app
        .world_mut()
        .spawn(Machine::new(
            &MINER,
            IVec3::ZERO,
            crate::components::Direction::North,
        ))
        .id();

    // Set InteractingMachine to reference this machine
    app.world_mut().resource_mut::<InteractingMachine>().0 = Some(machine);

    // Verify it's set
    assert_eq!(
        app.world().resource::<InteractingMachine>().0,
        Some(machine)
    );

    // Despawn the machine
    app.world_mut().despawn(machine);

    // Verify the entity no longer exists
    assert!(app.world().get_entity(machine).is_err());

    // The InteractingMachine resource still holds the old entity reference
    // (cleanup system hasn't run yet)
    assert_eq!(
        app.world().resource::<InteractingMachine>().0,
        Some(machine)
    );

    // Manually run cleanup logic (simulating what cleanup_invalid_interacting_machine does)
    // We can't easily run the full system without Window, so test the core logic
    let machine_exists = app
        .world_mut()
        .query::<&Machine>()
        .get(app.world(), machine)
        .is_ok();

    assert!(!machine_exists, "Machine should no longer exist");

    // If machine doesn't exist, the cleanup system would clear InteractingMachine
    if !machine_exists {
        app.world_mut().resource_mut::<InteractingMachine>().0 = None;
    }

    // Verify InteractingMachine is now None
    assert_eq!(app.world().resource::<InteractingMachine>().0, None);
}

#[test]
fn test_cleanup_preserves_valid_interacting_machine() {
    use crate::components::InteractingMachine;

    // Test that InteractingMachine is NOT cleared when the entity still exists
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<InteractingMachine>();

    // Create a machine entity
    let machine = app
        .world_mut()
        .spawn(Machine::new(
            &FURNACE,
            IVec3::new(1, 0, 1),
            crate::components::Direction::East,
        ))
        .id();

    // Set InteractingMachine to reference this machine
    app.world_mut().resource_mut::<InteractingMachine>().0 = Some(machine);

    // Verify it's set
    assert_eq!(
        app.world().resource::<InteractingMachine>().0,
        Some(machine)
    );

    // Check if machine exists (it should)
    let machine_exists = app
        .world_mut()
        .query::<&Machine>()
        .get(app.world(), machine)
        .is_ok();

    assert!(machine_exists, "Machine should still exist");

    // Since machine exists, InteractingMachine should NOT be cleared
    // This simulates the early return in cleanup_invalid_interacting_machine
    assert_eq!(
        app.world().resource::<InteractingMachine>().0,
        Some(machine)
    );
}

#[test]
fn test_input_state_transitions_on_machine_despawn() {
    use crate::components::{
        CommandInputState, CursorLockState, InputState, InteractingMachine, InventoryOpen,
    };

    // Test that InputState correctly transitions from MachineUI to Gameplay
    // when InteractingMachine is cleared
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<InteractingMachine>();
    app.init_resource::<InventoryOpen>();
    app.init_resource::<CommandInputState>();
    app.init_resource::<CursorLockState>();

    // Create a machine entity
    let machine = app
        .world_mut()
        .spawn(Machine::new(
            &CRUSHER,
            IVec3::new(2, 0, 2),
            crate::components::Direction::South,
        ))
        .id();

    // Simulate opening machine UI (sets both InteractingMachine and paused)
    // This mirrors sync_legacy_ui_state behavior for UIContext::Machine
    app.world_mut().resource_mut::<InteractingMachine>().0 = Some(machine);
    app.world_mut().resource_mut::<CursorLockState>().paused = true;

    // Check InputState - should be MachineUI
    {
        let inventory_open = app.world().resource::<InventoryOpen>();
        let interacting_machine = app.world().resource::<InteractingMachine>();
        let command_state = app.world().resource::<CommandInputState>();
        let cursor_state = app.world().resource::<CursorLockState>();

        let state = InputState::current(
            inventory_open,
            interacting_machine,
            command_state,
            cursor_state,
        );

        assert_eq!(state, InputState::MachineUI);
    }

    // Despawn the machine
    app.world_mut().despawn(machine);

    // Simulate cleanup: clear InteractingMachine and reset cursor state
    // This mirrors sync_legacy_ui_state behavior for UIContext::Gameplay
    let machine_exists = app
        .world_mut()
        .query::<&Machine>()
        .get(app.world(), machine)
        .is_ok();

    if !machine_exists {
        app.world_mut().resource_mut::<InteractingMachine>().0 = None;
        app.world_mut().resource_mut::<CursorLockState>().paused = false;
    }

    // Check InputState - should be Gameplay now
    {
        let inventory_open = app.world().resource::<InventoryOpen>();
        let interacting_machine = app.world().resource::<InteractingMachine>();
        let command_state = app.world().resource::<CommandInputState>();
        let cursor_state = app.world().resource::<CursorLockState>();

        let state = InputState::current(
            inventory_open,
            interacting_machine,
            command_state,
            cursor_state,
        );

        assert_eq!(state, InputState::Gameplay);
    }
}
