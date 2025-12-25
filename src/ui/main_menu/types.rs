// src/ui/main_menu/types.rs
//! 型定義

use bevy::prelude::*;
use crate::gameplay::commands::GameMode;
use crate::core::worldgen::WorldType;

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
    // ワールドタイプ選択
    SelectWorldType(WorldType),
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

/// 選択中のワールドタイプ（ワールド作成時）
#[derive(Resource)]
pub struct SelectedWorldType(pub WorldType);

impl Default for SelectedWorldType {
    fn default() -> Self {
        Self(WorldType::Normal)
    }
}

/// ゲームモードボタンのマーカー
#[derive(Component)]
pub struct GameModeButtonMarker(pub GameMode);

/// ワールドタイプボタンのマーカー
#[derive(Component)]
pub struct WorldTypeButtonMarker(pub WorldType);

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
