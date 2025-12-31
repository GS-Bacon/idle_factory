//! Idle Factory - Milestone 1: Minimal Voxel Game
//! Goal: Walk, mine blocks, collect in inventory

mod block_type;
mod components;
mod constants;
mod events;
mod game_spec;
mod logging;
mod machines;
mod meshes;
mod player;
mod save;
pub mod setup;
mod systems;
mod ui;
mod utils;
mod world;

use components::*;
use events::GameEventsPlugin;
use logging::GameLoggingPlugin;
use player::Inventory;
use setup::{setup_initial_items, setup_lighting, setup_player, setup_ui};
use utils::ray_aabb_intersection;
use systems::{
    // Block operation systems
    block_break, block_place,
    // Machine systems
    conveyor_transfer, crusher_interact, crusher_output, crusher_processing, crusher_ui_input,
    furnace_interact, furnace_output, furnace_smelting, furnace_ui_input, miner_interact,
    miner_mining, miner_output, miner_ui_input, miner_visual_feedback,
    update_conveyor_item_visuals, update_crusher_ui, update_furnace_ui, update_miner_ui,
    // Player systems
    player_look, player_move, tick_action_timers, toggle_cursor_lock, tutorial_dismiss,
    // Chunk systems
    receive_chunk_meshes, spawn_chunk_tasks, unload_distant_chunks,
    // UI systems
    command_input_handler, command_input_toggle, creative_inventory_click,
    inventory_continuous_shift_click, inventory_slot_click, inventory_toggle,
    inventory_update_slots, select_block_type, set_ui_open_state, toggle_debug_hud,
    trash_slot_click, update_debug_hud, update_held_item_display, update_hotbar_item_name,
    update_hotbar_ui, update_inventory_tooltip, update_window_title_fps, export_e2e_state,
    E2EExportConfig, TeleportEvent, LookEvent, SetBlockEvent, handle_teleport_event,
    handle_look_event, handle_setblock_event,
    // Quest systems
    load_machine_models, quest_claim_rewards, quest_progress_check, setup_delivery_platform,
    update_delivery_ui, update_quest_ui,
    // Save systems
    auto_save_system, handle_load_event, handle_save_event,
    // Targeting systems
    rotate_conveyor_placement, update_conveyor_shapes, update_guide_markers,
    update_target_block, update_target_highlight,
};
use world::{ChunkMesh, ChunkMeshTasks, WorldData};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::PresentMode;

pub use block_type::BlockType;
pub use constants::*;

