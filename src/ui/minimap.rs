// src/ui/minimap.rs
//! ミニマップUI
//! - プレイヤー周辺の地形表示
//! - 機械の位置表示
//! - 資源バイオーム境界表示

use bevy::prelude::*;
use crate::gameplay::player::Player;
use crate::gameplay::grid::SimulationGrid;
use crate::ui::main_menu::AppState;

/// ミニマップの設定
#[derive(Resource)]
pub struct MinimapSettings {
    pub size: f32,           // マップサイズ（ピクセル）
    pub range: i32,          // 表示範囲（ブロック数）
    pub zoom: f32,           // ズームレベル
    pub show_machines: bool, // 機械を表示するか
    pub show_players: bool,  // 他プレイヤーを表示するか
    pub opacity: f32,        // 透明度
}

impl Default for MinimapSettings {
    fn default() -> Self {
        Self {
            size: 150.0,
            range: 32,
            zoom: 1.0,
            show_machines: true,
            show_players: true,
            opacity: 0.8,
        }
    }
}

/// ミニマップのルートノード
#[derive(Component)]
pub struct MinimapRoot;

/// ミニマップの背景
#[derive(Component)]
pub struct MinimapBackground;

/// ミニマップ上のドット（機械、プレイヤーなど）
#[derive(Component)]
pub struct MinimapDot {
    pub dot_type: MinimapDotType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MinimapDotType {
    Player,      // 自分
    OtherPlayer, // 他プレイヤー
    Machine,     // 機械
}

/// ミニマッププラグイン
pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapSettings>()
            .add_systems(OnEnter(AppState::InGame), spawn_minimap)
            .add_systems(OnExit(AppState::InGame), despawn_minimap)
            .add_systems(Update, update_minimap.run_if(in_state(AppState::InGame)));
    }
}

/// ミニマップをスポーン
fn spawn_minimap(
    mut commands: Commands,
    settings: Res<MinimapSettings>,
) {
    let size = settings.size;
    let border_width = 2.0;

    // ミニマップのルートノード（右上に配置）
    commands.spawn((
        MinimapRoot,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            width: Val::Px(size + border_width * 2.0),
            height: Val::Px(size + border_width * 2.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, settings.opacity)),
        BorderRadius::all(Val::Px(4.0)),
    )).with_children(|parent| {
        // ミニマップ本体（背景）
        parent.spawn((
            MinimapBackground,
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.15, 0.1, 1.0)),
            BorderRadius::all(Val::Px(2.0)),
        )).with_children(|map_parent| {
            // プレイヤーマーカー（中央に固定）
            map_parent.spawn((
                MinimapDot { dot_type: MinimapDotType::Player },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(size / 2.0 - 4.0),
                    top: Val::Px(size / 2.0 - 4.0),
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                BorderRadius::all(Val::Px(4.0)),
            ));
        });

        // 方位表示（N）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(2.0),
                left: Val::Px(size / 2.0 + border_width - 5.0),
                ..default()
            },
            Text::new("N"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        ));
    });
}

/// ミニマップを削除
fn despawn_minimap(
    mut commands: Commands,
    query: Query<Entity, With<MinimapRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

/// ミニマップを更新
fn update_minimap(
    settings: Res<MinimapSettings>,
    player_query: Query<&Transform, With<Player>>,
    grid: Res<SimulationGrid>,
    mut background_query: Query<&mut BackgroundColor, With<MinimapBackground>>,
    mut dot_query: Query<(&MinimapDot, &mut Node, &mut BackgroundColor), Without<MinimapBackground>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let _player_yaw = player_transform.rotation.to_euler(EulerRot::YXZ).0;

    // 背景色を地形に基づいて更新（簡易実装）
    if let Ok(mut bg_color) = background_query.get_single_mut() {
        // プレイヤーの高さに基づいて色を調整
        let height_factor = (player_pos.y / 128.0).clamp(0.0, 1.0);
        let base_color = if player_pos.y < 0.0 {
            // 地下は暗い茶色
            Color::srgba(0.15 + height_factor * 0.1, 0.1, 0.05, 1.0)
        } else {
            // 地上は緑系
            Color::srgba(0.1, 0.15 + height_factor * 0.1, 0.1, 1.0)
        };
        *bg_color = BackgroundColor(base_color);
    }

    // 機械のドット表示を更新
    // 現在はシンプルな実装：グリッド内の機械をスキャン
    let range = settings.range;
    let player_grid_pos = IVec3::new(
        player_pos.x.round() as i32,
        player_pos.y.round() as i32,
        player_pos.z.round() as i32,
    );

    // 機械の数をカウント（デバッグ用）
    let mut machine_count = 0;
    for dx in -range..=range {
        for dz in -range..=range {
            let check_pos = IVec3::new(
                player_grid_pos.x + dx,
                player_grid_pos.y,
                player_grid_pos.z + dz,
            );
            if grid.machines.contains_key(&check_pos) {
                machine_count += 1;
            }
        }
    }

    // プレイヤードットの色を更新（機械が近くにあると色が変わる）
    for (dot, _node, mut color) in dot_query.iter_mut() {
        if dot.dot_type == MinimapDotType::Player {
            let intensity = if machine_count > 0 {
                1.0
            } else {
                0.7
            };
            *color = BackgroundColor(Color::srgba(intensity, intensity, intensity, 1.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimap_settings_default() {
        let settings = MinimapSettings::default();
        assert_eq!(settings.size, 150.0);
        assert_eq!(settings.range, 32);
        assert!(settings.show_machines);
    }
}
