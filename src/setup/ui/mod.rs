//! UI setup systems
//!
//! Creates all UI panels (hotbar, machine UIs, inventory, quests, etc.)

mod inventory_ui;
pub mod settings_ui;
pub mod tokens;
pub mod widgets;

pub use inventory_ui::{
    setup_inventory_ui, UpperPanel, UpperPanelGrid, UpperPanelPageText, UpperPanelSearchInput,
    UpperPanelSlot, UpperPanelSlotCount, UpperPanelSlotImage, UpperPanelTabs, UPPER_PANEL_SLOTS,
};
pub use settings_ui::{
    handle_settings_back, handle_settings_sliders, handle_settings_toggles,
    handle_slider_drag_state, setup_settings_ui, update_settings_ui, update_settings_visibility,
    SliderDragState,
};

// Re-export machine UI setup from ui module
pub use crate::ui::machine_ui::setup_generic_machine_ui;

use crate::components::*;
use crate::game_spec::{UIElementRegistry, UIElementTag};
use bevy::prelude::*;
use tokens::{color, font, size};

/// Helper to create TextFont with the game font
pub fn text_font(font: &Handle<Font>, size: f32) -> TextFont {
    TextFont {
        font: font.clone(),
        font_size: size,
        ..default()
    }
}

/// Create a complete Text bundle with font (white text)
/// Use this instead of manually creating Text + TextFont + TextColor
pub fn game_text(content: impl Into<String>, font: &Handle<Font>, size: f32) -> impl Bundle {
    (
        Text::new(content.into()),
        text_font(font, size),
        TextColor(Color::WHITE),
    )
}

/// Create a complete Text bundle with font and custom color
pub fn game_text_colored(
    content: impl Into<String>,
    font: &Handle<Font>,
    size: f32,
    color: Color,
) -> impl Bundle {
    (
        Text::new(content.into()),
        text_font(font, size),
        TextColor(color),
    )
}

// =============================================================================
// Factory Theme - UI Constants
// Re-exported from tokens module for backward compatibility
// =============================================================================

// Slot dimensions (from tokens)
pub const SLOT_SIZE: f32 = size::SLOT_SIZE;
pub const SLOT_GAP: f32 = size::SLOT_GAP;
pub const SLOT_BORDER: f32 = size::SLOT_BORDER;
pub const SLOT_RADIUS: f32 = size::SLOT_RADIUS;
pub const SPRITE_SIZE: f32 = size::SPRITE_SIZE;
/// Size for held item display (slightly larger for visibility)
pub const HELD_ITEM_SIZE: f32 = size::HELD_ITEM_SIZE;

// Slot colors (from tokens)
pub const SLOT_BG: Color = color::SLOT_BG;
pub const SLOT_BORDER_COLOR: Color = color::ACCENT;
pub const SLOT_HOVER_BG: Color = color::SLOT_HOVER;
pub const SLOT_HOVER_BORDER: Color = color::ACCENT_LIGHT;
pub const SLOT_SELECTED_BG: Color = color::SLOT_SELECTED;
pub const SLOT_SELECTED_BORDER: Color = color::ACCENT_GOLD;

// Text sizes (from tokens)
pub const TEXT_TINY: f32 = font::TINY;
pub const TEXT_MINI: f32 = font::MINI;
pub const TEXT_SMALL: f32 = font::SMALL;
pub const TEXT_CAPTION: f32 = font::CAPTION;
pub const TEXT_BODY: f32 = font::BODY;
pub const TEXT_BUTTON: f32 = font::BUTTON;
pub const TEXT_SECTION: f32 = font::SECTION;
pub const TEXT_TITLE: f32 = font::TITLE;
pub const TEXT_LARGE: f32 = font::LARGE;
pub const TEXT_HEADING: f32 = font::HEADING;
pub const TEXT_HUGE: f32 = font::HUGE;

// Legacy aliases (for backward compatibility)
pub const SLOT_NUMBER_SIZE: f32 = font::TINY;
pub const SLOT_NUMBER_COLOR: Color = color::TEXT_SLOT_NUMBER;
pub const SLOT_COUNT_SIZE: f32 = font::SMALL;
pub const ITEM_NAME_SIZE: f32 = font::LARGE;

