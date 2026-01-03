//! Auto-updater module for native builds.
//!
//! Provides automatic update checking, downloading, and installation
//! using GitHub Releases as the backend.
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

use self::state::{DownloadProgressEvent, UpdateCheckCompleteEvent, UpdateInstalledEvent};

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
            .add_event::<DownloadProgressEvent>()
            .add_event::<UpdateInstalledEvent>()
            .add_systems(Startup, setup_updater)
            .add_systems(Startup, ui::spawn_update_ui)
            .add_systems(
                Update,
                (
                    startup_check_timer,
                    handle_check_event,
                    poll_check_result,
                    handle_update_event,
                    poll_update_result,
                    handle_dismiss_button,
                    handle_update_button,
                    handle_restart_button,
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
    update_rx: Option<Receiver<Result<(), String>>>,
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
fn poll_check_result(
    mut channels: ResMut<UpdateChannels>,
    mut state: ResMut<UpdateState>,
    mut events: EventWriter<StartUpdateEvent>,
) {
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

                // Auto-download if enabled
                if state.auto_download {
                    events.send(StartUpdateEvent);
                }
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

/// Handle start update events.
#[allow(clippy::type_complexity)]
fn handle_update_event(
    mut events: EventReader<StartUpdateEvent>,
    mut state: ResMut<UpdateState>,
    mut channels: ResMut<UpdateChannels>,
) {
    for _ in events.read() {
        // Only proceed if update is available
        if !matches!(state.phase, UpdatePhase::Available { .. }) {
            continue;
        }

        tracing::info!("Starting update download...");
        state.phase = UpdatePhase::Downloading {
            downloaded: 0,
            total: 0,
        };

        // Spawn background task
        let (tx, rx): (Sender<Result<(), String>>, Receiver<Result<(), String>>) =
            crossbeam_channel::unbounded();
        channels.update_rx = Some(rx);

        IoTaskPool::get()
            .spawn(async move {
                let result = checker::perform_update();
                let _ = tx.send(result);
            })
            .detach();
    }
}

/// Poll for update results.
fn poll_update_result(mut channels: ResMut<UpdateChannels>, mut state: ResMut<UpdateState>) {
    let Some(ref rx) = channels.update_rx else {
        return;
    };

    // Non-blocking receive
    if let Ok(result) = rx.try_recv() {
        channels.update_rx = None;

        match result {
            Ok(()) => {
                tracing::info!("Update installed successfully");
                state.phase = UpdatePhase::RequiresRestart;
            }
            Err(e) => {
                tracing::error!("Update failed: {}", e);
                state.phase = UpdatePhase::Failed(e);
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

/// Handle restart button clicks.
fn handle_restart_button(
    query: Query<&Interaction, (Changed<Interaction>, With<ui::RestartButton>)>,
    mut events: EventWriter<RestartAppEvent>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            tracing::info!("Restart button clicked");
            events.send(RestartAppEvent);
        }
    }
}
