// src/ui/main_menu/mod.rs
//! メインメニューUI - モジュール構成

mod types;
mod systems;
mod screens;

pub use types::{
    AppState, MainMenuUi, ProfileSelectUi, ProfileSettingsUi, SaveSelectUi, WorldGenUi, PauseMenuUi,
    MenuButtonAction, TextInput, TextInputType, TextInputDisplay, SelectedSlotIndex,
    SelectedGameMode, SelectedWorldType, GameModeButtonMarker, WorldTypeButtonMarker,
    ProfileList, ProfileInfo,
};

use bevy::prelude::*;
use systems::{
    button_interaction_system,
    main_menu_buttons, profile_select_buttons, profile_settings_buttons,
    save_select_buttons, world_gen_buttons, pause_menu_buttons,
    text_input_system, update_text_input_display,
    handle_menu_escape_key, handle_ingame_escape_key,
    start_play_session, end_play_session,
};
use screens::{
    spawn_main_menu, spawn_profile_select, spawn_profile_settings,
    spawn_save_select, spawn_world_generation, spawn_pause_menu,
    despawn_with,
};

/// メインメニュープラグイン
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<ProfileList>()
            .init_resource::<SelectedGameMode>()
            .init_resource::<SelectedWorldType>()
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
