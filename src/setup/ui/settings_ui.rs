//! Settings UI setup and systems

use bevy::prelude::*;

use crate::components::UIContext;
use crate::game_spec::{UIElementRegistry, UIElementTag};
use crate::settings::GameSettings;
use crate::setup::ui::{
    text_font, SLOT_BORDER_COLOR, SLOT_RADIUS, TEXT_BODY, TEXT_HEADING, TEXT_SECTION, TEXT_SMALL,
};
use crate::updater::{StartUpdateEvent, UpdatePhase, UpdateState};

/// Marker for the settings panel root
#[derive(Component)]
pub struct SettingsPanel;

/// Marker for settings UI elements that need updating
#[derive(Component)]
pub struct SettingsSlider {
    pub setting: SettingType,
    pub min: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct SettingsToggle {
    pub setting: SettingType,
}

#[derive(Component)]
pub struct SettingsSliderFill;

#[derive(Component)]
pub struct SettingsValueText {
    pub setting: SettingType,
}

/// Types of settings
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingType {
    MouseSensitivity,
    ViewDistance,
    Fov,
    MasterVolume,
    SfxVolume,
    MusicVolume,
    VSync,
    Fullscreen,
    InvertY,
}

/// Back button on settings panel
#[derive(Component)]
pub struct SettingsBackButton;

/// Update button in settings panel
#[derive(Component)]
pub struct SettingsUpdateButton;

/// Update status text in settings panel
#[derive(Component)]
pub struct SettingsUpdateStatusText;

/// Resource to track slider drag state
#[derive(Resource, Default)]
pub struct SliderDragState {
    /// Currently dragging slider entity
    pub dragging: Option<Entity>,
}

/// Setup the settings UI panel
pub fn setup_settings_ui(
    commands: &mut Commands,
    font: &Handle<Font>,
    ui_registry: &UIElementRegistry,
) {
    commands
        .spawn((
            SettingsPanel,
            ui_registry
                .get_id("base:settings_menu")
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
                ..default()
            },
            BackgroundColor(Color::NONE), // No background (use PauseUI background)
            GlobalZIndex(101),            // Above pause menu
            Visibility::Hidden,
        ))
        .with_children(|root| {
            // Settings panel container
            root.spawn((
                Node {
                    width: Val::Px(500.0),
                    max_height: Val::Percent(85.0), // 画面の85%まで
                    padding: UiRect::all(Val::Px(20.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(15.0),
                    border: UiRect::all(Val::Px(2.0)),
                    overflow: Overflow::scroll_y(), // 縦スクロール有効
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.12, 0.14, 0.98)),
                BorderColor(SLOT_BORDER_COLOR),
                BorderRadius::all(Val::Px(SLOT_RADIUS)),
            ))
            .with_children(|panel| {
                // Title
                panel.spawn((
                    Text::new("設定"),
                    text_font(font, TEXT_HEADING),
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ));

                // Graphics section
                spawn_section_header(panel, font, "グラフィック");
                spawn_slider(panel, font, "描画距離", SettingType::ViewDistance, 1.0, 8.0);
                spawn_slider(panel, font, "視野角", SettingType::Fov, 45.0, 120.0);
                spawn_toggle(panel, font, "VSync", SettingType::VSync);
                spawn_toggle(panel, font, "フルスクリーン", SettingType::Fullscreen);

                // Controls section
                spawn_section_header(panel, font, "操作");
                spawn_slider(
                    panel,
                    font,
                    "マウス感度",
                    SettingType::MouseSensitivity,
                    0.0001,
                    0.01,
                );
                spawn_toggle(panel, font, "Y軸反転", SettingType::InvertY);

                // Audio section
                spawn_section_header(panel, font, "音声");
                spawn_slider(
                    panel,
                    font,
                    "マスター音量",
                    SettingType::MasterVolume,
                    0.0,
                    1.0,
                );
                spawn_slider(panel, font, "効果音", SettingType::SfxVolume, 0.0, 1.0);
                spawn_slider(panel, font, "BGM", SettingType::MusicVolume, 0.0, 1.0);

                // Update section
                spawn_section_header(panel, font, "アップデート");
                spawn_update_row(panel, font, ui_registry);

                // Back button
                panel
                    .spawn((
                        Button,
                        SettingsBackButton,
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(20.0)),
                            align_self: AlignSelf::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                        BorderColor(Color::srgb(0.8, 0.5, 0.0)),
                        BorderRadius::all(Val::Px(6.0)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("戻る"),
                            text_font(font, TEXT_SECTION),
                            TextColor(Color::WHITE),
                        ));
                    });
            });
        });
}

