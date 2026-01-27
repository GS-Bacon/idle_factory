//! Player movement, camera, and input handling systems
//!
//! CAD-style controls:
//! - Cursor always visible
//! - Middle-drag or Alt+left-drag to rotate camera
//! - WASD + Space/Shift for fly movement (no collision)

use crate::components::{
    CommandInputState, ContinuousActionTimer, CursorLockState, InputStateResourcesWithCursor,
    InteractingMachine, InventoryOpen, PauseUI, Player, PlayerCamera, TutorialShown, UIAction,
    UIContext, UIState,
};
use crate::input::{GameAction, InputManager};
use crate::settings::GameSettings;
use crate::systems::cursor;
use crate::{KEY_ROTATION_SPEED, PLAYER_SPEED};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorOptions, PrimaryWindow};
use tracing::info;

/// CAD-style controls: no cursor lock needed
/// Left as no-op for compatibility with system registration
pub fn toggle_cursor_lock(
    _input: Res<InputManager>,
    _cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    _ui_state: Res<UIState>,
    _cursor_state: ResMut<CursorLockState>,
) {
    // CAD-style: cursor is never locked, always visible
    // Camera rotation is handled by middle-drag in player_look
}

#[allow(clippy::too_many_arguments)]
pub fn player_look(
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    input: Res<InputManager>,
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut cursor_lock_state: ResMut<CursorLockState>,
    inventory_open: Res<InventoryOpen>,
    interacting_machine: Res<InteractingMachine>,
    command_state: Res<CommandInputState>,
    tutorial_shown: Res<TutorialShown>,
    settings: Res<GameSettings>,
) {
    // Block look while tutorial is showing
    if !tutorial_shown.0 {
        return;
    }

    // Don't look around while any UI is open or game is paused
    if inventory_open.0
        || interacting_machine.0.is_some()
        || command_state.open
        || cursor_lock_state.paused
    {
        return;
    }

    // Get camera component
    let Ok((mut camera_transform, mut camera)) = camera_query.single_mut() else {
        return;
    };
    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };

    // Pitch limit to prevent gimbal lock (±89 degrees)
    const PITCH_LIMIT: f32 = 1.54; // ~88 degrees in radians

    // --- Arrow keys for camera control (always works, time-based) ---
    if input.pressed(GameAction::LookLeft) {
        camera.yaw += KEY_ROTATION_SPEED * time.delta_secs();
    }
    if input.pressed(GameAction::LookRight) {
        camera.yaw -= KEY_ROTATION_SPEED * time.delta_secs();
    }
    if input.pressed(GameAction::LookUp) {
        camera.pitch += KEY_ROTATION_SPEED * time.delta_secs();
    }
    if input.pressed(GameAction::LookDown) {
        camera.pitch -= KEY_ROTATION_SPEED * time.delta_secs();
    }

    // --- CAD-style mouse rotation ---
    // Middle mouse button drag OR Alt + Left mouse button drag
    let alt_held = key_input.pressed(KeyCode::AltLeft) || key_input.pressed(KeyCode::AltRight);
    let middle_drag = mouse_button.pressed(MouseButton::Middle);
    let alt_left_drag = alt_held && mouse_button.pressed(MouseButton::Left);
    let is_rotating = middle_drag || alt_left_drag;

    // Track rotation state for skip frames
    if is_rotating && !cursor_lock_state.was_locked {
        cursor_lock_state.skip_frames = 1;
    }
    cursor_lock_state.was_locked = is_rotating;

    // Apply mouse motion only during rotation
    if is_rotating && cursor_lock_state.skip_frames == 0 {
        let raw_delta = accumulated_mouse_motion.delta;

        // Clamp delta to prevent extreme camera jumps
        const MAX_DELTA: f32 = 100.0;
        let clamped_delta = Vec2::new(
            raw_delta.x.clamp(-MAX_DELTA, MAX_DELTA),
            raw_delta.y.clamp(-MAX_DELTA, MAX_DELTA),
        );

        // Only apply if delta is non-trivial
        if clamped_delta.length() > 0.1 {
            let (sens_x, sens_y) = settings.effective_sensitivity();
            camera.yaw -= clamped_delta.x * sens_x;
            camera.pitch -= clamped_delta.y * sens_y;
        }
    }

    // Decrement skip counter
    if cursor_lock_state.skip_frames > 0 {
        cursor_lock_state.skip_frames -= 1;
    }

    // Clamp pitch
    camera.pitch = camera.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

    // --- Apply rotation (YXZ order to prevent roll) ---
    // Player rotates horizontally (yaw only)
    player_transform.rotation = Quat::from_rotation_y(camera.yaw);

    // Camera rotates vertically (pitch) relative to player
    camera_transform.rotation = Quat::from_rotation_x(camera.pitch);
}

