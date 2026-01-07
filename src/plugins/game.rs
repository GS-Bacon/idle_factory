//! Game Plugin
//!
//! Main game plugin that bundles all core systems and resources.
//! This plugin is the entry point for all game functionality.

use bevy::prelude::*;

use crate::achievements::AchievementsPlugin;
use crate::audio::AudioPlugin;
use crate::blueprint::BlueprintPlugin;
use crate::components::*;
use crate::craft::CraftPlugin;
use crate::events::GameEventsPlugin;
use crate::game_spec;
use crate::game_spec::RegistryPlugin;
use crate::map::MapPlugin;
use crate::modding::ModdingPlugin;
use crate::player::GlobalInventory;
use crate::plugins::{DebugPlugin, MachineSystemsPlugin, SavePlugin, UIPlugin};
use crate::robot::RobotPlugin;
use crate::settings::SettingsPlugin;
use crate::setup::{
    handle_settings_back, handle_settings_sliders, handle_settings_toggles, setup_initial_items,
    setup_lighting, setup_player, setup_ui, update_settings_ui, update_settings_visibility,
};
use crate::skin::SkinPlugin;
use crate::statistics::StatisticsPlugin;
use crate::storage::StoragePlugin;
use crate::systems::{
    block_break, block_place, handle_assert_machine_event, handle_debug_event, handle_look_event,
    handle_pause_menu_buttons, handle_screenshot_event, handle_setblock_event,
    handle_spawn_machine_event, handle_teleport_event, load_machine_models, player_look,
    player_move, process_dirty_chunks, quest_claim_rewards, quest_deliver_button,
    quest_progress_check, receive_chunk_meshes, rotate_conveyor_placement, select_block_type,
    setup_highlight_cache, spawn_chunk_tasks, sync_legacy_ui_state, tick_action_timers,
    toggle_cursor_lock, tutorial_dismiss, ui_action_handler, ui_escape_handler,
    ui_global_inventory_handler, ui_inventory_handler, unload_distant_chunks,
    update_conveyor_shapes, update_delivery_ui, update_guide_markers, update_pause_ui,
    update_quest_ui, update_target_block, update_target_highlight, AssertMachineEvent, DebugEvent,
    LookEvent, ScreenshotEvent, SetBlockEvent, TeleportEvent,
};
use crate::ui::{
    global_inventory_category_click, global_inventory_page_nav, global_inventory_search_input,
    setup_global_inventory_ui, update_global_inventory_ui, update_global_inventory_visibility,
};
use crate::world::{BiomeMap, ChunkMeshTasks, DirtyChunks, WorldData};

/// Main game plugin that bundles all game systems.
///
/// This plugin should be added after DefaultPlugins and VoxLoaderPlugin.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Add sub-plugins
        app.add_plugins(GameEventsPlugin)
            .add_plugins(RegistryPlugin)
            .add_plugins(SettingsPlugin)
            .add_plugins(MachineSystemsPlugin)
            .add_plugins(UIPlugin)
            .add_plugins(SavePlugin)
            .add_plugins(DebugPlugin)
            // Phase D plugins (基盤強化)
            .add_plugins(MapPlugin)
            .add_plugins(BlueprintPlugin)
            .add_plugins(CraftPlugin)
            .add_plugins(StoragePlugin)
            .add_plugins(StatisticsPlugin)
            .add_plugins(AudioPlugin)
            .add_plugins(AchievementsPlugin)
            .add_plugins(SkinPlugin)
            .add_plugins(RobotPlugin)
            .add_plugins(ModdingPlugin);

        // Initialize resources
        app.insert_resource(GlobalInventory::with_items(game_spec::INITIAL_EQUIPMENT))
            .init_resource::<WorldData>()
            .insert_resource(BiomeMap::new(12345)) // Fixed seed for deterministic biomes
            .init_resource::<CursorLockState>()
            .init_resource::<CurrentQuest>()
            .init_resource::<crate::systems::quest::QuestCache>()
            // NOTE: ActiveSubQuests removed (dead code) - reimplement with sub-quest UI
            .init_resource::<GameFont>()
            .init_resource::<ChunkMeshTasks>()
            .init_resource::<DirtyChunks>()
            .init_resource::<CreativeMode>()
            .init_resource::<ContinuousActionTimer>()
            .init_resource::<GlobalInventoryOpen>()
            .init_resource::<GlobalInventoryPage>()
            .init_resource::<GlobalInventoryCategory>()
            .init_resource::<GlobalInventorySearch>()
            .init_resource::<BreakingProgress>()
            // Sky blue background color (simple skybox)
            .insert_resource(ClearColor(Color::srgb(0.47, 0.66, 0.88)));

        // Register events
        app.add_event::<TeleportEvent>()
            .add_event::<LookEvent>()
            .add_event::<SetBlockEvent>()
            .add_event::<DebugEvent>()
            .add_event::<AssertMachineEvent>()
            .add_event::<ScreenshotEvent>()
            .add_event::<UIAction>();

        // UI state management
        app.init_resource::<UIState>();

        // Startup systems
        app.add_systems(
            Startup,
            (
                setup_lighting,
                setup_player,
                setup_ui,
                setup_initial_items,
                // setup_delivery_platform removed - now a tutorial reward
                load_machine_models,
                setup_highlight_cache,
                setup_global_inventory_ui,
            ),
        );

        // Update systems
        self.add_update_systems(app);
    }
}

