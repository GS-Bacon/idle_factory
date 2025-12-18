// src/ui/quest_hud.rs
//! クエストHUD
//! - 現在のアクティブクエスト表示
//! - 進捗バー
//! - クエスト詳細画面

use bevy::prelude::*;
use crate::gameplay::quest::{QuestRegistry, QuestManager, QuestStatus};
use crate::ui::main_menu::AppState;

/// クエストHUDの状態
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuestUiState {
    #[default]
    Minimized,  // 最小表示（アクティブクエストのみ）
    Expanded,   // 拡大表示（クエスト一覧）
}

/// クエストHUDのルートノード
#[derive(Component)]
pub struct QuestHudRoot;

/// クエスト一覧パネル
#[derive(Component)]
pub struct QuestListPanel;

/// アクティブクエスト表示
#[derive(Component)]
pub struct ActiveQuestDisplay;

/// クエストアイテム（一覧内の各クエスト）
#[derive(Component)]
pub struct QuestListItem {
    pub quest_id: String,
}

/// クエスト進捗バー
#[derive(Component)]
pub struct QuestProgressBar {
    pub quest_id: String,
}

/// クエストHUDプラグイン
pub struct QuestHudPlugin;

impl Plugin for QuestHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<QuestUiState>()
            .add_systems(OnEnter(AppState::InGame), spawn_quest_hud)
            .add_systems(OnExit(AppState::InGame), despawn_quest_hud)
            .add_systems(Update, (
                toggle_quest_panel,
                update_active_quest_display,
                update_quest_list,
            ).run_if(in_state(AppState::InGame)));
    }
}

/// クエストHUDをスポーン
fn spawn_quest_hud(mut commands: Commands) {
    // アクティブクエスト表示（右側中央）
    commands.spawn((
        QuestHudRoot,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(180.0), // ミニマップの下
            width: Val::Px(200.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
        BorderRadius::all(Val::Px(4.0)),
    )).with_children(|parent| {
        // タイトル
        parent.spawn((
            Node {
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            },
            Text::new("Quest"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.9, 0.3, 1.0)),
        ));

        // アクティブクエスト表示エリア
        parent.spawn((
            ActiveQuestDisplay,
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
        )).with_children(|quest_parent| {
            // プレースホルダー
            quest_parent.spawn((
                Text::new("No active quest"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
            ));
        });

        // ヒント
        parent.spawn((
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
            Text::new("[J] Quest List"),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
        ));
    });

    // クエスト一覧パネル（初期は非表示）
    commands.spawn((
        QuestListPanel,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(25.0),
            top: Val::Percent(15.0),
            width: Val::Percent(50.0),
            height: Val::Percent(70.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            display: Display::None, // 初期は非表示
            ..default()
        },
        BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.95)),
        BorderRadius::all(Val::Px(8.0)),
    )).with_children(|parent| {
        // タイトルバー
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
        )).with_children(|title_parent| {
            title_parent.spawn((
                Text::new("Quest List"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.9, 0.3, 1.0)),
            ));

            title_parent.spawn((
                Text::new("[J] Close"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
            ));
        });

        // メインクエストセクション
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        )).with_children(|section_parent| {
            section_parent.spawn((
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
                Text::new("Main Quest"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.9, 0.7, 0.2, 1.0)),
            ));
        });

        // サブクエストセクション
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
        )).with_children(|section_parent| {
            section_parent.spawn((
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
                Text::new("Sub Quests"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.5, 0.7, 0.9, 1.0)),
            ));
        });
    });
}

/// クエストHUDを削除
fn despawn_quest_hud(
    mut commands: Commands,
    hud_query: Query<Entity, With<QuestHudRoot>>,
    panel_query: Query<Entity, With<QuestListPanel>>,
) {
    for entity in &hud_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &panel_query {
        commands.entity(entity).despawn_recursive();
    }
}

/// クエストパネルの表示切り替え
fn toggle_quest_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    quest_state: Res<State<QuestUiState>>,
    mut next_state: ResMut<NextState<QuestUiState>>,
    mut panel_query: Query<&mut Node, With<QuestListPanel>>,
) {
    if keyboard.just_pressed(KeyCode::KeyJ) {
        let new_state = match quest_state.get() {
            QuestUiState::Minimized => QuestUiState::Expanded,
            QuestUiState::Expanded => QuestUiState::Minimized,
        };
        next_state.set(new_state);

        // パネルの表示/非表示を切り替え
        if let Ok(mut node) = panel_query.get_single_mut() {
            node.display = match new_state {
                QuestUiState::Minimized => Display::None,
                QuestUiState::Expanded => Display::Flex,
            };
        }
    }
}

/// アクティブクエスト表示を更新
fn update_active_quest_display(
    quest_registry: Res<QuestRegistry>,
    quest_manager: Res<QuestManager>,
    mut commands: Commands,
    display_query: Query<Entity, With<ActiveQuestDisplay>>,
) {
    let Ok(display_entity) = display_query.get_single() else {
        return;
    };

    // 子要素をクリアして再構築
    commands.entity(display_entity).despawn_descendants();

    // アクティブなクエストを取得
    let active_quest_ids: Vec<&String> = quest_manager.progress
        .iter()
        .filter(|(_, progress)| progress.status == QuestStatus::Active)
        .map(|(id, _)| id)
        .collect();

    commands.entity(display_entity).with_children(|parent| {
        if active_quest_ids.is_empty() {
            parent.spawn((
                Text::new("No active quest"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
            ));
        } else {
            // 最初のアクティブクエストを表示
            if let Some(quest_id) = active_quest_ids.first() {
                if let Some(quest_data) = quest_registry.get(quest_id) {
                    if let Some(progress) = quest_manager.progress.get(*quest_id) {
                        // クエスト名
                        parent.spawn((
                            Text::new(&quest_data.i18n_key),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                        ));

                        // 進捗計算
                        let (current, required) = quest_data.requirements.iter().fold(
                            (0u32, 0u32),
                            |(cur, req), requirement| {
                                let delivered = progress.get_delivered(&requirement.item_id);
                                (cur + delivered.min(requirement.amount), req + requirement.amount)
                            }
                        );
                        let progress_ratio = if required > 0 {
                            current as f32 / required as f32
                        } else {
                            0.0
                        };

                        // 進捗バー背景
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(8.0),
                                margin: UiRect::vertical(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 1.0)),
                            BorderRadius::all(Val::Px(2.0)),
                        )).with_children(|bar_parent| {
                            // 進捗バー
                            bar_parent.spawn((
                                Node {
                                    width: Val::Percent(progress_ratio * 100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.3, 0.8, 0.3, 1.0)),
                                BorderRadius::all(Val::Px(2.0)),
                            ));
                        });

                        // 進捗テキスト
                        let progress_text = format!("{} / {}", current, required);
                        parent.spawn((
                            Text::new(progress_text),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                        ));
                    }
                }
            }
        }
    });
}

/// クエスト一覧を更新
fn update_quest_list(
    _quest_registry: Res<QuestRegistry>,
    _quest_manager: Res<QuestManager>,
    _quest_state: Res<State<QuestUiState>>,
) {
    // クエスト一覧パネルが開いている時のみ更新
    // 実装簡略化のため、現在はアクティブクエスト表示のみ
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_ui_state_default() {
        let state = QuestUiState::default();
        assert_eq!(state, QuestUiState::Minimized);
    }
}
