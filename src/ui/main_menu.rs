// src/ui/main_menu.rs
//! メインメニューUI
//! - MainMenu: タイトル、Play/Settings/Quitボタン
//! - SaveSelect: セーブスロット選択
//! - WorldGeneration: 新規ワールド作成設定

use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::input::keyboard::{Key, KeyboardInput};
use crate::core::save_system::{
    SaveSlotData, SaveMetadata, WorldGenerationParams, save_metadata,
    PlayTimeTracker, WorldSaveData, SavedPlayerData, SavedInventorySlot,
    save_world_data, load_world_data,
};
use crate::gameplay::inventory::PlayerInventory;
use crate::gameplay::player::Player;
use crate::gameplay::commands::GameMode;
use crate::ui::settings_ui::SettingsUiState;

/// メインメニュープラグイン
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<ProfileList>()
            .init_resource::<SelectedGameMode>()
            // メインメニュー
            .add_systems(OnEnter(AppState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(AppState::MainMenu), despawn_with::<MainMenuUi>)
            // プロファイル選択
            .add_systems(OnEnter(AppState::ProfileSelect), spawn_profile_select)
            .add_systems(OnExit(AppState::ProfileSelect), despawn_with::<ProfileSelectUi>)
            // プロファイル設定
            .add_systems(OnEnter(AppState::ProfileSettings), spawn_profile_settings)
            .add_systems(OnExit(AppState::ProfileSettings), despawn_with::<ProfileSettingsUi>)
            // セーブ選択
            .add_systems(OnEnter(AppState::SaveSelect), spawn_save_select)
            .add_systems(OnExit(AppState::SaveSelect), despawn_with::<SaveSelectUi>)
            // ワールド生成
            .add_systems(OnEnter(AppState::WorldGeneration), spawn_world_generation)
            .add_systems(OnExit(AppState::WorldGeneration), despawn_with::<WorldGenUi>)
            // ポーズメニュー
            .add_systems(OnEnter(AppState::PauseMenu), spawn_pause_menu)
            .add_systems(OnExit(AppState::PauseMenu), despawn_with::<PauseMenuUi>)
            // InGame
            .add_systems(OnEnter(AppState::InGame), start_play_session)
            .add_systems(OnExit(AppState::InGame), end_play_session)
            .add_systems(Update, (
                button_interaction_system,
                main_menu_buttons.run_if(in_state(AppState::MainMenu)),
                profile_select_buttons.run_if(in_state(AppState::ProfileSelect)),
                profile_settings_buttons.run_if(in_state(AppState::ProfileSettings)),
                save_select_buttons.run_if(in_state(AppState::SaveSelect)),
                world_gen_buttons.run_if(in_state(AppState::WorldGeneration)),
                text_input_system.run_if(in_state(AppState::WorldGeneration)),
                update_text_input_display.run_if(in_state(AppState::WorldGeneration)),
                pause_menu_buttons.run_if(in_state(AppState::PauseMenu)),
                handle_menu_escape_key,
                handle_ingame_escape_key.run_if(in_state(AppState::InGame)),
            ));
    }
}

/// アプリケーション状態
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    ProfileSelect,
    ProfileSettings,
    SaveSelect,
    WorldGeneration,
    InGame,
    PauseMenu,
}

// ========================================
// マーカーコンポーネント
// ========================================

#[derive(Component)]
pub struct MainMenuUi;

#[derive(Component)]
pub struct ProfileSelectUi;

#[derive(Component)]
pub struct ProfileSettingsUi;

#[derive(Component)]
pub struct SaveSelectUi;

#[derive(Component)]
pub struct WorldGenUi;

#[derive(Component)]
pub struct PauseMenuUi;

/// ボタンの種類
#[derive(Component, Clone)]
pub enum MenuButtonAction {
    Play,
    Settings,
    Quit,
    Back,
    SelectSlot(usize),
    CreateWorld,
    DeleteSlot(usize),
    // プロファイル関連
    SelectProfile(String),
    EditProfile,
    CreateProfile,
    // ポーズメニュー関連
    Resume,
    ReturnToMainMenu,
    SaveAndQuit,
    // ゲームモード選択
    SelectGameMode(GameMode),
}

