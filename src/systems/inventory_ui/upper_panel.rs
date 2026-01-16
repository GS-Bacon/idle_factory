//! Upper panel (creative/platform inventory) systems

use crate::components::*;
use crate::core::{items, ItemId};
use crate::player::{LocalPlatform, PlatformInventory};
use crate::setup::ui::{
    UpperPanelPageText, UpperPanelSlot, UpperPanelSlotCount, UpperPanelSlotImage, SLOT_BG,
    SLOT_BORDER_COLOR, SLOT_HOVER_BG, SLOT_HOVER_BORDER, SLOT_SELECTED_BORDER, UPPER_PANEL_SLOTS,
};
use bevy::prelude::*;

/// Get all available items for the upper panel based on creative mode and category
pub(super) fn get_filtered_items(creative_mode: bool, category: &ItemCategory) -> Vec<ItemId> {
    if creative_mode {
        // In creative mode, show all items
        let all_items = [
            items::grass(),
            items::stone(),
            items::iron_ore(),
            items::copper_ore(),
            items::coal(),
            items::iron_ingot(),
            items::copper_ingot(),
            items::miner_block(),
            items::conveyor_block(),
            items::furnace_block(),
            items::crusher_block(),
            items::assembler_block(),
        ];

        all_items
            .into_iter()
            .filter(|item| category.matches(*item))
            .collect()
    } else {
        // Without creative mode, nothing is shown from creative catalog
        vec![]
    }
}

/// Update upper panel slots with items (creative catalog or platform inventory)
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn update_upper_panel_slots(
    inventory_open: Res<InventoryOpen>,
    creative_mode: Res<CreativeMode>,
    category: Res<GlobalInventoryCategory>,
    page: Res<GlobalInventoryPage>,
    local_platform: Option<Res<LocalPlatform>>,
    platform_query: Query<&PlatformInventory>,
    item_sprites: Res<ItemSprites>,
    mut slot_query: Query<(&UpperPanelSlot, &mut BackgroundColor, &mut BorderColor)>,
    mut image_query: Query<(&UpperPanelSlotImage, &mut ImageNode, &mut Visibility)>,
    mut count_query: Query<(&UpperPanelSlotCount, &mut Text)>,
    mut page_text_query: Query<&mut Text, (With<UpperPanelPageText>, Without<UpperPanelSlotCount>)>,
) {
    // Only update when inventory is open
    if !inventory_open.0 {
        // Hide all slot images when closed
        for (_slot, _image, mut vis) in image_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    // Determine if upper panel should be visible
    let has_platform = local_platform.is_some();
    let show_upper_panel = creative_mode.enabled || has_platform;

    if !show_upper_panel {
        // Hide all slot images when upper panel is not visible
        for (_slot, _image, mut vis) in image_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    // Build the items to display
    // Priority: Platform inventory if available, otherwise creative catalog
    let items_to_display: Vec<(ItemId, u32)> = if let Some(ref platform) = local_platform {
        // Get platform inventory items
        if let Ok(platform_inv) = platform_query.get(platform.0) {
            let mut items: Vec<(ItemId, u32)> = platform_inv
                .items_by_id()
                .iter()
                .filter(|(item_id, _)| category.0.matches(**item_id))
                .map(|(&id, &count)| (id, count))
                .collect();
            items.sort_by_key(|(id, _)| id.raw()); // Sort by ID for consistent ordering
            items
        } else {
            vec![]
        }
    } else if creative_mode.enabled {
        // Creative mode - show all items with count of 64
        get_filtered_items(true, &category.0)
            .into_iter()
            .map(|id| (id, 64))
            .collect()
    } else {
        vec![]
    };

    // Calculate pagination
    let total_items = items_to_display.len();
    let total_pages = total_items.div_ceil(UPPER_PANEL_SLOTS);
    let current_page = page.0.min(total_pages.saturating_sub(1));
    let start_idx = current_page * UPPER_PANEL_SLOTS;

    // Update page text
    for mut text in page_text_query.iter_mut() {
        text.0 = format!("{}/{}", current_page + 1, total_pages.max(1));
    }

    // Update slot visuals
    for (slot, mut bg_color, _border_color) in slot_query.iter_mut() {
        let slot_idx = slot.0;
        let item_idx = start_idx + slot_idx;

        if item_idx < total_items {
            let (_item_id, _count) = items_to_display[item_idx];
            *bg_color = BackgroundColor(SLOT_BG);
        } else {
            *bg_color = BackgroundColor(SLOT_BG);
        }
    }

    // Update slot images
    for (slot_image, mut image_node, mut visibility) in image_query.iter_mut() {
        let slot_idx = slot_image.0;
        let item_idx = start_idx + slot_idx;

        if item_idx < total_items {
            let (item_id, _count) = items_to_display[item_idx];
            if let Some(sprite) = item_sprites.get_id(item_id) {
                image_node.image = sprite;
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    // Update slot counts
    for (slot_count, mut text) in count_query.iter_mut() {
        let slot_idx = slot_count.0;
        let item_idx = start_idx + slot_idx;

        if item_idx < total_items {
            let (_item_id, count) = items_to_display[item_idx];
            text.0 = if count > 1 {
                format!("{}", count)
            } else {
                String::new()
            };
        } else {
            text.0.clear();
        }
    }
}

/// Handle upper panel slot clicks - pick up items from creative/platform
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn upper_panel_slot_click(
    inventory_open: Res<InventoryOpen>,
    creative_mode: Res<CreativeMode>,
    category: Res<GlobalInventoryCategory>,
    page: Res<GlobalInventoryPage>,
    local_platform: Option<Res<LocalPlatform>>,
    mut platform_query: Query<&mut PlatformInventory>,
    mut held_item: ResMut<HeldItem>,
    mut slot_query: Query<
        (
            &Interaction,
            &UpperPanelSlot,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    if !inventory_open.0 {
        return;
    }

    // Determine if upper panel should be visible
    let has_platform = local_platform.is_some();
    let show_upper_panel = creative_mode.enabled || has_platform;

    if !show_upper_panel {
        return;
    }

    // Build the items to display (same logic as update_upper_panel_slots)
    let items_to_display: Vec<(ItemId, u32)> = if let Some(ref platform) = local_platform {
        if let Ok(platform_inv) = platform_query.get(platform.0) {
            let mut items: Vec<(ItemId, u32)> = platform_inv
                .items_by_id()
                .iter()
                .filter(|(item_id, _)| category.0.matches(**item_id))
                .map(|(&id, &count)| (id, count))
                .collect();
            items.sort_by_key(|(id, _)| id.raw());
            items
        } else {
            vec![]
        }
    } else if creative_mode.enabled {
        get_filtered_items(true, &category.0)
            .into_iter()
            .map(|id| (id, 64))
            .collect()
    } else {
        vec![]
    };

    let total_items = items_to_display.len();
    let total_pages = total_items.div_ceil(UPPER_PANEL_SLOTS);
    let current_page = page.0.min(total_pages.saturating_sub(1));
    let start_idx = current_page * UPPER_PANEL_SLOTS;

    for (interaction, slot, mut bg_color, mut border_color) in slot_query.iter_mut() {
        let slot_idx = slot.0;
        let item_idx = start_idx + slot_idx;

        match *interaction {
            Interaction::Pressed => {
                if item_idx < total_items {
                    let (item_id, _count) = items_to_display[item_idx];

                    if creative_mode.enabled {
                        // Creative mode: Pick up 64 items (infinite)
                        held_item.0 = Some((item_id, 64));
                    } else if let Some(ref platform) = local_platform {
                        // Platform mode: Take from platform inventory
                        if let Ok(mut platform_inv) = platform_query.get_mut(platform.0) {
                            // Take up to 64 items
                            let take_count = platform_inv.get_count_by_id(item_id).min(64);
                            if take_count > 0 {
                                platform_inv.remove_item_by_id(item_id, take_count);
                                held_item.0 = Some((item_id, take_count));
                            }
                        }
                    }
                }
                *border_color = BorderColor(SLOT_SELECTED_BORDER);
            }
            Interaction::Hovered => {
                *border_color = BorderColor(SLOT_HOVER_BORDER);
                *bg_color = BackgroundColor(SLOT_HOVER_BG);
            }
            Interaction::None => {
                *border_color = BorderColor(SLOT_BORDER_COLOR);
                *bg_color = BackgroundColor(SLOT_BG);
            }
        }
    }
}

/// Handle upper panel page navigation buttons
pub fn upper_panel_page_nav(
    mut page: ResMut<GlobalInventoryPage>,
    mut interaction_query: Query<
        (
            &Interaction,
            &GlobalInventoryPageButton,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if button.next {
                    page.0 = page.0.saturating_add(1);
                } else {
                    page.0 = page.0.saturating_sub(1);
                }
                *bg_color = BackgroundColor(Color::srgba(0.4, 0.4, 0.45, 1.0));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 1.0));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 1.0));
            }
        }
    }
}

