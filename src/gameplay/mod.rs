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

use grid::SimulationGrid;
use crate::ui::inventory_ui::InventoryUiState;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(interaction::InteractionPlugin)
            .add_plugins(power::PowerPlugin)
            .add_plugins(multiblock::MultiblockPlugin)
            .add_plugins(inventory::InventoryPlugin)
            .init_resource::<SimulationGrid>()
            .init_resource::<building::BuildTool>()
            .add_event::<building::MachinePlacedEvent>()
            .add_systems(Startup, player::spawn_player)
            .add_systems(Update, (
                building::handle_building.run_if(in_state(InventoryUiState::Closed)),
                player::move_player.run_if(in_state(InventoryUiState::Closed)),
                player::look_player.run_if(in_state(InventoryUiState::Closed)),
                player::grab_cursor.run_if(in_state(InventoryUiState::Closed)),
                player::handle_hotbar_selection,
                items::update_visual_items,
            ));

        machines::register_machines(app);
    }
}