/// テキスト入力フィールド
#[derive(Component)]
pub struct TextInput {
    pub field_type: TextInputType,
    pub value: String,
    pub active: bool,
    /// 最初のクリックでデフォルト値をクリアするかどうか
    pub is_default: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextInputType {
    WorldName,
    Seed,
}

/// テキスト入力の表示用
#[derive(Component)]
pub struct TextInputDisplay(pub TextInputType);

/// 選択中のスロット
#[derive(Resource, Default)]
pub struct SelectedSlotIndex(pub Option<usize>);

/// 選択中のゲームモード（ワールド作成時）
#[derive(Resource)]
pub struct SelectedGameMode(pub GameMode);

impl Default for SelectedGameMode {
    fn default() -> Self {
        Self(GameMode::Creative) // デフォルトはクリエイティブ
    }
}

/// ゲームモードボタンのマーカー
#[derive(Component)]
pub struct GameModeButtonMarker(pub GameMode);

/// 利用可能なプロファイル一覧
#[derive(Resource)]
pub struct ProfileList {
    pub profiles: Vec<ProfileInfo>,
    pub active: String,
}

impl Default for ProfileList {
    fn default() -> Self {
        Self {
            profiles: vec![
                ProfileInfo { id: "vanilla".to_string(), name: "Vanilla".to_string(), description: "Official content".to_string() },
            ],
            active: "vanilla".to_string(),
        }
    }
}

/// プロファイル情報
#[derive(Clone)]
pub struct ProfileInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

// ========================================
// UIスタイル定数
// ========================================

const NORMAL_BUTTON: Color = Color::srgb(0.25, 0.25, 0.30);
const HOVERED_BUTTON: Color = Color::srgb(0.35, 0.35, 0.42);
const PRESSED_BUTTON: Color = Color::srgb(0.20, 0.20, 0.25);
const PANEL_BG: Color = Color::srgba(0.12, 0.12, 0.16, 0.95);
const TEXT_PRIMARY: Color = Color::WHITE;
const TEXT_SECONDARY: Color = Color::srgb(0.7, 0.7, 0.7);

// ========================================
// 汎用システム
// ========================================

/// 指定マーカーを持つエンティティを再帰的に削除
fn despawn_with<T: Component>(
    mut commands: Commands,
    query: Query<Entity, With<T>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

/// ボタンのインタラクション処理（色変更）
#[allow(clippy::type_complexity)]
fn button_interaction_system(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut bg_color) in &mut query {
        *bg_color = match *interaction {
            Interaction::Pressed => BackgroundColor(PRESSED_BUTTON),
            Interaction::Hovered => BackgroundColor(HOVERED_BUTTON),
            Interaction::None => BackgroundColor(NORMAL_BUTTON),
        };
    }
}

// ========================================
// メインメニュー
// ========================================

fn spawn_main_menu(mut commands: Commands) {
    commands.insert_resource(SelectedSlotIndex::default());

    // ルートノード
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
        MainMenuUi,
    )).with_children(|parent| {
        // パネル
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(40.0)),
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new("Infinite Voxel Factory"),
                TextFont { font_size: 36.0, ..default() },
                TextColor(TEXT_PRIMARY),
                Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
            ));

            // Play
            spawn_button(panel, "Play", MenuButtonAction::Play, 220.0);

            // Settings
            spawn_button(panel, "Settings", MenuButtonAction::Settings, 220.0);

            // Quit
            spawn_button(panel, "Quit", MenuButtonAction::Quit, 220.0);
        });
    });
}

#[allow(clippy::type_complexity)]
fn main_menu_buttons(
    query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed { continue; }

        match action {
            MenuButtonAction::Play => { next_state.set(AppState::ProfileSelect); }
            MenuButtonAction::Settings => { info!("Settings (not implemented)"); }
            MenuButtonAction::Quit => { exit.send(AppExit::Success); }
            _ => {}
        }
    }
}

