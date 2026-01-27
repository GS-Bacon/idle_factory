//! Breaking progress bar UI

use crate::components::{BreakingProgress, BreakingProgressBarFill, BreakingProgressUI};
use bevy::prelude::*;

/// Spawn the breaking progress bar UI (hidden by default)
pub fn spawn_breaking_progress_ui(mut commands: Commands) {
    // Progress bar container - centered on screen, slightly above crosshair
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(45.0), // Slightly above center
                width: Val::Px(200.0),
                height: Val::Px(10.0),
                margin: UiRect::left(Val::Px(-100.0)), // Center horizontally
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            Visibility::Hidden,
            BreakingProgressUI,
        ))
        .with_children(|parent| {
            // Progress bar fill
            parent.spawn((
                Node {
                    width: Val::Percent(0.0), // Will be updated based on progress
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                BreakingProgressBarFill,
            ));
        });
}

/// Update the breaking progress bar UI based on breaking progress
pub fn update_breaking_progress_ui(
    breaking_progress: Res<BreakingProgress>,
    mut container_query: Query<&mut Visibility, With<BreakingProgressUI>>,
    mut fill_query: Query<&mut Node, With<BreakingProgressBarFill>>,
) {
    let Ok(mut container_visibility) = container_query.single_mut() else {
        return;
    };

    if breaking_progress.is_breaking() && breaking_progress.progress < 1.0 {
        // Show progress bar
        *container_visibility = Visibility::Visible;

        // Update fill width
        if let Ok(mut fill_node) = fill_query.single_mut() {
            fill_node.width = Val::Percent(breaking_progress.progress * 100.0);
        }
    } else {
        // Hide progress bar
        *container_visibility = Visibility::Hidden;
    }
}
