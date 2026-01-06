//! UI systems plugin
//!
//! Consolidates all UI-related systems and resources.

use bevy::prelude::*;

use crate::systems::{
    command_input_handler, command_input_toggle, creative_inventory_click,
    inventory_continuous_shift_click, inventory_slot_click, inventory_update_slots,
    process_tutorial_events, spawn_breaking_progress_ui, track_inventory_open, track_movement,
    track_production, trash_slot_click, update_breaking_progress_ui, update_command_suggestions,
    update_creative_catalog_sprites, update_held_item_3d, update_held_item_display,
    update_hotbar_item_name, update_hotbar_ui, update_inventory_tooltip,
    update_inventory_visibility, update_tutorial_ui, TutorialEvent,
};
use crate::{
    CommandInputState, GuideMarkers, HeldItem, InventoryOpen, ItemSprites, TargetBlock,
    TutorialProgress, TutorialShown,
};

/// Plugin for all UI-related systems
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        // UI resources (DebugHudState is in DebugPlugin to avoid duplication)
        app.init_resource::<TargetBlock>()
            .init_resource::<InventoryOpen>()
            .init_resource::<TutorialShown>()
            .init_resource::<TutorialProgress>()
            .init_resource::<HeldItem>()
            .init_resource::<CommandInputState>()
            .init_resource::<GuideMarkers>()
            .init_resource::<ItemSprites>();

        // Tutorial event
        app.add_event::<TutorialEvent>();

        // Spawn breaking progress UI
        app.add_systems(Startup, spawn_breaking_progress_ui);

        // UI update systems (debug HUD systems are in DebugPlugin)
        app.add_systems(Update, (update_hotbar_ui, update_held_item_3d))
            .add_systems(Update, update_breaking_progress_ui)
            .add_systems(
                Update,
                (
                    // Inventory systems
                    update_inventory_visibility,
                    inventory_slot_click,
                    inventory_continuous_shift_click,
                    inventory_update_slots,
                    update_held_item_display,
                    update_hotbar_item_name,
                    update_inventory_tooltip,
                    update_creative_catalog_sprites,
                    trash_slot_click,
                    creative_inventory_click,
                ),
            )
            .add_systems(
                Update,
                (
                    // Command input systems
                    command_input_toggle,
                    command_input_handler,
                    update_command_suggestions,
                ),
            )
            .add_systems(
                Update,
                (
                    // Tutorial systems
                    track_movement,
                    track_inventory_open,
                    track_production,
                    process_tutorial_events,
                    update_tutorial_ui,
                ),
            );
    }
}
