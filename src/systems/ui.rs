//! UI-related systems (inventory, hotbar, debug HUD, command input)

use crate::components::*;
use crate::player::Inventory;
use crate::components::{LoadGameEvent, SaveGameEvent};
use crate::{BlockType, HOTBAR_SLOTS, MAX_STACK_SIZE, NUM_SLOTS};
use bevy::color::Srgba;
use bevy::diagnostic::DiagnosticsStore;
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
                let _ = canvas.set_attribute("data-ui-open", if ui_open { "true" } else { "false" });
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_ui_open_state(_ui_open: bool) {
    // No-op on native
}

// === Hotbar Systems ===

pub fn update_hotbar_ui(
    inventory: Res<Inventory>,
    mut slot_query: Query<(&HotbarSlot, &mut BackgroundColor, &mut BorderColor)>,
    mut count_query: Query<(&HotbarSlotCount, &mut Text)>,
) {
    if !inventory.is_changed() {
        return;
    }

    // Update slot backgrounds - use slot index for selection
    for (slot, mut bg, mut border) in slot_query.iter_mut() {
        let is_selected = inventory.selected_slot == slot.0;
        let has_item = inventory.get_slot(slot.0).is_some();

        if is_selected {
            // Selected slot - same highlight for empty and filled
            *bg = BackgroundColor(Color::srgba(0.4, 0.4, 0.2, 0.9));
            *border = BorderColor(Color::srgba(1.0, 1.0, 0.5, 1.0));
        } else if has_item {
            // Non-selected filled slot
            *bg = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8));
            *border = BorderColor(Color::srgba(0.5, 0.5, 0.5, 1.0));
        } else {
            // Non-selected empty slot
            *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8));
            *border = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
        }
    }

    // Update slot counts
    for (slot_count, mut text) in count_query.iter_mut() {
        if let Some(block_type) = inventory.get_slot(slot_count.0) {
            let count = inventory.get_slot_count(slot_count.0);
            // Show abbreviated name and count
            let name = match block_type {
                BlockType::Grass => "Grs",
                BlockType::Stone => "Stn",
                BlockType::IronOre => "Fe",
                BlockType::Coal => "C",
                BlockType::IronIngot => "FeI",
                BlockType::MinerBlock => "Min",
                BlockType::ConveyorBlock => "Cnv",
                BlockType::CopperOre => "Cu",
                BlockType::CopperIngot => "CuI",
                BlockType::CrusherBlock => "Cru",
                BlockType::FurnaceBlock => "Fur",
            };
            **text = format!("{}\n{}", name, count);
        } else {
            **text = String::new();
        }
    }
}

/// Update the hotbar item name display to show the selected item's name
pub fn update_hotbar_item_name(
    inventory: Res<Inventory>,
    inventory_open: Res<InventoryOpen>,
    mut text_query: Query<(&mut Text, &mut Node), With<HotbarItemNameText>>,
) {
    let Ok((mut text, mut node)) = text_query.get_single_mut() else {
        return;
    };

    // Hide when inventory is open
    if inventory_open.0 {
        text.0 = String::new();
        return;
    }

    // Show selected item name
    if let Some(block_type) = inventory.selected_block() {
        let name = block_type.name();
        text.0 = name.to_string();
        // Center the text by adjusting margin based on text length
        let char_width = 8.0; // Approximate character width
        node.margin.left = Val::Px(-(name.len() as f32 * char_width / 2.0));
    } else {
        text.0 = String::new();
    }
}

// === Debug HUD Systems ===

pub fn update_window_title_fps(diagnostics: Res<DiagnosticsStore>, mut windows: Query<&mut Window>) {
    if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            if let Ok(mut window) = windows.get_single_mut() {
                window.title = format!("Idle Factory - FPS: {:.0}", value);
            }
        }
    }
}

