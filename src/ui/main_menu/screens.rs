// src/ui/main_menu/screens.rs
//! 各画面のUI構築関数

use bevy::prelude::*;
use crate::core::save_system::{SaveSlotData, SaveMetadata};
use crate::gameplay::commands::GameMode;
use crate::core::worldgen::WorldType;
use crate::ui::styles::{colors, sizes, fonts};
use crate::ui::styles::colors::{BUTTON_DEFAULT, TEXT_PRIMARY, TEXT_SECONDARY};
use super::types::*;

// ========================================
// 汎用システム
// ========================================

/// 指定マーカーを持つエンティティを再帰的に削除
pub fn despawn_with<T: Component>(
    mut commands: Commands,
    query: Query<Entity, With<T>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}


// ========================================
// メインメニュー
// ========================================

pub fn spawn_main_menu(mut commands: Commands) {
    commands.insert_resource(SelectedSlotIndex::default());

    // ルートノード - グラデーション背景
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(colors::BG_DARK),
        GlobalZIndex(100),
        MainMenuUi,
    )).with_children(|parent| {
        // モダンなパネル - Glassmorphism風
        parent.spawn((
            Node {
                width: Val::Px(420.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(sizes::PANEL_PADDING + 16.0)),
                row_gap: Val::Px(sizes::PANEL_GAP),
                border: UiRect::all(Val::Px(sizes::BORDER_THIN)),
                ..default()
            },
            BackgroundColor(colors::BG_PANEL),
            BorderColor(colors::BORDER),
            BorderRadius::all(Val::Px(sizes::RADIUS_LG)),
        )).with_children(|panel| {
            // タイトル - より大きく、アクセント付き
            panel.spawn((
                Text::new("Infinite Voxel Factory"),
                TextFont { font_size: fonts::TITLE_LG, ..default() },
                TextColor(colors::TEXT_PRIMARY),
                Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
            ));

            // サブタイトル
            panel.spawn((
                Text::new("Build Your Dream Factory"),
                TextFont { font_size: fonts::BODY_SM, ..default() },
                TextColor(colors::TEXT_SECONDARY),
                Node { margin: UiRect::bottom(Val::Px(24.0)), ..default() },
            ));

            // セパレータ
            panel.spawn((
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(colors::BORDER),
            ));

            // Play - プライマリボタン
            spawn_modern_button(panel, "Play", MenuButtonAction::Play, true);

            // Settings
            spawn_modern_button(panel, "Settings", MenuButtonAction::Settings, false);

            // Quit
            spawn_modern_button(panel, "Quit", MenuButtonAction::Quit, false);

            // バージョン情報
            panel.spawn((
                Text::new("v0.1.0 - Early Development"),
                TextFont { font_size: fonts::CAPTION, ..default() },
                TextColor(colors::TEXT_DISABLED),
                Node { margin: UiRect::top(Val::Px(16.0)), ..default() },
            ));
        });
    });
}

// ========================================
// プロファイル選択画面
// ========================================

pub fn spawn_profile_select(
    mut commands: Commands,
    profile_list: Res<ProfileList>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95)),
        GlobalZIndex(100),
        ProfileSelectUi,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(450.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(colors::BG_PANEL),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new("Select Profile"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(colors::TEXT_PRIMARY),
                Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
            ));

            // プロファイル一覧
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(250.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
            )).with_children(|scroll| {
                for profile in &profile_list.profiles {
                    spawn_profile_slot(scroll, profile, profile.id == profile_list.active);
                }
            });

            // ボタン行
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            )).with_children(|row| {
                spawn_button(row, "Back", MenuButtonAction::Back, 120.0);
                spawn_button(row, "Settings", MenuButtonAction::EditProfile, 120.0);
                spawn_button(row, "Continue", MenuButtonAction::SelectProfile(profile_list.active.clone()), 120.0);
            });
        });
    });
}

// ========================================
// プロファイル選択画面
// ========================================

