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
                building::handle_building,
                player::move_player,
                player::look_player,
                player::grab_cursor,
                items::update_visual_items,
            ));

        machines::register_machines(app);
    }
}