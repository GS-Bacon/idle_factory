//! Hotbar UI systems

use crate::components::*;
use crate::player::Inventory;
use crate::BlockType;
use bevy::prelude::*;

/// Update hotbar UI display
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

/// Select slot with number keys (1-9) or scroll wheel
pub fn select_block_type(
    key_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut inventory: ResMut<Inventory>,
    input_resources: InputStateResourcesWithCursor,
) {
    use crate::HOTBAR_SLOTS;

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
