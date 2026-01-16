//! Inventory UI systems

use crate::components::*;
use crate::core::{items, ItemId};
use crate::input::{GameAction, InputManager};
use crate::player::{LocalPlatform, LocalPlayer, PlatformInventory, PlayerInventory};
use crate::setup::ui::{
    UpperPanel, UpperPanelPageText, UpperPanelSlot, UpperPanelSlotCount, UpperPanelSlotImage,
    UpperPanelTabs, SLOT_BG, SLOT_BORDER_COLOR, SLOT_HOVER_BG, SLOT_HOVER_BORDER,
    SLOT_SELECTED_BORDER, UPPER_PANEL_SLOTS,
};
use crate::{HOTBAR_SLOTS, MAX_STACK_SIZE, NUM_SLOTS};
use bevy::color::Srgba;
use bevy::prelude::*;
use tracing::{info, warn};

/// Set the UI open state (no-op, kept for API compatibility)
pub fn set_ui_open_state(_ui_open: bool) {
    // No-op
}

/// Return held item to inventory when closing
fn return_held_item_to_inventory(inventory: &mut PlayerInventory, held_item: &mut HeldItem) {
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
    local_platform: Option<Res<LocalPlatform>>,
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

/// Handle creative inventory item button clicks (only in creative mode)
pub fn creative_inventory_click(
    creative_inv_open: Res<InventoryOpen>,
    creative_mode: Res<CreativeMode>,
    mut held_item: ResMut<HeldItem>,
    mut interaction_query: Query<
        (
            &Interaction,
            &CreativeItemButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    // Only handle clicks in creative mode with inventory open
    if !creative_inv_open.0 || !creative_mode.enabled {
        return;
    }

    for (interaction, button, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        let block_type = button.0;

        match *interaction {
            Interaction::Pressed => {
                // Pick up 64 of this item for drag and drop
                // Replace any existing held item (in creative mode, no item loss)
                held_item.0 = Some((block_type, 64));
                // Visual feedback (selected/pressed uses yellow border)
                *border_color = BorderColor(SLOT_SELECTED_BORDER);
            }
            Interaction::Hovered => {
                // Highlight on hover
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                // Slightly brighter background
                let base = block_type.color();
                let Srgba {
                    red,
                    green,
                    blue,
                    alpha,
                } = base.to_srgba();
                *bg_color = BackgroundColor(Color::srgba(
                    (red + 0.2).min(1.0),
                    (green + 0.2).min(1.0),
                    (blue + 0.2).min(1.0),
                    alpha,
                ));
            }
            Interaction::None => {
                // Reset to normal
                *border_color = BorderColor(Color::srgba(0.3, 0.3, 0.3, 1.0));
                *bg_color = BackgroundColor(block_type.color());
            }
        }
    }
}

/// Handle inventory slot clicks (pick up / place items)
pub fn inventory_slot_click(
    inventory_open: Res<InventoryOpen>,
    local_player: Option<Res<LocalPlayer>>,
    mut inventory_query: Query<&mut PlayerInventory>,
    mut held_item: ResMut<HeldItem>,
    input: Res<InputManager>,
    mut interaction_query: Query<
        (
            &Interaction,
            &InventorySlotUI,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    if !inventory_open.0 {
        return;
    }

    let Some(local_player) = local_player else {
        return;
    };
    let Ok(mut inventory) = inventory_query.get_mut(local_player.0) else {
        return;
    };

    let shift_held = input.pressed(GameAction::ModifierShift);

    for (interaction, slot_ui, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        let slot_idx = slot_ui.0;

        match *interaction {
            Interaction::Pressed => {
                if shift_held {
                    // Shift+Click: Quick move between hotbar and main inventory
                    if let Some((block_type, count)) = inventory.slots[slot_idx].take() {
                        // Determine target area
                        let target_range = if slot_idx < HOTBAR_SLOTS {
                            // From hotbar -> main inventory
                            HOTBAR_SLOTS..NUM_SLOTS
                        } else {
                            // From main -> hotbar
                            0..HOTBAR_SLOTS
                        };

                        // Try to stack first
                        let mut remaining = count;
                        for target_idx in target_range.clone() {
                            if remaining == 0 {
                                break;
                            }
                            if let Some((bt, ref mut c)) = inventory.slots[target_idx] {
                                if bt == block_type && *c < MAX_STACK_SIZE {
                                    let space = MAX_STACK_SIZE - *c;
                                    let to_add = remaining.min(space);
                                    *c += to_add;
                                    remaining -= to_add;
                                }
                            }
                        }

                        // Then find empty slots
                        for target_idx in target_range {
                            if remaining == 0 {
                                break;
                            }
                            if inventory.slots[target_idx].is_none() {
                                let to_add = remaining.min(MAX_STACK_SIZE);
                                inventory.slots[target_idx] = Some((block_type, to_add));
                                remaining -= to_add;
                            }
                        }

                        // Put back any remaining
                        if remaining > 0 {
                            inventory.slots[slot_idx] = Some((block_type, remaining));
                        }
                    }
                } else {
                    // Normal click: pick up or place
                    let slot_item = inventory.slots[slot_idx].take();
                    let held = held_item.0.take();

                    match (slot_item, held) {
                        (None, None) => {
                            // Both empty, do nothing
                        }
                        (Some(item), None) => {
                            // Pick up item from slot
                            held_item.0 = Some(item);
                        }
                        (None, Some(item)) => {
                            // Place held item into slot
                            inventory.slots[slot_idx] = Some(item);
                        }
                        (Some((slot_type, slot_count)), Some((held_type, held_count))) => {
                            if slot_type == held_type {
                                // Same type - try to stack
                                let total = slot_count + held_count;
                                if total <= MAX_STACK_SIZE {
                                    inventory.slots[slot_idx] = Some((slot_type, total));
                                } else {
                                    inventory.slots[slot_idx] = Some((slot_type, MAX_STACK_SIZE));
                                    held_item.0 = Some((held_type, total - MAX_STACK_SIZE));
                                }
                            } else {
                                // Different types - swap
                                inventory.slots[slot_idx] = Some((held_type, held_count));
                                held_item.0 = Some((slot_type, slot_count));
                            }
                        }
                    }
                }

                // Visual feedback (selected/pressed uses yellow border)
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

/// Helper function to perform shift-click move on a slot
fn perform_shift_click_move(inventory: &mut PlayerInventory, slot_idx: usize) -> bool {
    if let Some((block_type, count)) = inventory.slots[slot_idx].take() {
        // Determine target area
        let target_range = if slot_idx < HOTBAR_SLOTS {
            // From hotbar -> main inventory
            HOTBAR_SLOTS..NUM_SLOTS
        } else {
            // From main -> hotbar
            0..HOTBAR_SLOTS
        };

        // Try to stack first
        let mut remaining = count;
        for target_idx in target_range.clone() {
            if remaining == 0 {
                break;
            }
            if let Some((bt, ref mut c)) = inventory.slots[target_idx] {
                if bt == block_type && *c < MAX_STACK_SIZE {
                    let space = MAX_STACK_SIZE - *c;
                    let to_add = remaining.min(space);
                    *c += to_add;
                    remaining -= to_add;
                }
            }
        }

        // Then find empty slots
        for target_idx in target_range {
            if remaining == 0 {
                break;
            }
            if inventory.slots[target_idx].is_none() {
                let to_add = remaining.min(MAX_STACK_SIZE);
                inventory.slots[target_idx] = Some((block_type, to_add));
                remaining -= to_add;
            }
        }

        // Put back any remaining
        if remaining > 0 {
            inventory.slots[slot_idx] = Some((block_type, remaining));
        }
        return remaining < count; // Return true if anything was moved
    }
    false
}

/// Continuous shift+click support for inventory
pub fn inventory_continuous_shift_click(
    inventory_open: Res<InventoryOpen>,
    local_player: Option<Res<LocalPlayer>>,
    mut inventory_query: Query<&mut PlayerInventory>,
    input: Res<InputManager>,
    mut action_timer: ResMut<ContinuousActionTimer>,
    interaction_query: Query<(&Interaction, &InventorySlotUI)>,
) {
    if !inventory_open.0 {
        return;
    }

    let Some(local_player) = local_player else {
        return;
    };
    let Ok(mut inventory) = inventory_query.get_mut(local_player.0) else {
        return;
    };

    let shift_held = input.pressed(GameAction::ModifierShift);
    if !shift_held || !input.pressed(GameAction::PrimaryAction) {
        return;
    }

    // Skip if timer hasn't finished (and this isn't the first click handled by inventory_slot_click)
    if !action_timer.inventory_timer.finished() {
        return;
    }

    // Find hovered slot
    for (interaction, slot_ui) in interaction_query.iter() {
        if *interaction == Interaction::Hovered {
            let slot_idx = slot_ui.0;
            if perform_shift_click_move(&mut inventory, slot_idx) {
                action_timer.inventory_timer.reset();
            }
            break;
        }
    }
}

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

/// Handle trash slot clicks (delete held item)
#[allow(clippy::type_complexity)]
pub fn trash_slot_click(
    inventory_open: Res<InventoryOpen>,
    mut held_item: ResMut<HeldItem>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<TrashSlot>),
    >,
) {
    if !inventory_open.0 {
        return;
    }

    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Delete held item
                held_item.0 = None;
                *border_color = BorderColor(Color::srgb(1.0, 0.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(1.0, 0.5, 0.5));
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.1, 0.1));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgb(0.6, 0.2, 0.2));
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.1, 0.1));
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

// ============================================================================
// Breaking Progress Bar UI
// ============================================================================

/// Spawn the breaking progress bar UI (hidden by default)
pub fn spawn_breaking_progress_ui(mut commands: Commands) {
    // Progress bar container - centered on screen, slightly above crosshair
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(45.0), // Slightly above center
                width: Val::Px(200.0),
                height: Val::Px(10.0),
                margin: UiRect::left(Val::Px(-100.0)), // Center horizontally
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            BorderRadius::all(Val::Px(3.0)),
            Visibility::Hidden,
            BreakingProgressUI,
        ))
        .with_children(|parent| {
            // Progress bar fill
            parent.spawn((
                Node {
                    width: Val::Percent(0.0), // Will be updated based on progress
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                BorderRadius::all(Val::Px(3.0)),
                BreakingProgressBarFill,
            ));
        });
}

/// Update the breaking progress bar UI based on breaking progress
pub fn update_breaking_progress_ui(
    breaking_progress: Res<BreakingProgress>,
    mut container_query: Query<&mut Visibility, With<BreakingProgressUI>>,
    mut fill_query: Query<&mut Node, With<BreakingProgressBarFill>>,
) {
    let Ok(mut container_visibility) = container_query.get_single_mut() else {
        return;
    };

    if breaking_progress.is_breaking() && breaking_progress.progress < 1.0 {
        // Show progress bar
        *container_visibility = Visibility::Visible;

        // Update fill width
        if let Ok(mut fill_node) = fill_query.get_single_mut() {
            fill_node.width = Val::Percent(breaking_progress.progress * 100.0);
        }
    } else {
        // Hide progress bar
        *container_visibility = Visibility::Hidden;
    }
}

// ============================================================================
// Upper Panel (Creative / Platform Inventory) Systems
// ============================================================================

/// Get all available items for the upper panel based on creative mode and category
fn get_filtered_items(creative_mode: bool, category: &ItemCategory) -> Vec<ItemId> {
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
