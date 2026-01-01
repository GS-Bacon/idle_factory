//! UI systems plugin
//!
//! Consolidates all UI-related systems and resources.

use bevy::prelude::*;

use crate::systems::{
    command_input_handler, command_input_toggle, creative_inventory_click,
    inventory_continuous_shift_click, inventory_slot_click, inventory_toggle,
    inventory_update_slots, toggle_debug_hud, trash_slot_click, update_creative_catalog_sprites,
    update_debug_hud, update_held_item_display, update_hotbar_item_name, update_hotbar_ui,
    update_inventory_tooltip,
};
use crate::{
    CommandInputState, DebugHudState, GuideMarkers, HeldItem, InventoryOpen, ItemSprites,
    TargetBlock, TutorialShown,
};

/// Plugin for all UI-related systems
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        // UI resources
        app.init_resource::<TargetBlock>()
            .init_resource::<InventoryOpen>()
            .init_resource::<TutorialShown>()
            .init_resource::<HeldItem>()
            .init_resource::<CommandInputState>()
            .init_resource::<GuideMarkers>()
            .init_resource::<DebugHudState>()
            .init_resource::<ItemSprites>();

        // UI update systems
        app.add_systems(
            Update,
            (
                update_hotbar_ui,
                toggle_debug_hud,
                update_debug_hud,
            ),
        )
        .add_systems(
            Update,
            (
                // Inventory systems
                inventory_toggle,
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
            ),
        );
    }
}