/// CAD-style fly movement (no collision, no gravity)
#[allow(clippy::too_many_arguments)]
pub fn player_move(
    time: Res<Time>,
    input: Res<InputManager>,
    mut player_query: Query<&mut Transform, With<Player>>,
    camera_query: Query<&PlayerCamera>,
    input_resources: InputStateResourcesWithCursor,
    tutorial_shown: Res<TutorialShown>,
) {
    // Block movement while tutorial is showing
    if !tutorial_shown.0 {
        return;
    }

    // Use InputState to check if movement is allowed
    let input_state = input_resources.get_state();
    if !input_state.allows_movement() {
        return;
    }

    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };
    let Ok(camera) = camera_query.single() else {
        return;
    };

    // Calculate forward/right from yaw
    let (sin_yaw, cos_yaw) = camera.yaw.sin_cos();
    let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
    let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);

    let dt = time.delta_secs();

    // CAD-style: always fly movement, no collision
    let mut direction = Vec3::ZERO;

    if input.pressed(GameAction::MoveForward) {
        direction += forward;
    }
    if input.pressed(GameAction::MoveBackward) {
        direction -= forward;
    }
    if input.pressed(GameAction::MoveLeft) {
        direction -= right;
    }
    if input.pressed(GameAction::MoveRight) {
        direction += right;
    }
    if input.pressed(GameAction::Jump) {
        direction.y += 1.0;
    }
    if input.pressed(GameAction::Descend) {
        direction.y -= 1.0;
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
        player_transform.translation += direction * PLAYER_SPEED * dt;
    }
}

/// Tick all action timers (separate system to reduce parameter count)
pub fn tick_action_timers(time: Res<Time>, mut action_timer: ResMut<ContinuousActionTimer>) {
    action_timer.break_timer.tick(time.delta());
    action_timer.place_timer.tick(time.delta());
    action_timer.inventory_timer.tick(time.delta());
}

/// Update pause UI visibility based on UIState
/// Note: Cursor control moved to sync_cursor_to_ui_state (PostUpdate)
pub fn update_pause_ui(
    ui_state: Res<UIState>,
    mut pause_query: Query<&mut Visibility, With<PauseUI>>,
) {
    // Only update when ui_state changes
    if !ui_state.is_changed() {
        return;
    }

    let Ok(mut visibility) = pause_query.single_mut() else {
        return;
    };

    // PauseUI is visible only when PauseMenu is active
    let show_pause = ui_state.is_active(&UIContext::PauseMenu);
    *visibility = if show_pause {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

/// Initialize cursor state at startup (CAD-style: always visible)
///
/// In CAD-style controls, cursor is always visible.
/// Camera rotation is controlled by middle-drag or Alt+left-drag.
pub fn initialize_cursor(
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut frame_count: Local<u32>,
) {
    // Wait until window is ready (5 frames after startup)
    *frame_count += 1;
    if *frame_count < 5 {
        return;
    }
    if *frame_count > 5 {
        return; // Only run once on frame 5
    }

    if let Ok(mut cursor_options) = cursor_query.single_mut() {
        // CAD-style: always visible, never grabbed
        cursor::release_cursor(&mut cursor_options);
        info!(
            "[CURSOR] Initialized (frame {}): CAD-style (always visible)",
            *frame_count
        );
    }
}

/// Handle pause menu button clicks
pub fn handle_pause_menu_buttons(
    mut interaction_query: Query<
        (
            &Interaction,
            &crate::setup::ui::PauseMenuButton,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
    mut cursor_state: ResMut<CursorLockState>,
    mut app_exit: MessageWriter<bevy::app::AppExit>,
    mut action_writer: MessageWriter<UIAction>,
) {
    for (interaction, button_type, mut bg_color) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                match button_type {
                    crate::setup::ui::PauseMenuButton::Resume => {
                        // Resume game
                        cursor_state.paused = false;
                        action_writer.write(UIAction::Pop);
                    }
                    crate::setup::ui::PauseMenuButton::Settings => {
                        // Open settings (will implement in D.3)
                        action_writer.write(UIAction::Push(UIContext::Settings));
                    }
                    crate::setup::ui::PauseMenuButton::Quit => {
                        // Exit application (native only)
                        #[cfg(not(target_arch = "wasm32"))]
                        app_exit.write(bevy::app::AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.95));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9));
            }
        }
    }
}
