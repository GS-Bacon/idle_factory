//! Inventory slot interaction systems

use crate::components::*;
use crate::input::{GameAction, InputManager};
use crate::player::{LocalPlayer, PlayerInventory};
use crate::setup::ui::{
    SLOT_BG, SLOT_BORDER_COLOR, SLOT_HOVER_BG, SLOT_HOVER_BORDER, SLOT_SELECTED_BORDER,
};
use crate::{HOTBAR_SLOTS, MAX_STACK_SIZE, NUM_SLOTS};
use bevy::color::Srgba;
use bevy::prelude::*;

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
                *border_color = BorderColor::all(SLOT_SELECTED_BORDER);
            }
            Interaction::Hovered => {
                // Highlight on hover
                *border_color = BorderColor::all(Color::srgb(0.8, 0.8, 0.8));
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
                *border_color = BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 1.0));
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
                *border_color = BorderColor::all(SLOT_SELECTED_BORDER);
            }
            Interaction::Hovered => {
                *border_color = BorderColor::all(SLOT_HOVER_BORDER);
                *bg_color = BackgroundColor(SLOT_HOVER_BG);
            }
            Interaction::None => {
                *border_color = BorderColor::all(SLOT_BORDER_COLOR);
                *bg_color = BackgroundColor(SLOT_BG);
            }
        }
    }
}

/// Helper function to perform shift-click move on a slot
pub(super) fn perform_shift_click_move(inventory: &mut PlayerInventory, slot_idx: usize) -> bool {
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
    if !action_timer.inventory_timer.is_finished() {
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
                *border_color = BorderColor::all(Color::srgb(1.0, 0.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor::all(Color::srgb(1.0, 0.5, 0.5));
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.1, 0.1));
            }
            Interaction::None => {
                *border_color = BorderColor::all(Color::srgb(0.6, 0.2, 0.2));
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.1, 0.1));
            }
        }
    }
}