// ========================================
// プロファイル選択画面
// ========================================

fn spawn_profile_select(
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
            BackgroundColor(PANEL_BG),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new("Select Profile"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(TEXT_PRIMARY),
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

fn spawn_profile_slot(parent: &mut ChildBuilder, profile: &ProfileInfo, is_active: bool) {
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
            TextColor(TEXT_PRIMARY),
        ));
        slot.spawn((
            Text::new(&profile.description),
            TextFont { font_size: 13.0, ..default() },
            TextColor(TEXT_SECONDARY),
        ));
    });
}

#[allow(clippy::type_complexity)]
fn profile_select_buttons(
    query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut profile_list: ResMut<ProfileList>,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed { continue; }

        match action {
            MenuButtonAction::Back => next_state.set(AppState::MainMenu),
            MenuButtonAction::EditProfile => next_state.set(AppState::ProfileSettings),
            MenuButtonAction::SelectProfile(id) => {
                profile_list.active = id.clone();
                next_state.set(AppState::SaveSelect);
            }
            _ => {}
        }
    }
}

// ========================================
// プロファイル設定画面
// ========================================

fn spawn_profile_settings(
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
            BackgroundColor(PANEL_BG),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new("Profile Settings"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(TEXT_PRIMARY),
                Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
            ));

            // 現在のプロファイル
            panel.spawn((
                Text::new(format!("Active: {}", profile_list.active)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(TEXT_SECONDARY),
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
                    TextColor(TEXT_PRIMARY),
                ));
                section.spawn((
                    Text::new("• Create new profiles in the Factory Data Architect editor"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(TEXT_SECONDARY),
                ));
                section.spawn((
                    Text::new("• Edit items, recipes, and quests for each profile"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(TEXT_SECONDARY),
                ));
                section.spawn((
                    Text::new("• Download MODs from Steam Workshop (coming soon)"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(TEXT_SECONDARY),
                ));
            });

            // ボタン
            spawn_button(panel, "Back", MenuButtonAction::Back, 180.0);
        });
    });
}

#[allow(clippy::type_complexity)]
fn profile_settings_buttons(
    query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed { continue; }

        if let MenuButtonAction::Back = action {
            next_state.set(AppState::ProfileSelect);
        }
    }
}

// ========================================
// ポーズメニュー
// ========================================

fn spawn_pause_menu(mut commands: Commands) {
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
            BackgroundColor(PANEL_BG),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new("Paused"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(TEXT_PRIMARY),
                Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
            ));

            spawn_button(panel, "Resume", MenuButtonAction::Resume, 200.0);
            spawn_button(panel, "Settings", MenuButtonAction::Settings, 200.0);
            spawn_button(panel, "Save & Quit", MenuButtonAction::SaveAndQuit, 200.0);
            spawn_button(panel, "Main Menu", MenuButtonAction::ReturnToMainMenu, 200.0);
        });
    });
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn pause_menu_buttons(
    query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut settings_state: ResMut<NextState<SettingsUiState>>,
    player_query: Query<(&Transform, &Player)>,
    player_inventory: Res<PlayerInventory>,
    world_params: Res<WorldGenerationParams>,
    game_mode: Res<GameMode>,
    mut slot_data: ResMut<SaveSlotData>,
    mut play_time: ResMut<PlayTimeTracker>,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed { continue; }

        match action {
            MenuButtonAction::Resume => next_state.set(AppState::InGame),
            MenuButtonAction::ReturnToMainMenu => next_state.set(AppState::MainMenu),
            MenuButtonAction::SaveAndQuit => {
                // セーブ処理
                if let Some(slot_index) = world_params.slot_index {
                    // プレイヤーデータを収集
                    if let Ok((transform, player)) = player_query.get_single() {
                        let saved_inventory: Vec<SavedInventorySlot> = player_inventory.slots.iter()
                            .map(|slot| SavedInventorySlot {
                                item_id: slot.item_id.clone(),
                                count: slot.count,
                            })
                            .collect();

                        let world_data = WorldSaveData {
                            player: SavedPlayerData {
                                position: [transform.translation.x, transform.translation.y, transform.translation.z],
                                yaw: player.yaw,
                                pitch: player.pitch,
                                inventory: saved_inventory,
                                selected_hotbar_slot: player_inventory.selected_hotbar_slot,
                            },
                            game_mode: format!("{:?}", *game_mode),
                        };

                        if let Err(e) = save_world_data(&world_data, slot_index) {
                            warn!("Failed to save world data: {}", e);
                        }

                        // プレイ時間を更新してメタデータを保存
                        play_time.end_session();
                        if let Some(meta) = slot_data.slots[slot_index].as_mut() {
                            meta.play_time = play_time.current_total();
                            meta.last_played_date = chrono::Utc::now();
                            if let Err(e) = save_metadata(meta, slot_index) {
                                warn!("Failed to save metadata: {}", e);
                            }
                        }

                        info!("Game saved to slot {}", slot_index);
                    }
                }
                next_state.set(AppState::MainMenu);
            }
            MenuButtonAction::Settings => {
                // 設定画面を開く
                settings_state.set(SettingsUiState::SettingsOpen);
            }
            _ => {}
        }
    }
}

