//! Machine UI setup (Furnace, Crusher, Miner)
//!
//! Follows design rules from .specify/memory/ui-design-rules.md

use crate::components::*;
use bevy::prelude::*;

// === Design Rule Constants ===
const SLOT_SIZE: f32 = 54.0;
const PANEL_PADDING: f32 = 20.0;
const HEADER_HEIGHT: f32 = 25.0;

// Colors from design rules
const PANEL_BG: Color = Color::srgba(0.12, 0.12, 0.12, 0.9);
const SLOT_BG: Color = Color::srgb(0.2, 0.2, 0.2);
const SLOT_BORDER: Color = Color::srgb(0.33, 0.33, 0.33);
const PROGRESS_BG: Color = Color::srgb(0.13, 0.13, 0.13);
const PROGRESS_FILL: Color = Color::srgb(0.30, 0.69, 0.31);
const TEXT_PRIMARY: Color = Color::WHITE;
const TEXT_SECONDARY: Color = Color::srgb(0.67, 0.67, 0.67);

/// Spawn furnace slot with proper component
fn spawn_furnace_slot(parent: &mut ChildBuilder, slot_type: MachineSlotType) {
    parent
        .spawn((
            Button,
            MachineSlotButton(slot_type),
            Node {
                width: Val::Px(SLOT_SIZE),
                height: Val::Px(SLOT_SIZE),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(SLOT_BG),
            BorderColor(SLOT_BORDER),
        ))
        .with_children(|slot| {
            slot.spawn((
                MachineSlotCount(slot_type),
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));
        });
}

/// Spawn crusher slot with proper component
fn spawn_crusher_slot_inner(parent: &mut ChildBuilder, slot_type: MachineSlotType) {
    parent
        .spawn((
            Button,
            CrusherSlotButton(slot_type),
            Node {
                width: Val::Px(SLOT_SIZE),
                height: Val::Px(SLOT_SIZE),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(SLOT_BG),
            BorderColor(SLOT_BORDER),
        ))
        .with_children(|slot| {
            slot.spawn((
                CrusherSlotCount(slot_type),
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));
        });
}

pub fn setup_furnace_ui(commands: &mut Commands) {
    commands
        .spawn((
            FurnaceUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(25.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-150.0),
                    ..default()
                },
                width: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            BorderColor(SLOT_BORDER),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            // === Header ===
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(HEADER_HEIGHT),
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(SLOT_BORDER),
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("Furnace"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(TEXT_PRIMARY),
                    ));
                    // Close button placeholder (X)
                    header.spawn((
                        Text::new("×"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));
                });

            // Hidden state text for backwards compatibility
            panel.spawn((
                FurnaceUIText,
                Text::new(""),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));

            // === Content area ===
            panel
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(PANEL_PADDING)),
                    row_gap: Val::Px(12.0),
                    ..default()
                },))
                .with_children(|content| {
                    // Fuel slot row
                    content
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            ..default()
                        },))
                        .with_children(|row| {
                            spawn_furnace_slot(row, MachineSlotType::Fuel);
                            row.spawn((
                                Text::new("Fuel"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(TEXT_SECONDARY),
                            ));
                        });

                    // Fire icon (shown when working)
                    content.spawn((
                        Text::new("🔥"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 0.5, 0.0, 0.8)),
                    ));

                    // Input -> Output row
                    content
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            ..default()
                        },))
                        .with_children(|row| {
                            spawn_furnace_slot(row, MachineSlotType::Input);
                            row.spawn((
                                Text::new("→"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_SECONDARY),
                            ));
                            spawn_furnace_slot(row, MachineSlotType::Output);
                        });

                    // Progress bar
                    content
                        .spawn((
                            Node {
                                width: Val::Px(SLOT_SIZE * 2.0 + 12.0),
                                height: Val::Px(8.0),
                                ..default()
                            },
                            BackgroundColor(PROGRESS_BG),
                        ))
                        .with_children(|bar| {
                            bar.spawn((
                                MachineProgressBar,
                                Node {
                                    width: Val::Percent(0.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(PROGRESS_FILL),
                            ));
                        });

                    // Instructions
                    content.spawn((
                        Text::new("E or ESC to close"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));
                });
        });
}

pub fn setup_crusher_ui(commands: &mut Commands) {
    commands
        .spawn((
            CrusherUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(25.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-140.0),
                    ..default()
                },
                width: Val::Px(280.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            BorderColor(SLOT_BORDER),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            // === Header ===
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(HEADER_HEIGHT),
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(SLOT_BORDER),
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("Crusher"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(TEXT_PRIMARY),
                    ));
                    header.spawn((
                        Text::new("×"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));
                });

            // === Content ===
            panel
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(PANEL_PADDING)),
                    row_gap: Val::Px(12.0),
                    ..default()
                },))
                .with_children(|content| {
                    // Input -> Output row
                    content
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            ..default()
                        },))
                        .with_children(|row| {
                            spawn_crusher_slot_inner(row, MachineSlotType::Input);
                            row.spawn((
                                Text::new("→"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_SECONDARY),
                            ));
                            spawn_crusher_slot_inner(row, MachineSlotType::Output);
                        });

                    // Progress bar
                    content
                        .spawn((
                            Node {
                                width: Val::Px(SLOT_SIZE * 2.0 + 12.0),
                                height: Val::Px(8.0),
                                ..default()
                            },
                            BackgroundColor(PROGRESS_BG),
                        ))
                        .with_children(|bar| {
                            bar.spawn((
                                CrusherProgressBar,
                                Node {
                                    width: Val::Percent(0.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(PROGRESS_FILL),
                            ));
                        });

                    // Info
                    content.spawn((
                        Text::new("Doubles ore output!"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));

                    content.spawn((
                        Text::new("E or ESC to close"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));
                });
        });
}

pub fn setup_miner_ui(commands: &mut Commands) {
    commands
        .spawn((
            MinerUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(25.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-125.0),
                    ..default()
                },
                width: Val::Px(250.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            BorderColor(SLOT_BORDER),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            // === Header ===
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(HEADER_HEIGHT),
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(SLOT_BORDER),
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("Miner"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(TEXT_PRIMARY),
                    ));
                    header.spawn((
                        Text::new("×"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));
                });

            // === Content ===
            panel
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(PANEL_PADDING)),
                    row_gap: Val::Px(12.0),
                    ..default()
                },))
                .with_children(|content| {
                    // Buffer slot
                    content
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        },))
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                MinerBufferButton,
                                Node {
                                    width: Val::Px(SLOT_SIZE),
                                    height: Val::Px(SLOT_SIZE),
                                    border: UiRect::all(Val::Px(1.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(SLOT_BG),
                                BorderColor(SLOT_BORDER),
                            ))
                            .with_children(|slot| {
                                slot.spawn((
                                    MinerBufferCountText,
                                    Text::new(""),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(TEXT_PRIMARY),
                                ));
                            });

                            // Clear button
                            row.spawn((
                                Button,
                                MinerClearButton,
                                Node {
                                    width: Val::Px(60.0),
                                    height: Val::Px(30.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                                BorderColor(Color::srgb(0.7, 0.3, 0.3)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Clear"),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(TEXT_PRIMARY),
                                ));
                            });
                        });

                    // Instructions
                    content.spawn((
                        Text::new("Click buffer to take items"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));

                    content.spawn((
                        Text::new("E or ESC to close"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));
                });
        });
}