/// Handle upper panel category tab clicks
pub fn upper_panel_category_click(
    mut category: ResMut<GlobalInventoryCategory>,
    mut page: ResMut<GlobalInventoryPage>,
    mut interaction_query: Query<
        (
            &Interaction,
            &GlobalInventoryCategoryTab,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, tab, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        let is_selected = category.0 == tab.0;

        match *interaction {
            Interaction::Pressed => {
                // Change category and reset to page 0
                if category.0 != tab.0 {
                    category.0 = tab.0;
                    page.0 = 0;
                }
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 1.0));
                *border_color = BorderColor(SLOT_BORDER_COLOR);
            }
            Interaction::Hovered => {
                if !is_selected {
                    *bg_color = BackgroundColor(Color::srgba(0.25, 0.25, 0.3, 1.0));
                }
            }
            Interaction::None => {
                if is_selected {
                    *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 1.0));
                    *border_color = BorderColor(SLOT_BORDER_COLOR);
                } else {
                    *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 1.0));
                    *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                }
            }
        }
    }
}

/// Update creative catalog item sprites
pub fn update_creative_catalog_sprites(
    inventory_open: Res<InventoryOpen>,
    creative_mode: Res<CreativeMode>,
    item_sprites: Res<ItemSprites>,
    mut query: Query<(&CreativeItemImage, &mut ImageNode, &mut Visibility)>,
) {
    // Only show sprites when inventory is open in creative mode
    let should_show = inventory_open.0 && creative_mode.enabled;

    for (item, mut image, mut visibility) in query.iter_mut() {
        if should_show {
            if let Some(sprite) = item_sprites.get_id(ItemId::from(item.0)) {
                image.image = sprite;
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
