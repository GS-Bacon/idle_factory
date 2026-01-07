//! Global Inventory (Storage) UI
//!
//! Displays the global inventory in a grid layout with pagination.
//! - 8 columns, 4 rows per page (32 slots visible)
//! - Tab key to toggle
//! - Page navigation buttons

use bevy::prelude::*;

use crate::components::{
    GlobalInventoryCategory, GlobalInventoryCategoryTab, GlobalInventoryOpen, GlobalInventoryPage,
    GlobalInventoryPageButton, GlobalInventoryPageText, GlobalInventorySearch,
    GlobalInventorySearchInput, GlobalInventorySlot, GlobalInventorySlotCount,
    GlobalInventorySlotImage, GlobalInventoryUI, ItemCategory, ItemSprites,
};
use crate::constants::ui_colors;
use crate::core::ItemId;
use crate::player::GlobalInventory;
use crate::systems::cursor;
use crate::BlockType;

/// Slots per page (8 columns x 4 rows)
pub const SLOTS_PER_PAGE: usize = 32;
/// Grid columns
pub const GRID_COLUMNS: usize = 8;
/// Slot size in pixels
pub const SLOT_SIZE: f32 = 54.0;
/// Slot spacing
pub const SLOT_GAP: f32 = 4.0;

/// Update global inventory UI visibility when GlobalInventoryOpen changes
/// (Key handling moved to ui_navigation.rs)
pub fn update_global_inventory_visibility(
    global_inv_open: Res<GlobalInventoryOpen>,
    mut ui_query: Query<&mut Visibility, With<GlobalInventoryUI>>,
    mut windows: Query<&mut Window>,
) {
    // Only update when GlobalInventoryOpen changes
    if !global_inv_open.is_changed() {
        return;
    }

    // Update UI visibility
    for mut vis in ui_query.iter_mut() {
        *vis = if global_inv_open.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Handle cursor lock
    if let Ok(mut window) = windows.get_single_mut() {
        if global_inv_open.0 {
            cursor::unlock_cursor(&mut window);
        } else {
            cursor::lock_cursor(&mut window);
        }
    }
}

/// Handle page navigation button clicks
pub fn global_inventory_page_nav(
    mut page: ResMut<GlobalInventoryPage>,
    global_inventory: Res<GlobalInventory>,
    mut button_query: Query<
        (
            &Interaction,
            &GlobalInventoryPageButton,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
) {
    let items = global_inventory.get_all_items_by_id();
    let total_pages = items.len().max(1).div_ceil(SLOTS_PER_PAGE);

    for (interaction, page_button, mut bg_color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if page_button.next {
                    if page.0 + 1 < total_pages {
                        page.0 += 1;
                    }
                } else if page.0 > 0 {
                    page.0 -= 1;
                }
                *bg_color = BackgroundColor(ui_colors::TAB_SELECTED);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(ui_colors::BTN_HOVER);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(ui_colors::BTN_BG);
            }
        }
    }
}

