//! UI setup systems
//!
//! Creates all UI panels (hotbar, machine UIs, inventory, quests, etc.)

mod inventory_ui;
mod machine_ui;

pub use inventory_ui::setup_inventory_ui;
pub use machine_ui::{setup_crusher_ui, setup_furnace_ui, setup_miner_ui};

use crate::components::*;
use bevy::prelude::*;

/// Helper to create TextFont with the game font
pub fn text_font(font: &Handle<Font>, size: f32) -> TextFont {
    TextFont {
        font: font.clone(),
        font_size: size,
        ..default()
    }
}

/// Minecraft-style slot size (18px in MC scaled to 50px for this game)
pub const SLOT_SIZE: f32 = 52.0;
pub const SLOT_GAP: f32 = 3.0;
pub const SLOT_BORDER: f32 = 2.0;
pub const SPRITE_SIZE: f32 = 48.0;
/// Size for held item display (slightly larger for visibility)
pub const HELD_ITEM_SIZE: f32 = 56.0;

/// Helper to spawn an inventory slot button (Minecraft-style)
pub fn spawn_inventory_slot(parent: &mut ChildBuilder, slot_idx: usize, font: &Handle<Font>) {
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
            BorderColor(Color::srgba(0.25, 0.25, 0.25, 1.0)),      // Subtle border
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
                text_font(font, 14.0),
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

pub fn setup_ui(mut commands: Commands, game_font: Res<GameFont>) {
    let font = &game_font.0;

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
                let font = font.clone();
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
                            text_font(&font, 10.0),
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
                            text_font(&font, 12.0),
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
        text_font(font, 16.0),
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
    setup_furnace_ui(&mut commands, font);

    // Crusher UI panel (hidden by default)
    setup_crusher_ui(&mut commands, font);

    // Miner UI panel (hidden by default)
    setup_miner_ui(&mut commands, font);

    // Inventory UI panel (hidden by default)
    setup_inventory_ui(&mut commands, font);

    // Inventory tooltip (hidden by default, shown on hover)
    commands.spawn((
        InventoryTooltip,
        Text::new(""),
        text_font(font, 12.0),
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
    // Use GlobalZIndex to render on top of all other UI
    commands
        .spawn((
            HeldItemDisplay,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(HELD_ITEM_SIZE),
                height: Val::Px(HELD_ITEM_SIZE),
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            GlobalZIndex(100), // Render on top of everything
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Sprite image for held item
            parent.spawn((
                HeldItemImage,
                ImageNode::default(),
                Node {
                    width: Val::Px(HELD_ITEM_SIZE),
                    height: Val::Px(HELD_ITEM_SIZE),
                    ..default()
                },
            ));
        });
    // Held item count text (separate entity, follows cursor)
    commands.spawn((
        HeldItemText,
        Text::new(""),
        text_font(font, 14.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        },
        GlobalZIndex(101), // On top of held item
        Visibility::Hidden,
    ));

    // Quest UI panel (top-right)
    let font_clone = font.clone();
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
                text_font(&font_clone, 16.0),
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
            ));
            parent.spawn((
                QuestUIText,
                Text::new("Loading..."),
                text_font(&font_clone, 12.0),
                TextColor(Color::WHITE),
            ));
            // Deliver button (shown when quest is completable)
            let font_btn = font_clone.clone();
            parent
                .spawn((
                    Button,
                    QuestDeliverButton,
                    Node {
                        width: Val::Px(100.0),
                        height: Val::Px(30.0),
                        margin: UiRect::top(Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.4, 0.2, 0.9)),
                    BorderColor(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                    Visibility::Hidden,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("納品"),
                        text_font(&font_btn, 14.0),
                        TextColor(Color::WHITE),
                    ));
                });
        });

    // Command input UI (hidden by default)
    let font_cmd = font.clone();
    commands
        .spawn((
            CommandInputUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(80.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                min_width: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                CommandInputText,
                Text::new("> "),
                text_font(&font_cmd, 14.0),
                TextColor(Color::WHITE),
            ));
            // Suggestions container
            let font_sug = font_cmd.clone();
            parent
                .spawn((
                    CommandSuggestionsUI,
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::top(Val::Px(4.0)),
                        ..default()
                    },
                    Visibility::Hidden,
                ))
                .with_children(|suggestions| {
                    // Pre-spawn suggestion text slots (up to 5)
                    for i in 0..5 {
                        suggestions.spawn((
                            CommandSuggestionText(i),
                            Text::new(""),
                            text_font(&font_sug, 12.0),
                            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                            Visibility::Hidden,
                        ));
                    }
                });
        });

    // Tutorial popup (shown at game start, dismiss on any input)
    let font_tut = font.clone();
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
                text_font(&font_tut, 24.0),
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
                text_font(&font_tut, 14.0),
                TextColor(Color::WHITE),
            ));
        });
}
