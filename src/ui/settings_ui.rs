// src/ui/settings_ui.rs
//! 設定UIシステム
//! - Escキーで設定ボタン表示
//! - 設定ボタンクリックで設定画面を開く
//! - FPS、マウス感度などを調整可能

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use crate::core::config::GameConfig;

/// 設定UIのステート
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum SettingsUiState {
    #[default]
    Closed,          // 設定UI非表示
    ButtonVisible,   // 設定ボタンのみ表示（Esc押下時）
    SettingsOpen,    // 設定画面表示
}

/// 設定ボタンマーカー
#[derive(Component)]
pub struct SettingsButton;

/// 設定画面ルート
#[derive(Component)]
pub struct SettingsUiRoot;

/// FPS増加ボタン
#[derive(Component)]
pub struct FpsIncreaseButton;

/// FPS減少ボタン
#[derive(Component)]
pub struct FpsDecreaseButton;

/// FPS値表示
#[derive(Component)]
pub struct FpsValueText;

/// マウス感度増加ボタン
#[derive(Component)]
pub struct SensitivityIncreaseButton;

/// マウス感度減少ボタン
#[derive(Component)]
pub struct SensitivityDecreaseButton;

/// マウス感度値表示
#[derive(Component)]
pub struct SensitivityValueText;

/// ハイライト有効/無効トグル
#[derive(Component)]
pub struct HighlightToggleButton;

/// UIブラー有効/無効トグル
#[derive(Component)]
pub struct UiBlurToggleButton;

/// 閉じるボタン
#[derive(Component)]
pub struct CloseButton;

pub struct SettingsUiPlugin;

impl Plugin for SettingsUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<SettingsUiState>()
            .add_systems(Update, handle_escape_key)
            .add_systems(OnEnter(SettingsUiState::ButtonVisible), spawn_settings_button)
            .add_systems(OnExit(SettingsUiState::ButtonVisible), despawn_settings_button)
            .add_systems(OnEnter(SettingsUiState::SettingsOpen), (spawn_settings_ui, release_cursor))
            .add_systems(OnExit(SettingsUiState::SettingsOpen), despawn_settings_ui)
            .add_systems(Update, (
                handle_settings_button,
                handle_close_button,
                handle_fps_buttons,
                handle_sensitivity_buttons,
                handle_toggle_buttons,
            ).run_if(not(in_state(SettingsUiState::Closed))));
    }
}

/// Escキーで設定ボタンを表示/非表示
fn handle_escape_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<SettingsUiState>>,
    mut next_state: ResMut<NextState<SettingsUiState>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.get() {
            SettingsUiState::Closed => {
                next_state.set(SettingsUiState::ButtonVisible);
            }
            SettingsUiState::ButtonVisible => {
                next_state.set(SettingsUiState::Closed);
                // カーソルを自動的にグラブして通常プレイに戻る
                if let Ok(mut window) = window_query.get_single_mut() {
                    window.cursor_options.grab_mode = CursorGrabMode::Locked;
                    window.cursor_options.visible = false;
                }
            }
            SettingsUiState::SettingsOpen => {
                next_state.set(SettingsUiState::Closed);
                // カーソルを自動的にグラブして通常プレイに戻る
                if let Ok(mut window) = window_query.get_single_mut() {
                    window.cursor_options.grab_mode = CursorGrabMode::Locked;
                    window.cursor_options.visible = false;
                }
            }
        }
    }
}

