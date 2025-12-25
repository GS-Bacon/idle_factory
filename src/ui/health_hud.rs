// src/ui/health_hud.rs
//! HP表示HUD
//! - プレイヤーのHP表示（ゆるめの表示）
//! - ダメージ時の視覚効果

use bevy::prelude::*;
use crate::gameplay::player::Player;
use crate::gameplay::player_stats::PlayerHealth;
use crate::ui::main_menu::AppState;

/// HP HUDのルートノード
#[derive(Component)]
pub struct HealthHudRoot;

/// HPバーの背景
#[derive(Component)]
pub struct HealthBarBackground;

/// HPバーの前景（実際のHP量を表示）
#[derive(Component)]
pub struct HealthBarFill;

/// HPテキスト
#[derive(Component)]
pub struct HealthText;

/// ダメージオーバーレイ（赤い画面）
#[derive(Component)]
pub struct DamageOverlay {
    pub fade_timer: f32,
}

/// HP HUDプラグイン
pub struct HealthHudPlugin;

impl Plugin for HealthHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_health_hud)
            .add_systems(OnExit(AppState::InGame), despawn_health_hud)
            .add_systems(Update, (
                update_health_bar,
                update_damage_overlay,
            ).run_if(in_state(AppState::InGame)));
    }
}

/// HP HUDをスポーン
fn spawn_health_hud(mut commands: Commands) {
    // ダメージオーバーレイ（全画面、初期は透明）
    commands.spawn((
        DamageOverlay { fade_timer: 0.0 },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.0)),
    ));

    // HP HUD（左下に配置、サイズ拡大）
    commands.spawn((
        HealthHudRoot,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(90.0), // ホットバーの上
            width: Val::Px(200.0),
            height: Val::Px(50.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            row_gap: Val::Px(4.0),
            ..default()
        },
    )).with_children(|parent| {
        // HPラベル
        parent.spawn((
            Text::new("HP"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        ));

        // HPバーコンテナ
        parent.spawn((
            HealthBarBackground,
            Node {
                width: Val::Px(180.0),
                height: Val::Px(20.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            BorderColor(Color::srgba(0.6, 0.6, 0.6, 0.9)),
            BorderRadius::all(Val::Px(4.0)),
        )).with_children(|bar_parent| {
            // HPバー本体
            bar_parent.spawn((
                HealthBarFill,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.8, 0.2, 0.2, 1.0)),
                BorderRadius::all(Val::Px(2.0)),
            ));
        });

        // HPテキスト
        parent.spawn((
            HealthText,
            Text::new("10 / 10"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
        ));
    });
}

/// HP HUDを削除
fn despawn_health_hud(
    mut commands: Commands,
    hud_query: Query<Entity, With<HealthHudRoot>>,
    overlay_query: Query<Entity, With<DamageOverlay>>,
) {
    for entity in &hud_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &overlay_query {
        commands.entity(entity).despawn_recursive();
    }
}

/// HPバーを更新
fn update_health_bar(
    player_query: Query<&PlayerHealth, With<Player>>,
    mut bar_query: Query<(&mut Node, &mut BackgroundColor), With<HealthBarFill>>,
    mut text_query: Query<&mut Text, With<HealthText>>,
    mut overlay_query: Query<(&mut DamageOverlay, &mut BackgroundColor), Without<HealthBarFill>>,
) {
    let Ok(health) = player_query.get_single() else {
        return;
    };

    let health_percent = (health.current / health.max * 100.0).clamp(0.0, 100.0);

    // バーの幅を更新
    if let Ok((mut node, mut color)) = bar_query.get_single_mut() {
        node.width = Val::Percent(health_percent);

        // HPに応じて色を変更
        let bar_color = if health_percent > 50.0 {
            // 緑〜黄色
            let t = (health_percent - 50.0) / 50.0;
            Color::srgba(0.2 + 0.6 * (1.0 - t), 0.8, 0.2, 1.0)
        } else if health_percent > 25.0 {
            // 黄色〜オレンジ
            let t = (health_percent - 25.0) / 25.0;
            Color::srgba(0.9, 0.5 + 0.3 * t, 0.1, 1.0)
        } else {
            // 赤（点滅）
            let pulse = (std::f32::consts::PI * 4.0 * health_percent / 25.0).sin().abs();
            Color::srgba(0.8 + 0.2 * pulse, 0.1, 0.1, 1.0)
        };
        *color = BackgroundColor(bar_color);
    }

    // テキストを更新
    if let Ok(mut text) = text_query.get_single_mut() {
        **text = format!("{:.0} / {:.0}", health.current, health.max);
    }

    // ダメージ時のオーバーレイ
    if let Ok((mut overlay, _)) = overlay_query.get_single_mut() {
        // 無敵時間中 = ダメージを受けた直後
        if health.invincibility_timer > 0.0 && overlay.fade_timer <= 0.0 {
            overlay.fade_timer = 0.3; // 0.3秒のフラッシュ
        }
    }
}

/// ダメージオーバーレイを更新
fn update_damage_overlay(
    time: Res<Time>,
    mut query: Query<(&mut DamageOverlay, &mut BackgroundColor)>,
) {
    for (mut overlay, mut color) in query.iter_mut() {
        if overlay.fade_timer > 0.0 {
            overlay.fade_timer -= time.delta_secs();
            let alpha = (overlay.fade_timer / 0.3).clamp(0.0, 1.0) * 0.3;
            *color = BackgroundColor(Color::srgba(1.0, 0.0, 0.0, alpha));
        } else {
            *color = BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_overlay_fade() {
        let mut overlay = DamageOverlay { fade_timer: 0.3 };
        overlay.fade_timer -= 0.1;
        assert!(overlay.fade_timer > 0.0);
        overlay.fade_timer -= 0.3;
        assert!(overlay.fade_timer <= 0.0);
    }
}