/// Toggle debug HUD with F3 key
pub fn toggle_debug_hud(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugHudState>,
    debug_query: Query<Entity, With<DebugHudText>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        debug_state.visible = !debug_state.visible;

        if debug_state.visible {
            // Spawn debug HUD
            commands.spawn((
                Text::new("Debug Info"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                DebugHudText,
            ));
        } else {
            // Remove debug HUD
            for entity in debug_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Update debug HUD content
pub fn update_debug_hud(
    debug_state: Res<DebugHudState>,
    mut text_query: Query<&mut Text, With<DebugHudText>>,
    diagnostics: Res<DiagnosticsStore>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&PlayerCamera>,
    world_data: Res<crate::world::WorldData>,
    creative_mode: Res<CreativeMode>,
    cursor_state: Res<CursorLockState>,
) {
    if !debug_state.visible {
        return;
    }

    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let (pos_str, dir_str) = if let Ok(transform) = player_query.get_single() {
        let pos = transform.translation;
        let dir = if let Ok(camera) = camera_query.get_single() {
            format!("Pitch: {:.1}° Yaw: {:.1}°", camera.pitch.to_degrees(), camera.yaw.to_degrees())
        } else {
            "N/A".to_string()
        };
        (format!("X: {:.1} Y: {:.1} Z: {:.1}", pos.x, pos.y, pos.z), dir)
    } else {
        ("N/A".to_string(), "N/A".to_string())
    };

    let chunk_count = world_data.chunks.len();
    let mode_str = if creative_mode.enabled { "Creative" } else { "Survival" };
    let pause_str = if cursor_state.paused { " [PAUSED]" } else { "" };

    text.0 = format!(
        "FPS: {:.0}\nPos: {}\nDir: {}\nChunks: {}\nMode: {}{}",
        fps, pos_str, dir_str, chunk_count, mode_str, pause_str
    );
}

// === Inventory Systems ===

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
pub fn inventory_toggle(
    key_input: Res<ButtonInput<KeyCode>>,
    mut inventory_open: ResMut<InventoryOpen>,
    mut inventory: ResMut<Inventory>,
    mut held_item: ResMut<HeldItem>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    command_state: Res<CommandInputState>,
    cursor_state: Res<CursorLockState>,
    creative_mode: Res<CreativeMode>,
    mut ui_query: Query<&mut Visibility, With<InventoryUI>>,
    mut creative_panel_query: Query<&mut Visibility, (With<CreativePanel>, Without<InventoryUI>)>,
    mut windows: Query<&mut Window>,
) {
    // Don't toggle if other UIs are open or game is paused (input matrix: E key)
    if interacting_furnace.0.is_some() || interacting_crusher.0.is_some() || command_state.open || cursor_state.paused {
        if key_input.just_pressed(KeyCode::KeyE) {
            info!("[INVENTORY] E pressed but blocked: furnace={}, crusher={}, command={}, paused={}",
                interacting_furnace.0.is_some(),
                interacting_crusher.0.is_some(),
                command_state.open,
                cursor_state.paused);
        }
        return;
    }

    // E key to toggle inventory
    if key_input.just_pressed(KeyCode::KeyE) {
        info!("[INVENTORY] E key pressed, toggling from {} to {}", inventory_open.0, !inventory_open.0);
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
        info!("[INVENTORY] Updated {} UI entities, now open={}", ui_count, inventory_open.0);

        if ui_count == 0 {
            warn!("[INVENTORY] No InventoryUI entity found! UI will not display.");
        }

        // Show/hide creative panel based on creative mode
        for mut vis in creative_panel_query.iter_mut() {
            *vis = if inventory_open.0 && creative_mode.enabled {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
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

    // ESC to close
    if inventory_open.0 && key_input.just_pressed(KeyCode::Escape) {
        inventory_open.0 = false;

        // Return held item when closing
        return_held_item_to_inventory(&mut inventory, &mut held_item);

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Also hide creative panel
        for mut vis in creative_panel_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Unlock cursor - JS will auto-relock via data-ui-open observer (BUG-6 fix)
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(false);
        }
    }
}

/// Handle creative inventory item button clicks (only in creative mode)
pub fn creative_inventory_click(
    creative_inv_open: Res<InventoryOpen>,
    creative_mode: Res<CreativeMode>,
    mut inventory: ResMut<Inventory>,
    mut interaction_query: Query<
        (&Interaction, &CreativeItemButton, &mut BackgroundColor, &mut BorderColor),
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
                let Srgba { red, green, blue, alpha } = base.to_srgba();
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
        (&Interaction, &InventorySlotUI, &mut BackgroundColor, &mut BorderColor),
        Changed<Interaction>,
    >,
) {
    if !inventory_open.0 {
        return;
    }

    let shift_held = key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight);

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
                            if remaining == 0 { break; }
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
                            if remaining == 0 { break; }
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
            if remaining == 0 { break; }
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
            if remaining == 0 { break; }
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

    let shift_held = key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight);
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
    mut slot_query: Query<(&InventorySlotUI, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    if !inventory_open.0 {
        return;
    }

    for (slot_ui, mut bg_color, children) in slot_query.iter_mut() {
        let slot_idx = slot_ui.0;

        if let Some((block_type, count)) = inventory.slots[slot_idx] {
            // Show item color and count
            *bg_color = BackgroundColor(block_type.color());

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
            // Empty slot
            *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9));

            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = String::new();
                }
            }
        }
    }
}