/// Update the global inventory UI display
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_global_inventory_ui(
    global_inv_open: Res<GlobalInventoryOpen>,
    global_inventory: Res<GlobalInventory>,
    page: Res<GlobalInventoryPage>,
    category: Res<GlobalInventoryCategory>,
    search: Res<GlobalInventorySearch>,
    item_sprites: Res<ItemSprites>,
    mut slot_query: Query<
        (&GlobalInventorySlot, &mut BackgroundColor),
        Without<GlobalInventorySlotImage>,
    >,
    mut slot_image_query: Query<(&GlobalInventorySlotImage, &mut Visibility, &mut ImageNode)>,
    mut slot_count_query: Query<(&GlobalInventorySlotCount, &mut Text)>,
    mut page_text_query: Query<
        &mut Text,
        (
            With<GlobalInventoryPageText>,
            Without<GlobalInventorySlotCount>,
        ),
    >,
) {
    // Hide all slot images when UI is closed
    if !global_inv_open.0 {
        for (_slot_image, mut vis, _) in slot_image_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    // Filter items by category and search
    let search_lower = search.0.to_lowercase();
    let items: Vec<(ItemId, u32)> = global_inventory
        .get_all_items_by_id()
        .into_iter()
        .filter(|(item_id, _)| {
            // Convert to BlockType for category matching (returns false for mod items)
            let block_type: Option<BlockType> = (*item_id).try_into().ok();

            // Filter by category
            if let Some(bt) = block_type {
                if !category.0.matches(bt) {
                    return false;
                }
            } else if category.0 != ItemCategory::All {
                // Mod items only show in "All" category
                return false;
            }

            // Filter by search text
            if !search_lower.is_empty() {
                let name = item_id.name().unwrap_or("").to_lowercase();
                if !name.contains(&search_lower) {
                    return false;
                }
            }
            true
        })
        .collect();

    let total_pages = items.len().max(1).div_ceil(SLOTS_PER_PAGE);
    let start_idx = page.0 * SLOTS_PER_PAGE;

    // Update page text
    for mut text in page_text_query.iter_mut() {
        **text = format!("{}/{}", page.0 + 1, total_pages.max(1));
    }

    // Update slots
    for (slot, mut bg_color) in slot_query.iter_mut() {
        let item_idx = start_idx + slot.0;
        if item_idx < items.len() {
            *bg_color = BackgroundColor(ui_colors::SLOT_FILLED);
        } else {
            *bg_color = BackgroundColor(ui_colors::SLOT_EMPTY);
        }
    }

    // Update slot images and counts
    for (slot_image, mut vis, mut image_node) in slot_image_query.iter_mut() {
        let item_idx = start_idx + slot_image.0;
        if item_idx < items.len() {
            let (item_id, _) = items[item_idx];
            if let Some(sprite_handle) = item_sprites.get_id(item_id) {
                image_node.image = sprite_handle;
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        } else {
            *vis = Visibility::Hidden;
        }
    }

    for (slot_count, mut text) in slot_count_query.iter_mut() {
        let item_idx = start_idx + slot_count.0;
        if item_idx < items.len() {
            let (_, count) = items[item_idx];
            **text = if count > 1 {
                format!("{}", count)
            } else {
                String::new()
            };
        } else {
            **text = String::new();
        }
    }
}

/// Handle category tab clicks
pub fn global_inventory_category_click(
    mut category: ResMut<GlobalInventoryCategory>,
    mut page: ResMut<GlobalInventoryPage>,
    mut button_query: Query<
        (
            &Interaction,
            &GlobalInventoryCategoryTab,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, tab, mut bg_color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                category.0 = tab.0;
                page.0 = 0; // Reset to first page when category changes
                *bg_color = BackgroundColor(ui_colors::TAB_SELECTED);
            }
            Interaction::Hovered => {
                if category.0 != tab.0 {
                    *bg_color = BackgroundColor(ui_colors::TAB_HOVER);
                }
            }
            Interaction::None => {
                if category.0 == tab.0 {
                    *bg_color = BackgroundColor(ui_colors::TAB_SELECTED);
                } else {
                    *bg_color = BackgroundColor(ui_colors::TAB_UNSELECTED);
                }
            }
        }
    }
}

/// Handle search input (simple keyboard-based input)
pub fn global_inventory_search_input(
    global_inv_open: Res<GlobalInventoryOpen>,
    mut search: ResMut<GlobalInventorySearch>,
    mut page: ResMut<GlobalInventoryPage>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut search_text_query: Query<&mut Text, With<GlobalInventorySearchInput>>,
) {
    if !global_inv_open.0 {
        return;
    }

    let mut changed = false;

    // Handle backspace
    if key_input.just_pressed(KeyCode::Backspace) {
        search.0.pop();
        changed = true;
    }

    // Handle letter keys A-Z
    let letter_keys = [
        (KeyCode::KeyA, 'a'),
        (KeyCode::KeyB, 'b'),
        (KeyCode::KeyC, 'c'),
        (KeyCode::KeyD, 'd'),
        (KeyCode::KeyE, 'e'),
        (KeyCode::KeyF, 'f'),
        (KeyCode::KeyG, 'g'),
        (KeyCode::KeyH, 'h'),
        (KeyCode::KeyI, 'i'),
        (KeyCode::KeyJ, 'j'),
        (KeyCode::KeyK, 'k'),
        (KeyCode::KeyL, 'l'),
        (KeyCode::KeyM, 'm'),
        (KeyCode::KeyN, 'n'),
        (KeyCode::KeyO, 'o'),
        (KeyCode::KeyP, 'p'),
        (KeyCode::KeyQ, 'q'),
        (KeyCode::KeyR, 'r'),
        (KeyCode::KeyS, 's'),
        (KeyCode::KeyT, 't'),
        (KeyCode::KeyU, 'u'),
        (KeyCode::KeyV, 'v'),
        (KeyCode::KeyW, 'w'),
        (KeyCode::KeyX, 'x'),
        (KeyCode::KeyY, 'y'),
        (KeyCode::KeyZ, 'z'),
    ];

    for (key, c) in letter_keys {
        if key_input.just_pressed(key) {
            search.0.push(c);
            changed = true;
        }
    }

    // Handle space
    if key_input.just_pressed(KeyCode::Space) {
        search.0.push(' ');
        changed = true;
    }

    if changed {
        page.0 = 0; // Reset to first page when search changes
        for mut text in search_text_query.iter_mut() {
            if search.0.is_empty() {
                **text = "Search...".to_string();
            } else {
                **text = search.0.clone();
            }
        }
    }
}

