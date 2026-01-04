//! Machine UI setup (Furnace, Crusher, Miner)
//!
//! Follows design rules from .specify/memory/ui-design-rules.md

use crate::components::*;
use crate::setup::ui::{
    text_font, QUEST_BORDER_COLOR, QUEST_PROGRESS_COLOR, QUEST_RADIUS, SLOT_BG, SLOT_BORDER,
    SLOT_BORDER_COLOR, SLOT_RADIUS, SLOT_SIZE,
};
use bevy::prelude::*;

// === Design Rule Constants ===
const PANEL_PADDING: f32 = 20.0;
const HEADER_HEIGHT: f32 = 30.0;

// Factory theme colors (consistent with other UI)
const PANEL_BG: Color = Color::srgba(0.10, 0.10, 0.10, 0.95);
const PANEL_BORDER: Color = QUEST_BORDER_COLOR;
const PROGRESS_BG: Color = Color::srgb(0.15, 0.15, 0.2);
const PROGRESS_FILL: Color = QUEST_PROGRESS_COLOR; // Orange theme
const TEXT_PRIMARY: Color = Color::WHITE;
const TEXT_SECONDARY: Color = Color::srgb(0.67, 0.67, 0.67);
const HEADER_COLOR: Color = Color::srgb(1.0, 0.8, 0.0); // Yellow header

/// Spawn furnace slot with Factory theme
fn spawn_furnace_slot(parent: &mut ChildBuilder, slot_type: MachineSlotType, font: &Handle<Font>) {
    parent
        .spawn((
            Button,
            MachineSlotButton(slot_type),
            Node {
                width: Val::Px(SLOT_SIZE),
                height: Val::Px(SLOT_SIZE),
                border: UiRect::all(Val::Px(SLOT_BORDER)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(SLOT_BG),
            BorderColor(SLOT_BORDER_COLOR),
            BorderRadius::all(Val::Px(SLOT_RADIUS)),
        ))
        .with_children(|slot| {
            slot.spawn((
                MachineSlotCount(slot_type),
                Text::new(""),
                text_font(font, 14.0),
                TextColor(TEXT_PRIMARY),
            ));
        });
}

/// Spawn crusher slot with Factory theme
fn spawn_crusher_slot_inner(
    parent: &mut ChildBuilder,
    slot_type: MachineSlotType,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            CrusherSlotButton(slot_type),
            Node {
                width: Val::Px(SLOT_SIZE),
                height: Val::Px(SLOT_SIZE),
                border: UiRect::all(Val::Px(SLOT_BORDER)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(SLOT_BG),
            BorderColor(SLOT_BORDER_COLOR),
            BorderRadius::all(Val::Px(SLOT_RADIUS)),
        ))
        .with_children(|slot| {
            slot.spawn((
                CrusherSlotCount(slot_type),
                Text::new(""),
                text_font(font, 14.0),
                TextColor(TEXT_PRIMARY),
            ));
        });
}

pub fn setup_furnace_ui(commands: &mut Commands, font: &Handle<Font>) {
    let font = font.clone();
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
            BorderColor(PANEL_BORDER),
            BorderRadius::all(Val::Px(QUEST_RADIUS)),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            let font = font.clone();
            // === Header (Factory theme) ===
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(HEADER_HEIGHT),
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(PANEL_BORDER),
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("🔥 精錬炉"),
                        text_font(&font, 16.0),
                        TextColor(HEADER_COLOR),
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
            let font_content = font.clone();
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
                    let font_row = font_content.clone();
                    content
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            ..default()
                        },))
                        .with_children(|row| {
                            spawn_furnace_slot(row, MachineSlotType::Fuel, &font_row);
                            row.spawn((
                                Text::new("Fuel"),
                                text_font(&font_row, 12.0),
                                TextColor(TEXT_SECONDARY),
                            ));
                        });

                    // Fire icon (shown when working)
                    content.spawn((
                        Text::new("🔥"),
                        text_font(&font_content, 20.0),
                        TextColor(Color::srgba(1.0, 0.5, 0.0, 0.8)),
                    ));

                    // Input -> Output row
                    let font_io = font_content.clone();
                    content
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            ..default()
                        },))
                        .with_children(|row| {
                            spawn_furnace_slot(row, MachineSlotType::Input, &font_io);
                            row.spawn((
                                Text::new("→"),
                                text_font(&font_io, 20.0),
                                TextColor(TEXT_SECONDARY),
                            ));
                            spawn_furnace_slot(row, MachineSlotType::Output, &font_io);
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
                        Text::new("E/ESC で閉じる"),
                        text_font(&font_content, 11.0),
                        TextColor(TEXT_SECONDARY),
                    ));
                });
        });
}

