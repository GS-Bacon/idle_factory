//! Inventory UI systems

use crate::components::*;
use crate::player::Inventory;
use crate::{BlockType, HOTBAR_SLOTS, MAX_STACK_SIZE, NUM_SLOTS};
use bevy::color::Srgba;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use tracing::{info, warn};

/// Set the UI open state for JavaScript overlay control (WASM only)
#[cfg(target_arch = "wasm32")]
pub fn set_ui_open_state(ui_open: bool) {
    use web_sys::window;
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(canvas) = doc.get_element_by_id("bevy-canvas") {
                let _ =
                    canvas.set_attribute("data-ui-open", if ui_open { "true" } else { "false" });
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_ui_open_state(_ui_open: bool) {
    // No-op on native
}

/// Return held item to inventory when closing
fn return_held_item_to_inventory(inventory: &mut Inventory, held_item: &mut HeldItem) {
    if let Some((block_type, count)) = held_item.0.take() {
        // Try to add to inventory
        let remaining = inventory.add_item(block_type, count);
        if remaining > 0 {
            // If inventory is full, item is lost (or could be dropped later)
            // For now, just put back what couldn't fit
            held_item.0 = Some((block_type, remaining));
        }
    }
}

/// Toggle inventory with E key (works in both survival and creative mode)
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn inventory_toggle(
    key_input: Res<ButtonInput<KeyCode>>,
    mut inventory_open: ResMut<InventoryOpen>,
    mut inventory: ResMut<Inventory>,
    mut held_item: ResMut<HeldItem>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    command_state: Res<CommandInputState>,
    mut cursor_state: ResMut<CursorLockState>,
    creative_mode: Res<CreativeMode>,
    mut ui_query: Query<&mut Visibility, With<InventoryUI>>,
    mut overlay_query: Query<
        &mut Visibility,
        (
            With<InventoryBackgroundOverlay>,
            Without<InventoryUI>,
            Without<CreativePanel>,
        ),
    >,
    mut creative_panel_query: Query<
        (&mut Visibility, &mut Node),
        (
            With<CreativePanel>,
            Without<InventoryUI>,
            Without<InventoryBackgroundOverlay>,
        ),
    >,
    mut windows: Query<&mut Window>,
) {
    // Don't toggle if other UIs are open or game is paused (input matrix: E key)
    if interacting_furnace.0.is_some()
        || interacting_crusher.0.is_some()
        || command_state.open
        || cursor_state.paused
    {
        if key_input.just_pressed(KeyCode::KeyE) {
            info!(
                "[INVENTORY] E pressed but blocked: furnace={}, crusher={}, command={}, paused={}",
                interacting_furnace.0.is_some(),
                interacting_crusher.0.is_some(),
                command_state.open,
                cursor_state.paused
            );
        }
        return;
    }

    // E key to toggle inventory
    if key_input.just_pressed(KeyCode::KeyE) {
        info!(
            "[INVENTORY] E key pressed, toggling from {} to {}",
            inventory_open.0, !inventory_open.0
        );
        inventory_open.0 = !inventory_open.0;

        // Return held item when closing
        if !inventory_open.0 {
            return_held_item_to_inventory(&mut inventory, &mut held_item);
        }

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

        // Show/hide creative panel based on creative mode
        // Use Display::None to also remove layout space in survival mode
        for (mut vis, mut node) in creative_panel_query.iter_mut() {
            if inventory_open.0 && creative_mode.enabled {
                *vis = Visibility::Visible;
                node.display = Display::Flex;
            } else {
                *vis = Visibility::Hidden;
                node.display = Display::None;
            }
        }

        // Unlock/lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            if inventory_open.0 {
                window.cursor_options.grab_mode = CursorGrabMode::None;
                window.cursor_options.visible = true;
                set_ui_open_state(true);
            } else {
                window.cursor_options.grab_mode = CursorGrabMode::Locked;
                window.cursor_options.visible = false;
                set_ui_open_state(false);
            }
        }
    }

    // ESC to close (just close inventory, don't trigger pause)
    if inventory_open.0 && key_input.just_pressed(KeyCode::Escape) {
        inventory_open.0 = false;

        // Return held item when closing
        return_held_item_to_inventory(&mut inventory, &mut held_item);

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Hide background overlay
        for mut vis in overlay_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Also hide creative panel
        for (mut vis, mut node) in creative_panel_query.iter_mut() {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }

        // Re-lock cursor after closing inventory (keep playing, don't pause)
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        // Ensure we don't get paused by toggle_cursor_lock running after this
        cursor_state.paused = false;
    }
}

