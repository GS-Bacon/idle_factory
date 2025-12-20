use bevy::prelude::*;

pub mod grid;
pub mod building;
pub mod player;
pub mod items;
pub mod machines;
pub mod interaction;
pub mod power;
pub mod multiblock;
pub mod inventory;
pub mod commands;
pub mod held_item;
pub mod scripting;
pub mod signals;
pub mod quest;
pub mod delivery_platform;
pub mod player_stats;
pub mod weather;
pub mod fluid;
pub mod heat;
pub mod vibration;

use grid::SimulationGrid;
use crate::ui::inventory_ui::InventoryUiState;
use crate::ui::command_ui::CommandUiState;
use crate::ui::settings_ui::SettingsUiState;
use crate::ui::main_menu::AppState;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(interaction::InteractionPlugin)
            .add_plugins(power::PowerPlugin)
            .add_plugins(multiblock::MultiblockPlugin)
            .add_plugins(inventory::InventoryPlugin)
            .add_plugins(commands::CommandsPlugin)
            .add_plugins(held_item::HeldItemPlugin)
            .add_plugins(scripting::ScriptingPlugin)
            .add_plugins(signals::SignalPlugin)
            .add_plugins(quest::QuestPlugin)
            .add_plugins(delivery_platform::DeliveryPlatformPlugin)
            .add_plugins(player_stats::PlayerStatsPlugin)
            .add_plugins(weather::WeatherPlugin)
            .add_plugins(fluid::FluidPlugin)
            .add_plugins(heat::HeatPlugin)
            .add_plugins(vibration::VibrationPlugin)
            .init_resource::<SimulationGrid>()
            .init_resource::<building::BuildTool>()
            .init_resource::<building::HologramState>()
            .add_event::<building::MachinePlacedEvent>()
            // プレイヤーはInGame開始時にスポーン
            .add_systems(OnEnter(AppState::InGame), player::spawn_player)
            // InGame退出時にプレイヤーを削除
            .add_systems(OnExit(AppState::InGame), player::despawn_player)
            .add_systems(Update, (
                building::handle_building.run_if(
                    in_state(AppState::InGame)
                    .and(in_state(InventoryUiState::Closed))
                    .and(in_state(CommandUiState::Closed))
                    .and(in_state(SettingsUiState::Closed))
                ),
                player::move_player.run_if(
                    in_state(AppState::InGame)
                    .and(in_state(InventoryUiState::Closed))
                    .and(in_state(CommandUiState::Closed))
                    .and(in_state(SettingsUiState::Closed))
                ),
                player::look_player.run_if(
                    in_state(AppState::InGame)
                    .and(in_state(InventoryUiState::Closed))
                    .and(in_state(CommandUiState::Closed))
                    .and(in_state(SettingsUiState::Closed))
                ),
                player::grab_cursor.run_if(
                    in_state(AppState::InGame)
                    .and(in_state(InventoryUiState::Closed))
                    .and(in_state(CommandUiState::Closed))
                    .and(in_state(SettingsUiState::Closed))
                ),
                player::handle_hotbar_selection.run_if(in_state(AppState::InGame)),
                player::handle_hotbar_scroll.run_if(in_state(AppState::InGame)),
                items::update_visual_items.run_if(in_state(AppState::InGame)),
            ));

        machines::register_machines(app);
    }
}