pub fn spawn_profile_slot(parent: &mut ChildBuilder, profile: &ProfileInfo, is_active: bool) {
    let bg = if is_active {
        Color::srgb(0.20, 0.35, 0.25)
    } else {
        Color::srgb(0.18, 0.18, 0.22)
    };

    parent.spawn((
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            padding: UiRect::horizontal(Val::Px(15.0)),
            ..default()
        },
        BackgroundColor(bg),
        BorderRadius::all(Val::Px(6.0)),
        MenuButtonAction::SelectProfile(profile.id.clone()),
    )).with_children(|slot| {
        slot.spawn((
            Text::new(format!("{}{}", profile.name, if is_active { " ✓" } else { "" })),
            TextFont { font_size: 18.0, ..default() },
            TextColor(colors::TEXT_PRIMARY),
        ));
        slot.spawn((
            Text::new(&profile.description),
            TextFont { font_size: 13.0, ..default() },
            TextColor(colors::TEXT_SECONDARY),
        ));
    });
}

// ========================================
// プロファイル設定画面
// ========================================

pub fn spawn_profile_settings(
    mut commands: Commands,
    profile_list: Res<ProfileList>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95)),
        GlobalZIndex(100),
        ProfileSettingsUi,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(500.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(colors::BG_PANEL),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new("Profile Settings"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(colors::TEXT_PRIMARY),
                Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
            ));

            // 現在のプロファイル
            panel.spawn((
                Text::new(format!("Active: {}", profile_list.active)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(colors::TEXT_SECONDARY),
            ));

            // プロファイル管理セクション
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(15.0)),
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderRadius::all(Val::Px(6.0)),
            )).with_children(|section| {
                section.spawn((
                    Text::new("Profile Management"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(colors::TEXT_PRIMARY),
                ));
                section.spawn((
                    Text::new("• Create new profiles in the Factory Data Architect editor"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(colors::TEXT_SECONDARY),
                ));
                section.spawn((
                    Text::new("• Edit items, recipes, and quests for each profile"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(colors::TEXT_SECONDARY),
                ));
                section.spawn((
                    Text::new("• Download MODs from Steam Workshop (coming soon)"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(colors::TEXT_SECONDARY),
                ));
            });

            // ボタン
            spawn_button(panel, "Back", MenuButtonAction::Back, 180.0);
        });
    });
}

// ========================================
// ポーズメニュー
// ========================================

pub fn spawn_pause_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        GlobalZIndex(200), // ゲームUIより前面
        PauseMenuUi,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(350.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(colors::BG_PANEL),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new("Paused"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(colors::TEXT_PRIMARY),
                Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
            ));

            spawn_button(panel, "Resume", MenuButtonAction::Resume, 200.0);
            spawn_button(panel, "Settings", MenuButtonAction::Settings, 200.0);
            spawn_button(panel, "Save & Quit", MenuButtonAction::SaveAndQuit, 200.0);
            spawn_button(panel, "Main Menu", MenuButtonAction::ReturnToMainMenu, 200.0);
        });
    });
}

// ========================================
// セーブ選択画面
// ========================================

pub fn spawn_save_select(
    mut commands: Commands,
    slot_data: Res<SaveSlotData>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95)),
        GlobalZIndex(100), // 最前面に表示
        SaveSelectUi,
    )).with_children(|parent| {
        // パネル
        parent.spawn((
            Node {
                width: Val::Px(500.0),
                height: Val::Px(550.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(25.0)),
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(colors::BG_PANEL),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new("Select World"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(colors::TEXT_PRIMARY),
                Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
            ));

            // スクロールエリア
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(380.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
            )).with_children(|scroll| {
                for i in 0..8 {
                    spawn_save_slot(scroll, i, slot_data.get(i));
                }
            });

            // Backボタン
            spawn_button(panel, "Back", MenuButtonAction::Back, 180.0);
        });
    });
}

// ========================================
// セーブ選択画面
// ========================================

