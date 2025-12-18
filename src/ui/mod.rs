use bevy::prelude::*;

pub mod hud;
pub mod machine_ui;
pub mod inventory_ui;
pub mod command_ui;
pub mod settings_ui;
pub mod main_menu;
pub mod menu_camera;
pub mod minimap;
pub mod health_hud;
pub mod quest_hud;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Main menu and camera (must be first for state initialization)
        app.add_plugins(main_menu::MainMenuPlugin);
        app.add_plugins(menu_camera::MenuCameraPlugin);
        // HUD（クロスヘア）のシステムを登録
        app.add_systems(Startup, hud::spawn_crosshair);
        // Machine UI plugin
        app.add_plugins(machine_ui::MachineUiPlugin);
        // Inventory UI plugin
        app.add_plugins(inventory_ui::InventoryUiPlugin);
        // Command UI plugin
        app.add_plugins(command_ui::CommandUiPlugin);
        // Settings UI plugin
        app.add_plugins(settings_ui::SettingsUiPlugin);
        // Minimap plugin
        app.add_plugins(minimap::MinimapPlugin);
        // Health HUD plugin
        app.add_plugins(health_hud::HealthHudPlugin);
        // Quest HUD plugin
        app.add_plugins(quest_hud::QuestHudPlugin);
    }
}