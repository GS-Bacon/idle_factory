//! Player movement, camera, and input handling systems

use crate::components::{
    CommandInputState, ContinuousActionTimer, CursorLockState, InputStateResourcesWithCursor,
    InteractingCrusher, InteractingFurnace, InventoryOpen, Player, PlayerCamera, TutorialPopup,
    TutorialShown,
};
use crate::{KEY_ROTATION_SPEED, MOUSE_SENSITIVITY, PLAYER_SPEED};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use tracing::info;

/// Toggle cursor lock with Escape key
pub fn toggle_cursor_lock(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    creative_inv_open: Res<InventoryOpen>,
    mut cursor_state: ResMut<CursorLockState>,
) {
    let mut window = windows.single_mut();

    // Check if any UI is open
    let any_ui_open =
        interacting_furnace.0.is_some() || interacting_crusher.0.is_some() || creative_inv_open.0;

    // Escape to unlock cursor (but not if a UI is open - that UI handles ESC itself)
    if key_input.just_pressed(KeyCode::Escape) && !any_ui_open {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        cursor_state.paused = true;
    }

    // Click to lock cursor (when not locked or paused, and no UI open)
    // Also handle case where browser may have released the lock
    let cursor_not_locked = window.cursor_options.grab_mode == CursorGrabMode::None;

    // On WASM, also check if we think we should be locked but aren't (browser may have released)
    #[cfg(target_arch = "wasm32")]
    let should_relock = cursor_not_locked && !cursor_state.paused && !any_ui_open;
    #[cfg(not(target_arch = "wasm32"))]
    let should_relock = false;

    if (mouse_button.just_pressed(MouseButton::Left)
        && (cursor_not_locked || cursor_state.paused)
        && !any_ui_open)
        || should_relock
    {
        // Use Locked mode - it properly captures relative mouse motion
        // Confined mode causes issues where mouse hits window edge and spins
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
        // Mark that we just locked - skip next block break to avoid accidental destruction
        if mouse_button.just_pressed(MouseButton::Left) {
            cursor_state.just_locked = true;
        }
        cursor_state.paused = false;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn player_look(
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    windows: Query<&Window>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut cursor_lock_state: ResMut<CursorLockState>,
    inventory_open: Res<InventoryOpen>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    command_state: Res<CommandInputState>,
    tutorial_shown: Res<TutorialShown>,
) {
    // Block look while tutorial is showing
    if !tutorial_shown.0 {
        return;
    }

    // Don't look around while any UI is open or game is paused (input matrix: Mouse Move)
    if inventory_open.0
        || interacting_furnace.0.is_some()
        || interacting_crusher.0.is_some()
        || command_state.open
        || cursor_lock_state.paused
    {
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Get camera component
    let Ok((mut camera_transform, mut camera)) = camera_query.get_single_mut() else {
        return;
    };
    let Ok(mut player_transform) = player_query.get_single_mut() else {
        return;
    };

    // Pitch limit to prevent gimbal lock (±89 degrees)
    const PITCH_LIMIT: f32 = 1.54; // ~88 degrees in radians

    // --- Arrow keys for camera control (always works, time-based) ---
    if key_input.pressed(KeyCode::ArrowLeft) {
        camera.yaw += KEY_ROTATION_SPEED * time.delta_secs();
    }
    if key_input.pressed(KeyCode::ArrowRight) {
        camera.yaw -= KEY_ROTATION_SPEED * time.delta_secs();
    }
    if key_input.pressed(KeyCode::ArrowUp) {
        camera.pitch += KEY_ROTATION_SPEED * time.delta_secs();
    }
    if key_input.pressed(KeyCode::ArrowDown) {
        camera.pitch -= KEY_ROTATION_SPEED * time.delta_secs();
    }

    // --- Track cursor lock state changes ---
    if cursor_locked && !cursor_lock_state.was_locked {
        // Just became locked - reset state
        cursor_lock_state.skip_frames = 2;
        cursor_lock_state.last_mouse_pos = None;
    }
    if !cursor_locked {
        cursor_lock_state.last_mouse_pos = None;
    }
    cursor_lock_state.was_locked = cursor_locked;

    // --- Mouse motion ---
    // Use AccumulatedMouseMotion for camera control
    // Works well with Pointer Lock API in both native and WASM
    if cursor_locked && cursor_lock_state.skip_frames == 0 {
        let raw_delta = accumulated_mouse_motion.delta;

        // Clamp delta to prevent extreme camera jumps
        // Can happen during pointer lock transitions or system lag
        const MAX_DELTA: f32 = 100.0;
        let clamped_delta = Vec2::new(
            raw_delta.x.clamp(-MAX_DELTA, MAX_DELTA),
            raw_delta.y.clamp(-MAX_DELTA, MAX_DELTA),
        );

        // Only apply if delta is non-trivial (avoid jitter from small movements)
        if clamped_delta.length() > 0.1 {
            camera.yaw -= clamped_delta.x * MOUSE_SENSITIVITY;
            camera.pitch -= clamped_delta.y * MOUSE_SENSITIVITY;
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

/// Dismiss tutorial popup on any input
pub fn tutorial_dismiss(
    mut commands: Commands,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut tutorial_shown: ResMut<TutorialShown>,
    popup_query: Query<Entity, With<TutorialPopup>>,
    mut windows: Query<&mut Window>,
) {
    // Already dismissed
    if tutorial_shown.0 {
        return;
    }

    // Check for any key or mouse button press
    let any_key = key_input.get_just_pressed().next().is_some();
    let any_click = mouse_button.just_pressed(MouseButton::Left)
        || mouse_button.just_pressed(MouseButton::Right);

    if any_key || any_click {
        tutorial_shown.0 = true;

        // Despawn tutorial popup
        for entity in popup_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Lock cursor for gameplay
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        }

        info!("[TUTORIAL] Dismissed, starting gameplay");
    }
}

pub fn player_move(
    time: Res<Time>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    camera_query: Query<&PlayerCamera>,
    input_resources: InputStateResourcesWithCursor,
    tutorial_shown: Res<TutorialShown>,
) {
    // Block movement while tutorial is showing
    if !tutorial_shown.0 {
        return;
    }

    // Use InputState to check if movement is allowed (see CLAUDE.md 入力マトリクス)
    let input_state = input_resources.get_state();
    if !input_state.allows_movement() {
        return;
    }

    let Ok(mut player_transform) = player_query.get_single_mut() else {
        return;
    };
    let Ok(camera) = camera_query.get_single() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    // Calculate forward/right from yaw (more stable than transform.forward())
    let (sin_yaw, cos_yaw) = camera.yaw.sin_cos();
    let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
    let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);

    if key_input.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if key_input.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if key_input.pressed(KeyCode::KeyA) {
        direction -= right;
    }
    if key_input.pressed(KeyCode::KeyD) {
        direction += right;
    }
    if key_input.pressed(KeyCode::Space) {
        direction.y += 1.0;
    }
    if key_input.pressed(KeyCode::ShiftLeft) {
        direction.y -= 1.0;
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
        player_transform.translation += direction * PLAYER_SPEED * time.delta_secs();
    }
}

/// Tick all action timers (separate system to reduce parameter count)
pub fn tick_action_timers(time: Res<Time>, mut action_timer: ResMut<ContinuousActionTimer>) {
    action_timer.break_timer.tick(time.delta());
    action_timer.place_timer.tick(time.delta());
    action_timer.inventory_timer.tick(time.delta());
}
