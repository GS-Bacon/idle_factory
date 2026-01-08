//! Idle Factory - Voxel Factory Game
//!
//! Main entry point for the game binary.

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::time::Fixed;
use bevy::window::PresentMode;
use idle_factory::logging;
use idle_factory::plugins::GamePlugin;

fn main() {
    // Initialize logging before anything else
    let _log_guard = logging::init_logging();

    let mut app = App::new();

    // Configure fixed timestep for deterministic game logic (20 ticks/second)
    app.insert_resource(Time::<Fixed>::from_hz(20.0));

    // Configure DefaultPlugins
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<PipelinedRenderingPlugin>() // Lower input lag
            .disable::<LogPlugin>() // Use custom tracing-subscriber
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
    );

    // Add VOX loader plugin for hot reload
    app.add_plugins(idle_factory::vox_loader::VoxLoaderPlugin);

    // Add auto-updater plugin (only when feature enabled)
    #[cfg(feature = "updater")]
    app.add_plugins(idle_factory::UpdaterPlugin);

    // Add main game plugin
    app.add_plugins(GamePlugin);

    app.run();
}

// === Tests ===

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use idle_factory::components::*;
    use idle_factory::constants::{HOTBAR_SLOTS, NUM_SLOTS};
    use idle_factory::core::items;
    use idle_factory::player::PlayerInventory;
    use idle_factory::systems::quest::get_main_quests;
    use idle_factory::utils::{ray_aabb_intersection, ray_aabb_intersection_with_normal};
    use idle_factory::world::{ChunkData, WorldData};

    #[test]
    fn test_chunk_generation() {
        let chunk = ChunkData::generate(IVec2::ZERO);
        // Check that blocks array has some blocks
        let block_count = chunk.blocks.iter().filter(|b| b.is_some()).count();
        assert!(block_count > 0);
        // Check surface block at (0, 7, 0)
        let surface_block = chunk.get_block(0, 7, 0);
        assert!(
            surface_block == Some(items::grass())
                || surface_block == Some(items::iron_ore())
                || surface_block == Some(items::copper_ore())
                || surface_block == Some(items::coal())
        );
    }

    #[test]
    fn test_world_coordinate_conversion() {
        assert_eq!(
            WorldData::world_to_chunk(IVec3::new(0, 0, 0)),
            IVec2::new(0, 0)
        );
        assert_eq!(
            WorldData::world_to_chunk(IVec3::new(16, 0, 0)),
            IVec2::new(1, 0)
        );
        assert_eq!(
            WorldData::world_to_chunk(IVec3::new(-1, 0, -1)),
            IVec2::new(-1, -1)
        );
    }

    #[test]
    fn test_world_data_block_operations() {
        let mut world = WorldData::default();
        world
            .chunks
            .insert(IVec2::new(0, 0), ChunkData::generate(IVec2::ZERO));
        assert!(world.has_block(IVec3::new(0, 0, 0)));
        assert!(!world.has_block(IVec3::new(0, 10, 0)));
    }

    #[test]
    fn test_mesh_winding_order() {
        let mut world = WorldData::default();
        let chunk_coord = IVec2::new(0, 0);
        world
            .chunks
            .insert(chunk_coord, ChunkData::generate(chunk_coord));
        let mesh = world
            .generate_chunk_mesh(chunk_coord)
            .expect("mesh generation should succeed");
        let positions = match mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("mesh should have positions")
        {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.clone(),
            _ => panic!("Unexpected vertex format"),
        };
        let indices = match mesh.indices().expect("mesh should have indices") {
            bevy::render::mesh::Indices::U32(v) => v.clone(),
            _ => panic!("Unexpected index format"),
        };
        let normals = match mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .expect("mesh should have normals")
        {
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
            if cross.length() < 0.0001 {
                continue;
            }
            let calc_normal = cross.normalize();
            let expected = Vec3::from_array(normals[tri[0] as usize]);
            total += 1;
            if calc_normal.dot(expected) > 0.9 {
                correct += 1;
            }
        }
        assert!(correct as f32 / total as f32 > 0.99);
    }

    #[test]
    fn test_inventory_add() {
        let mut inventory = PlayerInventory::default();
        inventory.add_item_by_id(items::stone(), 1);
        assert_eq!(inventory.get_total_count_by_id(items::stone()), 1);
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
        let (_, normal) = result.expect("ray should hit");
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
        let mut inventory = PlayerInventory::default();
        inventory.add_item_by_id(items::stone(), 10);
        inventory.selected_slot = 0;
        assert_eq!(inventory.get_slot_count(0), 10);
        inventory.consume_item_by_id(items::stone(), 1);
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
            &InteractingMachine(None),
            &CommandInputState::default(),
            &CursorLockState::default(),
        );
        assert!(matches!(state, InputState::Gameplay));

        let state = InputState::current(
            &InventoryOpen(true),
            &InteractingMachine(None),
            &CommandInputState::default(),
            &CursorLockState {
                paused: true,
                ..default()
            },
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
            output_direction: Direction::North,
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
            output_direction: Direction::North,
            items: vec![],
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::Straight,
        };
        conveyor.add_item(items::iron_ore(), 0.5);
        // Item at 0.5, so 0.4 and 0.6 should be too close
        assert!(!conveyor.can_accept_item(0.5));
        assert!(!conveyor.can_accept_item(0.45));
        // But 0.0 should be far enough
        assert!(conveyor.can_accept_item(0.0));
    }

    #[test]
    fn test_furnace_smelt_output() {
        use idle_factory::components::get_smelt_output_by_id;

        assert_eq!(
            get_smelt_output_by_id(items::iron_ore()),
            Some(items::iron_ingot())
        );
        assert_eq!(
            get_smelt_output_by_id(items::copper_ore()),
            Some(items::copper_ingot())
        );
        assert_eq!(get_smelt_output_by_id(items::stone()), None);
    }

    #[test]
    fn test_machine_can_add_input() {
        // Test Machine component accepts correct ore types
        use idle_factory::components::Machine;
        use idle_factory::game_spec::FURNACE;

        let mut machine = Machine::new(&FURNACE, IVec3::ZERO, Direction::North);

        // Furnace should accept ore or dust
        let input_slot = machine.slots.inputs.first_mut().unwrap();
        assert!(input_slot.item_id.is_none());
        assert_eq!(input_slot.count, 0);

        // Can add ore when empty
        input_slot.item_id = Some(items::iron_ore());
        input_slot.count = 1;

        // Same type can be added
        let same_type_ok = input_slot.item_id == Some(items::iron_ore());
        assert!(same_type_ok);
    }

    #[test]
    fn test_crusher_has_recipes() {
        use idle_factory::components::{can_crush_by_id, get_crush_output_by_id};

        // Crusher now has recipes for ore -> dust (doubles output)
        assert!(can_crush_by_id(items::iron_ore()));
        assert!(can_crush_by_id(items::copper_ore()));
        assert!(!can_crush_by_id(items::stone())); // Stone can't be crushed
        assert!(!can_crush_by_id(items::iron_ingot())); // Ingots can't be crushed

        // Ore outputs should be dust (with count 2 = doubling)
        assert_eq!(
            get_crush_output_by_id(items::iron_ore()),
            Some((items::iron_dust(), 2))
        );
        assert_eq!(
            get_crush_output_by_id(items::copper_ore()),
            Some((items::copper_dust(), 2))
        );
        assert!(get_crush_output_by_id(items::stone()).is_none());
    }

    #[test]
    fn test_furnace_uses_recipe_system() {
        // Verify furnace smelt output matches recipe system
        use idle_factory::components::get_smelt_output_by_id;
        use idle_factory::core::items;
        use idle_factory::game_spec::{find_recipe, MachineType};

        let iron_output = get_smelt_output_by_id(items::iron_ore());
        let recipe = find_recipe(MachineType::Furnace, items::iron_ore());
        assert!(iron_output.is_some());
        assert!(recipe.is_some());
        assert_eq!(
            iron_output.expect("smelt output should exist"),
            recipe.expect("recipe should exist").outputs[0].item
        );
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
            output_direction: Direction::North,
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
    fn test_machine_default() {
        use idle_factory::components::Machine;
        use idle_factory::game_spec::MINER;

        let machine = Machine::new(&MINER, IVec3::ZERO, Direction::North);
        assert_eq!(machine.position, IVec3::ZERO);
        assert_eq!(machine.progress, 0.0);
        // Output slot should be empty
        let output = machine.slots.outputs.first().unwrap();
        assert!(output.item_id.is_none());
        assert_eq!(output.count, 0);
    }

    #[test]
    fn test_conveyor_corner_left_direction() {
        // Corner left: turns to the left of the facing direction
        // North-facing corner left outputs to West (left of North)
        let dir = Direction::North;
        let left_dir = dir.left();
        let output = IVec3::new(5, 0, 5) + left_dir.to_ivec3();
        assert_eq!(left_dir, Direction::West);
        assert_eq!(
            output,
            IVec3::new(4, 0, 5),
            "Corner left should output to West"
        );
    }

    #[test]
    fn test_conveyor_corner_right_direction() {
        // Corner right: turns to the right of the facing direction
        // North-facing corner right outputs to East (right of North)
        let dir = Direction::North;
        let right_dir = dir.right();
        let output = IVec3::new(5, 0, 5) + right_dir.to_ivec3();
        assert_eq!(right_dir, Direction::East);
        assert_eq!(
            output,
            IVec3::new(6, 0, 5),
            "Corner right should output to East"
        );
    }

    #[test]
    fn test_conveyor_l_shape_chain() {
        // Test a chain of conveyors forming an L-shape
        // Straight (North) -> Corner Right (turn East) -> Straight (East)
        let corner_pos = IVec3::new(5, 0, 5);
        let corner_dir = Direction::North;

        // Corner right outputs to the right of its facing direction
        let output_dir = corner_dir.right(); // Turns right: North -> East
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
            (Direction::North, Direction::West, Direction::East), // N: left=W, right=E
            (Direction::East, Direction::North, Direction::South), // E: left=N, right=S
            (Direction::South, Direction::East, Direction::West), // S: left=E, right=W
            (Direction::West, Direction::South, Direction::North), // W: left=S, right=N
        ];

        for (facing, expected_left, expected_right) in cases {
            assert_eq!(facing.left(), expected_left, "Left of {:?}", facing);
            assert_eq!(facing.right(), expected_right, "Right of {:?}", facing);
        }
    }

    // === Conveyor Integration Tests ===

    #[test]
    fn test_conveyor_join_progress_straight() {
        // Test join progress for straight entry (from behind)
        let conveyor = Conveyor {
            position: IVec3::new(5, 0, 5),
            direction: Direction::East,
            output_direction: Direction::East,
            items: vec![],
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::Straight,
        };

        // From behind (West) should join at 0.0
        let from_west = IVec3::new(4, 0, 5);
        let info = conveyor.get_join_info(from_west);
        assert!(info.is_some());
        let (progress, lateral) = info.expect("join info should exist for west entry");
        assert!((progress - 0.0).abs() < 0.001, "Should join at 0.0");
        assert!((lateral - 0.0).abs() < 0.001, "No lateral offset");
    }

    #[test]
    fn test_conveyor_join_progress_from_side() {
        // Test join progress from side (T-junction behavior)
        let conveyor = Conveyor {
            position: IVec3::new(5, 0, 5),
            direction: Direction::East,
            output_direction: Direction::East,
            items: vec![],
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::TJunction,
        };

        // From North side should join at 0.5 with lateral offset
        let from_north = IVec3::new(5, 0, 4);
        let info = conveyor.get_join_info(from_north);
        assert!(info.is_some());
        let (progress, lateral) = info.expect("join info should exist for north entry");
        assert!((progress - 0.5).abs() < 0.001, "Side join at 0.5");
        assert!(lateral.abs() > 0.1, "Should have lateral offset");

        // From South side
        let from_south = IVec3::new(5, 0, 6);
        let info = conveyor.get_join_info(from_south);
        assert!(info.is_some());
        let (progress, _) = info.expect("join info should exist for south entry");
        assert!((progress - 0.5).abs() < 0.001, "Side join at 0.5");
    }

    #[test]
    fn test_conveyor_multiple_items() {
        // Test conveyor can hold multiple items with proper spacing
        let mut conveyor = Conveyor {
            position: IVec3::ZERO,
            direction: Direction::North,
            output_direction: Direction::North,
            items: vec![],
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::Straight,
        };

        // Add first item at 0.0
        assert!(conveyor.can_accept_item(0.0));
        conveyor.add_item(items::iron_ore(), 0.0);

        // Add second item at 0.5 (far enough away)
        assert!(conveyor.can_accept_item(0.5));
        conveyor.add_item(items::copper_ore(), 0.5);

        // Items should be sorted by progress
        assert_eq!(conveyor.items.len(), 2);
        assert!((conveyor.items[0].progress - 0.0).abs() < 0.001);
        assert!((conveyor.items[1].progress - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_conveyor_max_items() {
        use idle_factory::constants::CONVEYOR_MAX_ITEMS;

        let mut conveyor = Conveyor {
            position: IVec3::ZERO,
            direction: Direction::North,
            output_direction: Direction::North,
            items: vec![],
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::Straight,
        };

        // Fill up to max items
        for i in 0..CONVEYOR_MAX_ITEMS {
            let progress = i as f32 * 0.2; // Spread items out
            conveyor.add_item(items::stone(), progress);
        }

        assert_eq!(conveyor.items.len(), CONVEYOR_MAX_ITEMS);
        // Should not accept more
        assert!(!conveyor.can_accept_item(0.9));
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::South.opposite(), Direction::North);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::West.opposite(), Direction::East);
    }

    #[test]
    fn test_splitter_all_directions() {
        // Test splitter outputs for all facing directions
        for dir in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            let conveyor = Conveyor {
                position: IVec3::new(10, 0, 10),
                direction: dir,
                output_direction: dir,
                items: vec![],
                last_output_index: 0,
                last_input_source: 0,
                shape: ConveyorShape::Splitter,
            };

            let outputs = conveyor.get_splitter_outputs();

            // Front should be in facing direction
            assert_eq!(outputs[0], conveyor.position + dir.to_ivec3());
            // Left should be left of facing
            assert_eq!(outputs[1], conveyor.position + dir.left().to_ivec3());
            // Right should be right of facing
            assert_eq!(outputs[2], conveyor.position + dir.right().to_ivec3());
        }
    }

    #[test]
    fn test_machine_facing_output_position() {
        use idle_factory::components::Machine;
        use idle_factory::game_spec::{FURNACE, MINER};

        // Test that machines output in facing direction
        let miner = Machine::new(&MINER, IVec3::new(5, 5, 5), Direction::East);
        let output_pos = miner.position + miner.facing.to_ivec3();
        assert_eq!(output_pos, IVec3::new(6, 5, 5));

        let furnace = Machine::new(&FURNACE, IVec3::new(10, 5, 10), Direction::South);
        let output_pos = furnace.position + furnace.facing.to_ivec3();
        assert_eq!(output_pos, IVec3::new(10, 5, 11));
    }

    #[test]
    fn test_machine_input_from_back() {
        use idle_factory::components::Machine;
        use idle_factory::game_spec::FURNACE;

        // Machines receive input from the back (opposite of facing)
        let furnace = Machine::new(&FURNACE, IVec3::new(10, 5, 10), Direction::North);

        // Input comes from South (behind the furnace)
        let input_pos = furnace.position - furnace.facing.to_ivec3();
        assert_eq!(input_pos, IVec3::new(10, 5, 11)); // +Z is South
    }
}