/// Update held item display to follow cursor and show held item
pub fn update_held_item_display(
    inventory_open: Res<InventoryOpen>,
    held_item: Res<HeldItem>,
    windows: Query<&Window>,
    mut held_display_query: Query<(&mut Node, &mut BackgroundColor, &mut Visibility), With<HeldItemDisplay>>,
    mut held_text_query: Query<&mut Text, With<HeldItemText>>,
) {
    let Ok((mut node, mut bg_color, mut visibility)) = held_display_query.get_single_mut() else {
        return;
    };

    // Only show when inventory is open and we're holding something
    if !inventory_open.0 {
        *visibility = Visibility::Hidden;
        return;
    }

    match &held_item.0 {
        Some((block_type, count)) => {
            // Show the held item
            *visibility = Visibility::Visible;
            *bg_color = BackgroundColor(block_type.color());

            // Update count text
            if let Ok(mut text) = held_text_query.get_single_mut() {
                text.0 = if *count > 1 {
                    format!("{}", count)
                } else {
                    String::new()
                };
            }

            // Position at cursor
            if let Ok(window) = windows.get_single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    // Offset so item appears slightly below and to the right of cursor
                    node.left = Val::Px(cursor_pos.x + 8.0);
                    node.top = Val::Px(cursor_pos.y + 8.0);
                }
            }
        }
        None => {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Update inventory tooltip to show item name when hovering over slots
pub fn update_inventory_tooltip(
    inventory_open: Res<InventoryOpen>,
    inventory: Res<Inventory>,
    windows: Query<&Window>,
    slot_query: Query<(&Interaction, &InventorySlotUI, &GlobalTransform)>,
    creative_query: Query<(&Interaction, &CreativeItemButton, &GlobalTransform)>,
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
    let mut hovered_item: Option<(BlockType, Option<u32>, Vec2)> = None;
    for (interaction, slot_ui, global_transform) in slot_query.iter() {
        if *interaction == Interaction::Hovered {
            let slot_idx = slot_ui.0;
            if let Some((block_type, count)) = inventory.slots[slot_idx] {
                let pos = global_transform.translation();
                hovered_item = Some((block_type, Some(count), Vec2::new(pos.x, pos.y)));
                break;
            }
        }
    }

    // Check creative catalog items if no inventory slot is hovered
    if hovered_item.is_none() {
        for (interaction, creative_btn, global_transform) in creative_query.iter() {
            if *interaction == Interaction::Hovered {
                let pos = global_transform.translation();
                hovered_item = Some((creative_btn.0, None, Vec2::new(pos.x, pos.y)));
                break;
            }
        }
    }

    if let Some((block_type, count_opt, slot_pos)) = hovered_item {
        *visibility = Visibility::Inherited;

        // Position tooltip near the slot (offset to the right and up)
        if let Ok(window) = windows.get_single() {
            let half_width = window.width() / 2.0;
            let half_height = window.height() / 2.0;
            // Convert from global UI coords to absolute position
            node.left = Val::Px(slot_pos.x + half_width + 45.0);
            node.top = Val::Px(half_height - slot_pos.y - 10.0);
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

// === Command Input System ===

/// Toggle command input with T or / key
#[allow(clippy::too_many_arguments)]
pub fn command_input_toggle(
    key_input: Res<ButtonInput<KeyCode>>,
    mut command_state: ResMut<CommandInputState>,
    mut ui_query: Query<&mut Visibility, With<CommandInputUI>>,
    mut text_query: Query<&mut Text, With<CommandInputText>>,
    mut windows: Query<&mut Window>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    creative_inv_open: Res<InventoryOpen>,
) {
    // Don't open if other UI is open
    if interacting_furnace.0.is_some() || interacting_crusher.0.is_some() || creative_inv_open.0 {
        return;
    }

    // T or / to open command input (when not already open)
    if !command_state.open
        && (key_input.just_pressed(KeyCode::KeyT) || key_input.just_pressed(KeyCode::Slash))
    {
        command_state.open = true;
        command_state.text.clear();

        // Start with / if opened with slash key
        if key_input.just_pressed(KeyCode::Slash) {
            command_state.text.push('/');
        }

        // Show UI
        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Visible;
        }

        // Reset text display
        for mut text in text_query.iter_mut() {
            text.0 = format!("> {}|", command_state.text);
        }

        // Unlock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(true);
        }
    }
}

/// Handle command input text entry
#[allow(clippy::too_many_arguments)]
pub fn command_input_handler(
    key_input: Res<ButtonInput<KeyCode>>,
    mut command_state: ResMut<CommandInputState>,
    mut ui_query: Query<&mut Visibility, With<CommandInputUI>>,
    mut text_query: Query<&mut Text, With<CommandInputText>>,
    mut windows: Query<&mut Window>,
    mut creative_mode: ResMut<CreativeMode>,
    mut inventory: ResMut<Inventory>,
    mut save_events: EventWriter<SaveGameEvent>,
    mut load_events: EventWriter<LoadGameEvent>,
) {
    if !command_state.open {
        return;
    }

    // ESC to close without executing
    if key_input.just_pressed(KeyCode::Escape) {
        command_state.open = false;
        command_state.text.clear();

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Enter to execute command
    if key_input.just_pressed(KeyCode::Enter) {
        let command = command_state.text.clone();
        command_state.open = false;
        command_state.text.clear();

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }

        // Execute command
        execute_command(&command, &mut creative_mode, &mut inventory, &mut save_events, &mut load_events);
        return;
    }

    // Backspace to delete character
    if key_input.just_pressed(KeyCode::Backspace) {
        command_state.text.pop();
    }

    // Handle character input
    for key in key_input.get_just_pressed() {
        if let Some(c) = keycode_to_char(*key, key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight)) {
            command_state.text.push(c);
        }
    }

    // Update display text
    for mut text in text_query.iter_mut() {
        text.0 = format!("> {}|", command_state.text);
    }
}

/// Convert key code to character
fn keycode_to_char(key_code: KeyCode, shift: bool) -> Option<char> {
    match key_code {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some(if shift { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift { '(' } else { '9' }),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Equal => Some(if shift { '+' } else { '=' }),
        KeyCode::Slash => Some(if shift { '?' } else { '/' }),
        _ => None,
    }
}

/// Execute a command
fn execute_command(
    command: &str,
    creative_mode: &mut ResMut<CreativeMode>,
    inventory: &mut ResMut<Inventory>,
    save_events: &mut EventWriter<SaveGameEvent>,
    load_events: &mut EventWriter<LoadGameEvent>,
) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "/creative" | "creative" => {
            creative_mode.enabled = true;
            // Give all items when entering creative mode
            let all_items = [
                BlockType::Stone,
                BlockType::Grass,
                BlockType::IronOre,
                BlockType::Coal,
                BlockType::IronIngot,
                BlockType::CopperOre,
                BlockType::CopperIngot,
                BlockType::MinerBlock,
                BlockType::ConveyorBlock,
                BlockType::CrusherBlock,
            ];
            for (i, block_type) in all_items.iter().take(9).enumerate() {
                inventory.slots[i] = Some((*block_type, 64));
            }
            info!("Creative mode enabled");
        }
        "/survival" | "survival" => {
            creative_mode.enabled = false;
            info!("Survival mode enabled");
        }
        "/give" | "give" => {
            // /give <item> [count]
            if parts.len() >= 2 {
                let item_name = parts[1].to_lowercase();
                let count: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(64);

                if let Some(block_type) = parse_item_name(&item_name) {
                    inventory.add_item(block_type, count);
                    info!("Gave {} x{}", block_type.name(), count);
                }
            }
        }
        "/clear" | "clear" => {
            // Clear inventory
            for slot in inventory.slots.iter_mut() {
                *slot = None;
            }
            info!("Inventory cleared");
        }
        "/save" | "save" => {
            // /save [filename]
            let filename = parts.get(1).unwrap_or(&"quicksave").to_string();
            save_events.send(SaveGameEvent { filename });
        }
        "/load" | "load" => {
            // /load [filename]
            let filename = parts.get(1).unwrap_or(&"quicksave").to_string();
            load_events.send(LoadGameEvent { filename });
        }
        "/help" | "help" => {
            info!("Commands: /creative, /survival, /give <item> [count], /clear, /save [name], /load [name]");
        }
        _ => {
            info!("Unknown command: {}", command);
        }
    }
}