fn release_cursor(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

/// 設定ボタンを表示
fn spawn_settings_button(mut commands: Commands) {
    commands.spawn((
        SettingsButton,
        Button,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            right: Val::Px(20.0),
            padding: UiRect::all(Val::Px(15.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.9)),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Settings"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });

    info!("Settings button spawned");
}

fn despawn_settings_button(
    mut commands: Commands,
    query: Query<Entity, With<SettingsButton>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// 設定ボタンのクリック処理
fn handle_settings_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>,
    mut next_state: ResMut<NextState<SettingsUiState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(SettingsUiState::SettingsOpen);
        }
    }
}

/// 設定UI画面を生成
fn spawn_settings_ui(
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    commands.spawn((
        SettingsUiRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    ))
    .with_children(|parent| {
        // 設定パネル
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(20.0),
                width: Val::Px(500.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        ))
        .with_children(|parent| {
            // タイトル
            parent.spawn((
                Text::new("Settings"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::WHITE),
            ));

            // FPS設定
            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Max FPS"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                // FPS調整ボタン
                parent.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    // 減少ボタン
                    parent.spawn((
                        FpsDecreaseButton,
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new("-"),
                            TextFont { font_size: 24.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // 値表示
                    parent.spawn((
                        FpsValueText,
                        Text::new(format!("{}", config.max_fps as u32)),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                        Node {
                            width: Val::Px(80.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ));

                    // 増加ボタン
                    parent.spawn((
                        FpsIncreaseButton,
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new("+"),
                            TextFont { font_size: 24.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                });

                parent.spawn((
                    Text::new("(30, 60, 120, 144, 240)"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                ));
            });

            // マウス感度設定
            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Mouse Sensitivity"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                // マウス感度調整ボタン
                parent.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    // 減少ボタン
                    parent.spawn((
                        SensitivityDecreaseButton,
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new("-"),
                            TextFont { font_size: 24.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // 値表示
                    parent.spawn((
                        SensitivityValueText,
                        Text::new(format!("{:.4}", config.mouse_sensitivity)),
                        TextFont { font_size: 24.0, ..default() },
                        TextColor(Color::WHITE),
                        Node {
                            width: Val::Px(80.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ));

                    // 増加ボタン
                    parent.spawn((
                        SensitivityIncreaseButton,
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new("+"),
                            TextFont { font_size: 24.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                });

                parent.spawn((
                    Text::new("(0.001 ~ 0.01)"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                ));
            });

            // ハイライトトグル設定
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Enable Highlight"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                parent.spawn((
                    HighlightToggleButton,
                    Button,
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if config.enable_highlight {
                        Color::srgb(0.3, 0.7, 0.3) // 緑（ON）
                    } else {
                        Color::srgb(0.6, 0.3, 0.3) // 赤（OFF）
                    }),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(if config.enable_highlight { "ON" } else { "OFF" }),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            });

            // UIブラートグル設定
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Enable UI Blur"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                parent.spawn((
                    UiBlurToggleButton,
                    Button,
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if config.enable_ui_blur {
                        Color::srgb(0.3, 0.7, 0.3) // 緑（ON）
                    } else {
                        Color::srgb(0.6, 0.3, 0.3) // 赤（OFF）
                    }),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(if config.enable_ui_blur { "ON" } else { "OFF" }),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            });

            // ボタン
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|parent| {
                // Closeボタン
                parent.spawn((
                    CloseButton,
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(15.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.6, 0.3, 0.3)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Close"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });
    });

    info!("Settings UI spawned");
}

fn despawn_settings_ui(
    mut commands: Commands,
    query: Query<Entity, With<SettingsUiRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Closeボタンのクリック処理
fn handle_close_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CloseButton>)>,
    mut next_state: ResMut<NextState<SettingsUiState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(SettingsUiState::Closed);
        }
    }
}

/// FPS調整ボタンのクリック処理
fn handle_fps_buttons(
    increase_query: Query<&Interaction, (Changed<Interaction>, With<FpsIncreaseButton>)>,
    decrease_query: Query<&Interaction, (Changed<Interaction>, With<FpsDecreaseButton>)>,
    mut text_query: Query<&mut Text, With<FpsValueText>>,
    mut config: ResMut<GameConfig>,
) {
    let fps_options = [30.0, 60.0, 120.0, 144.0, 240.0];

    for interaction in &increase_query {
        if *interaction == Interaction::Pressed {
            let current_index = fps_options.iter().position(|&x| x == config.max_fps).unwrap_or(1);
            let new_index = (current_index + 1).min(fps_options.len() - 1);
            config.max_fps = fps_options[new_index];

            if let Ok(mut text) = text_query.get_single_mut() {
                **text = format!("{}", config.max_fps as u32);
            }
        }
    }

    for interaction in &decrease_query {
        if *interaction == Interaction::Pressed {
            let current_index = fps_options.iter().position(|&x| x == config.max_fps).unwrap_or(1);
            let new_index = current_index.saturating_sub(1);
            config.max_fps = fps_options[new_index];

            if let Ok(mut text) = text_query.get_single_mut() {
                **text = format!("{}", config.max_fps as u32);
            }
        }
    }
}

/// マウス感度調整ボタンのクリック処理
fn handle_sensitivity_buttons(
    increase_query: Query<&Interaction, (Changed<Interaction>, With<SensitivityIncreaseButton>)>,
    decrease_query: Query<&Interaction, (Changed<Interaction>, With<SensitivityDecreaseButton>)>,
    mut text_query: Query<&mut Text, With<SensitivityValueText>>,
    mut config: ResMut<GameConfig>,
) {
    const STEP: f32 = 0.0005;
    const MIN: f32 = 0.001;
    const MAX: f32 = 0.01;

    for interaction in &increase_query {
        if *interaction == Interaction::Pressed {
            config.mouse_sensitivity = (config.mouse_sensitivity + STEP).min(MAX);

            if let Ok(mut text) = text_query.get_single_mut() {
                **text = format!("{:.4}", config.mouse_sensitivity);
            }
        }
    }

    for interaction in &decrease_query {
        if *interaction == Interaction::Pressed {
            config.mouse_sensitivity = (config.mouse_sensitivity - STEP).max(MIN);

            if let Ok(mut text) = text_query.get_single_mut() {
                **text = format!("{:.4}", config.mouse_sensitivity);
            }
        }
    }
}

/// トグルボタンのクリック処理
fn handle_toggle_buttons(
    highlight_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<HighlightToggleButton>)>,
    blur_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<UiBlurToggleButton>)>,
    mut text_query: Query<&mut Text>,
    mut bg_query: Query<&mut BackgroundColor>,
    mut config: ResMut<GameConfig>,
) {
    for (interaction, children) in &highlight_query {
        if *interaction == Interaction::Pressed {
            config.enable_highlight = !config.enable_highlight;

            // ボタンの背景色を更新
            if let Ok(mut bg) = bg_query.get_mut(children[0]) {
                *bg = BackgroundColor(if config.enable_highlight {
                    Color::srgb(0.3, 0.7, 0.3)
                } else {
                    Color::srgb(0.6, 0.3, 0.3)
                });
            }

            // ボタンのテキストを更新
            if let Ok(mut text) = text_query.get_mut(children[0]) {
                **text = if config.enable_highlight { "ON".to_string() } else { "OFF".to_string() };
            }
        }
    }

    for (interaction, children) in &blur_query {
        if *interaction == Interaction::Pressed {
            config.enable_ui_blur = !config.enable_ui_blur;

            // ボタンの背景色を更新
            if let Ok(mut bg) = bg_query.get_mut(children[0]) {
                *bg = BackgroundColor(if config.enable_ui_blur {
                    Color::srgb(0.3, 0.7, 0.3)
                } else {
                    Color::srgb(0.6, 0.3, 0.3)
                });
            }

            // ボタンのテキストを更新
            if let Ok(mut text) = text_query.get_mut(children[0]) {
                **text = if config.enable_ui_blur { "ON".to_string() } else { "OFF".to_string() };
            }
        }
    }
}
