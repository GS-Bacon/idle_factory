//! Player movement, camera, and input handling systems

use crate::components::{
    CommandInputState, ContinuousActionTimer, CursorLockState, InputStateResourcesWithCursor,
    InteractingMachine, InventoryOpen, PauseUI, Player, PlayerCamera, PlayerPhysics, TutorialPopup,
    TutorialShown, UIAction, UIContext, UIState,
};
use crate::input::{GameAction, InputManager};
use crate::settings::GameSettings;
use crate::systems::cursor;
use crate::world::WorldData;
use crate::{
    CreativeMode, GRAVITY, JUMP_VELOCITY, KEY_ROTATION_SPEED, PLAYER_HEIGHT, PLAYER_SPEED,
    PLAYER_WIDTH, TERMINAL_VELOCITY,
};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use tracing::info;

/// Handle cursor lock on click (ESC handling moved to ui_navigation.rs)
pub fn toggle_cursor_lock(
    input: Res<InputManager>,
    mut windows: Query<&mut Window>,
    interacting_machine: Res<InteractingMachine>,
    creative_inv_open: Res<InventoryOpen>,
    mut cursor_state: ResMut<CursorLockState>,
) {
    let mut window = windows.single_mut();

    // Check if any UI is open
    let any_ui_open = interacting_machine.0.is_some() || creative_inv_open.0;

    // Click to lock cursor (when not locked or paused, and no UI open)
    // Also handle case where cursor may have been released
    if input.just_pressed(GameAction::PrimaryAction)
        && (cursor::is_unlocked(&window) || cursor_state.paused)
        && !any_ui_open
    {
        // Use Locked mode - it properly captures relative mouse motion
        // Confined mode causes issues where mouse hits window edge and spins
        cursor::lock_cursor(&mut window);
        // Mark that we just locked - skip next block break to avoid accidental destruction
        cursor_state.just_locked = true;
        cursor_state.paused = false;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn player_look(
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    input: Res<InputManager>,
    time: Res<Time>,
    windows: Query<&Window>,
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

    // Don't look around while any UI is open or game is paused (input matrix: Mouse Move)
    if inventory_open.0
        || interacting_machine.0.is_some()
        || command_state.open
        || cursor_lock_state.paused
    {
        return;
    }

    let window = windows.single();
    let cursor_locked = cursor::is_locked(window);

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
    // Works well with Pointer Lock API
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
            cursor::lock_cursor(&mut window);
        }

        info!("[TUTORIAL] Dismissed, starting gameplay");
    }
}

#[allow(clippy::too_many_arguments)]
pub fn player_move(
    time: Res<Time>,
    input: Res<InputManager>,
    mut player_query: Query<(&mut Transform, &mut PlayerPhysics), With<Player>>,
    camera_query: Query<&PlayerCamera>,
    input_resources: InputStateResourcesWithCursor,
    tutorial_shown: Res<TutorialShown>,
    creative_mode: Res<CreativeMode>,
    world_data: Res<WorldData>,
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

    let Ok((mut player_transform, mut physics)) = player_query.get_single_mut() else {
        return;
    };
    let Ok(camera) = camera_query.get_single() else {
        return;
    };

    // Calculate forward/right from yaw (more stable than transform.forward())
    let (sin_yaw, cos_yaw) = camera.yaw.sin_cos();
    let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
    let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);

    let dt = time.delta_secs();

    if creative_mode.enabled {
        // Creative mode: fly movement
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
    } else {
        // Survival mode: physics-based movement
        survival_movement(
            &mut player_transform,
            &mut physics,
            &input,
            &world_data,
            forward,
            right,
            dt,
        );
    }
}

