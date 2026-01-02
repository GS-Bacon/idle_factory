//! Idle Factory - Milestone 1: Minimal Voxel Game
//! Goal: Walk, mine blocks, collect in inventory

// Use library crate for all game logic
use idle_factory::components::*;
use idle_factory::events::GameEventsPlugin;
#[cfg(target_arch = "wasm32")]
use idle_factory::logging::GameLoggingPlugin;
use idle_factory::logging;
use idle_factory::player::Inventory;
use idle_factory::plugins::{DebugPlugin, MachineSystemsPlugin, SavePlugin, UIPlugin};
use idle_factory::setup::{setup_initial_items, setup_lighting, setup_player, setup_ui};
use idle_factory::systems::{
    // Block operation systems
    block_break, block_place,
    // Player systems
    player_look, player_move, tick_action_timers, toggle_cursor_lock, tutorial_dismiss,
    // Chunk systems
    receive_chunk_meshes, spawn_chunk_tasks, unload_distant_chunks,
    // UI systems
    select_block_type,
    // Command events and handlers
    TeleportEvent, LookEvent, SetBlockEvent, handle_teleport_event,
    handle_look_event, handle_setblock_event, handle_spawn_machine_event, handle_debug_conveyor_event,
    handle_assert_machine_event, DebugConveyorEvent, AssertMachineEvent,
    // Quest systems
    load_machine_models, quest_claim_rewards, quest_deliver_button, quest_progress_check,
    setup_delivery_platform, update_delivery_ui, update_quest_ui,
    // Targeting systems
    rotate_conveyor_placement, update_conveyor_shapes, update_guide_markers,
    update_target_block, update_target_highlight,
};
use idle_factory::world::{BiomeMap, ChunkMeshTasks, WorldData};
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::PresentMode;