fn spawn_section_header(parent: &mut ChildBuilder, font: &Handle<Font>, label: &str) {
    parent.spawn((
        Text::new(label),
        text_font(font, TEXT_SECTION),
        TextColor(Color::srgb(1.0, 0.8, 0.0)),
        Node {
            margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(15.0), Val::Px(5.0)),
            ..default()
        },
    ));
}

fn spawn_slider(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    setting: SettingType,
    min: f32,
    max: f32,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                text_font(font, TEXT_BODY),
                TextColor(Color::WHITE),
                Node {
                    width: Val::Px(120.0),
                    ..default()
                },
            ));

            // Slider track
            row.spawn((
                Button,
                SettingsSlider { setting, min, max },
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
                BorderRadius::all(Val::Px(4.0)),
            ))
            .with_children(|track| {
                // Fill
                track.spawn((
                    SettingsSliderFill,
                    SettingsValueText { setting },
                    Node {
                        width: Val::Percent(50.0), // Will be updated
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(1.0, 0.53, 0.0)),
                    BorderRadius::all(Val::Px(4.0)),
                ));
            });

            // Value text
            row.spawn((
                Text::new("0"),
                SettingsValueText { setting },
                text_font(font, TEXT_BODY),
                TextColor(Color::WHITE),
                Node {
                    width: Val::Px(60.0),
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                },
            ));
        });
}

fn spawn_update_row(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    ui_registry: &UIElementRegistry,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            // Status text
            row.spawn((
                SettingsUpdateStatusText,
                Text::new("確認中..."),
                text_font(font, TEXT_BODY),
                TextColor(Color::srgb(0.67, 0.67, 0.67)),
            ));

            // Update button
            row.spawn((
                Button,
                SettingsUpdateButton,
                ui_registry
                    .get_id("base:settings_update_button")
                    .map(UIElementTag::new)
                    .unwrap_or_else(|| UIElementTag::new(Default::default())),
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.25, 0.30)),
                BorderColor(Color::srgb(0.33, 0.33, 0.33)),
                BorderRadius::all(Val::Px(4.0)),
                Visibility::Hidden, // Hidden until update available
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("今すぐ更新"),
                    text_font(font, TEXT_BODY),
                    TextColor(Color::WHITE),
                ));
            });
        });
}

fn spawn_toggle(parent: &mut ChildBuilder, font: &Handle<Font>, label: &str, setting: SettingType) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                text_font(font, TEXT_BODY),
                TextColor(Color::WHITE),
            ));

            // Toggle button
            row.spawn((
                Button,
                SettingsToggle { setting },
                Node {
                    width: Val::Px(50.0),
                    height: Val::Px(26.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.15, 0.15)), // Will be updated
                BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                BorderRadius::all(Val::Px(13.0)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("OFF"),
                    SettingsValueText { setting },
                    text_font(font, TEXT_SMALL),
                    TextColor(Color::WHITE),
                ));
            });
        });
}