/// Parse item name to BlockType
fn parse_item_name(name: &str) -> Option<BlockType> {
    match name {
        "stone" => Some(BlockType::Stone),
        "grass" => Some(BlockType::Grass),
        "ironore" | "iron_ore" => Some(BlockType::IronOre),
        "copperore" | "copper_ore" => Some(BlockType::CopperOre),
        "coal" => Some(BlockType::Coal),
        "ironingot" | "iron_ingot" | "iron" => Some(BlockType::IronIngot),
        "copperingot" | "copper_ingot" | "copper" => Some(BlockType::CopperIngot),
        "miner" => Some(BlockType::MinerBlock),
        "conveyor" => Some(BlockType::ConveyorBlock),
        "crusher" => Some(BlockType::CrusherBlock),
        "furnace" => Some(BlockType::FurnaceBlock),
        _ => None,
    }
}

/// Select slot with number keys (1-9) or scroll wheel
pub fn select_block_type(
    key_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut inventory: ResMut<Inventory>,
    input_resources: InputStateResourcesWithCursor,
) {
    // Use InputState to check if hotbar selection is allowed (see CLAUDE.md 入力マトリクス)
    let input_state = input_resources.get_state();
    if !input_state.allows_hotbar() {
        // Still need to drain events to prevent accumulation
        for _ in mouse_wheel.read() {}
        return;
    }

    // Handle mouse wheel scroll (cycles through hotbar slots 0-8 only)
    for event in mouse_wheel.read() {
        let scroll = event.y;
        if scroll > 0.0 {
            // Scroll up - previous slot (within hotbar)
            if inventory.selected_slot > 0 {
                inventory.selected_slot -= 1;
            } else {
                inventory.selected_slot = HOTBAR_SLOTS - 1;
            }
        } else if scroll < 0.0 {
            // Scroll down - next slot (within hotbar)
            if inventory.selected_slot < HOTBAR_SLOTS - 1 {
                inventory.selected_slot += 1;
            } else {
                inventory.selected_slot = 0;
            }
        }
    }

    // Number keys 1-9 select hotbar slots directly
    let digit_keys = [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
    ];
    for (key, slot) in digit_keys {
        if key_input.just_pressed(key) {
            inventory.selected_slot = slot;
        }
    }
}