fn main() {
    // WASM: Set panic hook to display errors in browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    // Configure plugins based on platform
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Native: Disable pipelined rendering for lower input lag
        // Use current working directory for assets (not executable path)
        app.add_plugins((
            DefaultPlugins
                .build()
                .disable::<PipelinedRenderingPlugin>()
                .set(AssetPlugin {
                    file_path: "assets".to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Idle Factory".into(),
                        present_mode: PresentMode::AutoNoVsync,
                        desired_maximum_frame_latency: std::num::NonZeroU32::new(1),
                        ..default()
                    }),
                    ..default()
                }),
            FrameTimeDiagnosticsPlugin,
        ));
    }

    #[cfg(target_arch = "wasm32")]
    {
        // WASM: Use default plugins with canvas selector
        // Disable LogPlugin to use tracing_wasm instead
        use bevy::log::LogPlugin;
        app.add_plugins((
            DefaultPlugins
                .build()
                .disable::<LogPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Idle Factory".into(),
                        canvas: Some("#bevy-canvas".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                }),
            FrameTimeDiagnosticsPlugin,
        ));
    }

    app
        .add_plugins(GameEventsPlugin)
        .add_plugins(GameLoggingPlugin)
        .init_resource::<Inventory>()
        .init_resource::<WorldData>()
        .init_resource::<CursorLockState>()
        .init_resource::<InteractingFurnace>()
        .init_resource::<InteractingCrusher>()
        .init_resource::<InteractingMiner>()
        .init_resource::<CurrentQuest>()
        .init_resource::<GameFont>()
        .init_resource::<ChunkMeshTasks>()
        .init_resource::<MachineModels>()
        .init_resource::<DebugHudState>()
        .init_resource::<TargetBlock>()
        .init_resource::<CreativeMode>()
        .init_resource::<InventoryOpen>()
        .init_resource::<TutorialShown>()
        .init_resource::<HeldItem>()
        .init_resource::<ContinuousActionTimer>()
        .init_resource::<CommandInputState>()
        .init_resource::<GuideMarkers>()
        .init_resource::<ConveyorRotationOffset>()
        .init_resource::<save::AutoSaveTimer>()
        .init_resource::<SaveLoadState>()
        .init_resource::<ItemSprites>()
        .init_resource::<E2EExportConfig>()
        .add_event::<SaveGameEvent>()
        .add_event::<LoadGameEvent>()
        .add_event::<TeleportEvent>()
        .add_event::<LookEvent>()
        .add_event::<SetBlockEvent>()
        .add_systems(Startup, (setup_lighting, setup_player, setup_ui, setup_initial_items, setup_delivery_platform, load_machine_models))
        .add_systems(
            Update,
            (
                // Core gameplay systems - chunk loading
                spawn_chunk_tasks,
                receive_chunk_meshes,
                unload_distant_chunks,
                toggle_cursor_lock,
                player_look,
                player_move,
                tick_action_timers,
            ),
        )
        .add_systems(Update, tutorial_dismiss)
        .add_systems(Update, block_break)
        .add_systems(Update, block_place)
        .add_systems(Update, select_block_type)
        .add_systems(
            Update,
            (
                // Machine interaction systems
                furnace_interact,
                furnace_ui_input,
                furnace_smelting,
                crusher_interact,
                crusher_ui_input,
                miner_interact,
                miner_ui_input,
            ),
        )
        .add_systems(
            Update,
            (
                // Machine systems
                miner_mining,
                miner_visual_feedback,
                miner_output,
                crusher_processing,
                crusher_output,
                furnace_output,
                conveyor_transfer,
                update_conveyor_item_visuals,
                systems::targeting::update_conveyor_shapes,
                quest_progress_check,
                quest_claim_rewards,
            ),
        )
        .add_systems(
            Update,
            (
                // UI update systems
                update_hotbar_ui,
                update_furnace_ui,
                update_crusher_ui,
                update_miner_ui,
                update_delivery_ui,
                update_quest_ui,
                update_window_title_fps,
                toggle_debug_hud,
            ),
        )
        .add_systems(
            Update,
            (
                // Debug and utility systems
                update_debug_hud,
                export_e2e_state,
                update_target_block,
                update_target_highlight,
                rotate_conveyor_placement,
                update_conveyor_shapes,
                update_guide_markers,
                inventory_toggle,
                inventory_slot_click,
                inventory_continuous_shift_click,
                inventory_update_slots,
            ),
        )
        .add_systems(
            Update,
            (
                // UI interaction systems
                update_held_item_display,
                update_hotbar_item_name,
                update_inventory_tooltip,
                trash_slot_click,
                creative_inventory_click,
                command_input_toggle,
                command_input_handler,
            ),
        )
        .add_systems(
            Update,
            (
                // Save/Load systems
                auto_save_system,
                handle_save_event,
                handle_load_event,
                // E2E command handlers
                handle_teleport_event,
                handle_look_event,
                handle_setblock_event,
            ),
        )
        .run();
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;
    use world::ChunkData;
    use crate::systems::get_quests;
    use crate::utils::ray_aabb_intersection_with_normal;

    #[test]
    fn test_chunk_generation() {
        let chunk = ChunkData::generate(IVec2::ZERO);
        assert!(!chunk.blocks_map.is_empty());
        let surface_block = chunk.blocks_map.get(&IVec3::new(0, 7, 0));
        assert!(matches!(
            surface_block,
            Some(BlockType::Grass) | Some(BlockType::IronOre) | Some(BlockType::CopperOre) | Some(BlockType::Coal)
        ));
    }

    #[test]
    fn test_world_coordinate_conversion() {
        assert_eq!(WorldData::world_to_chunk(IVec3::new(0, 0, 0)), IVec2::new(0, 0));
        assert_eq!(WorldData::world_to_chunk(IVec3::new(16, 0, 0)), IVec2::new(1, 0));
        assert_eq!(WorldData::world_to_chunk(IVec3::new(-1, 0, -1)), IVec2::new(-1, -1));
    }

    #[test]
    fn test_world_data_block_operations() {
        let mut world = WorldData::default();
        world.chunks.insert(IVec2::new(0, 0), ChunkData::generate(IVec2::ZERO));
        assert!(world.has_block(IVec3::new(0, 0, 0)));
        assert!(!world.has_block(IVec3::new(0, 10, 0)));
    }

    #[test]
    fn test_mesh_winding_order() {
        let mut world = WorldData::default();
        let chunk_coord = IVec2::new(0, 0);
        world.chunks.insert(chunk_coord, ChunkData::generate(chunk_coord));
        let mesh = world.generate_chunk_mesh(chunk_coord).unwrap();
        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.clone(),
            _ => panic!("Unexpected vertex format"),
        };
        let indices = match mesh.indices().unwrap() {
            bevy::render::mesh::Indices::U32(v) => v.clone(),
            _ => panic!("Unexpected index format"),
        };
        let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.clone(),
            _ => panic!("Unexpected normal format"),
        };
        let mut correct = 0;
        let mut total = 0;
        for tri in indices.chunks(3) {
            let v0 = Vec3::from_array(positions[tri[0] as usize]);
            let v1 = Vec3::from_array(positions[tri[1] as usize]);
            let v2 = Vec3::from_array(positions[tri[2] as usize]);
            let cross = (v1 - v0).cross(v2 - v0);
            if cross.length() < 0.0001 { continue; }
            let calc_normal = cross.normalize();
            let expected = Vec3::from_array(normals[tri[0] as usize]);
            total += 1;
            if calc_normal.dot(expected) > 0.9 { correct += 1; }
        }
        assert!(correct as f32 / total as f32 > 0.99);
    }

    #[test]
    fn test_inventory_add() {
        let mut inventory = Inventory::default();
        inventory.add_item(BlockType::Stone, 1);
        assert_eq!(inventory.get_item_count(BlockType::Stone), 1);
    }

    #[test]
    fn test_ray_aabb_hit() {
        let result = ray_aabb_intersection(
            Vec3::new(0.0, 0.0, -5.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        assert!(result.is_some());
    }

    #[test]
    fn test_ray_aabb_with_normal_z() {
        let result = ray_aabb_intersection_with_normal(
            Vec3::new(0.0, 0.0, -5.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        assert!(result.is_some());
        let (_, normal) = result.unwrap();
        assert_eq!(normal, Vec3::NEG_Z);
    }

    #[test]
    fn test_hotbar_scroll_stays_in_bounds() {
        assert_eq!(HOTBAR_SLOTS, 9);
        assert_eq!(NUM_SLOTS, 36);
        for start_slot in 0..HOTBAR_SLOTS {
            let next = (start_slot + 1) % HOTBAR_SLOTS;
            assert!(next < HOTBAR_SLOTS);
        }
    }

    #[test]
    fn test_inventory_consumption() {
        let mut inventory = Inventory::default();
        inventory.add_item(BlockType::Stone, 10);
        inventory.selected_slot = 0;
        assert_eq!(inventory.get_slot_count(0), 10);
        inventory.consume_selected();
        assert_eq!(inventory.get_slot_count(0), 9);
    }

    #[test]
    fn test_mode_constants() {
        let creative = CreativeMode::default();
        assert!(!creative.enabled);
    }

    #[test]
    fn test_quest_rewards() {
        let quests = get_quests();
        assert!(!quests.is_empty());
        for quest in &quests {
            assert!(quest.required_amount > 0);
        }
    }

    #[test]
    fn test_input_state_priority() {
        let state = InputState::current(
            &InventoryOpen(false),
            &InteractingFurnace(None),
            &InteractingCrusher(None),
            &InteractingMiner(None),
            &CommandInputState::default(),
            &CursorLockState::default(),
        );
        assert!(matches!(state, InputState::Gameplay));

        let state = InputState::current(
            &InventoryOpen(true),
            &InteractingFurnace(None),
            &InteractingCrusher(None),
            &InteractingMiner(None),
            &CommandInputState::default(),
            &CursorLockState { paused: true, ..default() },
        );
        assert!(matches!(state, InputState::Paused));
    }

    #[test]
    fn test_input_state_allows() {
        assert!(InputState::Gameplay.allows_movement());
        assert!(InputState::Gameplay.allows_block_actions());
        assert!(!InputState::Inventory.allows_movement());
        assert!(!InputState::Command.allows_block_actions());
    }

    #[test]
    fn test_biome_generation() {
        let biome1 = ChunkData::get_biome(0, 0);
        let biome2 = ChunkData::get_biome(0, 0);
        assert_eq!(biome1, biome2);
    }
}
