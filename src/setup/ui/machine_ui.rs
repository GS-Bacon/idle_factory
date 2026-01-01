//! Machine UI setup (Furnace, Crusher, Miner)

use crate::components::*;
use bevy::prelude::*;

use super::{spawn_crusher_slot, spawn_machine_slot};

pub fn setup_furnace_ui(commands: &mut Commands) {
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

pub fn setup_crusher_ui(commands: &mut Commands) {
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

pub fn setup_miner_ui(commands: &mut Commands) {
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