/// InGame中のESCキー処理
fn handle_ingame_escape_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::PauseMenu);
    }
}

/// プレイセッション開始時の処理
fn start_play_session(
    mut play_time: ResMut<PlayTimeTracker>,
    world_params: Res<WorldGenerationParams>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    mut player_inventory: ResMut<PlayerInventory>,
    slot_data: Res<SaveSlotData>,
) {
    // プレイ時間トラッキング開始
    if let Some(slot_index) = world_params.slot_index {
        // メタデータからプレイ時間を復元
        if let Some(meta) = slot_data.get(slot_index) {
            play_time.total_seconds = meta.play_time;
        }
    }
    play_time.start_session();

    // 既存のワールドをロード
    if !world_params.is_new_world {
        if let Some(slot_index) = world_params.slot_index {
            match load_world_data(slot_index) {
                Ok(world_data) => {
                    // プレイヤー位置を復元
                    if let Ok((mut transform, mut player)) = player_query.get_single_mut() {
                        transform.translation = Vec3::new(
                            world_data.player.position[0],
                            world_data.player.position[1],
                            world_data.player.position[2],
                        );
                        player.yaw = world_data.player.yaw;
                        player.pitch = world_data.player.pitch;
                    }

                    // インベントリを復元
                    for (i, saved_slot) in world_data.player.inventory.iter().enumerate() {
                        if i < player_inventory.slots.len() {
                            player_inventory.slots[i].item_id = saved_slot.item_id.clone();
                            player_inventory.slots[i].count = saved_slot.count;
                        }
                    }
                    player_inventory.selected_hotbar_slot = world_data.player.selected_hotbar_slot;

                    info!("Loaded world from slot {}", slot_index);
                }
                Err(e) => {
                    info!("No world data to load: {}", e);
                }
            }
        }
    }
}

/// プレイセッション終了時の処理
fn end_play_session(
    mut play_time: ResMut<PlayTimeTracker>,
) {
    play_time.end_session();
}

// ========================================
// セーブ選択画面
// ========================================

