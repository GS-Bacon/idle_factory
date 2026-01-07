//! Hotbar UI systems

use crate::components::*;
use crate::player::{LocalPlayer, PlayerInventory};
use crate::systems::block_operations::LocalPlayerInventory;
use bevy::prelude::*;

/// Update hotbar UI display
pub fn update_hotbar_ui(
    local_player: Option<Res<LocalPlayer>>,
    inventory_query: Query<&PlayerInventory>,
    item_sprites: Res<ItemSprites>,
    asset_server: Res<AssetServer>,
    mut slot_query: Query<(&HotbarSlot, &mut BackgroundColor, &mut BorderColor)>,
    mut count_query: Query<(&HotbarSlotCount, &mut Text)>,
    mut image_query: Query<(&HotbarSlotImage, &mut ImageNode, &mut Visibility)>,
) {
    // Get local player's inventory
    let Some(local_player) = local_player else {
        return;
    };
    let Ok(inventory) = inventory_query.get(local_player.0) else {
        return;
    };

    // Check if any sprite assets are still loading
    let sprites_loading = item_sprites.textures.values().any(|h| {
        !matches!(
            asset_server.get_load_state(h),
            Some(bevy::asset::LoadState::Loaded)
        )
    });

    // Update when sprites resource changes or sprites are loading
    // (need to keep checking while loading to catch when they finish)
    // Note: is_changed() on Query component requires different approach
    if !item_sprites.is_changed() && !sprites_loading {
        return;
    }

    // Update slot backgrounds - use slot index for selection
    for (slot, mut bg, mut border) in slot_query.iter_mut() {
        let is_selected = inventory.selected_slot == slot.0;
        let has_item = inventory.get_slot_item_id(slot.0).is_some();

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

    // Update slot sprite images with visibility control
    for (slot_image, mut image_node, mut visibility) in image_query.iter_mut() {
        if let Some(item_id) = inventory.get_slot_item_id(slot_image.0) {
            if let Some(sprite_handle) = item_sprites.get_id(item_id) {
                image_node.image = sprite_handle.clone();
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    // Update slot counts (only show number when count > 1)
    for (slot_count, mut text) in count_query.iter_mut() {
        if inventory.get_slot_item_id(slot_count.0).is_some() {
            let count = inventory.get_slot_count(slot_count.0);
            if count > 1 {
                **text = count.to_string();
            } else {
                **text = String::new();
            }
        } else {
            **text = String::new();
        }
    }
}

/// Update the hotbar item name display to show the selected item's name
pub fn update_hotbar_item_name(
    local_player: Option<Res<LocalPlayer>>,
    inventory_query: Query<&PlayerInventory>,
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

    // Get local player's inventory
    let Some(local_player) = local_player else {
        text.0 = String::new();
        return;
    };
    let Ok(inventory) = inventory_query.get(local_player.0) else {
        text.0 = String::new();
        return;
    };

    // Show selected item name
    if let Some(item_id) = inventory.selected_item_id() {
        // Get display name - try to convert to BlockType for name
        let name = if let Ok(block_type) = crate::BlockType::try_from(item_id) {
            block_type.name().to_string()
        } else {
            // Fallback to ItemId string name
            item_id.name().unwrap_or("Unknown").to_string()
        };
        text.0 = name.clone();
        // Center the text by adjusting margin based on text length
        let char_width = 8.0; // Approximate character width
        node.margin.left = Val::Px(-(name.len() as f32 * char_width / 2.0));
    } else {
        text.0 = String::new();
    }
}

/// Select slot with number keys (1-9) or scroll wheel
pub fn select_block_type(
    key_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut local_player_inventory: LocalPlayerInventory,
    input_resources: InputStateResourcesWithCursor,
) {
    use crate::HOTBAR_SLOTS;

    // Use InputState to check if hotbar selection is allowed (see CLAUDE.md input matrix)
    let input_state = input_resources.get_state();
    if !input_state.allows_hotbar() {
        // Still need to drain events to prevent accumulation
        for _ in mouse_wheel.read() {}
        return;
    }

    // Get mutable access to local player's inventory
    let Some(mut inventory) = local_player_inventory.get_mut() else {
        // Still need to drain events to prevent accumulation
        for _ in mouse_wheel.read() {}
        return;
    };

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

/// Update 3D held item display based on selected hotbar item
pub fn update_held_item_3d(
    local_player: Option<Res<LocalPlayer>>,
    inventory_query: Query<&PlayerInventory>,
    cache: Option<Res<HeldItem3DCache>>,
    mut query: Query<(&mut MeshMaterial3d<StandardMaterial>, &mut Visibility), With<HeldItem3D>>,
) {
    let Some(cache) = cache else {
        return;
    };

    let Ok((mut material, mut visibility)) = query.get_single_mut() else {
        return;
    };

    // Get local player's inventory
    let Some(local_player) = local_player else {
        *visibility = Visibility::Hidden;
        return;
    };
    let Ok(inventory) = inventory_query.get(local_player.0) else {
        *visibility = Visibility::Hidden;
        return;
    };

    // Get selected item
    if let Some(item_id) = inventory.selected_item_id() {
        // Convert to BlockType for material lookup
        if let Ok(block_type) = crate::BlockType::try_from(item_id) {
            if let Some(block_material) = cache.materials.get(&block_type) {
                material.0 = block_material.clone();
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    } else {
        // No item selected - hide
        *visibility = Visibility::Hidden;
    }
}
