//! Common UI widgets for reusable UI components
//!
//! This module provides factory functions for creating standardized UI widgets
//! that can be used across different UI panels (inventory, machine UI, etc.)

use bevy::prelude::*;

// Import theme constants from setup/ui
use crate::setup::ui::{
    text_font, SLOT_BG, SLOT_BORDER_COLOR, SLOT_GAP, SLOT_RADIUS, SLOT_SIZE, SPRITE_SIZE,
    TEXT_SMALL,
};

/// Configuration for spawning a slot widget
#[derive(Clone, Debug)]
pub struct SlotConfig {
    /// Slot index (for inventory systems)
    pub index: usize,
    /// Whether to show item count text
    pub show_count: bool,
    /// Optional custom size (defaults to SLOT_SIZE)
    pub size: Option<f32>,
    /// Optional marker component type name
    pub marker: Option<&'static str>,
}

impl Default for SlotConfig {
    fn default() -> Self {
        Self {
            index: 0,
            show_count: true,
            size: None,
            marker: None,
        }
    }
}

/// Marker component for slot widgets
#[derive(Component)]
pub struct SlotWidget {
    pub index: usize,
}

/// Marker component for slot item image
#[derive(Component)]
pub struct SlotItemImage {
    pub index: usize,
}

/// Marker component for slot count text
#[derive(Component)]
pub struct SlotCountText {
    pub index: usize,
}

/// Spawn a standard inventory slot with item image and count text
/// Returns the slot entity
pub fn spawn_slot(commands: &mut Commands, config: SlotConfig, font: Handle<Font>) -> Entity {
    let size = config.size.unwrap_or(SLOT_SIZE);
    let index = config.index;

    commands
        .spawn((
            SlotWidget { index },
            Button,
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(SLOT_GAP / 2.0)),
                border_radius: BorderRadius::all(Val::Px(SLOT_RADIUS)),
                ..default()
            },
            BackgroundColor(SLOT_BG),
            BorderColor::all(SLOT_BORDER_COLOR),
            Interaction::None,
        ))
        .with_children(|slot| {
            // Item image
            slot.spawn((
                SlotItemImage { index },
                ImageNode {
                    image: Handle::default(),
                    ..default()
                },
                Node {
                    width: Val::Px(SPRITE_SIZE),
                    height: Val::Px(SPRITE_SIZE),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                Visibility::Hidden,
            ));

            // Count text
            if config.show_count {
                slot.spawn((
                    SlotCountText { index },
                    Text::new(""),
                    text_font(&font, TEXT_SMALL),
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(2.0),
                        right: Val::Px(4.0),
                        ..default()
                    },
                ));
            }
        })
        .id()
}

/// Configuration for spawning a button widget
#[derive(Clone, Debug)]
pub struct ButtonConfig {
    /// Button text
    pub text: String,
    /// Button width
    pub width: Val,
    /// Button height
    pub height: Val,
    /// Font size
    pub font_size: f32,
    /// Background color
    pub bg_color: Color,
    /// Hover background color
    pub hover_color: Color,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            text: "Button".to_string(),
            width: Val::Px(100.0),
            height: Val::Px(30.0),
            font_size: 14.0,
            bg_color: Color::srgb(0.25, 0.25, 0.30),
            hover_color: Color::srgb(0.35, 0.35, 0.40),
        }
    }
}

/// Marker component for button widgets
#[derive(Component)]
pub struct ButtonWidget;

/// Spawn a standard button with text
/// Returns the button entity
pub fn spawn_button(commands: &mut Commands, config: ButtonConfig, font: Handle<Font>) -> Entity {
    commands
        .spawn((
            ButtonWidget,
            Button,
            Node {
                width: config.width,
                height: config.height,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(config.bg_color),
            Interaction::None,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(config.text),
                text_font(&font, config.font_size),
                TextColor(Color::WHITE),
            ));
        })
        .id()
}

/// Spawn a row of slots (for inventory grids)
pub fn spawn_slot_row(
    commands: &mut Commands,
    start_index: usize,
    count: usize,
    font: Handle<Font>,
) -> Entity {
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            for i in 0..count {
                let config = SlotConfig {
                    index: start_index + i,
                    show_count: true,
                    size: None,
                    marker: None,
                };
                spawn_slot_child(row, config, font.clone());
            }
        })
        .id()
}

/// Helper to spawn a slot as a child (used in spawn_slot_row)
fn spawn_slot_child(parent: &mut ChildSpawnerCommands, config: SlotConfig, font: Handle<Font>) {
    let size = config.size.unwrap_or(SLOT_SIZE);
    let index = config.index;

    parent
        .spawn((
            SlotWidget { index },
            Button,
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(SLOT_GAP / 2.0)),
                border_radius: BorderRadius::all(Val::Px(SLOT_RADIUS)),
                ..default()
            },
            BackgroundColor(SLOT_BG),
            BorderColor::all(SLOT_BORDER_COLOR),
            Interaction::None,
        ))
        .with_children(|slot| {
            // Item image
            slot.spawn((
                SlotItemImage { index },
                ImageNode {
                    image: Handle::default(),
                    ..default()
                },
                Node {
                    width: Val::Px(SPRITE_SIZE),
                    height: Val::Px(SPRITE_SIZE),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                Visibility::Hidden,
            ));

            // Count text
            if config.show_count {
                slot.spawn((
                    SlotCountText { index },
                    Text::new(""),
                    text_font(&font, TEXT_SMALL),
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(2.0),
                        right: Val::Px(4.0),
                        ..default()
                    },
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_config_default() {
        let config = SlotConfig::default();
        assert_eq!(config.index, 0);
        assert!(config.show_count);
        assert!(config.size.is_none());
    }

    #[test]
    fn test_button_config_default() {
        let config = ButtonConfig::default();
        assert_eq!(config.text, "Button");
        assert_eq!(config.font_size, 14.0);
    }
}
