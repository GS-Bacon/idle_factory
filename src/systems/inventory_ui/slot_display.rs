//! Inventory slot display systems

use crate::components::*;
use crate::core::ItemId;
use crate::player::{LocalPlayer, PlayerInventory};
use crate::setup::ui::{SLOT_BG, SLOT_HOVER_BG};
use bevy::prelude::*;

/// Update inventory slot visuals to reflect current inventory state
pub fn inventory_update_slots(
    inventory_open: Res<InventoryOpen>,
    local_player: Option<Res<LocalPlayer>>,
    inventory_query: Query<&PlayerInventory>,
    item_sprites: Res<ItemSprites>,
    mut slot_query: Query<(
        &InventorySlotUI,
        &mut BackgroundColor,
        &Children,
        &Interaction,
    )>,
    mut text_query: Query<&mut Text>,
    mut image_query: Query<(&InventorySlotImage, &mut ImageNode, &mut Visibility)>,
) {
    if !inventory_open.0 {
        // Hide all slot images when inventory is closed
        for (_slot_image, _image_node, mut visibility) in image_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let Some(local_player) = local_player else {
        return;
    };
    let Ok(inventory) = inventory_query.get(local_player.0) else {
        return;
    };

    // Update slot sprite images
    for (slot_image, mut image_node, mut visibility) in image_query.iter_mut() {
        let slot_idx = slot_image.0;
        if let Some((block_type, _count)) = inventory.slots[slot_idx] {
            if let Some(sprite_handle) = item_sprites.get_id(ItemId::from(block_type)) {
                image_node.image = sprite_handle;
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    for (slot_ui, mut bg_color, children, interaction) in slot_query.iter_mut() {
        let slot_idx = slot_ui.0;

        if let Some((_block_type, count)) = inventory.slots[slot_idx] {
            // Use consistent dark background regardless of sprite availability
            *bg_color = BackgroundColor(SLOT_BG);

            // Update text (count)
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = if count > 1 {
                        format!("{}", count)
                    } else {
                        String::new()
                    };
                }
            }
        } else {
            // Empty slot - respect hover state using theme colors
            *bg_color = BackgroundColor(match interaction {
                Interaction::Hovered => SLOT_HOVER_BG,
                Interaction::Pressed => SLOT_BG,
                Interaction::None => SLOT_BG,
            });

            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = String::new();
                }
            }
        }
    }
}

/// Update held item display to follow cursor and show held item
#[allow(clippy::type_complexity)]
pub fn update_held_item_display(
    inventory_open: Res<InventoryOpen>,
    held_item: Res<HeldItem>,
    item_sprites: Res<ItemSprites>,
    windows: Query<&Window>,
    mut held_display_query: Query<(&mut Node, &mut Visibility), With<HeldItemDisplay>>,
    mut held_image_query: Query<&mut ImageNode, With<HeldItemImage>>,
    mut held_text_query: Query<
        (&mut Text, &mut Node, &mut Visibility),
        (With<HeldItemText>, Without<HeldItemDisplay>),
    >,
) {
    let Ok((mut node, mut visibility)) = held_display_query.get_single_mut() else {
        return;
    };

    // Only show when inventory is open and we're holding something
    if !inventory_open.0 {
        *visibility = Visibility::Hidden;
        if let Ok((_, _, mut text_vis)) = held_text_query.get_single_mut() {
            *text_vis = Visibility::Hidden;
        }
        return;
    }

    match &held_item.0 {
        Some((block_type, count)) => {
            // Show the held item
            *visibility = Visibility::Visible;

            // Update sprite image
            if let Ok(mut image) = held_image_query.get_single_mut() {
                if let Some(sprite) = item_sprites.get_id(ItemId::from(*block_type)) {
                    image.image = sprite;
                }
            }

            // Position at cursor
            if let Ok(window) = windows.get_single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    // Offset so item appears slightly below and to the right of cursor
                    let x = cursor_pos.x + 8.0;
                    let y = cursor_pos.y + 8.0;
                    node.left = Val::Px(x);
                    node.top = Val::Px(y);

                    // Update count text position and visibility
                    if let Ok((mut text, mut text_node, mut text_vis)) =
                        held_text_query.get_single_mut()
                    {
                        text.0 = if *count > 1 {
                            format!("{}", count)
                        } else {
                            String::new()
                        };
                        text_node.left = Val::Px(x + 30.0);
                        text_node.top = Val::Px(y + 30.0);
                        *text_vis = if *count > 1 {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        };
                    }
                }
            }
        }
        None => {
            *visibility = Visibility::Hidden;
            if let Ok((_, _, mut text_vis)) = held_text_query.get_single_mut() {
                *text_vis = Visibility::Hidden;
            }
        }
    }
}
