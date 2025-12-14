use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use crate::core::config::GameConfig;
use crate::core::input::KeyBindings;

// シンプルに1つのコンポーネントで管理
#[derive(Component)]
pub struct Player {
    pub yaw: f32,   // 左右 (Y軸)
    pub pitch: f32, // 上下 (X軸)
}

pub fn spawn_player(mut commands: Commands, config: Res<GameConfig>) {
    info!("🚀 SPAWN_PLAYER SYSTEM STARTED! (プレイヤー生成開始)"); // ★動作確認用ログ

    let start_pos = Vec3::new(16.0, 10.0, 16.0);

    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: config.fov.to_radians(),
            ..default()
        }),
        // ★初期化: Quat::IDENTITY は「回転ゼロ（北向き・水平）」です。
        // これで真下を向くなら、他の何かが悪さをしています。
        Transform::from_translation(start_pos).with_rotation(Quat::IDENTITY),
        Player { 
            yaw: 0.0, 
            pitch: 0.0 
        },
    ));
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

        // ★角度制限 (Clamp)
        // 89.5度 (1.56ラジアン) で確実に止める
        let limit = 89.5_f32.to_radians();
        player.pitch = player.pitch.clamp(-limit, limit);

        // ★回転の適用 (Y回転 * X回転)
        // 毎回ゼロから計算しなおすため、現在の傾きに関わらず正しい姿勢になります。
        transform.rotation = 
            Quat::from_axis_angle(Vec3::Y, player.yaw) * Quat::from_axis_angle(Vec3::X, player.pitch);
    }
}

pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    config: Res<GameConfig>,
    keybinds: Res<KeyBindings>, // 追加
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let mut move_dir = Vec3::ZERO;
        
        // 自分の向き(Yaw)を基準に進む
        let yaw_rot = Quat::from_rotation_y(transform.rotation.to_euler(EulerRot::YXZ).0);
        let forward = yaw_rot * Vec3::NEG_Z;
        let right = yaw_rot * Vec3::X;

        // キーバインドを使って判定
        if keyboard.pressed(keybinds.forward) { move_dir += forward; }
        if keyboard.pressed(keybinds.backward) { move_dir -= forward; }
        if keyboard.pressed(keybinds.right) { move_dir += right; }
        if keyboard.pressed(keybinds.left) { move_dir -= right; }
        
        // 上下移動
        if keyboard.pressed(keybinds.jump) { move_dir.y += 1.0; }
        if keyboard.pressed(keybinds.descend) { move_dir.y -= 1.0; } // ここでShiftLeftが効くようになる

        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
        }

        // ダッシュ判定
        let speed = if keyboard.pressed(keybinds.sprint) { config.run_speed } else { config.walk_speed };
        
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