impl GamePlugin {
    fn add_update_systems(&self, app: &mut App) {
        // Chunk systems: spawn → receive → LOD update (ordered)
        app.add_systems(
            Update,
            (
                spawn_chunk_tasks,
                receive_chunk_meshes,
                unload_distant_chunks,
                crate::systems::update_chunk_lod,
            )
                .chain(),
        );

        // Player systems: look → move (camera affects movement direction)
        app.add_systems(
            Update,
            (
                toggle_cursor_lock,
                player_look,
                player_move,
                tick_action_timers,
                update_pause_ui,
                handle_pause_menu_buttons,
            ),
        );

        app.add_systems(Update, tutorial_dismiss);

        // Targeting must run before block operations
        app.add_systems(Update, update_target_block);
        // Block operations (break/place blocks)
        // Note: No ordering constraint because systems have too many params for Bevy's trait impls
        app.add_systems(Update, block_break);
        app.add_systems(Update, block_place);

        // Process dirty chunks (batched mesh regeneration - runs every frame)
        app.add_systems(Update, process_dirty_chunks);

        app.add_systems(Update, select_block_type);

        // Quest systems
        app.add_systems(
            Update,
            (
                crate::systems::targeting::update_conveyor_shapes,
                quest_progress_check,
                quest_claim_rewards,
            ),
        );

        // Quest UI systems
        app.add_systems(
            Update,
            (update_delivery_ui, update_quest_ui, quest_deliver_button),
        );

        // Global inventory UI systems
        app.add_systems(
            Update,
            (
                update_global_inventory_visibility,
                global_inventory_page_nav,
                global_inventory_category_click,
                global_inventory_search_input,
                update_global_inventory_ui,
            ),
        );

        // Targeting highlight (after target_block update)
        app.add_systems(
            Update,
            (
                update_target_highlight,
                rotate_conveyor_placement,
                update_conveyor_shapes,
                update_guide_markers,
            )
                .after(update_target_block),
        );

        // E2E command handlers
        app.add_systems(
            Update,
            (
                handle_teleport_event,
                handle_look_event,
                handle_setblock_event,
                handle_spawn_machine_event,
                handle_debug_event,
                handle_assert_machine_event,
                handle_screenshot_event,
            ),
        );

        // UI navigation systems (must run early to process actions before other UI systems)
        // Order: input handlers emit events → action handler updates UIState → sync to legacy
        app.add_systems(
            Update,
            (
                ui_escape_handler,
                ui_inventory_handler,
                ui_global_inventory_handler,
                ui_action_handler,
                sync_legacy_ui_state,
            )
                .chain(),
        );

        // Settings UI systems
        app.add_systems(
            Update,
            (
                update_settings_visibility,
                update_settings_ui,
                handle_settings_sliders,
                handle_settings_toggles,
                handle_settings_back,
            ),
        );
    }
}
