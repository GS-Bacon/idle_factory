//! Inventory tooltip system

use crate::components::*;
use crate::player::{LocalPlayer, PlayerInventory};
use bevy::prelude::*;

/// Update inventory tooltip to show item name when hovering over slots
#[allow(clippy::too_many_arguments)]
pub fn update_inventory_tooltip(
    inventory_open: Res<InventoryOpen>,
    local_player: Option<Res<LocalPlayer>>,
    inventory_query: Query<&PlayerInventory>,
    windows: Query<&Window>,
    slot_query: Query<(&Interaction, &InventorySlotUI)>,
    creative_query: Query<(&Interaction, &CreativeItemButton)>,
    mut tooltip_query: Query<(&mut Node, &mut Visibility, &Children), With<InventoryTooltip>>,
    mut text_query: Query<&mut Text>,
) {
    let Ok((mut node, mut visibility, children)) = tooltip_query.get_single_mut() else {
        return;
    };

    // Hide tooltip if inventory is closed
    if !inventory_open.0 {
        *visibility = Visibility::Hidden;
        return;
    }

    // Get inventory if available
    let inventory = local_player
        .as_ref()
        .and_then(|lp| inventory_query.get(lp.0).ok());

    // Find hovered slot (inventory slots)
    let mut hovered_item: Option<(crate::core::ItemId, Option<u32>)> = None;
    if let Some(inventory) = inventory {
        for (interaction, slot_ui) in slot_query.iter() {
            if *interaction == Interaction::Hovered {
                let slot_idx = slot_ui.0;
                if let Some((item_id, count)) = inventory.slots[slot_idx] {
                    hovered_item = Some((item_id, Some(count)));
                    break;
                }
            }
        }
    }

    // Check creative catalog items if no inventory slot is hovered
    if hovered_item.is_none() {
        for (interaction, creative_btn) in creative_query.iter() {
            if *interaction == Interaction::Hovered {
                hovered_item = Some((creative_btn.0, None));
                break;
            }
        }
    }

    if let Some((item_id, count_opt)) = hovered_item {
        *visibility = Visibility::Inherited;

        // Position tooltip near the mouse cursor
        if let Ok(window) = windows.get_single() {
            if let Some(cursor_pos) = window.cursor_position() {
                // Offset tooltip to bottom-right of cursor
                node.left = Val::Px(cursor_pos.x + 15.0);
                node.top = Val::Px(cursor_pos.y + 15.0);
            }
        }

        // Update tooltip text
        if let Some(&child) = children.first() {
            if let Ok(mut text) = text_query.get_mut(child) {
                let name = item_id.name().unwrap_or("unknown");
                if let Some(count) = count_opt {
                    text.0 = format!("{} ({})", name, count);
                } else {
                    // Creative catalog item - just show name
                    text.0 = name.to_string();
                }
            }
        }
    } else {
        *visibility = Visibility::Hidden;
    }
}
