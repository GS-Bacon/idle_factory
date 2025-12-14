use bevy::prelude::*;

// クロスヘア（照準）を描画
pub fn spawn_crosshair(mut commands: Commands) {
    let color = Color::srgba(1.0, 1.0, 1.0, 0.5); // 半透明の白
    let thickness = Val::Px(2.0);
    let length = Val::Px(10.0);

    // 画面中央に配置する親Node (色は不要なので Node だけでOK)
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center, // 上下中央
        justify_content: JustifyContent::Center, // 左右中央
        ..default()
    }).with_children(|parent| {
        // 横棒
        parent.spawn((
            Node {
                width: length,
                height: thickness,
                ..default()
            },
            BackgroundColor(color), // ★修正: 色は別コンポーネントとして渡す
        ));
        
        // 縦棒
        parent.spawn((
            Node {
                width: thickness,
                height: length,
                position_type: PositionType::Absolute, // 重ねて表示
                ..default()
            },
            BackgroundColor(color), // ★修正: ここも同じ
        ));
    });
}