pub fn spawn_save_slot(parent: &mut ChildBuilder, index: usize, meta: Option<&SaveMetadata>) {
    let bg = if meta.is_some() {
        Color::srgb(0.18, 0.28, 0.20)
    } else {
        Color::srgb(0.18, 0.18, 0.22)
    };

    parent.spawn((
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(70.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            padding: UiRect::horizontal(Val::Px(15.0)),
            ..default()
        },
        BackgroundColor(bg),
        BorderRadius::all(Val::Px(6.0)),
        MenuButtonAction::SelectSlot(index),
    )).with_children(|slot| {
        if let Some(m) = meta {
            slot.spawn((
                Text::new(&m.world_name),
                TextFont { font_size: 18.0, ..default() },
                TextColor(colors::TEXT_PRIMARY),
            ));
            slot.spawn((
                Text::new(format!("{} | {}", m.formatted_play_time(), m.formatted_date())),
                TextFont { font_size: 13.0, ..default() },
                TextColor(colors::TEXT_SECONDARY),
            ));
        } else {
            slot.spawn((
                Text::new(format!("Slot {} - Empty", index + 1)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(colors::TEXT_SECONDARY),
            ));
            slot.spawn((
                Text::new("Click to create new world"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        }
    });
}

// ========================================
// ワールド生成画面
// ========================================

pub fn spawn_world_generation(
    mut commands: Commands,
    selected: Res<SelectedSlotIndex>,
    selected_game_mode: Res<SelectedGameMode>,
    selected_world_type: Res<SelectedWorldType>,
) {
    let slot_index = selected.0.unwrap_or(0);
    let current_mode = selected_game_mode.0;
    let current_world_type = selected_world_type.0;

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95)),
        GlobalZIndex(100), // 最前面に表示
        WorldGenUi,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(450.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(colors::BG_PANEL),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new(format!("New World (Slot {})", slot_index + 1)),
                TextFont { font_size: 28.0, ..default() },
                TextColor(colors::TEXT_PRIMARY),
                Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
            ));

            // ワールド名入力
            spawn_text_input(panel, "World Name", TextInputType::WorldName, "New World");

            // シード値入力
            spawn_text_input(panel, "Seed (optional)", TextInputType::Seed, "");

            // ゲームモード選択
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    ..default()
                },
            )).with_children(|container| {
                // ラベル
                container.spawn((
                    Text::new("Game Mode"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(colors::TEXT_SECONDARY),
                ));

                // ゲームモードボタン行
                container.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                )).with_children(|row| {
                    spawn_game_mode_button(row, "Survival", GameMode::Survival, current_mode == GameMode::Survival);
                    spawn_game_mode_button(row, "Creative", GameMode::Creative, current_mode == GameMode::Creative);
                });
            });

            // ワールドタイプ選択
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    ..default()
                },
            )).with_children(|container| {
                // ラベル
                container.spawn((
                    Text::new("World Type"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(TEXT_SECONDARY),
                ));

                // ワールドタイプボタン行
                container.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                )).with_children(|row| {
                    spawn_world_type_button(row, "Normal", WorldType::Normal, current_world_type == WorldType::Normal);
                    spawn_world_type_button(row, "Flat", WorldType::Flat, current_world_type == WorldType::Flat);
                });
            });

            // ボタン行
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                },
            )).with_children(|row| {
                spawn_button(row, "Back", MenuButtonAction::Back, 140.0);
                spawn_button(row, "Create", MenuButtonAction::CreateWorld, 140.0);
            });
        });
    });
}

// ========================================
// ワールド生成画面
// ========================================