/// Handle creative inventory item button clicks (only in creative mode)
pub fn creative_inventory_click(
    creative_inv_open: Res<InventoryOpen>,
    creative_mode: Res<CreativeMode>,
    mut inventory: ResMut<Inventory>,
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
                // Add 64 of this item to selected slot
                let slot = inventory.selected_slot;
                inventory.slots[slot] = Some((block_type, 64));
                // Visual feedback
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
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
    mut inventory: ResMut<Inventory>,
    mut held_item: ResMut<HeldItem>,
    key_input: Res<ButtonInput<KeyCode>>,
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

    let shift_held =
        key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight);

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

                // Visual feedback
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.7, 0.7, 0.7));
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.9));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9));
            }
        }
    }
}

/// Helper function to perform shift-click move on a slot
fn perform_shift_click_move(inventory: &mut Inventory, slot_idx: usize) -> bool {
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
    mut inventory: ResMut<Inventory>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut action_timer: ResMut<ContinuousActionTimer>,
    interaction_query: Query<(&Interaction, &InventorySlotUI)>,
) {
    if !inventory_open.0 {
        return;
    }

    let shift_held =
        key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight);
    if !shift_held || !mouse_button.pressed(MouseButton::Left) {
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
    inventory: Res<Inventory>,
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

    // Update slot sprite images
    for (slot_image, mut image_node, mut visibility) in image_query.iter_mut() {
        let slot_idx = slot_image.0;
        if let Some((block_type, _count)) = inventory.slots[slot_idx] {
            if let Some(sprite_handle) = item_sprites.get(block_type) {
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

        if let Some((block_type, count)) = inventory.slots[slot_idx] {
            // Use dark background if sprite exists, fallback color otherwise
            if item_sprites.get(block_type).is_some() {
                *bg_color = BackgroundColor(Color::srgba(0.14, 0.14, 0.14, 0.95));
            } else {
                let color = block_type.color();
                *bg_color = BackgroundColor(Color::srgba(
                    color.to_srgba().red * 0.5,
                    color.to_srgba().green * 0.5,
                    color.to_srgba().blue * 0.5,
                    0.6,
                ));
            }

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
            // Empty slot - respect hover state
            *bg_color = BackgroundColor(match interaction {
                Interaction::Hovered => Color::srgba(0.25, 0.25, 0.25, 0.9),
                Interaction::Pressed => Color::srgba(0.2, 0.2, 0.2, 0.9),
                Interaction::None => Color::srgba(0.14, 0.14, 0.14, 0.95),
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
                if let Some(sprite) = item_sprites.get(*block_type) {
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
pub fn update_inventory_tooltip(
    inventory_open: Res<InventoryOpen>,
    inventory: Res<Inventory>,
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

    // Find hovered slot (inventory slots)
    let mut hovered_item: Option<(BlockType, Option<u32>)> = None;
    for (interaction, slot_ui) in slot_query.iter() {
        if *interaction == Interaction::Hovered {
            let slot_idx = slot_ui.0;
            if let Some((block_type, count)) = inventory.slots[slot_idx] {
                hovered_item = Some((block_type, Some(count)));
                break;
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

    if let Some((block_type, count_opt)) = hovered_item {
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
                if let Some(count) = count_opt {
                    text.0 = format!("{} ({})", block_type.name(), count);
                } else {
                    // Creative catalog item - just show name
                    text.0 = block_type.name().to_string();
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
    item_sprites: Res<ItemSprites>,
    mut query: Query<(&CreativeItemImage, &mut ImageNode, &mut Visibility)>,
) {
    for (item, mut image, mut visibility) in query.iter_mut() {
        if let Some(sprite) = item_sprites.get(item.0) {
            image.image = sprite;
            // Use Inherited so visibility follows parent (CreativePanel)
            *visibility = Visibility::Inherited;
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
