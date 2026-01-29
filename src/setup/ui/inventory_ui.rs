//! Inventory UI setup
//!
//! Unified inventory panel that displays:
//! - Upper panel (Platform Inventory / Creative Catalog) with tabs, search, scrollable grid
//! - Main inventory (3x9)
//! - Hotbar (1x9)
//! - Trash slot

use crate::components::*;
use crate::game_spec::{UIElementRegistry, UIElementTag};
use bevy::prelude::*;

use super::tokens::{color, font};
use super::{
    spawn_inventory_slot, text_font, QUEST_BG, QUEST_BORDER_COLOR, SLOT_BG, SLOT_BORDER,
    SLOT_BORDER_COLOR, SLOT_GAP, SLOT_RADIUS, SLOT_SIZE, SPRITE_SIZE,
};

/// Calculate inventory UI width based on slot size
fn inventory_ui_width() -> f32 {
    // 9 slots + 8 gaps + padding
    SLOT_SIZE * 9.0 + SLOT_GAP * 8.0 + 24.0
}

/// Marker component for the upper panel (Platform Inventory or Creative Catalog)
#[derive(Component)]
pub struct UpperPanel;

/// Marker component for upper panel scrollable grid container
#[derive(Component)]
pub struct UpperPanelGrid;

/// Marker component for upper panel slot button
#[derive(Component)]
pub struct UpperPanelSlot(pub usize);

/// Marker component for upper panel slot image
#[derive(Component)]
pub struct UpperPanelSlotImage(pub usize);

/// Marker component for upper panel slot count text
#[derive(Component)]
pub struct UpperPanelSlotCount(pub usize);

/// Marker component for upper panel search input text
#[derive(Component)]
pub struct UpperPanelSearchInput;

/// Marker component for upper panel tab buttons container
#[derive(Component)]
pub struct UpperPanelTabs;

/// Marker component for upper panel page text
#[derive(Component)]
pub struct UpperPanelPageText;

/// Number of slots in the upper panel grid
pub const UPPER_PANEL_SLOTS: usize = 36; // 9 columns x 4 rows

pub fn setup_inventory_ui(
    commands: &mut Commands,
    font: &Handle<Font>,
    ui_registry: &UIElementRegistry,
) {
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
        BackgroundColor(color::OVERLAY),
        GlobalZIndex(40), // Below inventory UI
        Visibility::Hidden,
    ));

    commands
        .spawn((
            InventoryUI,
            ui_registry
                .get_id("base:inventory_panel")
                .map(UIElementTag::new)
                .unwrap_or_else(|| UIElementTag::new(Default::default())),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(10.0),
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(12.0)),
                margin: UiRect {
                    left: Val::Px(-ui_width / 2.0),
                    ..default()
                },
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(SLOT_RADIUS)),
                ..default()
            },
            BackgroundColor(QUEST_BG),
            BorderColor::all(QUEST_BORDER_COLOR),
            GlobalZIndex(50), // Above overlay, below held item
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // === Upper panel (Platform Inventory / Creative Catalog) ===
            // Conditionally visible when creative_mode.enabled || local_platform.entity.is_some()
            spawn_upper_panel(parent, font);

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
                BackgroundColor(color::ACCENT_TRANSPARENT),
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
                BackgroundColor(color::ACCENT_TRANSPARENT),
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
                                border_radius: BorderRadius::all(Val::Px(SLOT_RADIUS)),
                                ..default()
                            },
                            BackgroundColor(color::TRASH_BG),
                            BorderColor::all(color::TRASH_BORDER),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("X"),
                                text_font(font, font::BUTTON),
                                TextColor(color::TRASH_TEXT),
                            ));
                        });
                });
        });
}

