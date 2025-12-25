// src/ui/inventory_ui/mod.rs
//! インベントリUIシステム
//! - ステート管理: Closed/PlayerInventory/Container
//! - ドラッグ&ドロップ
//! - 動的ツールチップ
//! - クラフトリスト
//! - Minecraft風レイアウト

mod types;
mod systems;
mod render;

pub use types::*;
use systems::*;
use render::*;

use bevy::prelude::*;

/// インベントリUIプラグイン
pub struct InventoryUiPlugin;

impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InventoryUiState>()
            .init_resource::<OpenContainer>()
            .init_resource::<DraggedItem>()
            .init_resource::<CreativeViewMode>()
            .add_event::<OpenInventoryEvent>()
            .add_event::<CloseInventoryEvent>()
            .add_event::<OpenContainerEvent>()
            .add_event::<SortInventoryEvent>()
            .add_event::<CraftItemEvent>()
            .add_systems(Update, (
                handle_inventory_key.run_if(in_state(crate::ui::settings_ui::SettingsUiState::Closed)),
                handle_open_inventory_event,
                handle_close_inventory_event,
                handle_open_container_event,
                handle_escape_key.run_if(in_state(crate::ui::settings_ui::SettingsUiState::Closed)),
            ))
            .add_systems(OnEnter(InventoryUiState::PlayerInventory), (
                spawn_player_inventory_ui,
                initialize_creative_visibility,
                release_cursor,
            ))
            .add_systems(OnEnter(InventoryUiState::Container), (
                spawn_container_ui,
                release_cursor,
            ))
            .add_systems(OnExit(InventoryUiState::PlayerInventory), (despawn_inventory_ui, spawn_hotbar_hud_if_not_creative, grab_cursor))
            .add_systems(OnExit(InventoryUiState::Container), (despawn_inventory_ui, grab_cursor))
            // ホットバーHUDはInGame開始時にスポーン、終了時にデスポーン
            .add_systems(OnEnter(crate::ui::main_menu::AppState::InGame), spawn_hotbar_hud)
            .add_systems(OnExit(crate::ui::main_menu::AppState::InGame), despawn_hotbar_hud)
            // インベントリ開閉時の表示切替
            .add_systems(OnEnter(InventoryUiState::PlayerInventory), (spawn_hotbar_hud_if_creative, hide_hotbar_hud_if_not_creative))
            .add_systems(OnExit(InventoryUiState::Closed), hide_hotbar_hud_if_not_creative)
            .add_systems(Update, (
                (
                    handle_slot_interaction,
                    handle_creative_item_button,
                    handle_drag_drop_release,
                    update_slot_visuals, // ドラッグ&ドロップ処理の後に必ず実行
                ).chain(),
                handle_sort_button,
                handle_craft_button,
                handle_view_toggle_button,
                update_creative_view_visibility,
                update_tooltip,
                update_dragged_item_visual,
            ).run_if(not(in_state(InventoryUiState::Closed))))
            .add_systems(Update, update_hotbar_hud);
    }
}
