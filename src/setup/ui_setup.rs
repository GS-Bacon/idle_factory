//! UI setup systems
//!
//! Creates all UI panels (hotbar, machine UIs, inventory, quests, etc.)

use crate::components::*;
use bevy::prelude::*;

/// Helper to spawn a machine UI slot (fuel/input/output)
pub fn spawn_machine_slot(
    parent: &mut ChildBuilder,
    slot_type: MachineSlotType,
    label: &str,
    color: Color,
) {
    parent
        .spawn((
            Button,
            MachineSlotButton(slot_type),
            Node {
                width: Val::Px(60.0),
                height: Val::Px(60.0),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
        ))
        .with_children(|slot| {
            // Label
            slot.spawn((
                Text::new(label),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
            ));
            // Count
            slot.spawn((
                MachineSlotCount(slot_type),
                Text::new("0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Helper to spawn a crusher UI slot (input/output only, no fuel)
pub fn spawn_crusher_slot(
    parent: &mut ChildBuilder,
    slot_type: MachineSlotType,
    label: &str,
    color: Color,
) {
    parent
        .spawn((
            Button,
            CrusherSlotButton(slot_type),
            Node {
                width: Val::Px(55.0),
                height: Val::Px(55.0),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
        ))
        .with_children(|slot| {
            // Label
            slot.spawn((
                Text::new(label),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
            ));
            // Count
            slot.spawn((
                CrusherSlotCount(slot_type),
                Text::new("0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Helper to spawn an inventory slot button
pub fn spawn_inventory_slot(parent: &mut ChildBuilder, slot_idx: usize) {
    parent
        .spawn((
            Button,
            InventorySlotUI(slot_idx),
            Node {
                width: Val::Px(36.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
        ))
        .with_children(|btn| {
            // Slot number (small, top-left)
            btn.spawn((
                Text::new(""),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn setup_ui(mut commands: Commands) {
    // Hotbar UI - centered at bottom
    commands
        .spawn((
            HotbarUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-225.0), // Center 9 slots (9 * 50 = 450, half = 225)
                    ..default()
                },
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Create 9 hotbar slots
            for i in 0..9 {
                parent
                    .spawn((
                        HotbarSlot(i),
                        Node {
                            width: Val::Px(46.0),
                            height: Val::Px(46.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                        BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                    ))
                    .with_children(|slot| {
                        // Slot number
                        slot.spawn((
                            Text::new(format!("{}", i + 1)),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(2.0),
                                left: Val::Px(4.0),
                                ..default()
                            },
                        ));
                        // Item count
                        slot.spawn((
                            HotbarSlotCount(i),
                            Text::new(""),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });

    // Hotbar item name display (above hotbar)
    commands.spawn((
        HotbarItemNameText,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(75.0),
            left: Val::Percent(50.0),
            ..default()
        },
    ));

    // Crosshair
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(4.0),
            height: Val::Px(4.0),
            margin: UiRect {
                left: Val::Px(-2.0),
                top: Val::Px(-2.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ));

    // Furnace UI panel (hidden by default)
    setup_furnace_ui(&mut commands);

    // Crusher UI panel (hidden by default)
    setup_crusher_ui(&mut commands);

    // Miner UI panel (hidden by default)
    setup_miner_ui(&mut commands);

    // Inventory UI panel (hidden by default)
    setup_inventory_ui(&mut commands);

    // Inventory tooltip (hidden by default, shown on hover)
    commands.spawn((
        InventoryTooltip,
        Text::new(""),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        Visibility::Hidden,
    ));

    // Held item display (follows cursor when dragging)
    commands.spawn((
        HeldItemDisplay,
        Text::new(""),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        },
        Visibility::Hidden,
    ));
    commands.spawn((
        HeldItemText,
        Text::new(""),
        TextFont {
            font_size: 10.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(0.0),
            ..default()
        },
        Visibility::Hidden,
    ));

    // Quest UI panel (top-right)
    commands
        .spawn((
            QuestUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Quests"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
            ));
            parent.spawn((
                QuestUIText,
                Text::new("Loading..."),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    // Command input UI (hidden by default)
    commands
        .spawn((
            CommandInputUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(80.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                min_width: Val::Px(300.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                CommandInputText,
                Text::new("> "),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    // Tutorial popup (shown at game start, dismiss on any input)
    commands
        .spawn((
            TutorialPopup,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-200.0),
                    ..default()
                },
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Welcome to Idle Factory!"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
            ));
            parent.spawn((
                Text::new(
                    "Controls:\n\
                     WASD - Move\n\
                     Mouse - Look around\n\
                     Left Click - Break blocks\n\
                     Right Click - Place/Interact\n\
                     E - Open inventory\n\
                     1-9 / Scroll - Select hotbar\n\
                     R - Rotate conveyor\n\
                     T or / - Open command input\n\
                     F3 - Debug info\n\
                     ESC - Pause\n\n\
                     Press any key to start...",
                ),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn setup_furnace_ui(commands: &mut Commands) {
    commands
        .spawn((
            FurnaceUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(15.0)),
                margin: UiRect {
                    left: Val::Px(-175.0),
                    ..default()
                },
                width: Val::Px(350.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Furnace"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Keep FurnaceUIText for backwards compatibility (hidden, used for state)
            parent.spawn((
                FurnaceUIText,
                Text::new(""),
                TextFont {
                    font_size: 1.0,
                    ..default()
                },
                TextColor(Color::NONE),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));

            // Main slot layout: [Input] -> [Progress] -> [Output]
            //                      [Fuel]
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    ..default()
                },))
                .with_children(|layout| {
                    // Top row: Input -> Arrow -> Output
                    layout
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(15.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|row| {
                            // Input slot (Iron Ore / Copper Ore)
                            spawn_machine_slot(
                                row,
                                MachineSlotType::Input,
                                "Ore",
                                Color::srgb(0.6, 0.5, 0.4),
                            );

                            // Progress bar container
                            row.spawn((
                                Node {
                                    width: Val::Px(60.0),
                                    height: Val::Px(20.0),
                                    flex_direction: FlexDirection::Row,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                            ))
                            .with_children(|bar_container| {
                                // Progress fill
                                bar_container.spawn((
                                    MachineProgressBar,
                                    Node {
                                        width: Val::Percent(0.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(1.0, 0.5, 0.0)),
                                ));
                            });

                            // Output slot (Iron Ingot / Copper Ingot)
                            spawn_machine_slot(
                                row,
                                MachineSlotType::Output,
                                "Ingot",
                                Color::srgb(0.8, 0.8, 0.85),
                            );
                        });

                    // Bottom row: Fuel slot (centered)
                    spawn_machine_slot(
                        layout,
                        MachineSlotType::Fuel,
                        "Coal",
                        Color::srgb(0.15, 0.15, 0.15),
                    );
                });

            // Instructions
            parent.spawn((
                Text::new("Click slots to add/remove items\nE or ESC to close"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
            ));
        });
}

fn setup_crusher_ui(commands: &mut Commands) {
    commands
        .spawn((
            CrusherUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(15.0)),
                margin: UiRect {
                    left: Val::Px(-150.0),
                    ..default()
                },
                width: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.15, 0.1, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Crusher"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Slot layout: [Input] -> [Progress] -> [Output]
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|row| {
                    // Input slot
                    spawn_crusher_slot(
                        row,
                        MachineSlotType::Input,
                        "Ore",
                        Color::srgb(0.5, 0.4, 0.35),
                    );

                    // Progress bar
                    row.spawn((
                        Node {
                            width: Val::Px(50.0),
                            height: Val::Px(16.0),
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            CrusherProgressBar,
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.8, 0.4, 0.0)),
                        ));
                    });

                    // Output slot
                    spawn_crusher_slot(
                        row,
                        MachineSlotType::Output,
                        "2x Ore",
                        Color::srgb(0.6, 0.5, 0.45),
                    );
                });

            // Instructions
            parent.spawn((
                Text::new("Doubles ore output!\nE or ESC to close"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
            ));
        });
}

fn setup_miner_ui(commands: &mut Commands) {
    commands
        .spawn((
            MinerUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(15.0)),
                margin: UiRect {
                    left: Val::Px(-125.0),
                    ..default()
                },
                width: Val::Px(250.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.2, 0.15, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Miner"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Buffer display and take button
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|row| {
                    // Buffer contents display
                    row.spawn((
                        Button,
                        MinerBufferButton,
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(60.0),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.4, 0.35)),
                        BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Buffer"),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                        ));
                        btn.spawn((
                            MinerBufferCountText,
                            Text::new("Empty"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Clear button
                    row.spawn((
                        Button,
                        MinerClearButton,
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(30.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                        BorderColor(Color::srgba(0.8, 0.3, 0.3, 1.0)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Clear"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                });

            // Instructions
            parent.spawn((
                Text::new("Click buffer to take items\nE or ESC to close"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
            ));
        });
}

fn setup_inventory_ui(commands: &mut Commands) {
    commands
        .spawn((
            InventoryUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(20.0),
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(15.0)),
                margin: UiRect {
                    left: Val::Px(-250.0),
                    ..default()
                },
                width: Val::Px(500.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Left side: Player inventory
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },))
                .with_children(|left| {
                    // Title
                    left.spawn((
                        Text::new("Inventory"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Main inventory grid (slots 9-35, 3 rows of 9)
                    left.spawn((Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },))
                    .with_children(|grid| {
                        for row in 0..3 {
                            grid.spawn((Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(2.0),
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

                    // Hotbar slots in inventory (slots 0-8)
                    left.spawn((
                        Text::new("Hotbar"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    ));

                    left.spawn((Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(2.0),
                        ..default()
                    },))
                    .with_children(|hotbar_row| {
                        for slot_idx in 0..9 {
                            spawn_inventory_slot(hotbar_row, slot_idx);
                        }
                    });

                    // Trash slot
                    left.spawn((Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },))
                    .with_children(|trash_row| {
                        trash_row.spawn((
                            Text::new("Trash:"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.8, 0.5, 0.5, 1.0)),
                        ));
                        trash_row
                            .spawn((
                                Button,
                                TrashSlot,
                                Node {
                                    width: Val::Px(36.0),
                                    height: Val::Px(36.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.4, 0.2, 0.2, 0.9)),
                                BorderColor(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("🗑"),
                                    TextFont {
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    });
                });

            // Right side: Creative catalog (visible only in creative mode)
            parent
                .spawn((
                    CreativePanel,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                ))
                .with_children(|catalog| {
                    catalog.spawn((
                        Text::new("Creative Catalog"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 1.0, 0.5)),
                    ));

                    // Group items by category
                    let mut current_category = "";
                    for (block_type, category) in crate::components::CREATIVE_ITEMS.iter() {
                        if *category != current_category {
                            current_category = category;
                            catalog.spawn((
                                Text::new(*category),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                            ));
                        }

                        catalog
                            .spawn((
                                Button,
                                CreativeItemButton(*block_type),
                                Node {
                                    padding: UiRect::all(Val::Px(5.0)),
                                    margin: UiRect::left(Val::Px(10.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.2, 0.3, 0.2, 0.9)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(block_type.name()),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });
        });
}