/// Survival mode movement with gravity, jumping, and collision
fn survival_movement(
    player_transform: &mut Transform,
    physics: &mut PlayerPhysics,
    input: &InputManager,
    world_data: &WorldData,
    forward: Vec3,
    right: Vec3,
    dt: f32,
) {
    // Get horizontal movement input
    let mut horizontal = Vec3::ZERO;
    if input.pressed(GameAction::MoveForward) {
        horizontal += forward;
    }
    if input.pressed(GameAction::MoveBackward) {
        horizontal -= forward;
    }
    if input.pressed(GameAction::MoveLeft) {
        horizontal -= right;
    }
    if input.pressed(GameAction::MoveRight) {
        horizontal += right;
    }

    // Normalize horizontal movement
    if horizontal.length_squared() > 0.0 {
        horizontal = horizontal.normalize() * PLAYER_SPEED;
    }

    // Apply gravity
    physics.velocity.y -= GRAVITY * dt;
    physics.velocity.y = physics.velocity.y.max(-TERMINAL_VELOCITY);

    // Jump if on ground and space pressed
    if physics.on_ground && input.just_pressed(GameAction::Jump) {
        physics.velocity.y = JUMP_VELOCITY;
        physics.on_ground = false;
    }

    // Calculate new position
    let mut new_pos = player_transform.translation;
    new_pos.x += horizontal.x * dt;
    new_pos.z += horizontal.z * dt;
    new_pos.y += physics.velocity.y * dt;

    // Check collision and resolve
    let half_width = PLAYER_WIDTH / 2.0;

    // Check ground collision (feet position)
    let feet_y = new_pos.y - PLAYER_HEIGHT / 2.0;
    let ground_check_y = (feet_y - 0.01) as i32;

    // Check if player would be inside a block
    let mut on_ground = false;

    // Check 4 corners at feet level
    for &(dx, dz) in &[
        (-half_width, -half_width),
        (half_width, -half_width),
        (-half_width, half_width),
        (half_width, half_width),
    ] {
        let check_x = (new_pos.x + dx) as i32;
        let check_z = (new_pos.z + dz) as i32;

        // Check block below feet (Some = solid, None = air)
        if world_data
            .get_block(IVec3::new(check_x, ground_check_y, check_z))
            .is_some()
        {
            // Standing on a block
            on_ground = true;
            let ground_top = (ground_check_y + 1) as f32;
            if new_pos.y - PLAYER_HEIGHT / 2.0 < ground_top {
                new_pos.y = ground_top + PLAYER_HEIGHT / 2.0;
                physics.velocity.y = 0.0;
            }
        }

        // Check horizontal collision at multiple heights
        // Skip h=0.0 (feet level) as that's where ground blocks are - checking it causes
        // false positives when standing on ground
        for h in [0.3, PLAYER_HEIGHT / 2.0, PLAYER_HEIGHT - 0.1] {
            let check_y = (new_pos.y - PLAYER_HEIGHT / 2.0 + h) as i32;
            if world_data
                .get_block(IVec3::new(check_x, check_y, check_z))
                .is_some()
            {
                // Collision - push player out
                let block_center_x = check_x as f32 + 0.5;
                let block_center_z = check_z as f32 + 0.5;

                // Determine which direction to push
                let push_x = new_pos.x - block_center_x;
                let push_z = new_pos.z - block_center_z;

                if push_x.abs() > push_z.abs() {
                    new_pos.x = if push_x > 0.0 {
                        block_center_x + 0.5 + half_width + 0.01
                    } else {
                        block_center_x - 0.5 - half_width - 0.01
                    };
                } else {
                    new_pos.z = if push_z > 0.0 {
                        block_center_z + 0.5 + half_width + 0.01
                    } else {
                        block_center_z - 0.5 - half_width - 0.01
                    };
                }
            }
        }

        // Check ceiling collision
        let head_y = (new_pos.y + PLAYER_HEIGHT / 2.0 + 0.01) as i32;
        if world_data
            .get_block(IVec3::new(check_x, head_y, check_z))
            .is_some()
        {
            // Hit ceiling
            let ceiling_bottom = head_y as f32;
            new_pos.y = ceiling_bottom - PLAYER_HEIGHT / 2.0 - 0.01;
            if physics.velocity.y > 0.0 {
                physics.velocity.y = 0.0;
            }
        }
    }

    physics.on_ground = on_ground;
    player_transform.translation = new_pos;
}

/// Tick all action timers (separate system to reduce parameter count)
pub fn tick_action_timers(time: Res<Time>, mut action_timer: ResMut<ContinuousActionTimer>) {
    action_timer.break_timer.tick(time.delta());
    action_timer.place_timer.tick(time.delta());
    action_timer.inventory_timer.tick(time.delta());
}

/// Update pause UI visibility and cursor state based on UIState
pub fn update_pause_ui(
    ui_state: Res<UIState>,
    mut pause_query: Query<&mut Visibility, With<PauseUI>>,
    mut windows: Query<&mut Window>,
) {
    // Only update when ui_state changes
    if !ui_state.is_changed() {
        return;
    }

    let Ok(mut visibility) = pause_query.get_single_mut() else {
        return;
    };

    // PauseUI is visible only when PauseMenu is active
    let show_pause = ui_state.is_active(&UIContext::PauseMenu);
    *visibility = if show_pause {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    // Cursor is locked only when in Gameplay (no UI open)
    if let Ok(mut window) = windows.get_single_mut() {
        if ui_state.is_gameplay() {
            cursor::lock_cursor(&mut window);
        } else {
            cursor::release_cursor(&mut window);
        }
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
    mut app_exit: EventWriter<bevy::app::AppExit>,
    mut action_writer: EventWriter<UIAction>,
) {
    for (interaction, button_type, mut bg_color) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                match button_type {
                    crate::setup::ui::PauseMenuButton::Resume => {
                        // Resume game
                        cursor_state.paused = false;
                        action_writer.send(UIAction::Pop);
                    }
                    crate::setup::ui::PauseMenuButton::Settings => {
                        // Open settings (will implement in D.3)
                        action_writer.send(UIAction::Push(UIContext::Settings));
                    }
                    crate::setup::ui::PauseMenuButton::Quit => {
                        // Exit application (native only)
                        #[cfg(not(target_arch = "wasm32"))]
                        app_exit.send(bevy::app::AppExit::Success);
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
