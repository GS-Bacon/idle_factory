//! Auto-updater module for native builds.
//!
//! Uses external updater binary for actual downloads/installation.
//! The main game only checks for updates and launches the updater if needed.
//!
//! This module is only compiled for native targets (not WASM).

mod checker;
mod state;
mod ui;

pub use checker::CURRENT_VERSION;
pub use state::{
    CancelUpdateEvent, CheckForUpdateEvent, RestartAppEvent, StartUpdateEvent, UpdateCheckResult,
    UpdatePhase, UpdateState,
};

use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use crossbeam_channel::{Receiver, Sender};

use self::state::UpdateCheckCompleteEvent;

/// Plugin for automatic updates.
///
/// Only active on native platforms (Linux, Windows).
pub struct UpdaterPlugin;

impl Plugin for UpdaterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpdateState>()
            .init_resource::<UpdateChannels>()
            .add_event::<CheckForUpdateEvent>()
            .add_event::<StartUpdateEvent>()
            .add_event::<CancelUpdateEvent>()
            .add_event::<RestartAppEvent>()
            .add_event::<UpdateCheckCompleteEvent>()
            .add_systems(Startup, setup_updater)
            .add_systems(Startup, ui::spawn_update_ui)
            .add_systems(
                Update,
                (
                    startup_check_timer,
                    handle_check_event,
                    poll_check_result,
                    handle_start_update,
                    handle_dismiss_button,
                    handle_update_button,
                    ui::update_notification_ui,
                    ui::handle_button_hover,
                ),
            );
    }
}

/// Resource for communicating between async tasks and Bevy systems.
#[derive(Resource, Default)]
struct UpdateChannels {
    check_rx: Option<Receiver<UpdateCheckResult>>,
}

/// Timer for delayed startup check.
#[derive(Resource)]
struct StartupCheckTimer(Timer);

/// Setup the updater system.
fn setup_updater(mut commands: Commands) {
    // Delay the first update check by 5 seconds to let the game initialize
    commands.insert_resource(StartupCheckTimer(Timer::from_seconds(5.0, TimerMode::Once)));
    tracing::info!("Updater initialized (v{})", CURRENT_VERSION);
}

/// Trigger update check after startup delay.
fn startup_check_timer(
    mut timer: Option<ResMut<StartupCheckTimer>>,
    time: Res<Time>,
    mut events: EventWriter<CheckForUpdateEvent>,
    mut commands: Commands,
) {
    let Some(ref mut timer) = timer else {
        return;
    };

    if timer.0.tick(time.delta()).just_finished() {
        tracing::info!("Startup check timer finished, triggering update check");
        events.send(CheckForUpdateEvent);
        commands.remove_resource::<StartupCheckTimer>();
    }
}

/// Handle update check events.
fn handle_check_event(
    mut events: EventReader<CheckForUpdateEvent>,
    mut state: ResMut<UpdateState>,
    mut channels: ResMut<UpdateChannels>,
) {
    for _ in events.read() {
        // Rate limiting
        if !state.can_check() {
            tracing::debug!("Skipping update check (rate limited)");
            continue;
        }

        // Don't start a new check if one is in progress
        if matches!(state.phase, UpdatePhase::Checking) {
            tracing::debug!("Update check already in progress");
            continue;
        }

        tracing::info!("Starting update check...");
        state.phase = UpdatePhase::Checking;
        state.last_check = Some(std::time::Instant::now());

        // Spawn background task
        let (tx, rx): (Sender<UpdateCheckResult>, Receiver<UpdateCheckResult>) =
            crossbeam_channel::unbounded();
        channels.check_rx = Some(rx);

        IoTaskPool::get()
            .spawn(async move {
                let result = checker::check_for_update();
                let _ = tx.send(result);
            })
            .detach();
    }
}

/// Poll for update check results.
fn poll_check_result(mut channels: ResMut<UpdateChannels>, mut state: ResMut<UpdateState>) {
    let Some(ref rx) = channels.check_rx else {
        return;
    };

    // Non-blocking receive
    if let Ok(result) = rx.try_recv() {
        channels.check_rx = None;

        match result {
            UpdateCheckResult::Available {
                version,
                release_notes,
                download_url,
            } => {
                tracing::info!("Update available: v{}", version);
                state.phase = UpdatePhase::Available {
                    version,
                    release_notes,
                    download_url,
                };
            }
            UpdateCheckResult::UpToDate => {
                tracing::info!("Already up to date");
                state.phase = UpdatePhase::UpToDate;
            }
            UpdateCheckResult::Error(e) => {
                tracing::warn!("Update check failed: {}", e);
                state.phase = UpdatePhase::Failed(e);
            }
        }
    }
}

/// Handle start update events - launch external updater and exit game.
fn handle_start_update(
    mut events: EventReader<StartUpdateEvent>,
    state: Res<UpdateState>,
    mut exit: EventWriter<AppExit>,
) {
    for _ in events.read() {
        // Only proceed if update is available
        let (version, download_url) = match &state.phase {
            UpdatePhase::Available {
                version,
                download_url,
                ..
            } => (version.clone(), download_url.clone()),
            _ => continue,
        };

        tracing::info!("Launching external updater for v{}...", version);

        // Find the updater executable
        let updater_path = match std::env::current_exe() {
            Ok(exe) => {
                let dir = exe.parent().unwrap_or(std::path::Path::new("."));
                if cfg!(windows) {
                    dir.join("updater.exe")
                } else {
                    dir.join("updater")
                }
            }
            Err(e) => {
                tracing::error!("Failed to get executable path: {}", e);
                continue;
            }
        };

        if !updater_path.exists() {
            tracing::error!("Updater not found at {:?}", updater_path);
            // Fallback: open download URL in browser
            if let Err(e) = open::that(&download_url) {
                tracing::error!("Failed to open browser: {}", e);
            }
            continue;
        }

        // Launch updater with version and download URL as arguments
        match std::process::Command::new(&updater_path)
            .arg(&version)
            .arg(&download_url)
            .spawn()
        {
            Ok(_) => {
                tracing::info!("Updater launched, exiting game...");
                exit.send(AppExit::Success);
            }
            Err(e) => {
                tracing::error!("Failed to launch updater: {}", e);
                // Fallback: open download URL in browser
                if let Err(e) = open::that(&download_url) {
                    tracing::error!("Failed to open browser: {}", e);
                }
            }
        }
    }
}

/// Handle dismiss button clicks.
fn handle_dismiss_button(
    query: Query<&Interaction, (Changed<Interaction>, With<ui::DismissButton>)>,
    mut state: ResMut<UpdateState>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            tracing::info!("Update dismissed by user");
            state.phase = UpdatePhase::Idle;
        }
    }
}

/// Handle update button clicks.
fn handle_update_button(
    query: Query<&Interaction, (Changed<Interaction>, With<ui::UpdateButton>)>,
    mut events: EventWriter<StartUpdateEvent>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            tracing::info!("Update button clicked");
            events.send(StartUpdateEvent);
        }
    }
}