// Quest panel (from tokens)
pub const QUEST_BG: Color = color::PANEL;
pub const QUEST_BORDER_COLOR: Color = color::ACCENT;
pub const QUEST_BORDER_WIDTH: f32 = 2.0;
pub const QUEST_RADIUS: f32 = size::SLOT_RADIUS;
pub const QUEST_PADDING: f32 = 12.0;
pub const QUEST_HEADER_COLOR: Color = color::ACCENT_GOLD;
pub const QUEST_PROGRESS_COLOR: Color = color::ACCENT;

/// Helper to spawn an inventory slot button (Factory theme)
pub fn spawn_inventory_slot(
    parent: &mut ChildSpawnerCommands,
    slot_idx: usize,
    font: &Handle<Font>,
) {
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
                border_radius: BorderRadius::all(Val::Px(SLOT_RADIUS)),
                ..default()
            },
            BackgroundColor(SLOT_BG),
            BorderColor::all(SLOT_BORDER_COLOR),
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
                text_font(font, SLOT_COUNT_SIZE),
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

pub fn setup_ui(
    mut commands: Commands,
    game_font: Res<GameFont>,
    ui_registry: Res<UIElementRegistry>,
) {
    let font = &game_font.0;

    // Hotbar UI - centered at bottom (Factory theme)
    // Width calculation: 9 slots * SLOT_SIZE + 8 gaps * SLOT_GAP
    let hotbar_width = SLOT_SIZE * 9.0 + SLOT_GAP * 8.0;
    commands
        .spawn((
            HotbarUI,
            ui_registry
                .get_id("base:hotbar")
                .map(UIElementTag::new)
                .unwrap_or_else(|| UIElementTag::new(Default::default())),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-hotbar_width / 2.0),
                    ..default()
                },
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(SLOT_GAP),
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
                            width: Val::Px(SLOT_SIZE),
                            height: Val::Px(SLOT_SIZE),
                            border: UiRect::all(Val::Px(SLOT_BORDER)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            border_radius: BorderRadius::all(Val::Px(SLOT_RADIUS)),
                            ..default()
                        },
                        BackgroundColor(SLOT_BG),
                        BorderColor::all(SLOT_BORDER_COLOR),
                    ))
                    .with_children(|slot| {
                        // Slot number
                        slot.spawn((
                            Text::new(format!("{}", i + 1)),
                            text_font(&font, SLOT_NUMBER_SIZE),
                            TextColor(SLOT_NUMBER_COLOR),
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
                                width: Val::Px(SPRITE_SIZE),
                                height: Val::Px(SPRITE_SIZE),
                                ..default()
                            },
                        ));
                        // Item count
                        slot.spawn((
                            HotbarSlotCount(i),
                            Text::new(""),
                            text_font(&font, SLOT_COUNT_SIZE),
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
        text_font(font, ITEM_NAME_SIZE),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(90.0), // Adjusted for larger slots
            left: Val::Percent(50.0),
            ..default()
        },
    ));

    // Crosshair
    commands.spawn((
        ui_registry
            .get_id("base:crosshair")
            .map(UIElementTag::new)
            .unwrap_or_else(|| UIElementTag::new(Default::default())),
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

    // Machine UI panels (hidden by default, data-driven from MachineSpec)
    use crate::game_spec::{CRUSHER, FURNACE, MINER};
    setup_generic_machine_ui(&mut commands, &FURNACE, font, &ui_registry);
    setup_generic_machine_ui(&mut commands, &CRUSHER, font, &ui_registry);
    setup_generic_machine_ui(&mut commands, &MINER, font, &ui_registry);

    // Inventory UI panel (hidden by default)
    setup_inventory_ui(&mut commands, font, &ui_registry);

    // Inventory tooltip (hidden by default, shown on hover)
    commands.spawn((
        InventoryTooltip,
        Text::new(""),
        text_font(font, TEXT_SMALL),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        BackgroundColor(color::PANEL),
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
                Visibility::Inherited, // Inherit visibility from parent HeldItemDisplay
            ));
        });
    // Held item count text (separate entity, follows cursor)
    commands.spawn((
        HeldItemText,
        Text::new(""),
        text_font(font, TEXT_BODY),
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

    // Quest UI panel (top-right) - Factory theme
    // Start hidden if tutorial is active (controlled by update_tutorial_ui)
    let font_clone = font.clone();
    commands
        .spawn((
            QuestUI,
            ui_registry
                .get_id("base:quest_tracker")
                .map(UIElementTag::new)
                .unwrap_or_else(|| UIElementTag::new(Default::default())),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(QUEST_PADDING)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                min_width: Val::Px(220.0),
                border: UiRect::all(Val::Px(QUEST_BORDER_WIDTH)),
                border_radius: BorderRadius::all(Val::Px(QUEST_RADIUS)),
                ..default()
            },
            BackgroundColor(QUEST_BG),
            BorderColor::all(QUEST_BORDER_COLOR),
            Visibility::Hidden, // Hidden until tutorial completes
        ))
        .with_children(|parent| {
            // Header with quest icon
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    padding: UiRect::bottom(Val::Px(4.0)),
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                })
                .with_child((
                    Text::new("[Q] クエスト"),
                    text_font(&font_clone, TEXT_BUTTON),
                    TextColor(QUEST_HEADER_COLOR),
                ));

            // Quest description
            parent.spawn((
                QuestUIText,
                Text::new("Loading..."),
                text_font(&font_clone, TEXT_CAPTION),
                TextColor(color::TEXT_LIGHT),
                Node {
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
            ));

            // Progress container (holds progress bars for each required item)
            let font_progress = font_clone.clone();
            parent
                .spawn((
                    QuestProgressContainer,
                    ui_registry
                        .get_id("base:quest_progress_container")
                        .map(UIElementTag::new)
                        .unwrap_or_else(|| UIElementTag::new(Default::default())),
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        ..default()
                    },
                ))
                .with_children(|container| {
                    // Pre-spawn 3 progress item slots (max items per quest)
                    for i in 0..3 {
                        container
                            .spawn((
                                QuestProgressItem(i),
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(2.0),
                                    ..default()
                                },
                                Visibility::Hidden,
                            ))
                            .with_children(|item_row| {
                                // Item name and count text
                                item_row.spawn((
                                    QuestProgressText(i),
                                    Text::new(""),
                                    text_font(&font_progress, TEXT_MINI),
                                    TextColor(color::TEXT_MUTED),
                                ));

                                // Progress bar container
                                item_row
                                    .spawn((
                                        QuestProgressBarBg(i),
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(10.0),
                                            border: UiRect::all(Val::Px(1.0)),
                                            border_radius: BorderRadius::all(Val::Px(3.0)),
                                            ..default()
                                        },
                                        BackgroundColor(color::PROGRESS_BG),
                                        BorderColor::all(QUEST_BORDER_COLOR),
                                    ))
                                    .with_child((
                                        QuestProgressBarFill(i),
                                        Node {
                                            width: Val::Percent(0.0),
                                            height: Val::Percent(100.0),
                                            border_radius: BorderRadius::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(QUEST_PROGRESS_COLOR),
                                    ));
                            });
                    }
                });

            // Deliver button (shown when quest is completable)
            let font_btn = font_clone.clone();
            parent
                .spawn((
                    Button,
                    QuestDeliverButton,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(32.0),
                        margin: UiRect::top(Val::Px(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(color::BUTTON_DELIVER),
                    BorderColor::all(color::BUTTON_DELIVER_BORDER),
                    Visibility::Hidden,
                ))
                .with_child((
                    Text::new("[D] 納品する"),
                    text_font(&font_btn, TEXT_BODY),
                    TextColor(Color::WHITE),
                ));
        });

    // Command input UI (hidden by default)
    let font_cmd = font.clone();
    commands
        .spawn((
            CommandInputUI,
            ui_registry
                .get_id("base:command_input")
                .map(UIElementTag::new)
                .unwrap_or_else(|| UIElementTag::new(Default::default())),
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
            BackgroundColor(color::COMMAND_BG),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                CommandInputText,
                Text::new("> "),
                text_font(&font_cmd, TEXT_BODY),
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
                            text_font(&font_sug, TEXT_SMALL),
                            TextColor(color::TEXT_DIM),
                            Visibility::Hidden,
                        ));
                    }
                });
        });

    // Tutorial progress panel (shown during tutorial, hidden after completion)
    let font_panel = font.clone();
    commands
        .spawn((
            TutorialPanel,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-180.0),
                    ..default()
                },
                width: Val::Px(360.0),
                padding: UiRect::all(Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(color::PANEL_DARK),
            BorderColor::all(QUEST_BORDER_COLOR),
            Visibility::Hidden, // Hidden until tutorial popup is dismissed
        ))
        .with_children(|panel| {
            // Step text
            panel.spawn((
                TutorialStepText,
                Text::new("[T] チュートリアル"),
                text_font(&font_panel, TEXT_BODY),
                TextColor(QUEST_HEADER_COLOR),
            ));

            // Progress text (e.g., "3/5")
            panel.spawn((
                TutorialProgressText,
                Text::new(""),
                text_font(&font_panel, TEXT_SMALL),
                TextColor(color::TEXT_MUTED),
                Visibility::Hidden, // Hidden when no count-based action
            ));

            // Progress bar container
            panel
                .spawn((
                    TutorialProgressBarBg,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(10.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(color::PROGRESS_BG),
                    BorderColor::all(QUEST_BORDER_COLOR),
                    Visibility::Hidden, // Hidden when no count-based action
                ))
                .with_child((
                    TutorialProgressBarFill,
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(QUEST_PROGRESS_COLOR),
                ));
        });

    // Biome HUD - top left, always visible (Factory theme)
    commands.spawn((
        BiomeHudText,
        Text::new("バイオーム: 読み込み中..."),
        text_font(font, TEXT_CAPTION),
        TextColor(color::TEXT_LIGHT),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            padding: UiRect::all(Val::Px(QUEST_PADDING)),
            border: UiRect::all(Val::Px(QUEST_BORDER_WIDTH)),
            border_radius: BorderRadius::all(Val::Px(QUEST_RADIUS)),
            ..default()
        },
        BackgroundColor(QUEST_BG),
        BorderColor::all(QUEST_BORDER_COLOR),
    ));

    // Settings UI panel (hidden by default)
    setup_settings_ui(&mut commands, font, &ui_registry);

    // Pause overlay - shown when ESC pressed
    let font_pause = font.clone();
    commands
        .spawn((
            PauseUI,
            ui_registry
                .get_id("base:pause_menu")
                .map(UIElementTag::new)
                .unwrap_or_else(|| UIElementTag::new(Default::default())),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(color::OVERLAY_PAUSE),
            GlobalZIndex(100),   // Above all other UI
            Visibility::Visible, // Start visible (UIState defaults to PauseMenu)
        ))
        .with_children(|pause| {
            // Title
            pause.spawn((
                Text::new("一時停止"),
                text_font(&font_pause, TEXT_HUGE),
                TextColor(Color::WHITE),
            ));

            // Button container
            pause
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|btns| {
                    // Resume button
                    spawn_pause_button(btns, &font_pause, "再開", PauseMenuButton::Resume);
                    // Settings button
                    spawn_pause_button(btns, &font_pause, "設定", PauseMenuButton::Settings);
                    // Quit button (native only)
                    #[cfg(not(target_arch = "wasm32"))]
                    spawn_pause_button(btns, &font_pause, "終了", PauseMenuButton::Quit);
                });

            // Hint text
            pause.spawn((
                Text::new("ESCで再開"),
                text_font(&font_pause, TEXT_BODY),
                TextColor(color::TEXT_HINT),
                Node {
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
            ));
        });
}

/// Pause menu button types
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuButton {
    Resume,
    Settings,
    #[allow(dead_code)]
    Quit,
}

/// Spawn a pause menu button
fn spawn_pause_button(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    button_type: PauseMenuButton,
) {
    parent
        .spawn((
            Button,
            button_type,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(color::BUTTON_PAUSE),
            BorderColor::all(color::BORDER_ACCENT),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                text_font(font, TEXT_TITLE),
                TextColor(color::TEXT),
            ));
        });
}