/// Update settings panel visibility based on UIState
pub fn update_settings_visibility(
    ui_state: Res<crate::components::UIState>,
    mut settings_query: Query<&mut Visibility, With<SettingsPanel>>,
) {
    let Ok(mut visibility) = settings_query.get_single_mut() else {
        return;
    };

    *visibility = if ui_state.is_active(&UIContext::Settings) {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

/// Update settings UI to reflect current values
pub fn update_settings_ui(
    settings: Res<GameSettings>,
    mut slider_fills: Query<(&mut Node, &SettingsValueText), With<SettingsSliderFill>>,
    mut value_texts: Query<(&mut Text, &SettingsValueText), Without<SettingsSliderFill>>,
    mut toggles: Query<(&SettingsToggle, &mut BackgroundColor), Without<SettingsSliderFill>>,
) {
    // Update slider fills and value texts
    for (mut node, value_text) in slider_fills.iter_mut() {
        let (value, min, max) = get_setting_value(&settings, value_text.setting);
        let percent = ((value - min) / (max - min)).clamp(0.0, 1.0) * 100.0;
        node.width = Val::Percent(percent);
    }

    // Update value text displays
    for (mut text, value_text) in value_texts.iter_mut() {
        let (value, _min, _max) = get_setting_value(&settings, value_text.setting);
        **text = format_setting_value(value_text.setting, value);
    }

    // Update toggle colors
    for (toggle, mut bg) in toggles.iter_mut() {
        let enabled = get_toggle_value(&settings, toggle.setting);
        *bg = if enabled {
            BackgroundColor(Color::srgb(0.15, 0.4, 0.15))
        } else {
            BackgroundColor(Color::srgb(0.3, 0.15, 0.15))
        };
    }
}

fn get_setting_value(settings: &GameSettings, setting: SettingType) -> (f32, f32, f32) {
    match setting {
        SettingType::MouseSensitivity => (settings.mouse_sensitivity, 0.0001, 0.01),
        SettingType::ViewDistance => (settings.view_distance as f32, 1.0, 8.0),
        SettingType::Fov => (settings.fov, 45.0, 120.0),
        SettingType::MasterVolume => (settings.master_volume, 0.0, 1.0),
        SettingType::SfxVolume => (settings.sfx_volume, 0.0, 1.0),
        SettingType::MusicVolume => (settings.music_volume, 0.0, 1.0),
        _ => (0.0, 0.0, 1.0),
    }
}

fn get_toggle_value(settings: &GameSettings, setting: SettingType) -> bool {
    match setting {
        SettingType::VSync => settings.vsync_enabled,
        SettingType::Fullscreen => settings.fullscreen,
        SettingType::InvertY => settings.invert_y,
        _ => false,
    }
}

fn format_setting_value(setting: SettingType, value: f32) -> String {
    match setting {
        SettingType::MouseSensitivity => format!("{:.4}", value),
        SettingType::ViewDistance => format!("{}", value as i32),
        SettingType::Fov => format!("{}°", value as i32),
        SettingType::MasterVolume | SettingType::SfxVolume | SettingType::MusicVolume => {
            format!("{}%", (value * 100.0) as i32)
        }
        SettingType::VSync | SettingType::Fullscreen | SettingType::InvertY => {
            if value > 0.5 {
                "ON".to_string()
            } else {
                "OFF".to_string()
            }
        }
    }
}

/// Handle slider drag start/end
#[allow(clippy::type_complexity)]
pub fn handle_slider_drag_state(
    mut drag_state: ResMut<SliderDragState>,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<SettingsSlider>)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    // Check for drag start
    for (entity, interaction) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            drag_state.dragging = Some(entity);
        }
    }

    // Check for drag end (mouse released)
    if !mouse_button.pressed(MouseButton::Left) {
        drag_state.dragging = None;
    }
}

/// Handle slider value updates during drag
pub fn handle_settings_sliders(
    drag_state: Res<SliderDragState>,
    slider_query: Query<(&SettingsSlider, &Node, &GlobalTransform)>,
    mut settings: ResMut<GameSettings>,
    mut settings_changed: EventWriter<crate::settings::SettingsChangedEvent>,
    windows: Query<&Window>,
) {
    // Only process when dragging
    let Some(dragging_entity) = drag_state.dragging else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Get the dragging slider
    let Ok((slider, node, transform)) = slider_query.get(dragging_entity) else {
        return;
    };

    // Calculate relative position within slider
    let slider_pos = transform.translation().truncate();
    let width = match node.width {
        Val::Px(w) => w,
        _ => 200.0,
    };

    let relative_x = cursor_pos.x - (slider_pos.x - width / 2.0);
    let percent = (relative_x / width).clamp(0.0, 1.0);
    let value = slider.min + percent * (slider.max - slider.min);

    // Update setting
    match slider.setting {
        SettingType::MouseSensitivity => settings.mouse_sensitivity = value,
        SettingType::ViewDistance => settings.view_distance = value.round() as i32,
        SettingType::Fov => settings.fov = value,
        SettingType::MasterVolume => settings.master_volume = value,
        SettingType::SfxVolume => settings.sfx_volume = value,
        SettingType::MusicVolume => settings.music_volume = value,
        _ => {}
    }

    settings.validate();
    settings_changed.send(crate::settings::SettingsChangedEvent);
}

