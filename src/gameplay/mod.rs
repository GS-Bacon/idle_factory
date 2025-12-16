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

use grid::SimulationGrid;
use crate::ui::inventory_ui::InventoryUiState;
use crate::ui::command_ui::CommandUiState;
use crate::ui::settings_ui::SettingsUiState;

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
            .init_resource::<SimulationGrid>()
            .init_resource::<building::BuildTool>()
            .init_resource::<building::HologramState>()
            .add_event::<building::MachinePlacedEvent>()
            .add_systems(Startup, player::spawn_player)
            .add_systems(Update, (
                building::handle_building.run_if(
                    in_state(InventoryUiState::Closed)
                    .and(in_state(CommandUiState::Closed))
                    .and(in_state(SettingsUiState::Closed))
                ),
                player::move_player.run_if(
                    in_state(InventoryUiState::Closed)
                    .and(in_state(CommandUiState::Closed))
                    .and(in_state(SettingsUiState::Closed))
                ),
                player::look_player.run_if(
                    in_state(InventoryUiState::Closed)
                    .and(in_state(CommandUiState::Closed))
                    .and(in_state(SettingsUiState::Closed))
                ),
                player::grab_cursor.run_if(
                    in_state(InventoryUiState::Closed)
                    .and(in_state(CommandUiState::Closed))
                    .and(in_state(SettingsUiState::Closed))
                ),
                player::handle_hotbar_selection,
                player::handle_hotbar_scroll,
                items::update_visual_items,
            ));

        machines::register_machines(app);
    }
}