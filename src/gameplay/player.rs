use crate::core::config::GameConfig;
use crate::core::input::KeyBindings;
use crate::gameplay::inventory::PlayerInventory;
use crate::gameplay::commands::GameMode;
use crate::gameplay::held_item::PlayerCamera;
use crate::gameplay::physics::{PlayerCollider, PlayerPhysics};
use crate::gameplay::player_stats::{FallTracker, PlayerHealth, PlayerExperience};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

#[derive(Component)]
pub struct Player {
    pub yaw: f32,   // 左右 (Y軸)
    pub pitch: f32, // 上下 (X軸)
    pub is_flying: bool, // クリエイティブモードの飛行状態
    pub velocity: Vec3, // 速度（重力や慣性用）
}

pub fn spawn_player(
    mut commands: Commands,
    existing: Query<Entity, With<Player>>,
) {
    // 既にプレイヤーが存在する場合はスポーンしない
    if !existing.is_empty() {
        return;
    }

    // プレイヤー本体 (カメラも含む)
    // シンプルにするため、プレイヤー自体が回転し、その視界＝カメラとします
    commands.spawn((
        // ★修正: 構造体のフィールド名を明記
        Player { yaw: 0.0, pitch: 0.0, is_flying: false, velocity: Vec3::ZERO },
        Transform::from_xyz(0.0, 5.0, 0.0),
        Visibility::default(),
        // 物理コンポーネント（サバイバルモード用）
        PlayerPhysics::default(),
        PlayerCollider::default(),
        FallTracker::default(),
        PlayerHealth::default(),
        PlayerExperience::default(),
    ))
    .with_children(|parent| {
        // カメラ (FPS視点)
        // 親(Player)が回転・移動するので、カメラはローカル座標で固定でOK
        parent.spawn((
            Camera3d::default(),
            Camera {
                order: 1, // メニューカメラ(order=0)より上
                ..default()
            },
            // ★重要追加: アンチエイリアス有効化
            Msaa::Sample4,
            Transform::from_xyz(0.0, 1.5, 0.0), // 目の高さ (PlayerCollider.eye_height)
            PlayerCamera, // マーカーコンポーネント
        ));
    });
}

/// プレイヤーを削除
pub fn despawn_player(
    mut commands: Commands,
    query: Query<Entity, With<Player>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn look_player(
    mut events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut Player)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    config: Res<GameConfig>,
) {
    let window = window_query.single();
    if window.cursor_options.grab_mode == CursorGrabMode::None {
        return;
    }

    // マウス移動量の合計を計算
    let mut delta_x = 0.0;
    let mut delta_y = 0.0;
    for event in events.read() {
        delta_x += event.delta.x;
        delta_y += event.delta.y;
    }

    if let Ok((mut transform, mut player)) = query.get_single_mut() {
        // 感度適用
        player.yaw -= delta_x * config.mouse_sensitivity;
        player.pitch -= delta_y * config.mouse_sensitivity;

        // 角度制限 (真上・真下付近で止める)
        let limit = 89.5_f32.to_radians();
        player.pitch = player.pitch.clamp(-limit, limit);

        // 回転の適用 (Y回転 * X回転)
        transform.rotation = Quat::from_axis_angle(Vec3::Y, player.yaw)
            * Quat::from_axis_angle(Vec3::X, player.pitch);
    }
}

/// クリエイティブモード専用の移動処理
/// サバイバルモードはPhysicsPluginが処理する
pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Player)>,
    config: Res<GameConfig>,
    keybinds: Res<KeyBindings>,
    game_mode: Res<GameMode>,
) {
    // サバイバルモードはPhysicsPluginが処理
    if *game_mode == GameMode::Survival {
        return;
    }

    if let Ok((mut transform, mut player)) = query.get_single_mut() {
        let mut move_dir = Vec3::ZERO;

        // 自分の向き(Yaw)を基準に進む
        let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw_rot = Quat::from_rotation_y(yaw);

        let forward = yaw_rot * Vec3::NEG_Z;
        let right = yaw_rot * Vec3::X;

        // キーバインド判定
        if keyboard.pressed(keybinds.forward) { move_dir += forward; }
        if keyboard.pressed(keybinds.backward) { move_dir -= forward; }
        if keyboard.pressed(keybinds.right) { move_dir += right; }
        if keyboard.pressed(keybinds.left) { move_dir -= right; }

        // クリエイティブモード：常に飛行可能
        player.is_flying = true;

        // 飛行中：Space/Shiftで上下移動
        if keyboard.pressed(keybinds.jump) { move_dir.y += 1.0; }
        if keyboard.pressed(keybinds.descend) { move_dir.y -= 1.0; }

        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
        }

        // ダッシュ判定（クリエイティブ飛行は2倍速）
        let base_speed = if keyboard.pressed(keybinds.sprint) {
            config.run_speed
        } else {
            config.walk_speed
        };

        let speed = base_speed * 2.0;

        transform.translation += move_dir * speed * time.delta_secs();
    }
}

pub fn grab_cursor(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut window = window_query.single_mut();
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
    if key.just_pressed(KeyCode::Escape) {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

/// ホットバースロット選択（1-9キー、0キー）
pub fn handle_hotbar_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<PlayerInventory>,
) {
    // 1-9キーでホットバースロット50-58を選択
    if keyboard.just_pressed(KeyCode::Digit1) {
        inventory.selected_hotbar_slot = 50;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        inventory.selected_hotbar_slot = 51;
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        inventory.selected_hotbar_slot = 52;
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        inventory.selected_hotbar_slot = 53;
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        inventory.selected_hotbar_slot = 54;
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        inventory.selected_hotbar_slot = 55;
    } else if keyboard.just_pressed(KeyCode::Digit7) {
        inventory.selected_hotbar_slot = 56;
    } else if keyboard.just_pressed(KeyCode::Digit8) {
        inventory.selected_hotbar_slot = 57;
    } else if keyboard.just_pressed(KeyCode::Digit9) {
        inventory.selected_hotbar_slot = 58;
    } else if keyboard.just_pressed(KeyCode::Digit0) {
        inventory.selected_hotbar_slot = 59;
    }
}

/// ホットバースロット選択（スクロールホイール）
pub fn handle_hotbar_scroll(
    mut scroll_events: EventReader<MouseWheel>,
    mut inventory: ResMut<PlayerInventory>,
) {
    for event in scroll_events.read() {
        if event.y != 0.0 {
            let current_index = inventory.selected_hotbar_slot - 50;
            let delta = if event.y > 0.0 { -1 } else { 1 };
            let new_index = (current_index as i32 + delta).rem_euclid(10) as usize;
            inventory.selected_hotbar_slot = new_index + 50;
        }
    }
}