fn spawn_save_select(
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
            BackgroundColor(PANEL_BG),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new("Select World"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(TEXT_PRIMARY),
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

fn spawn_save_slot(parent: &mut ChildBuilder, index: usize, meta: Option<&SaveMetadata>) {
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
                TextColor(TEXT_PRIMARY),
            ));
            slot.spawn((
                Text::new(format!("{} | {}", m.formatted_play_time(), m.formatted_date())),
                TextFont { font_size: 13.0, ..default() },
                TextColor(TEXT_SECONDARY),
            ));
        } else {
            slot.spawn((
                Text::new(format!("Slot {} - Empty", index + 1)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(TEXT_SECONDARY),
            ));
            slot.spawn((
                Text::new("Click to create new world"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        }
    });
}

#[allow(clippy::type_complexity)]
fn save_select_buttons(
    query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut selected: ResMut<SelectedSlotIndex>,
    slot_data: Res<SaveSlotData>,
    mut world_params: ResMut<WorldGenerationParams>,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed { continue; }

        match action {
            MenuButtonAction::Back => next_state.set(AppState::MainMenu),
            MenuButtonAction::SelectSlot(index) => {
                if let Some(meta) = slot_data.get(*index) {
                    // 既存ワールドをロード
                    world_params.world_name = meta.world_name.clone();
                    world_params.seed = meta.seed;
                    world_params.slot_index = Some(*index);
                    world_params.is_new_world = false;
                    next_state.set(AppState::InGame);
                } else {
                    // 新規作成画面へ
                    selected.0 = Some(*index);
                    next_state.set(AppState::WorldGeneration);
                }
            }
            _ => {}
        }
    }
}

// ========================================
// ワールド生成画面
// ========================================

fn spawn_world_generation(
    mut commands: Commands,
    selected: Res<SelectedSlotIndex>,
    selected_game_mode: Res<SelectedGameMode>,
) {
    let slot_index = selected.0.unwrap_or(0);
    let current_mode = selected_game_mode.0;

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
            BackgroundColor(PANEL_BG),
            BorderRadius::all(Val::Px(12.0)),
        )).with_children(|panel| {
            // タイトル
            panel.spawn((
                Text::new(format!("New World (Slot {})", slot_index + 1)),
                TextFont { font_size: 28.0, ..default() },
                TextColor(TEXT_PRIMARY),
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
                    TextColor(TEXT_SECONDARY),
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

/// ゲームモード選択ボタンを生成
fn spawn_game_mode_button(parent: &mut ChildBuilder, label: &str, mode: GameMode, is_selected: bool) {
    let bg_color = if is_selected {
        Color::srgb(0.3, 0.5, 0.3) // 選択中: 緑がかった色
    } else {
        NORMAL_BUTTON
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
            TextColor(TEXT_PRIMARY),
        ));
    });
}

fn spawn_text_input(parent: &mut ChildBuilder, label: &str, input_type: TextInputType, default_value: &str) {
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
            TextColor(TEXT_SECONDARY),
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
                TextColor(TEXT_PRIMARY),
                TextInputDisplay(input_type),
            ));
        });
    });
}

#[allow(clippy::type_complexity)]
fn world_gen_buttons(
    query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    input_query: Query<&TextInput>,
    mut next_state: ResMut<NextState<AppState>>,
    selected: Res<SelectedSlotIndex>,
    mut world_params: ResMut<WorldGenerationParams>,
    mut slot_data: ResMut<SaveSlotData>,
    mut selected_game_mode: ResMut<SelectedGameMode>,
    mut game_mode: ResMut<GameMode>,
    mut game_mode_buttons: Query<(&GameModeButtonMarker, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed { continue; }

        match action {
            MenuButtonAction::Back => next_state.set(AppState::SaveSelect),
            MenuButtonAction::SelectGameMode(mode) => {
                selected_game_mode.0 = *mode;
                // ボタンの見た目を更新
                for (marker, mut bg, mut border) in &mut game_mode_buttons {
                    if marker.0 == *mode {
                        *bg = BackgroundColor(Color::srgb(0.3, 0.5, 0.3));
                        *border = BorderColor(Color::srgb(0.5, 0.8, 0.5));
                    } else {
                        *bg = BackgroundColor(NORMAL_BUTTON);
                        *border = BorderColor(Color::srgb(0.3, 0.3, 0.35));
                    }
                }
            }
            MenuButtonAction::CreateWorld => {
                let mut world_name = "New World".to_string();
                let mut seed: u64 = generate_random_seed();

                for input in &input_query {
                    match input.field_type {
                        TextInputType::WorldName if !input.value.is_empty() => {
                            world_name = input.value.clone();
                        }
                        TextInputType::Seed if !input.value.is_empty() => {
                            seed = input.value.parse().unwrap_or_else(|_| {
                                // 文字列をハッシュ化
                                input.value.bytes().fold(0u64, |acc, b| {
                                    acc.wrapping_mul(31).wrapping_add(b as u64)
                                })
                            });
                        }
                        _ => {}
                    }
                }

                let slot_index = selected.0.unwrap_or(0);

                // メタデータを作成・保存
                let meta = SaveMetadata::new(slot_index, &world_name, seed);
                if let Err(e) = save_metadata(&meta, slot_index) {
                    warn!("Failed to save metadata: {}", e);
                }
                slot_data.set(slot_index, meta);

                // ゲームモードを設定
                *game_mode = selected_game_mode.0;

                // パラメータを設定
                world_params.world_name = world_name;
                world_params.seed = seed;
                world_params.slot_index = Some(slot_index);
                world_params.is_new_world = true;

                next_state.set(AppState::InGame);
            }
            _ => {}
        }
    }
}