/// Setup the global inventory UI (call in Startup)
pub fn setup_global_inventory_ui(mut commands: Commands) {
    let panel_width = (SLOT_SIZE + SLOT_GAP) * GRID_COLUMNS as f32 + 40.0;

    // Main panel (hidden by default)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(panel_width),
                height: Val::Px((SLOT_SIZE + SLOT_GAP) * 4.0 + 140.0), // Extra height for tabs/search
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(ui_colors::PANEL_BG),
            BorderColor(ui_colors::BORDER_HIGHLIGHT),
            GlobalInventoryUI,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Storage"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.95)), // Slightly off-white
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Category tabs row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|tabs| {
                    for cat in [
                        ItemCategory::All,
                        ItemCategory::Ores,
                        ItemCategory::Ingots,
                        ItemCategory::Machines,
                    ] {
                        let is_default = cat == ItemCategory::All;
                        tabs.spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(if is_default {
                                ui_colors::TAB_SELECTED
                            } else {
                                ui_colors::TAB_UNSELECTED
                            }),
                            BorderColor(if is_default {
                                ui_colors::BORDER_HIGHLIGHT
                            } else {
                                ui_colors::BORDER_SHADOW
                            }),
                            GlobalInventoryCategoryTab(cat),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(cat.label()),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.85, 0.85, 0.9)),
                            ));
                        });
                    }
                });

            // Search input
            parent
                .spawn((
                    Node {
                        width: Val::Px(panel_width - 40.0),
                        height: Val::Px(28.0),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        margin: UiRect::bottom(Val::Px(8.0)),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(ui_colors::INPUT_BG),
                    BorderColor(ui_colors::BORDER_SHADOW),
                ))
                .with_children(|search_box| {
                    search_box.spawn((
                        Text::new("Search..."),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                        GlobalInventorySearchInput,
                    ));
                });

            // Grid container
            parent
                .spawn((Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::px(GRID_COLUMNS as u16, SLOT_SIZE),
                    grid_template_rows: RepeatedGridTrack::px(4, SLOT_SIZE),
                    row_gap: Val::Px(SLOT_GAP),
                    column_gap: Val::Px(SLOT_GAP),
                    ..default()
                },))
                .with_children(|grid| {
                    // Create 32 slots
                    for i in 0..SLOTS_PER_PAGE {
                        grid.spawn((
                            Node {
                                width: Val::Px(SLOT_SIZE),
                                height: Val::Px(SLOT_SIZE),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(ui_colors::SLOT_EMPTY),
                            BorderColor(ui_colors::BORDER_HIGHLIGHT),
                            GlobalInventorySlot(i),
                        ))
                        .with_children(|slot| {
                            // Item image (hidden by default)
                            slot.spawn((
                                ImageNode::default(),
                                Node {
                                    width: Val::Px(40.0),
                                    height: Val::Px(40.0),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                GlobalInventorySlotImage(i),
                                Visibility::Hidden,
                            ));

                            // Count text
                            slot.spawn((
                                Text::new(""),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Node {
                                    position_type: PositionType::Absolute,
                                    right: Val::Px(2.0),
                                    bottom: Val::Px(2.0),
                                    ..default()
                                },
                                GlobalInventorySlotCount(i),
                            ));
                        });
                    }
                });

            // Page navigation
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(10.0)),
                    column_gap: Val::Px(10.0),
                    ..default()
                },))
                .with_children(|nav| {
                    // Previous button
                    nav.spawn((
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(ui_colors::BTN_BG),
                        BorderColor(ui_colors::BORDER_HIGHLIGHT),
                        GlobalInventoryPageButton { next: false },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("<"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Page indicator
                    nav.spawn((
                        Text::new("1/1"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        GlobalInventoryPageText,
                    ));

                    // Next button
                    nav.spawn((
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(ui_colors::BTN_BG),
                        BorderColor(ui_colors::BORDER_HIGHLIGHT),
                        GlobalInventoryPageButton { next: true },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(">"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                });
        });
}