pub fn setup_crusher_ui(commands: &mut Commands, font: &Handle<Font>) {
    let font = font.clone();
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
            BorderColor(PANEL_BORDER),
            BorderRadius::all(Val::Px(QUEST_RADIUS)),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            let font = font.clone();
            // === Header (Factory theme) ===
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(HEADER_HEIGHT),
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(PANEL_BORDER),
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("⚙ 粉砕機"),
                        text_font(&font, 16.0),
                        TextColor(HEADER_COLOR),
                    ));
                });

            // === Content ===
            let font_content = font.clone();
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
                    let font_io = font_content.clone();
                    content
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            ..default()
                        },))
                        .with_children(|row| {
                            spawn_crusher_slot_inner(row, MachineSlotType::Input, &font_io);
                            row.spawn((
                                Text::new("→"),
                                text_font(&font_io, 20.0),
                                TextColor(TEXT_SECONDARY),
                            ));
                            spawn_crusher_slot_inner(row, MachineSlotType::Output, &font_io);
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
                        Text::new("鉱石を2倍に増やす"),
                        text_font(&font_content, 11.0),
                        TextColor(TEXT_SECONDARY),
                    ));

                    content.spawn((
                        Text::new("E/ESC で閉じる"),
                        text_font(&font_content, 11.0),
                        TextColor(TEXT_SECONDARY),
                    ));
                });
        });
}

pub fn setup_miner_ui(commands: &mut Commands, font: &Handle<Font>) {
    let font = font.clone();
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
            BorderColor(PANEL_BORDER),
            BorderRadius::all(Val::Px(QUEST_RADIUS)),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            let font = font.clone();
            // === Header (Factory theme) ===
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(HEADER_HEIGHT),
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(PANEL_BORDER),
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("⛏ 採掘機"),
                        text_font(&font, 16.0),
                        TextColor(HEADER_COLOR),
                    ));
                });

            // === Content ===
            let font_content = font.clone();
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
                    let font_row = font_content.clone();
                    content
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        },))
                        .with_children(|row| {
                            let font_slot = font_row.clone();
                            row.spawn((
                                Button,
                                MinerBufferButton,
                                Node {
                                    width: Val::Px(SLOT_SIZE),
                                    height: Val::Px(SLOT_SIZE),
                                    border: UiRect::all(Val::Px(SLOT_BORDER)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(SLOT_BG),
                                BorderColor(SLOT_BORDER_COLOR),
                                BorderRadius::all(Val::Px(SLOT_RADIUS)),
                            ))
                            .with_children(|slot| {
                                slot.spawn((
                                    MinerBufferCountText,
                                    Text::new(""),
                                    text_font(&font_slot, 14.0),
                                    TextColor(TEXT_PRIMARY),
                                ));
                            });

                            // Clear button
                            let font_btn = font_row.clone();
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
                                    text_font(&font_btn, 12.0),
                                    TextColor(TEXT_PRIMARY),
                                ));
                            });
                        });

                    // Instructions
                    content.spawn((
                        Text::new("バッファをクリックでアイテム取得"),
                        text_font(&font_content, 11.0),
                        TextColor(TEXT_SECONDARY),
                    ));

                    content.spawn((
                        Text::new("E/ESC で閉じる"),
                        text_font(&font_content, 11.0),
                        TextColor(TEXT_SECONDARY),
                    ));
                });
        });
}