fn text_input_system(
    mut input_query: Query<(&Interaction, &mut TextInput, &mut BackgroundColor)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut key_events: EventReader<KeyboardInput>,
) {
    // キー入力を収集
    let mut chars_to_add: Vec<String> = Vec::new();
    for event in key_events.read() {
        if !event.state.is_pressed() { continue; }
        if let Key::Character(c) = &event.logical_key {
            chars_to_add.push(c.to_string());
        }
    }

    // 入力処理
    for (interaction, mut input, mut bg) in &mut input_query {
        // クリックでフォーカス切り替え
        if *interaction == Interaction::Pressed {
            // デフォルト値の場合、クリックでクリア
            if input.is_default {
                input.value.clear();
                input.is_default = false;
            }
            input.active = true;
            *bg = BackgroundColor(Color::srgb(0.22, 0.22, 0.28));
        }

        if !input.active { continue; }

        // Backspace
        if keyboard.just_pressed(KeyCode::Backspace) && !input.value.is_empty() {
            input.value.pop();
        }

        // Enter でフォーカス解除
        if keyboard.just_pressed(KeyCode::Enter) {
            input.active = false;
            *bg = BackgroundColor(Color::srgb(0.15, 0.15, 0.18));
        }

        // 文字入力 - デフォルト値フラグを解除
        for c in &chars_to_add {
            for ch in c.chars() {
                if (ch.is_ascii_alphanumeric() || ch == ' ' || ch == '_' || ch == '-') && input.value.len() < 32 {
                    input.value.push(ch);
                    input.is_default = false;
                }
            }
        }
    }
}

/// テキスト入力の表示を更新
fn update_text_input_display(
    input_query: Query<&TextInput>,
    mut display_query: Query<(&mut Text, &TextInputDisplay)>,
) {
    for input in &input_query {
        for (mut text, display) in &mut display_query {
            if display.0 == input.field_type {
                **text = if input.value.is_empty() { " ".to_string() } else { input.value.clone() };
            }
        }
    }
}

// ========================================
// ヘルパー関数
// ========================================

fn spawn_button(parent: &mut ChildBuilder, text: &str, action: MenuButtonAction, width: f32) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(width),
            height: Val::Px(48.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
        BorderRadius::all(Val::Px(6.0)),
        action,
    )).with_children(|btn| {
        btn.spawn((
            Text::new(text),
            TextFont { font_size: 18.0, ..default() },
            TextColor(TEXT_PRIMARY),
        ));
    });
}

fn generate_random_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

/// メニュー画面でのESCキー処理
fn handle_menu_escape_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::ProfileSelect => next_state.set(AppState::MainMenu),
            AppState::ProfileSettings => next_state.set(AppState::ProfileSelect),
            AppState::SaveSelect => next_state.set(AppState::ProfileSelect),
            AppState::WorldGeneration => next_state.set(AppState::SaveSelect),
            AppState::PauseMenu => next_state.set(AppState::InGame),
            _ => {}
        }
    }
}