fn main() {
    // WASM: Set panic hook to display errors in browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // Initialize logging before anything else
    #[cfg(not(target_arch = "wasm32"))]
    let _log_guard = logging::init_logging();

    let mut app = App::new();

    // Configure plugins based on platform
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Native: Disable pipelined rendering for lower input lag
        // Disable LogPlugin to use custom tracing-subscriber instead
        use bevy::log::LogPlugin;
        // Use current working directory for assets (not executable path)
        app.add_plugins((
            DefaultPlugins
                .build()
                .disable::<PipelinedRenderingPlugin>()
                .disable::<LogPlugin>()
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
        ));
    }

    #[cfg(target_arch = "wasm32")]
    {
        // WASM: Use default plugins with canvas selector
        // Disable LogPlugin to use tracing_wasm instead
        use bevy::log::LogPlugin;
        app.add_plugins(
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
        );
    }

    // WASM: Add logging plugin (Native logging is initialized above)
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(GameLoggingPlugin);

    // Add VOX loader plugin for hot reload (native only)
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(idle_factory::vox_loader::VoxLoaderPlugin);

    app
        .add_plugins(GameEventsPlugin)
        .add_plugins(MachineSystemsPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(SavePlugin)
        .add_plugins(DebugPlugin)
        .insert_resource(Inventory::with_initial_items(idle_factory::game_spec::INITIAL_EQUIPMENT))
        .init_resource::<WorldData>()
        .insert_resource(BiomeMap::new(12345)) // Fixed seed for deterministic biomes
        .init_resource::<CursorLockState>()
        .init_resource::<CurrentQuest>()
        .init_resource::<GameFont>()
        .init_resource::<ChunkMeshTasks>()
        .init_resource::<CreativeMode>()
        .init_resource::<ContinuousActionTimer>()
        .add_event::<TeleportEvent>()
        .add_event::<LookEvent>()
        .add_event::<SetBlockEvent>()
        .add_event::<DebugConveyorEvent>()
        .add_event::<AssertMachineEvent>()
        .add_systems(Startup, (setup_lighting, setup_player, setup_ui, setup_initial_items, setup_delivery_platform, load_machine_models))
        .add_systems(
            Update,
            (
                // Chunk systems: spawn → receive (ordered)
                spawn_chunk_tasks,
                receive_chunk_meshes,
                unload_distant_chunks,
            ).chain(),
        )
        .add_systems(
            Update,
            (
                // Player systems: look → move (camera affects movement direction)
                toggle_cursor_lock,
                player_look,
                player_move,
                tick_action_timers,
            ),
        )
        .add_systems(Update, tutorial_dismiss)
        // Targeting must run before block operations
        .add_systems(Update, update_target_block)
        .add_systems(Update, (block_break, block_place).after(update_target_block))
        .add_systems(Update, select_block_type)
        .add_systems(
            Update,
            (
                // Quest systems
                idle_factory::systems::targeting::update_conveyor_shapes,
                quest_progress_check,
                quest_claim_rewards,
            ),
        )
        .add_systems(
            Update,
            (
                // Quest UI systems
                update_delivery_ui,
                update_quest_ui,
                quest_deliver_button,
            ),
        )
        .add_systems(
            Update,
            (
                // Targeting highlight (after target_block update)
                update_target_highlight,
                rotate_conveyor_placement,
                update_conveyor_shapes,
                update_guide_markers,
            ).after(update_target_block),
        )
        .add_systems(
            Update,
            (
                // E2E command handlers
                handle_teleport_event,
                handle_look_event,
                handle_setblock_event,
                handle_spawn_machine_event,
                handle_debug_conveyor_event,
                handle_assert_machine_event,
            ),
        )
        .run();
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;
    use idle_factory::BlockType;
    use idle_factory::constants::{HOTBAR_SLOTS, NUM_SLOTS};
    use idle_factory::world::ChunkData;
    use idle_factory::systems::quest::get_main_quests;
    use idle_factory::utils::{ray_aabb_intersection, ray_aabb_intersection_with_normal};

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
        let quests = get_main_quests();
        assert!(!quests.is_empty());
        for quest in &quests {
            assert!(!quest.required_items.is_empty());
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

    #[test]
    fn test_conveyor_can_accept_item() {
        let conveyor = Conveyor {
            position: IVec3::ZERO,
            direction: Direction::North,
            items: vec![],
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::Straight,
        };
        assert!(conveyor.can_accept_item(0.0));
        assert!(conveyor.can_accept_item(0.5));
    }

    #[test]
    fn test_conveyor_item_spacing() {
        let mut conveyor = Conveyor {
            position: IVec3::ZERO,
            direction: Direction::North,
            items: vec![],
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::Straight,
        };
        conveyor.add_item(BlockType::IronOre, 0.5);
        // Item at 0.5, so 0.4 and 0.6 should be too close
        assert!(!conveyor.can_accept_item(0.5));
        assert!(!conveyor.can_accept_item(0.45));
        // But 0.0 should be far enough
        assert!(conveyor.can_accept_item(0.0));
    }

    #[test]
    fn test_furnace_smelt_output() {
        assert_eq!(Furnace::get_smelt_output(BlockType::IronOre), Some(BlockType::IronIngot));
        assert_eq!(Furnace::get_smelt_output(BlockType::CopperOre), Some(BlockType::CopperIngot));
        assert_eq!(Furnace::get_smelt_output(BlockType::Stone), None);
    }

    #[test]
    fn test_furnace_can_add_input() {
        let mut furnace = Furnace::default();
        assert!(furnace.can_add_input(BlockType::IronOre));
        furnace.input_type = Some(BlockType::IronOre);
        furnace.input_count = 1;
        assert!(furnace.can_add_input(BlockType::IronOre));
        // Different ore type should be rejected
        assert!(!furnace.can_add_input(BlockType::CopperOre));
    }

    #[test]
    fn test_crusher_can_crush() {
        assert!(Crusher::can_crush(BlockType::IronOre));
        assert!(Crusher::can_crush(BlockType::CopperOre));
        assert!(!Crusher::can_crush(BlockType::Stone));
        assert!(!Crusher::can_crush(BlockType::IronIngot));
    }

    #[test]
    fn test_creative_mode_toggle() {
        let mut creative = CreativeMode::default();
        assert!(!creative.enabled);
        creative.enabled = true;
        assert!(creative.enabled);
    }

    #[test]
    fn test_quest_structure() {
        let quests = get_main_quests();
        for quest in &quests {
            assert!(!quest.description.is_empty());
            assert!(!quest.required_items.is_empty());
            assert!(!quest.rewards.is_empty());
        }
    }

    #[test]
    fn test_direction_conversions() {
        // Test all directions have valid ivec3
        assert_eq!(Direction::North.to_ivec3(), IVec3::new(0, 0, -1));
        assert_eq!(Direction::South.to_ivec3(), IVec3::new(0, 0, 1));
        assert_eq!(Direction::East.to_ivec3(), IVec3::new(1, 0, 0));
        assert_eq!(Direction::West.to_ivec3(), IVec3::new(-1, 0, 0));

        // Test rotation consistency
        let mut dir = Direction::North;
        for _ in 0..4 {
            dir = dir.rotate_cw();
        }
        assert_eq!(dir, Direction::North);
    }

    #[test]
    fn test_conveyor_splitter_outputs() {
        let conveyor = Conveyor {
            position: IVec3::new(5, 0, 5),
            direction: Direction::North,
            items: vec![],
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::Splitter,
        };
        let outputs = conveyor.get_splitter_outputs();
        // front, left, right
        assert_eq!(outputs[0], IVec3::new(5, 0, 4)); // North (front)
        assert_eq!(outputs[1], IVec3::new(4, 0, 5)); // West (left)
        assert_eq!(outputs[2], IVec3::new(6, 0, 5)); // East (right)
    }

    #[test]
    fn test_miner_default() {
        let miner = Miner::default();
        assert_eq!(miner.position, IVec3::ZERO);
        assert_eq!(miner.progress, 0.0);
        assert!(miner.buffer.is_none());
    }

    #[test]
    fn test_conveyor_corner_left_direction() {
        // Corner left: turns to the left of the facing direction
        // North-facing corner left outputs to West (left of North)
        let dir = Direction::North;
        let left_dir = dir.left();
        let output = IVec3::new(5, 0, 5) + left_dir.to_ivec3();
        assert_eq!(left_dir, Direction::West);
        assert_eq!(output, IVec3::new(4, 0, 5), "Corner left should output to West");
    }

    #[test]
    fn test_conveyor_corner_right_direction() {
        // Corner right: turns to the right of the facing direction
        // North-facing corner right outputs to East (right of North)
        let dir = Direction::North;
        let right_dir = dir.right();
        let output = IVec3::new(5, 0, 5) + right_dir.to_ivec3();
        assert_eq!(right_dir, Direction::East);
        assert_eq!(output, IVec3::new(6, 0, 5), "Corner right should output to East");
    }

    #[test]
    fn test_conveyor_l_shape_chain() {
        // Test a chain of conveyors forming an L-shape
        // Straight (North) -> Corner Right (turn East) -> Straight (East)
        let corner_pos = IVec3::new(5, 0, 5);
        let corner_dir = Direction::North;

        // Corner right outputs to the right of its facing direction
        let output_dir = corner_dir.right();  // Turns right: North -> East
        let output_pos = corner_pos + output_dir.to_ivec3();

        assert_eq!(output_dir, Direction::East);
        assert_eq!(output_pos, IVec3::new(6, 0, 5));

        // Next straight conveyor faces East, outputs to East
        let next_output = output_pos + output_dir.to_ivec3();
        assert_eq!(next_output, IVec3::new(7, 0, 5));
    }

    #[test]
    fn test_all_corner_directions() {
        // Test all 4 directions for corner conveyors
        let cases = [
            (Direction::North, Direction::West, Direction::East),   // N: left=W, right=E
            (Direction::East, Direction::North, Direction::South),  // E: left=N, right=S
            (Direction::South, Direction::East, Direction::West),   // S: left=E, right=W
            (Direction::West, Direction::South, Direction::North),  // W: left=S, right=N
        ];

        for (facing, expected_left, expected_right) in cases {
            assert_eq!(facing.left(), expected_left, "Left of {:?}", facing);
            assert_eq!(facing.right(), expected_right, "Right of {:?}", facing);
        }
    }
}
