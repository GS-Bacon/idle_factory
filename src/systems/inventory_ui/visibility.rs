//! Inventory visibility systems

use crate::components::*;
use crate::player::{LocalPlayer, PlayerInventory};
use crate::setup::ui::{UpperPanel, UpperPanelTabs};
use bevy::prelude::*;
use tracing::{info, warn};

/// Set the UI open state (no-op, kept for API compatibility)
pub fn set_ui_open_state(_ui_open: bool) {
    // No-op
}

/// Return held item to inventory when closing
pub(super) fn return_held_item_to_inventory(
    inventory: &mut PlayerInventory,
    held_item: &mut HeldItem,
) {
    use crate::core::ItemId;

    if let Some((block_type, count)) = held_item.0.take() {
        // Try to add to inventory
        let remaining = inventory.add_item_by_id(ItemId::from(block_type), count);
        if remaining > 0 {
            // If inventory is full, item is lost (or could be dropped later)
            // For now, just put back what couldn't fit
            held_item.0 = Some((block_type, remaining));
        }
    }
}

/// Update inventory UI visibility when InventoryOpen changes
/// (Key handling moved to ui_navigation.rs)
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn update_inventory_visibility(
    inventory_open: Res<InventoryOpen>,
    local_player: Option<Res<LocalPlayer>>,
    local_platform: Option<Res<crate::player::LocalPlatform>>,
    mut inventory_query: Query<&mut PlayerInventory>,
    mut held_item: ResMut<HeldItem>,
    creative_mode: Res<CreativeMode>,
    mut ui_query: Query<&mut Visibility, With<InventoryUI>>,
    mut overlay_query: Query<
        &mut Visibility,
        (
            With<InventoryBackgroundOverlay>,
            Without<InventoryUI>,
            Without<CreativePanel>,
            Without<UpperPanel>,
        ),
    >,
    mut creative_panel_query: Query<
        (&mut Visibility, &mut Node),
        (
            With<CreativePanel>,
            Without<InventoryUI>,
            Without<InventoryBackgroundOverlay>,
            Without<UpperPanel>,
        ),
    >,
    mut upper_panel_query: Query<
        &mut Visibility,
        (
            With<UpperPanel>,
            Without<InventoryUI>,
            Without<InventoryBackgroundOverlay>,
            Without<CreativePanel>,
            Without<UpperPanelTabs>,
        ),
    >,
    mut upper_tabs_query: Query<
        &mut Visibility,
        (
            With<UpperPanelTabs>,
            Without<UpperPanel>,
            Without<InventoryUI>,
        ),
    >,
    windows: Query<&Window>,
) {
    // Only update when InventoryOpen changes
    if !inventory_open.is_changed() {
        return;
    }

    info!("[INVENTORY] InventoryOpen changed to {}", inventory_open.0);

    // Return held item when closing
    if !inventory_open.0 {
        if let Some(ref local_player) = local_player {
            if let Ok(mut inventory) = inventory_query.get_mut(local_player.0) {
                return_held_item_to_inventory(&mut inventory, &mut held_item);
            }
        }
    }

    // Update UI visibility
    let mut ui_count = 0;
    for mut vis in ui_query.iter_mut() {
        ui_count += 1;
        *vis = if inventory_open.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Show/hide background overlay
    for mut vis in overlay_query.iter_mut() {
        *vis = if inventory_open.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    info!(
        "[INVENTORY] Updated {} UI entities, now open={}",
        ui_count, inventory_open.0
    );

    if ui_count == 0 {
        warn!("[INVENTORY] No InventoryUI entity found! UI will not display.");
    }

    // Determine if upper panel should be visible
    // Condition: creative_mode.enabled || local_platform.is_some()
    let has_platform = local_platform.is_some();
    let show_upper_panel = inventory_open.0 && (creative_mode.enabled || has_platform);

    // Show/hide upper panel
    for mut vis in upper_panel_query.iter_mut() {
        *vis = if show_upper_panel {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Show/hide upper panel tabs (only when BOTH creative and platform are available)
    // For now, always show tabs when upper panel is visible
    for mut vis in upper_tabs_query.iter_mut() {
        *vis = if show_upper_panel {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Show/hide creative panel (old one, might be removed)
    for (mut vis, mut node) in creative_panel_query.iter_mut() {
        // Hide old creative panel - now using upper panel
        *vis = Visibility::Hidden;
        node.display = Display::None;
    }

    // Note: Cursor control removed - UIState is the single source of truth
    // Cursor is now controlled by update_pause_ui in player.rs
    let _ = windows; // Suppress unused warning
}
