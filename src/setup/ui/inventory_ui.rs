//! Inventory UI setup

use crate::components::*;
use crate::BlockType;
use bevy::prelude::*;

use super::{spawn_inventory_slot, SLOT_BORDER, SLOT_GAP, SLOT_SIZE, SPRITE_SIZE};

/// Machine types to display in GlobalInventory panel
const GLOBAL_INVENTORY_ITEMS: &[BlockType] = &[
    BlockType::MinerBlock,
    BlockType::ConveyorBlock,
    BlockType::CrusherBlock,
    BlockType::FurnaceBlock,
];

/// Calculate inventory UI width based on slot size
fn inventory_ui_width() -> f32 {
    // 9 slots + 8 gaps + padding
    SLOT_SIZE * 9.0 + SLOT_GAP * 8.0 + 16.0
}

pub fn setup_inventory_ui(commands: &mut Commands) {
    let ui_width = inventory_ui_width();

    commands
        .spawn((
            InventoryUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(15.0),
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(8.0)),
                margin: UiRect {
                    left: Val::Px(-ui_width / 2.0),
                    ..default()
                },
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.12, 0.12, 0.96)), // MC-style dark gray
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
                                        BackgroundColor(Color::srgba(0.14, 0.14, 0.14, 0.95)),
                                        BorderColor(Color::srgba(0.3, 0.3, 0.3, 1.0)),
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
                                            TextFont {
                                                font_size: 9.0,
                                                ..default()
                                            },
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
                                spawn_inventory_slot(row_node, slot_idx);
                            }
                        });
                    }
                });

            // Separator line
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8)),
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
                        spawn_inventory_slot(hotbar_row, slot_idx);
                    }
                });

            // Separator line
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8)),
            ));

            // === GlobalInventory panel (machine storage) ===
            // Industrial dark theme (Gemini review suggestion)
            parent
                .spawn((
                    GlobalInventoryPanel,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        padding: UiRect::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.22, 0.25, 0.95)), // Industrial dark gray
                ))
                .with_children(|panel| {
                    // Title with accent color
                    panel.spawn((
                        Text::new("機械ストレージ"),
                        TextFont {
                            font_size: 14.0, // Increased from 12.0
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 0.6, 0.2, 1.0)), // Orange accent
                        Node {
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                    ));

                    // Item rows (2x2 grid)
                    panel
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(12.0),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },))
                        .with_children(|grid| {
                            for &block_type in GLOBAL_INVENTORY_ITEMS {
                                grid.spawn((
                                    GlobalInventoryRow(block_type),
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(6.0),
                                        min_width: Val::Px(110.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|row| {
                                    // Slot-like icon background (darker)
                                    row.spawn((
                                        Node {
                                            width: Val::Px(SLOT_SIZE * 0.65),
                                            height: Val::Px(SLOT_SIZE * 0.65),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(SLOT_BORDER)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.9)), // Dark slot
                                        BorderColor(Color::srgba(0.35, 0.35, 0.4, 1.0)), // Subtle gray border
                                    ))
                                    .with_children(|slot| {
                                        slot.spawn((
                                            Text::new(block_type.short_name()),
                                            TextFont {
                                                font_size: 10.0, // Increased from 8.0
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });

                                    // Item name and count
                                    row.spawn((
                                        GlobalInventoryCountText(block_type),
                                        Text::new(format!("{}: 0", block_type.name())),
                                        TextFont {
                                            font_size: 13.0, // Increased from 11.0
                                            ..default()
                                        },
                                        TextColor(Color::srgba(0.9, 0.9, 0.95, 1.0)), // Bright text
                                    ));
                                });
                            }
                        });
                });

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
                            BackgroundColor(Color::srgba(0.35, 0.15, 0.15, 0.95)),
                            BorderColor(Color::srgba(0.5, 0.25, 0.25, 1.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("X"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(1.0, 0.5, 0.5, 1.0)),
                            ));
                        });
                });
        });
}