/// Handle toggle interactions
pub fn handle_settings_toggles(
    mut interaction_query: Query<(&Interaction, &SettingsToggle), Changed<Interaction>>,
    mut settings: ResMut<GameSettings>,
    mut settings_changed: EventWriter<crate::settings::SettingsChangedEvent>,
) {
    for (interaction, toggle) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Toggle the setting
        match toggle.setting {
            SettingType::VSync => settings.vsync_enabled = !settings.vsync_enabled,
            SettingType::Fullscreen => settings.fullscreen = !settings.fullscreen,
            SettingType::InvertY => settings.invert_y = !settings.invert_y,
            _ => {}
        }

        settings_changed.send(crate::settings::SettingsChangedEvent);
    }
}

/// Handle back button
#[allow(clippy::type_complexity)]
pub fn handle_settings_back(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SettingsBackButton>),
    >,
    mut action_writer: EventWriter<crate::components::UIAction>,
) {
    for (interaction, mut bg) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                action_writer.send(crate::components::UIAction::Pop);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.95));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9));
            }
        }
    }
}

/// Update the update section UI based on UpdateState
pub fn update_settings_update_ui(
    ui_state: Res<crate::components::UIState>,
    state: Option<Res<UpdateState>>,
    mut status_query: Query<(&mut Text, &mut TextColor), With<SettingsUpdateStatusText>>,
    mut button_query: Query<&mut Visibility, With<SettingsUpdateButton>>,
) {
    // 設定画面がアクティブでない場合は、ボタンを非表示にして早期リターン
    // Bevy 0.15では Visibility::Visible は親の状態に関係なく表示されるため、
    // 明示的にHiddenに設定する必要がある
    if !ui_state.is_active(&crate::components::UIContext::Settings) {
        for mut vis in button_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    let Ok((mut text, mut color)) = status_query.get_single_mut() else {
        return;
    };

    let Some(state) = state else {
        text.0 = "アップデータ未初期化".to_string();
        return;
    };

    match &state.phase {
        UpdatePhase::Idle => {
            text.0 = "確認中...".to_string();
            *color = TextColor(Color::srgb(0.67, 0.67, 0.67));
        }
        UpdatePhase::Checking => {
            text.0 = "アップデートを確認中...".to_string();
            *color = TextColor(Color::srgb(0.67, 0.67, 0.67));
        }
        UpdatePhase::UpToDate => {
            text.0 = "✓ 最新バージョンです".to_string();
            *color = TextColor(Color::srgb(0.5, 0.9, 0.5));
        }
        UpdatePhase::Available { version, .. } => {
            text.0 = format!("v{} が利用可能", version);
            *color = TextColor(Color::srgb(1.0, 0.8, 0.0));
            // Show button
            for mut vis in button_query.iter_mut() {
                *vis = Visibility::Visible;
            }
            return; // Don't hide button below
        }
        UpdatePhase::Failed(error) => {
            if error.contains("ブラウザでダウンロードページを開きました") {
                text.0 = "✓ ブラウザで開きました".to_string();
                *color = TextColor(Color::srgb(0.5, 0.9, 0.5));
            } else {
                text.0 = format!("エラー: {}", error);
                *color = TextColor(Color::srgb(1.0, 0.4, 0.4));
            }
        }
    }

    // Hide button for non-available states
    for mut vis in button_query.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

/// Handle update button click
#[allow(clippy::type_complexity)]
pub fn handle_settings_update_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SettingsUpdateButton>),
    >,
    mut update_event: EventWriter<StartUpdateEvent>,
) {
    for (interaction, mut bg) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                update_event.send(StartUpdateEvent);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.35, 0.35, 0.42));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.25, 0.25, 0.30));
            }
        }
    }
}