pub fn spawn_game_mode_button(parent: &mut ChildBuilder, label: &str, mode: GameMode, is_selected: bool) {
    let bg_color = if is_selected {
        Color::srgb(0.3, 0.5, 0.3) // 選択中: 緑がかった色
    } else {
        colors::BUTTON_DEFAULT
    };
    let border_color = if is_selected {
        Color::srgb(0.5, 0.8, 0.5)
    } else {
        Color::srgb(0.3, 0.3, 0.35)
    };

    parent.spawn((
        Button,
        Node {
            width: Val::Percent(50.0),
            height: Val::Px(40.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(bg_color),
        BorderColor(border_color),
        BorderRadius::all(Val::Px(6.0)),
        MenuButtonAction::SelectGameMode(mode),
        GameModeButtonMarker(mode),
    )).with_children(|btn| {
        btn.spawn((
            Text::new(label),
            TextFont { font_size: 16.0, ..default() },
            TextColor(colors::TEXT_PRIMARY),
        ));
    });
}

// ========================================
// ワールド生成画面
// ========================================

pub fn spawn_world_type_button(parent: &mut ChildBuilder, label: &str, world_type: WorldType, is_selected: bool) {
    let bg_color = if is_selected {
        Color::srgb(0.3, 0.4, 0.5) // 選択中: 青みがかった色
    } else {
        BUTTON_DEFAULT
    };
    let border_color = if is_selected {
        Color::srgb(0.5, 0.7, 0.9)
    } else {
        Color::srgb(0.3, 0.3, 0.35)
    };

    parent.spawn((
        Button,
        Node {
            width: Val::Percent(50.0),
            height: Val::Px(40.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(bg_color),
        BorderColor(border_color),
        BorderRadius::all(Val::Px(6.0)),
        MenuButtonAction::SelectWorldType(world_type),
        WorldTypeButtonMarker(world_type),
    )).with_children(|btn| {
        btn.spawn((
            Text::new(label),
            TextFont { font_size: 16.0, ..default() },
            TextColor(TEXT_PRIMARY),
        ));
    });
}

// ========================================
// ワールド生成画面
// ========================================

pub fn spawn_text_input(parent: &mut ChildBuilder, label: &str, input_type: TextInputType, default_value: &str) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(5.0),
            ..Default::default()
        },
    )).with_children(|container| {
        // ラベル
        container.spawn((
            Text::new(label),
            TextFont { font_size: 14.0, ..Default::default() },
            TextColor(colors::TEXT_SECONDARY),
        ));

        // 入力フィールド
        container.spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                padding: UiRect::horizontal(Val::Px(10.0)),
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
            BorderRadius::all(Val::Px(4.0)),
            TextInput {
                field_type: input_type,
                value: default_value.to_string(),
                active: false,
                is_default: true,
            },
        )).with_children(|field| {
            field.spawn((
                Text::new(if default_value.is_empty() { " " } else { default_value }),
                TextFont { font_size: 16.0, ..Default::default() },
                TextColor(colors::TEXT_PRIMARY),
                TextInputDisplay(input_type),
            ));
        });
    });
}

// ========================================
// ヘルパー関数
// ========================================

pub fn spawn_button(parent: &mut ChildBuilder, text: &str, action: MenuButtonAction, width: f32) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(width),
            height: Val::Px(sizes::BUTTON_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(sizes::BORDER_THIN)),
            ..default()
        },
        BackgroundColor(colors::BUTTON_DEFAULT),
        BorderColor(colors::BORDER),
        BorderRadius::all(Val::Px(sizes::RADIUS_MD)),
        action,
    )).with_children(|btn| {
        btn.spawn((
            Text::new(text),
            TextFont { font_size: fonts::BODY_MD, ..default() },
            TextColor(colors::TEXT_PRIMARY),
        ));
    });
}

// ========================================
// ヘルパー関数
// ========================================

pub fn spawn_modern_button(parent: &mut ChildBuilder, text: &str, action: MenuButtonAction, is_primary: bool) {
    let (bg_color, border_color) = if is_primary {
        (colors::BUTTON_PRIMARY, colors::ACCENT_PRIMARY)
    } else {
        (colors::BUTTON_DEFAULT, colors::BORDER)
    };

    parent.spawn((
        Button,
        Node {
            width: Val::Px(240.0),
            height: Val::Px(sizes::BUTTON_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(sizes::BORDER_THIN)),
            ..default()
        },
        BackgroundColor(bg_color),
        BorderColor(border_color),
        BorderRadius::all(Val::Px(sizes::RADIUS_MD)),
        action,
    )).with_children(|btn| {
        btn.spawn((
            Text::new(text),
            TextFont { font_size: fonts::BODY_LG, ..default() },
            TextColor(colors::TEXT_PRIMARY),
        ));
    });
}
