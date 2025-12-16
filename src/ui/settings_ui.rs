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

/// FPSスライダー
#[derive(Component)]
pub struct FpsSlider;

/// マウス感度スライダー
#[derive(Component)]
pub struct MouseSensitivitySlider;

/// 適用ボタン
#[derive(Component)]
pub struct ApplyButton;

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
                handle_apply_button,
                handle_close_button,
            ).run_if(not(in_state(SettingsUiState::Closed))));
    }
}

/// Escキーで設定ボタンを表示/非表示
fn handle_escape_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<SettingsUiState>>,
    mut next_state: ResMut<NextState<SettingsUiState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.get() {
            SettingsUiState::Closed => {
                next_state.set(SettingsUiState::ButtonVisible);
            }
            SettingsUiState::ButtonVisible => {
                next_state.set(SettingsUiState::Closed);
            }
            SettingsUiState::SettingsOpen => {
                next_state.set(SettingsUiState::Closed);
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
                    Text::new(format!("Max FPS: {}", config.max_fps as u32)),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                ));

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
                    Text::new(format!("Mouse Sensitivity: {:.3}", config.mouse_sensitivity)),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                parent.spawn((
                    Text::new("(0.001 ~ 0.01)"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                ));
            });

            // ボタン
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|parent| {
                // Applyボタン
                parent.spawn((
                    ApplyButton,
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(15.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.6, 0.3)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Apply"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

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

/// Applyボタンのクリック処理
fn handle_apply_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ApplyButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Apply settings (not yet implemented)");
            // TODO: 設定値を保存
        }
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
