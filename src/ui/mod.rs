use bevy::prelude::*;

pub mod hud;
pub mod machine_ui;
pub mod inventory_ui;
pub mod command_ui;

// ここで UiPlugin を定義する必要があります
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // HUD（クロスヘア）のシステムを登録
        app.add_systems(Startup, hud::spawn_crosshair);
        // Machine UI plugin
        app.add_plugins(machine_ui::MachineUiPlugin);
        // Inventory UI plugin
        app.add_plugins(inventory_ui::InventoryUiPlugin);
        // Command UI plugin
        app.add_plugins(command_ui::CommandUiPlugin);
    }
}