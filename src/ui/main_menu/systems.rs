// src/ui/main_menu/systems.rs
//! システム関数

use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::input::keyboard::{Key, KeyboardInput};
use crate::core::save_system::{
    SaveSlotData, SaveMetadata, WorldGenerationParams,
    PlayTimeTracker, WorldSaveData, SavedPlayerData, SavedInventorySlot,
    save_world_data, load_world_data, save_metadata,
};
use crate::gameplay::inventory::PlayerInventory;
use crate::gameplay::player::Player;
use crate::gameplay::commands::GameMode;
use crate::ui::settings_ui::SettingsUiState;
use crate::ui::styles::colors;
use super::types::*;

// ========================================
// 汎用システム
// ========================================

/// ボタンのインタラクション処理（モダン：色変更 + ボーダー）
#[allow(clippy::type_complexity)]
pub fn button_interaction_system(
    mut query: Query<(&Interaction, &mut BackgroundColor, &mut BorderColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(colors::BUTTON_PRESSED);
                *border_color = BorderColor(colors::BORDER_ACTIVE);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(colors::BUTTON_HOVER);
                *border_color = BorderColor(colors::BORDER_ACTIVE);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(colors::BUTTON_DEFAULT);
                *border_color = BorderColor(colors::BORDER);
            }
        }
    }
}


// ========================================
// main_menu_buttons
// ========================================

pub fn main_menu_buttons(
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
// ========================================

pub fn profile_select_buttons(
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
// ========================================

pub fn profile_settings_buttons(
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
// ========================================

pub fn save_select_buttons(
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
                    world_params.world_type = meta.world_type;
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
// ========================================

pub fn world_gen_buttons(
    query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    input_query: Query<&TextInput>,
    mut next_state: ResMut<NextState<AppState>>,
    selected: Res<SelectedSlotIndex>,
    mut world_params: ResMut<WorldGenerationParams>,
    mut slot_data: ResMut<SaveSlotData>,
    mut selected_game_mode: ResMut<SelectedGameMode>,
    mut selected_world_type: ResMut<SelectedWorldType>,
    mut game_mode: ResMut<GameMode>,
    mut game_mode_buttons: Query<(&GameModeButtonMarker, &mut BackgroundColor, &mut BorderColor), Without<WorldTypeButtonMarker>>,
    mut world_type_buttons: Query<(&WorldTypeButtonMarker, &mut BackgroundColor, &mut BorderColor), Without<GameModeButtonMarker>>,
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
                        *bg = BackgroundColor(colors::BUTTON_DEFAULT);
                        *border = BorderColor(Color::srgb(0.3, 0.3, 0.35));
                    }
                }
            }
            MenuButtonAction::SelectWorldType(world_type) => {
                selected_world_type.0 = *world_type;
                // ボタンの見た目を更新
                for (marker, mut bg, mut border) in &mut world_type_buttons {
                    if marker.0 == *world_type {
                        *bg = BackgroundColor(Color::srgb(0.3, 0.4, 0.5));
                        *border = BorderColor(Color::srgb(0.5, 0.7, 0.9));
                    } else {
                        *bg = BackgroundColor(colors::BUTTON_DEFAULT);
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

                // メタデータを作成・保存（ワールドタイプを含む）
                let meta = SaveMetadata::with_world_type(
                    slot_index,
                    &world_name,
                    seed,
                    selected_world_type.0,
                );
                if let Err(e) = save_metadata(&meta, slot_index) {
                    warn!("Failed to save metadata: {}", e);
                }
                slot_data.set(slot_index, meta);

                // ゲームモードを設定
                *game_mode = selected_game_mode.0;

                // パラメータを設定（ワールドタイプを含む）
                world_params.world_name = world_name;
                world_params.seed = seed;
                world_params.slot_index = Some(slot_index);
                world_params.is_new_world = true;
                world_params.world_type = selected_world_type.0;

                next_state.set(AppState::InGame);
            }
            _ => {}
        }
    }
}

// ========================================
// pause_menu_buttons
// ========================================

pub fn pause_menu_buttons(
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

// ========================================
// テキスト入力
// ========================================

pub fn text_input_system(
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

// ========================================
// テキスト入力
// ========================================

pub fn update_text_input_display(
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
// キーボード入力
// ========================================

pub fn handle_ingame_escape_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::PauseMenu);
    }
}

// ========================================
// キーボード入力
// ========================================

pub fn handle_menu_escape_key(
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

// ========================================
// セッション管理
// ========================================

pub fn start_play_session(
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

// ========================================
// セッション管理
// ========================================

pub fn end_play_session(
    mut play_time: ResMut<PlayTimeTracker>,
) {
    play_time.end_session();
}

// ========================================
// ヘルパー関数
// ========================================

fn generate_random_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
