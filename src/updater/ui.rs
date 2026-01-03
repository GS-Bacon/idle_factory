//! Update notification UI components.

use bevy::prelude::*;

use super::state::{UpdatePhase, UpdateState};

// UI Colors (matching existing design rules)
const PANEL_BG: Color = Color::srgba(0.12, 0.12, 0.12, 0.95);
const TEXT_PRIMARY: Color = Color::WHITE;
const TEXT_SECONDARY: Color = Color::srgb(0.67, 0.67, 0.67);
const BUTTON_BG: Color = Color::srgb(0.25, 0.25, 0.30);
const BUTTON_HOVER: Color = Color::srgb(0.35, 0.35, 0.42);
const PROGRESS_BG: Color = Color::srgb(0.13, 0.13, 0.13);
const PROGRESS_FILL: Color = Color::srgb(0.30, 0.69, 0.31);
const BORDER_COLOR: Color = Color::srgb(0.33, 0.33, 0.33);

/// Marker component for the update notification UI root.
#[derive(Component)]
pub struct UpdateNotificationUI;

/// Marker for the progress bar fill.
#[derive(Component)]
pub struct UpdateProgressBar;

/// Marker for the status text.
#[derive(Component)]
pub struct UpdateStatusText;

/// Marker for the update button.
#[derive(Component)]
pub struct UpdateButton;

/// Marker for the dismiss button.
#[derive(Component)]
pub struct DismissButton;

/// Marker for the restart button.
#[derive(Component)]
pub struct RestartButton;

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
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Update Available"),
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

            // Progress bar container
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(PROGRESS_BG),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        UpdateProgressBar,
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(PROGRESS_FILL),
                    ));
                });

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
                    spawn_button(row, "Later", DismissButton);
                    // Update button
                    spawn_button(row, "Update Now", UpdateButton);
                    // Restart button (initially hidden)
                    spawn_button(row, "Restart", RestartButton);
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
    mut status_query: Query<&mut Text, With<UpdateStatusText>>,
    mut progress_query: Query<&mut Node, With<UpdateProgressBar>>,
    mut update_btn_query: Query<
        &mut Visibility,
        (
            With<UpdateButton>,
            Without<UpdateNotificationUI>,
            Without<RestartButton>,
            Without<DismissButton>,
        ),
    >,
    mut restart_btn_query: Query<
        &mut Visibility,
        (
            With<RestartButton>,
            Without<UpdateNotificationUI>,
            Without<UpdateButton>,
            Without<DismissButton>,
        ),
    >,
    mut dismiss_btn_query: Query<
        &mut Visibility,
        (
            With<DismissButton>,
            Without<UpdateNotificationUI>,
            Without<UpdateButton>,
            Without<RestartButton>,
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

            if let Ok(mut text) = status_query.get_single_mut() {
                text.0 = format!("New version v{} is available!", version);
            }

            // Show update and dismiss buttons, hide restart
            for mut vis in update_btn_query.iter_mut() {
                *vis = Visibility::Visible;
            }
            for mut vis in dismiss_btn_query.iter_mut() {
                *vis = Visibility::Visible;
            }
            for mut vis in restart_btn_query.iter_mut() {
                *vis = Visibility::Hidden;
            }

            // Reset progress bar
            if let Ok(mut node) = progress_query.get_single_mut() {
                node.width = Val::Percent(0.0);
            }
        }

        UpdatePhase::Downloading { downloaded, total } => {
            *ui_vis = Visibility::Visible;

            let percent = if *total > 0 {
                (*downloaded as f32 / *total as f32) * 100.0
            } else {
                0.0
            };

            if let Ok(mut text) = status_query.get_single_mut() {
                let downloaded_mb = *downloaded as f32 / 1_048_576.0;
                let total_mb = *total as f32 / 1_048_576.0;
                text.0 = format!(
                    "Downloading... {:.1}/{:.1} MB ({:.0}%)",
                    downloaded_mb, total_mb, percent
                );
            }

            if let Ok(mut node) = progress_query.get_single_mut() {
                node.width = Val::Percent(percent);
            }

            // Hide all buttons during download
            for mut vis in update_btn_query.iter_mut() {
                *vis = Visibility::Hidden;
            }
            for mut vis in dismiss_btn_query.iter_mut() {
                *vis = Visibility::Hidden;
            }
            for mut vis in restart_btn_query.iter_mut() {
                *vis = Visibility::Hidden;
            }
        }

        UpdatePhase::Installing => {
            *ui_vis = Visibility::Visible;

            if let Ok(mut text) = status_query.get_single_mut() {
                text.0 = "Installing update...".to_string();
            }

            if let Ok(mut node) = progress_query.get_single_mut() {
                node.width = Val::Percent(100.0);
            }
        }

        UpdatePhase::RequiresRestart => {
            *ui_vis = Visibility::Visible;

            if let Ok(mut text) = status_query.get_single_mut() {
                text.0 = "Update installed! Restart to apply.".to_string();
            }

            // Show only restart button
            for mut vis in update_btn_query.iter_mut() {
                *vis = Visibility::Hidden;
            }
            for mut vis in dismiss_btn_query.iter_mut() {
                *vis = Visibility::Hidden;
            }
            for mut vis in restart_btn_query.iter_mut() {
                *vis = Visibility::Visible;
            }
        }

        UpdatePhase::Failed(error) => {
            *ui_vis = Visibility::Visible;

            if let Ok(mut text) = status_query.get_single_mut() {
                text.0 = format!("Update failed: {}", error);
            }

            // Show dismiss button only
            for mut vis in update_btn_query.iter_mut() {
                *vis = Visibility::Hidden;
            }
            for mut vis in dismiss_btn_query.iter_mut() {
                *vis = Visibility::Visible;
            }
            for mut vis in restart_btn_query.iter_mut() {
                *vis = Visibility::Hidden;
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
            Or<(With<UpdateButton>, With<DismissButton>, With<RestartButton>)>,
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
