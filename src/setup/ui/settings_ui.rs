//! Settings UI setup and systems

use bevy::prelude::*;

use crate::components::UIContext;
use crate::game_spec::{UIElementRegistry, UIElementTag};
use crate::settings::GameSettings;
use crate::setup::ui::text_font;
use crate::setup::ui::tokens::{color, font, size};
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
                    border_radius: BorderRadius::all(size::BORDER_RADIUS_MD),
                    ..default()
                },
                BackgroundColor(color::PANEL_SETTINGS),
                BorderColor::all(color::ACCENT),
            ))
            .with_children(|panel| {
                // Title
                panel.spawn((
                    Text::new("設定"),
                    text_font(font, font::HEADING),
                    TextColor(color::TEXT),
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
                            width: size::BUTTON_WIDTH_MD,
                            height: size::BUTTON_HEIGHT_MD,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(20.0)),
                            align_self: AlignSelf::Center,
                            border: UiRect::all(size::BORDER_MEDIUM),
                            border_radius: BorderRadius::all(size::BORDER_RADIUS_MD),
                            ..default()
                        },
                        BackgroundColor(color::BUTTON_PAUSE),
                        BorderColor::all(color::BORDER_ACCENT),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("戻る"),
                            text_font(font, font::SECTION),
                            TextColor(color::TEXT),
                        ));
                    });
            });
        });
}

fn spawn_section_header(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, label: &str) {
    parent.spawn((
        Text::new(label),
        text_font(font, font::SECTION),
        TextColor(color::ACCENT_GOLD),
        Node {
            margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(15.0), Val::Px(5.0)),
            ..default()
        },
    ));
}

fn spawn_slider(
    parent: &mut ChildSpawnerCommands,
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
                text_font(font, font::BODY),
                TextColor(color::TEXT),
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
                    width: size::SLIDER_WIDTH,
                    height: size::SLIDER_HEIGHT,
                    border_radius: BorderRadius::all(size::BORDER_RADIUS_SM),
                    ..default()
                },
                BackgroundColor(color::SLIDER_TRACK),
            ))
            .with_children(|track| {
                // Fill
                track.spawn((
                    SettingsSliderFill,
                    SettingsValueText { setting },
                    Node {
                        width: Val::Percent(50.0), // Will be updated
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(size::BORDER_RADIUS_SM),
                        ..default()
                    },
                    BackgroundColor(color::ACCENT),
                ));
            });

            // Value text
            row.spawn((
                Text::new("0"),
                SettingsValueText { setting },
                text_font(font, font::BODY),
                TextColor(color::TEXT),
                Node {
                    width: Val::Px(60.0),
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                },
            ));
        });
}

fn spawn_update_row(
    parent: &mut ChildSpawnerCommands,
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
                text_font(font, font::BODY),
                TextColor(color::TEXT_CHECKING),
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
                    border: UiRect::all(size::BORDER_THIN),
                    border_radius: BorderRadius::all(size::BORDER_RADIUS_SM),
                    ..default()
                },
                BackgroundColor(color::BUTTON_UPDATE),
                BorderColor::all(color::BORDER_UPDATE),
                Visibility::Hidden, // Hidden until update available
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("今すぐ更新"),
                    text_font(font, font::BODY),
                    TextColor(color::TEXT),
                ));
            });
        });
}

fn spawn_toggle(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    setting: SettingType,
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
                text_font(font, font::BODY),
                TextColor(color::TEXT),
            ));

            // Toggle button
            row.spawn((
                Button,
                SettingsToggle { setting },
                Node {
                    width: size::TOGGLE_WIDTH,
                    height: size::TOGGLE_HEIGHT,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(size::BORDER_MEDIUM),
                    border_radius: BorderRadius::all(size::BORDER_RADIUS_PILL),
                    ..default()
                },
                BackgroundColor(color::TOGGLE_OFF), // Will be updated
                BorderColor::all(color::BORDER_INACTIVE),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("OFF"),
                    SettingsValueText { setting },
                    text_font(font, font::SMALL),
                    TextColor(color::TEXT),
                ));
            });
        });
}

/// Update settings panel visibility based on UIState
pub fn update_settings_visibility(
    ui_state: Res<crate::components::UIState>,
    mut settings_query: Query<&mut Visibility, With<SettingsPanel>>,
) {
    let Ok(mut visibility) = settings_query.single_mut() else {
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
            BackgroundColor(color::TOGGLE_ON)
        } else {
            BackgroundColor(color::TOGGLE_OFF)
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
    mut settings_changed: MessageWriter<crate::settings::SettingsChangedEvent>,
    windows: Query<&Window>,
) {
    // Only process when dragging
    let Some(dragging_entity) = drag_state.dragging else {
        return;
    };

    let Ok(window) = windows.single() else {
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
    settings_changed.write(crate::settings::SettingsChangedEvent);
}

/// Handle toggle interactions
pub fn handle_settings_toggles(
    mut interaction_query: Query<(&Interaction, &SettingsToggle), Changed<Interaction>>,
    mut settings: ResMut<GameSettings>,
    mut settings_changed: MessageWriter<crate::settings::SettingsChangedEvent>,
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

        settings_changed.write(crate::settings::SettingsChangedEvent);
    }
}

/// Handle back button
#[allow(clippy::type_complexity)]
pub fn handle_settings_back(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SettingsBackButton>),
    >,
    mut action_writer: MessageWriter<crate::components::UIAction>,
) {
    for (interaction, mut bg) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                action_writer.write(crate::components::UIAction::Pop);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(color::BUTTON_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(color::BUTTON_PAUSE);
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

    let Ok((mut text, mut text_color)) = status_query.single_mut() else {
        return;
    };

    let Some(state) = state else {
        text.0 = "アップデータ未初期化".to_string();
        return;
    };

    match &state.phase {
        UpdatePhase::Idle => {
            text.0 = "確認中...".to_string();
            *text_color = TextColor(color::TEXT_CHECKING);
        }
        UpdatePhase::Checking => {
            text.0 = "アップデートを確認中...".to_string();
            *text_color = TextColor(color::TEXT_CHECKING);
        }
        UpdatePhase::UpToDate => {
            text.0 = "✓ 最新バージョンです".to_string();
            *text_color = TextColor(color::SUCCESS);
        }
        UpdatePhase::Available { version, .. } => {
            text.0 = format!("v{} が利用可能", version);
            *text_color = TextColor(color::WARNING);
            // Show button
            for mut vis in button_query.iter_mut() {
                *vis = Visibility::Visible;
            }
            return; // Don't hide button below
        }
        UpdatePhase::Failed(error) => {
            if error.contains("ブラウザでダウンロードページを開きました") {
                text.0 = "✓ ブラウザで開きました".to_string();
                *text_color = TextColor(color::SUCCESS);
            } else {
                text.0 = format!("エラー: {}", error);
                *text_color = TextColor(color::DANGER);
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
    mut update_event: MessageWriter<StartUpdateEvent>,
) {
    for (interaction, mut bg) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                update_event.write(StartUpdateEvent);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(color::BUTTON_UPDATE_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(color::BUTTON_UPDATE);
            }
        }
    }
}
