//! UI setup systems
//!
//! Creates all UI panels (hotbar, machine UIs, inventory, quests, etc.)

mod inventory_ui;
mod machine_ui;

pub use inventory_ui::setup_inventory_ui;
pub use machine_ui::{setup_crusher_ui, setup_furnace_ui, setup_miner_ui};

use crate::components::*;
use bevy::prelude::*;

/// Minecraft-style slot size (18px in MC scaled to 50px for this game)
pub const SLOT_SIZE: f32 = 50.0;
pub const SLOT_GAP: f32 = 3.0;
pub const SLOT_BORDER: f32 = 2.0;
pub const SPRITE_SIZE: f32 = 46.0;

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

/// Helper to spawn an inventory slot button (Minecraft-style)
pub fn spawn_inventory_slot(parent: &mut ChildBuilder, slot_idx: usize) {
    parent
        .spawn((
            Button,
            InventorySlotUI(slot_idx),
            Node {
                width: Val::Px(SLOT_SIZE),
                height: Val::Px(SLOT_SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(SLOT_BORDER)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.14, 0.14, 0.14, 0.95)), // Darker MC-style
            BorderColor(Color::srgba(0.25, 0.25, 0.25, 1.0)), // Subtle border
        ))
        .with_children(|btn| {
            // Item sprite image
            btn.spawn((
                InventorySlotImage(slot_idx),
                ImageNode::default(),
                Visibility::Hidden, // Hide when no image
                Node {
                    width: Val::Px(SPRITE_SIZE),
                    height: Val::Px(SPRITE_SIZE),
                    ..default()
                },
            ));
            // Item count (bottom-right)
            btn.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(2.0),
                    right: Val::Px(4.0),
                    ..default()
                },
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
                        // Item sprite image
                        slot.spawn((
                            HotbarSlotImage(i),
                            ImageNode::default(),
                            Visibility::Hidden, // Hide when no image
                            Node {
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
                                ..default()
                            },
                        ));
                        // Item count
                        slot.spawn((
                            HotbarSlotCount(i),
                            Text::new(""),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(2.0),
                                right: Val::Px(4.0),
                                ..default()
                            },
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
    commands
        .spawn((
            HeldItemDisplay,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(SPRITE_SIZE),
                height: Val::Px(SPRITE_SIZE),
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Sprite image for held item
            parent.spawn((
                HeldItemImage,
                ImageNode::default(),
                Node {
                    width: Val::Px(SPRITE_SIZE),
                    height: Val::Px(SPRITE_SIZE),
                    ..default()
                },
            ));
        });
    // Held item count text (separate entity, follows cursor)
    commands.spawn((
        HeldItemText,
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
