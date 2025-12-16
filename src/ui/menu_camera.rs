// src/ui/menu_camera.rs
//! メニュー背景カメラ
//! - 原点中心にゆっくり旋回（Orbit）
//! - メニュー画面でのみ有効

use bevy::prelude::*;
use crate::ui::main_menu::AppState;

/// メニューカメラプラグイン
pub struct MenuCameraPlugin;

impl Plugin for MenuCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), spawn_menu_camera)
            .add_systems(OnExit(AppState::InGame), spawn_menu_camera)
            .add_systems(OnEnter(AppState::InGame), despawn_menu_camera)
            .add_systems(Update, orbit_camera.run_if(not(in_state(AppState::InGame))));
    }
}

/// メニューカメラマーカー
#[derive(Component)]
pub struct MenuCamera {
    /// 旋回中心
    pub target: Vec3,
    /// 旋回半径
    pub radius: f32,
    /// 現在の角度（ラジアン）
    pub angle: f32,
    /// 旋回速度（ラジアン/秒）
    pub speed: f32,
    /// カメラの高さ
    pub height: f32,
}

impl Default for MenuCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            radius: 50.0,
            angle: 0.0,
            speed: 0.1,  // ゆっくり旋回
            height: 20.0,
        }
    }
}

/// メニューカメラを生成
fn spawn_menu_camera(
    mut commands: Commands,
    existing: Query<Entity, With<MenuCamera>>,
) {
    // 既存カメラがあれば何もしない
    if !existing.is_empty() {
        return;
    }

    let menu_cam = MenuCamera::default();
    let initial_pos = calculate_orbit_position(&menu_cam);

    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.05, 0.05, 0.08)),
            order: 0,
            ..default()
        },
        Transform::from_translation(initial_pos)
            .looking_at(menu_cam.target, Vec3::Y),
        menu_cam,
    ));

    info!("Menu camera spawned");
}

/// メニューカメラを削除
fn despawn_menu_camera(
    mut commands: Commands,
    query: Query<Entity, With<MenuCamera>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    info!("Menu camera despawned");
}

/// カメラを旋回させる
fn orbit_camera(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut MenuCamera)>,
) {
    for (mut transform, mut menu_cam) in &mut query {
        // 角度を更新
        menu_cam.angle += menu_cam.speed * time.delta_secs();

        // 位置を計算
        let new_pos = calculate_orbit_position(&menu_cam);
        transform.translation = new_pos;

        // ターゲットを向く
        transform.look_at(menu_cam.target, Vec3::Y);
    }
}

/// 旋回位置を計算
fn calculate_orbit_position(cam: &MenuCamera) -> Vec3 {
    Vec3::new(
        cam.target.x + cam.radius * cam.angle.cos(),
        cam.target.y + cam.height,
        cam.target.z + cam.radius * cam.angle.sin(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_camera_default() {
        let cam = MenuCamera::default();
        assert_eq!(cam.target, Vec3::ZERO);
        assert!(cam.radius > 0.0);
        assert!(cam.speed > 0.0);
    }

    #[test]
    fn test_calculate_orbit_position() {
        let cam = MenuCamera {
            target: Vec3::ZERO,
            radius: 10.0,
            angle: 0.0,
            speed: 1.0,
            height: 5.0,
        };

        let pos = calculate_orbit_position(&cam);
        assert!((pos.x - 10.0).abs() < 0.001);
        assert!((pos.y - 5.0).abs() < 0.001);
        assert!((pos.z - 0.0).abs() < 0.001);
    }
}
