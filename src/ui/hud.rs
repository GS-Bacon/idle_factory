use bevy::prelude::*;

/// クロスヘアマーカー
#[derive(Component)]
pub struct CrosshairRoot;

// クロスヘア（照準）を描画 - モダンなスタイル
pub fn spawn_crosshair(mut commands: Commands) {
    // モダンなクロスヘア: 微細なドット + 十字
    let outer_color = Color::srgba(0.95, 0.95, 0.97, 0.6);
    let inner_color = Color::srgba(0.0, 0.0, 0.0, 0.3);
    let thickness = Val::Px(2.0);
    let length = Val::Px(12.0);

    // 画面中央に配置する親Node
    commands.spawn((
        CrosshairRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )).with_children(|parent| {
        // 中央ドット（背景）
        parent.spawn((
            Node {
                width: Val::Px(4.0),
                height: Val::Px(4.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(inner_color),
            BorderRadius::all(Val::Px(2.0)),
        ));

        // 中央ドット（前景）
        parent.spawn((
            Node {
                width: Val::Px(2.0),
                height: Val::Px(2.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(outer_color),
            BorderRadius::all(Val::Px(1.0)),
        ));

        // 横棒（左）
        parent.spawn((
            Node {
                width: length,
                height: thickness,
                position_type: PositionType::Absolute,
                left: Val::Px(-18.0),
                ..default()
            },
            BackgroundColor(outer_color),
            BorderRadius::all(Val::Px(1.0)),
        ));

        // 横棒（右）
        parent.spawn((
            Node {
                width: length,
                height: thickness,
                position_type: PositionType::Absolute,
                right: Val::Px(-18.0),
                ..default()
            },
            BackgroundColor(outer_color),
            BorderRadius::all(Val::Px(1.0)),
        ));

        // 縦棒（上）
        parent.spawn((
            Node {
                width: thickness,
                height: length,
                position_type: PositionType::Absolute,
                top: Val::Px(-18.0),
                ..default()
            },
            BackgroundColor(outer_color),
            BorderRadius::all(Val::Px(1.0)),
        ));

        // 縦棒（下）
        parent.spawn((
            Node {
                width: thickness,
                height: length,
                position_type: PositionType::Absolute,
                bottom: Val::Px(-18.0),
                ..default()
            },
            BackgroundColor(outer_color),
            BorderRadius::all(Val::Px(1.0)),
        ));
    });
}

/// クロスヘアを削除
pub fn despawn_crosshair(
    mut commands: Commands,
    query: Query<Entity, With<CrosshairRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}