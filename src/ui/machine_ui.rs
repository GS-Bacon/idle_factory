//! Machine UI setup (Furnace, Crusher, Miner)
//!
//! Follows design rules from .specify/memory/ui-design-rules.md

use crate::components::*;
use crate::game_spec::{MachineSpec, UiSlotDef, UiSlotType};
use crate::setup::ui::{
    text_font, QUEST_BORDER_COLOR, QUEST_PROGRESS_COLOR, QUEST_RADIUS, SLOT_BG, SLOT_BORDER,
    SLOT_BORDER_COLOR, SLOT_RADIUS, SLOT_SIZE,
};
use crate::ui::{UIId, UIVisibilityTarget};
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

// =============================================================================
// Generic Machine UI Generator
// =============================================================================

/// Spawn a generic slot button based on UiSlotDef
fn spawn_generic_slot(parent: &mut ChildBuilder, slot_def: &UiSlotDef, font: &Handle<Font>) {
    let is_input = matches!(slot_def.slot_type, UiSlotType::Input);
    let is_fuel = matches!(slot_def.slot_type, UiSlotType::Fuel);

    parent
        .spawn((
            Button,
            GenericMachineSlotButton {
                slot_id: slot_def.slot_id,
                is_input,
                is_fuel,
            },
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
                GenericMachineSlotCount {
                    slot_id: slot_def.slot_id,
                    is_input,
                    is_fuel,
                },
                Text::new(""),
                text_font(font, 14.0),
                TextColor(TEXT_PRIMARY),
            ));
        });
}

/// Setup generic machine UI from MachineSpec
///
/// This generates UI based on the spec's `ui_slots` definition.
/// Layout:
/// - Header with machine name
/// - Input slots row
/// - Progress bar
/// - Fuel slot (if present)
/// - Output slots row
/// - Instructions
pub fn setup_generic_machine_ui(
    commands: &mut Commands,
    spec: &'static MachineSpec,
    font: &Handle<Font>,
) {
    let font = font.clone();
    let panel_width = calculate_panel_width(spec);

    commands
        .spawn((
            GenericMachineUI {
                machine_id: spec.id,
            },
            UIVisibilityTarget::new(UIId::Machine),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(25.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-panel_width / 2.0),
                    ..default()
                },
                width: Val::Px(panel_width),
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
            // === Header ===
            spawn_header(panel, spec, &font);

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
                    spawn_io_row(content, spec, &font_content);

                    // Progress bar
                    spawn_progress_bar(content);

                    // Fuel slot (if any)
                    spawn_fuel_row(content, spec, &font_content);

                    // Instructions
                    content.spawn((
                        Text::new("E/ESC で閉じる"),
                        text_font(&font_content, 11.0),
                        TextColor(TEXT_SECONDARY),
                    ));
                });
        });
}

/// Calculate panel width based on slot count
fn calculate_panel_width(spec: &MachineSpec) -> f32 {
    let input_count = spec
        .ui_slots
        .iter()
        .filter(|s| matches!(s.slot_type, UiSlotType::Input))
        .count();
    let output_count = spec
        .ui_slots
        .iter()
        .filter(|s| matches!(s.slot_type, UiSlotType::Output))
        .count();

    let slot_count = input_count.max(1) + output_count.max(1);
    let base_width = (slot_count as f32 * (SLOT_SIZE + 12.0)) + 60.0;
    base_width.max(250.0)
}

/// Spawn header row
fn spawn_header(panel: &mut ChildBuilder, spec: &MachineSpec, font: &Handle<Font>) {
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
                GenericMachineHeaderText,
                Text::new(spec.name),
                text_font(font, 16.0),
                TextColor(HEADER_COLOR),
            ));
        });
}

/// Spawn input -> output row
fn spawn_io_row(content: &mut ChildBuilder, spec: &MachineSpec, font: &Handle<Font>) {
    let input_slots: Vec<_> = spec
        .ui_slots
        .iter()
        .filter(|s| matches!(s.slot_type, UiSlotType::Input))
        .collect();
    let output_slots: Vec<_> = spec
        .ui_slots
        .iter()
        .filter(|s| matches!(s.slot_type, UiSlotType::Output))
        .collect();

    content
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|row| {
            // Input slots
            for slot_def in input_slots {
                spawn_generic_slot(row, slot_def, font);
            }

            // Arrow
            if !output_slots.is_empty() {
                row.spawn((
                    Text::new("→"),
                    text_font(font, 20.0),
                    TextColor(TEXT_SECONDARY),
                ));
            }

            // Output slots
            for slot_def in output_slots {
                spawn_generic_slot(row, slot_def, font);
            }
        });
}

/// Spawn progress bar
fn spawn_progress_bar(content: &mut ChildBuilder) {
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
                GenericMachineProgressBar,
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(PROGRESS_FILL),
            ));
        });
}

/// Spawn fuel row if machine requires fuel
fn spawn_fuel_row(content: &mut ChildBuilder, spec: &MachineSpec, font: &Handle<Font>) {
    let fuel_slots: Vec<_> = spec
        .ui_slots
        .iter()
        .filter(|s| matches!(s.slot_type, UiSlotType::Fuel))
        .collect();

    if fuel_slots.is_empty() {
        return;
    }

    // Fire icon
    content.spawn((
        Text::new("▼"),
        text_font(font, 20.0),
        TextColor(Color::srgba(1.0, 0.5, 0.0, 0.8)),
    ));

    content
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|row| {
            for slot_def in fuel_slots {
                spawn_generic_slot(row, slot_def, font);
                row.spawn((
                    Text::new(slot_def.label),
                    text_font(font, 12.0),
                    TextColor(TEXT_SECONDARY),
                ));
            }
        });
}