/// Spawn the upper panel (Platform Inventory / Creative Catalog)
fn spawn_upper_panel(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            UpperPanel,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(4.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(SLOT_RADIUS)),
                ..default()
            },
            BackgroundColor(QUEST_BG),
            Visibility::Hidden, // Start hidden, shown based on conditions
        ))
        .with_children(|upper| {
            // Tab buttons row (Creative / Platform tabs)
            upper
                .spawn((
                    UpperPanelTabs,
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                    Visibility::Hidden, // Only shown when both creative and platform are available
                ))
                .with_children(|tabs| {
                    // Creative tab
                    tabs.spawn((
                        Button,
                        GlobalInventoryCategoryTab(ItemCategory::All),
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(color::TAB_ACTIVE),
                        BorderColor::all(QUEST_BORDER_COLOR),
                    ))
                    .with_child((
                        Text::new("All"),
                        text_font(font, font::BODY),
                        TextColor(color::TEXT),
                    ));

                    // Ores tab
                    tabs.spawn((
                        Button,
                        GlobalInventoryCategoryTab(ItemCategory::Ores),
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(color::TAB_INACTIVE),
                        BorderColor::all(color::BORDER),
                    ))
                    .with_child((
                        Text::new("Ores"),
                        text_font(font, font::BODY),
                        TextColor(color::TEXT_MUTED),
                    ));

                    // Ingots tab
                    tabs.spawn((
                        Button,
                        GlobalInventoryCategoryTab(ItemCategory::Ingots),
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(color::TAB_INACTIVE),
                        BorderColor::all(color::BORDER),
                    ))
                    .with_child((
                        Text::new("Ingots"),
                        text_font(font, font::BODY),
                        TextColor(color::TEXT_MUTED),
                    ));

                    // Machines tab
                    tabs.spawn((
                        Button,
                        GlobalInventoryCategoryTab(ItemCategory::Machines),
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(color::TAB_INACTIVE),
                        BorderColor::all(color::BORDER),
                    ))
                    .with_child((
                        Text::new("Machines"),
                        text_font(font, font::BODY),
                        TextColor(color::TEXT_MUTED),
                    ));
                });

            // Search input
            upper
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(28.0),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(color::INPUT_BG),
                    BorderColor::all(color::BORDER),
                ))
                .with_child((
                    Text::new("Search..."),
                    text_font(font, font::BODY),
                    TextColor(color::TEXT_DIM),
                    UpperPanelSearchInput,
                ));

            // Scrollable grid container
            upper
                .spawn((
                    UpperPanelGrid,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(SLOT_GAP),
                        max_height: Val::Px(252.0), // 4 rows of slots
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                ))
                .with_children(|grid| {
                    // Spawn 4 rows x 9 columns = 36 slots
                    for row in 0..4 {
                        grid.spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(SLOT_GAP),
                            ..default()
                        },))
                            .with_children(|row_node| {
                                for col in 0..9 {
                                    let slot_idx = row * 9 + col;
                                    spawn_upper_panel_slot(row_node, slot_idx, font);
                                }
                            });
                    }
                });

            // Page navigation row
            upper
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },))
                .with_children(|nav| {
                    // Previous page button
                    nav.spawn((
                        Button,
                        GlobalInventoryPageButton { next: false },
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(color::BUTTON),
                        BorderColor::all(QUEST_BORDER_COLOR),
                    ))
                    .with_child((
                        Text::new("<"),
                        text_font(font, font::BODY),
                        TextColor(color::TEXT),
                    ));

                    // Page text
                    nav.spawn((
                        Text::new("1/1"),
                        text_font(font, font::BODY),
                        TextColor(color::TEXT),
                        UpperPanelPageText,
                    ));

                    // Next page button
                    nav.spawn((
                        Button,
                        GlobalInventoryPageButton { next: true },
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(color::BUTTON),
                        BorderColor::all(QUEST_BORDER_COLOR),
                    ))
                    .with_child((
                        Text::new(">"),
                        text_font(font, font::BODY),
                        TextColor(color::TEXT),
                    ));
                });
        });
}

/// Spawn an upper panel slot button
fn spawn_upper_panel_slot(parent: &mut ChildSpawnerCommands, slot_idx: usize, font: &Handle<Font>) {
    parent
        .spawn((
            Button,
            UpperPanelSlot(slot_idx),
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
                UpperPanelSlotImage(slot_idx),
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
                UpperPanelSlotCount(slot_idx),
                Text::new(""),
                text_font(font, font::TINY),
                TextColor(color::TEXT),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(2.0),
                    right: Val::Px(4.0),
                    ..default()
                },
            ));
        });
}
