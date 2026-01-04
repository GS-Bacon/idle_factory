//! Inventory UI setup

use crate::components::*;
use bevy::prelude::*;

use super::{
    spawn_inventory_slot, text_font, QUEST_BG, QUEST_BORDER_COLOR, QUEST_RADIUS, SLOT_BG,
    SLOT_BORDER, SLOT_BORDER_COLOR, SLOT_GAP, SLOT_RADIUS, SLOT_SIZE, SPRITE_SIZE,
};

/// Calculate inventory UI width based on slot size
fn inventory_ui_width() -> f32 {
    // 9 slots + 8 gaps + padding
    SLOT_SIZE * 9.0 + SLOT_GAP * 8.0 + 16.0
}

pub fn setup_inventory_ui(commands: &mut Commands, font: &Handle<Font>) {
    let ui_width = inventory_ui_width();

    // Background overlay (darkens the screen when inventory is open)
    commands.spawn((
        InventoryBackgroundOverlay,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)), // 50% black overlay
        GlobalZIndex(40),                                  // Below inventory UI
        Visibility::Hidden,
    ));

    commands
        .spawn((
            InventoryUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(15.0),
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(12.0)),
                margin: UiRect {
                    left: Val::Px(-ui_width / 2.0),
                    ..default()
                },
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(QUEST_BG),
            BorderColor(QUEST_BORDER_COLOR),
            BorderRadius::all(Val::Px(QUEST_RADIUS)),
            GlobalZIndex(50), // Above overlay, below held item
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // === Creative catalog (top, only visible in creative mode) ===
            parent
                .spawn((
                    CreativePanel,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        padding: UiRect::all(Val::Px(4.0)),
                        margin: UiRect::bottom(Val::Px(8.0)),
                        max_height: Val::Px(200.0), // Limit height for scrolling
                        overflow: Overflow::clip_y(), // Enable vertical scroll
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.08, 0.9)),
                    BorderRadius::all(Val::Px(SLOT_RADIUS)),
                    Visibility::Hidden, // Start hidden, shown when inventory open + creative mode
                ))
                .with_children(|catalog| {
                    // Creative catalog grid (5x9)
                    let items: Vec<_> = crate::components::CREATIVE_ITEMS.iter().collect();
                    for row_items in items.chunks(9) {
                        catalog
                            .spawn((Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(SLOT_GAP),
                                ..default()
                            },))
                            .with_children(|row| {
                                for (block_type, _category) in row_items {
                                    row.spawn((
                                        Button,
                                        CreativeItemButton(*block_type),
                                        Node {
                                            width: Val::Px(SLOT_SIZE),
                                            height: Val::Px(SLOT_SIZE),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(SLOT_BORDER)),
                                            ..default()
                                        },
                                        // Factory theme colors
                                        BackgroundColor(SLOT_BG),
                                        BorderColor(SLOT_BORDER_COLOR),
                                        BorderRadius::all(Val::Px(SLOT_RADIUS)),
                                    ))
                                    .with_children(|btn| {
                                        // Sprite image
                                        btn.spawn((
                                            CreativeItemImage(*block_type),
                                            ImageNode::default(),
                                            Visibility::Hidden,
                                            Node {
                                                width: Val::Px(SPRITE_SIZE),
                                                height: Val::Px(SPRITE_SIZE),
                                                position_type: PositionType::Absolute,
                                                ..default()
                                            },
                                        ));
                                        // Text fallback
                                        btn.spawn((
                                            Text::new(block_type.short_name()),
                                            text_font(font, 9.0),
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                                }
                            });
                    }
                });

            // === Main inventory grid (3x9, slots 9-35) ===
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(SLOT_GAP),
                    ..default()
                },))
                .with_children(|grid| {
                    for row in 0..3 {
                        grid.spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(SLOT_GAP),
                            ..default()
                        },))
                            .with_children(|row_node| {
                                for col in 0..9 {
                                    let slot_idx = 9 + row * 9 + col;
                                    spawn_inventory_slot(row_node, slot_idx, font);
                                }
                            });
                    }
                });

            // Separator line (Factory orange tint)
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 0.53, 0.0, 0.4)), // Orange tint
            ));

            // === Hotbar slots (1x9, slots 0-8) ===
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(SLOT_GAP),
                    ..default()
                },))
                .with_children(|hotbar_row| {
                    for slot_idx in 0..9 {
                        spawn_inventory_slot(hotbar_row, slot_idx, font);
                    }
                });

            // Separator line (Factory orange tint)
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 0.53, 0.0, 0.4)), // Orange tint
            ));

            // === Bottom row: Trash slot ===
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexEnd,
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },))
                .with_children(|bottom_row| {
                    bottom_row
                        .spawn((
                            Button,
                            TrashSlot,
                            Node {
                                width: Val::Px(SLOT_SIZE),
                                height: Val::Px(SLOT_SIZE),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(SLOT_BORDER)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.4, 0.15, 0.1, 0.95)),
                            BorderColor(Color::srgb(0.8, 0.3, 0.2)), // Red-orange
                            BorderRadius::all(Val::Px(SLOT_RADIUS)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("X"),
                                text_font(font, 16.0),
                                TextColor(Color::srgb(1.0, 0.5, 0.4)),
                            ));
                        });
                });
        });
}
