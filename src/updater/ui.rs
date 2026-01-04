//! Update notification UI components.

use bevy::prelude::*;

use super::state::{UpdatePhase, UpdateState};

// UI Colors (matching existing design rules)
const PANEL_BG: Color = Color::srgba(0.12, 0.12, 0.12, 0.95);
const TEXT_PRIMARY: Color = Color::WHITE;
const TEXT_SECONDARY: Color = Color::srgb(0.67, 0.67, 0.67);
const TEXT_ERROR: Color = Color::srgb(1.0, 0.4, 0.4);
const BUTTON_BG: Color = Color::srgb(0.25, 0.25, 0.30);
const BUTTON_HOVER: Color = Color::srgb(0.35, 0.35, 0.42);
const BORDER_COLOR: Color = Color::srgb(0.33, 0.33, 0.33);

/// Marker component for the update notification UI root.
#[derive(Component)]
pub struct UpdateNotificationUI;

/// Marker for the status text.
#[derive(Component)]
pub struct UpdateStatusText;

/// Marker for the update button.
#[derive(Component)]
pub struct UpdateButton;

/// Marker for the dismiss button.
#[derive(Component)]
pub struct DismissButton;

/// Spawn the update notification UI.
pub fn spawn_update_ui(mut commands: Commands) {
    commands
        .spawn((
            UpdateNotificationUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                width: Val::Px(320.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            BorderColor(BORDER_COLOR),
            BorderRadius::all(Val::Px(8.0)),
            GlobalZIndex(110), // Above PauseUI (100) so update button is clickable during pause
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("🔄 アップデート"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));

            // Status text
            parent.spawn((
                UpdateStatusText,
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
            ));

            // Button row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                },))
                .with_children(|row| {
                    // Dismiss button
                    spawn_button(row, "後で", DismissButton);
                    // Update button
                    spawn_button(row, "今すぐ更新", UpdateButton);
                });
        });
}

fn spawn_button<T: Component>(parent: &mut ChildBuilder, label: &str, marker: T) {
    parent
        .spawn((
            Button,
            marker,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(BUTTON_BG),
            BorderColor(BORDER_COLOR),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));
        });
}

/// Update the notification UI based on current state.
#[allow(clippy::type_complexity)]
pub fn update_notification_ui(
    state: Res<UpdateState>,
    mut ui_query: Query<&mut Visibility, With<UpdateNotificationUI>>,
    mut status_query: Query<(&mut Text, &mut TextColor), With<UpdateStatusText>>,
    mut update_btn_query: Query<
        &mut Visibility,
        (
            With<UpdateButton>,
            Without<UpdateNotificationUI>,
            Without<DismissButton>,
        ),
    >,
    mut dismiss_btn_query: Query<
        &mut Visibility,
        (
            With<DismissButton>,
            Without<UpdateNotificationUI>,
            Without<UpdateButton>,
        ),
    >,
) {
    let Ok(mut ui_vis) = ui_query.get_single_mut() else {
        return;
    };

    match &state.phase {
        UpdatePhase::Idle | UpdatePhase::Checking | UpdatePhase::UpToDate => {
            *ui_vis = Visibility::Hidden;
        }

        UpdatePhase::Available { version, .. } => {
            *ui_vis = Visibility::Visible;

            if let Ok((mut text, mut color)) = status_query.get_single_mut() {
                text.0 = format!("新バージョン v{} が利用可能です！", version);
                *color = TextColor(TEXT_SECONDARY);
            }

            // Show update and dismiss buttons
            for mut vis in update_btn_query.iter_mut() {
                *vis = Visibility::Visible;
            }
            for mut vis in dismiss_btn_query.iter_mut() {
                *vis = Visibility::Visible;
            }
        }

        UpdatePhase::Failed(error) => {
            *ui_vis = Visibility::Visible;

            if let Ok((mut text, mut color)) = status_query.get_single_mut() {
                // Make error message more user-friendly
                let (user_message, is_success) = if error
                    .contains("ブラウザでダウンロードページを開きました")
                {
                    // Browser was opened successfully - show as info rather than error
                    ("✓ ブラウザでダウンロードページを\n開きました。手動でダウンロード\nしてください。".to_string(), true)
                } else if error.contains("Updater not found") {
                    ("アップデータが見つかりません。".to_string(), false)
                } else {
                    (format!("更新失敗: {}", error), false)
                };
                text.0 = user_message;
                // Use green-ish color for success (browser opened), red for errors
                *color = if is_success {
                    TextColor(Color::srgb(0.5, 0.9, 0.5))
                } else {
                    TextColor(TEXT_ERROR)
                };
            }

            // Show dismiss button only
            for mut vis in update_btn_query.iter_mut() {
                *vis = Visibility::Hidden;
            }
            for mut vis in dismiss_btn_query.iter_mut() {
                *vis = Visibility::Visible;
            }
        }
    }
}

/// Handle button hover effects.
#[allow(clippy::type_complexity)]
pub fn handle_button_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            Or<(With<UpdateButton>, With<DismissButton>)>,
        ),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(BUTTON_HOVER),
            Interaction::None => BackgroundColor(BUTTON_BG),
        };
    }
}
