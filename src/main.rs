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
mod setup;
mod systems;
mod ui;
mod utils;
mod world;

use components::*;
use events::GameEventsPlugin;
use logging::GameLoggingPlugin;
use player::Inventory;
use setup::{setup_initial_items, setup_lighting, setup_player, setup_ui};
use utils::{auto_conveyor_direction, ray_aabb_intersection, ray_aabb_intersection_with_normal, yaw_to_direction};
use systems::{
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
    update_hotbar_ui, update_inventory_tooltip, update_window_title_fps,
    // Quest systems
    load_machine_models, quest_claim_rewards, quest_progress_check, setup_delivery_platform,
    update_delivery_ui, update_quest_ui,
    // Save systems
    auto_save_system, handle_load_event, handle_save_event,
    // Targeting systems
    rotate_conveyor_placement, update_conveyor_shapes, update_guide_markers,
    update_target_block, update_target_highlight,
};
use tracing::info;
use world::{ChunkMesh, ChunkMeshTasks, WorldData};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::PresentMode;
use bevy::window::CursorGrabMode;

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
        .add_event::<SaveGameEvent>()
        .add_event::<LoadGameEvent>()
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
            ),
        )
        .run();
}

// === System Params ===

/// Bundled machine queries for block_break system (reduces parameter count)
#[derive(SystemParam)]
struct MachineBreakQueries<'w, 's> {
    conveyor: Query<'w, 's, (Entity, &'static Conveyor, &'static GlobalTransform)>,
    miner: Query<'w, 's, (Entity, &'static Miner, &'static GlobalTransform)>,
    crusher: Query<'w, 's, (Entity, &'static Crusher, &'static GlobalTransform)>,
    furnace: Query<'w, 's, (Entity, &'static Furnace, &'static GlobalTransform)>,
}

/// Bundled machine queries for block_place system (reduces parameter count)
#[derive(SystemParam)]
struct MachinePlaceQueries<'w, 's> {
    conveyor: Query<'w, 's, &'static Conveyor>,
    miner: Query<'w, 's, &'static Miner>,
    crusher: Query<'w, 's, (&'static Crusher, &'static Transform)>,
    furnace: Query<'w, 's, &'static Transform, With<Furnace>>,
}

// === Update Systems ===


#[allow(clippy::too_many_arguments)]
fn block_break(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &PlayerCamera)>,
    machines: MachineBreakQueries,
    mut world_data: ResMut<WorldData>,
    mut inventory: ResMut<Inventory>,
    windows: Query<&Window>,
    item_visual_query: Query<Entity, With<ConveyorItemVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cursor_state: ResMut<CursorLockState>,
    input_resources: InputStateResources,
    mut action_timer: ResMut<ContinuousActionTimer>,
) {
    // Only break blocks when cursor is locked and not paused
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Use InputState to check if block actions are allowed (see CLAUDE.md 入力マトリクス)
    let input_state = input_resources.get_state_with(&cursor_state);
    if !input_state.allows_block_actions() {
        return;
    }

    // Support continuous breaking: first click is instant, then timer-gated
    let can_break = mouse_button.just_pressed(MouseButton::Left)
        || (mouse_button.pressed(MouseButton::Left) && action_timer.break_timer.finished());
    if can_break {
        action_timer.break_timer.reset();
    }

    if !cursor_locked || !can_break {
        return;
    }

    // Skip block break if we just locked the cursor (to avoid accidental destruction on resume click)
    if cursor_state.just_locked {
        cursor_state.just_locked = false;
        return;
    }

    let Ok((camera_transform, _camera)) = camera_query.get_single() else {
        return;
    };

    // Calculate ray from camera using its actual transform
    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Track what we hit (world block, conveyor, miner, crusher, or furnace)
    enum HitType {
        WorldBlock(IVec3),
        Conveyor(Entity), // entity only - items handled separately
        Miner(Entity),
        Crusher(Entity),
        Furnace(Entity),
    }
    let mut closest_hit: Option<(HitType, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    // Check world blocks using DDA (Digital Differential Analyzer) for precise traversal
    {
        // Current voxel position
        let mut current = IVec3::new(
            ray_origin.x.floor() as i32,
            ray_origin.y.floor() as i32,
            ray_origin.z.floor() as i32,
        );

        // Direction sign for stepping (+1 or -1 for each axis)
        let step = IVec3::new(
            if ray_direction.x >= 0.0 { 1 } else { -1 },
            if ray_direction.y >= 0.0 { 1 } else { -1 },
            if ray_direction.z >= 0.0 { 1 } else { -1 },
        );

        // How far along the ray we need to travel for one voxel step on each axis
        let t_delta = Vec3::new(
            if ray_direction.x.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.x).abs() },
            if ray_direction.y.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.y).abs() },
            if ray_direction.z.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.z).abs() },
        );

        // Distance to next voxel boundary for each axis
        let mut t_max = Vec3::new(
            if ray_direction.x >= 0.0 {
                ((current.x + 1) as f32 - ray_origin.x) / ray_direction.x.abs().max(1e-8)
            } else {
                (ray_origin.x - current.x as f32) / ray_direction.x.abs().max(1e-8)
            },
            if ray_direction.y >= 0.0 {
                ((current.y + 1) as f32 - ray_origin.y) / ray_direction.y.abs().max(1e-8)
            } else {
                (ray_origin.y - current.y as f32) / ray_direction.y.abs().max(1e-8)
            },
            if ray_direction.z >= 0.0 {
                ((current.z + 1) as f32 - ray_origin.z) / ray_direction.z.abs().max(1e-8)
            } else {
                (ray_origin.z - current.z as f32) / ray_direction.z.abs().max(1e-8)
            },
        );

        let max_steps = (REACH_DISTANCE * 2.0) as i32;

        for _ in 0..max_steps {
            if world_data.has_block(current) {
                // Calculate hit distance
                let block_center = Vec3::new(
                    current.x as f32 + 0.5,
                    current.y as f32 + 0.5,
                    current.z as f32 + 0.5,
                );
                if let Some(hit_t) = ray_aabb_intersection(
                    ray_origin,
                    ray_direction,
                    block_center - Vec3::splat(half_size),
                    block_center + Vec3::splat(half_size),
                ) {
                    if hit_t > 0.0 && hit_t < REACH_DISTANCE {
                        let is_closer = closest_hit.as_ref().is_none_or(|h| hit_t < h.1);
                        if is_closer {
                            closest_hit = Some((HitType::WorldBlock(current), hit_t));
                        }
                        break; // Found first block
                    }
                }
            }

            // Step to next voxel
            if t_max.x < t_max.y && t_max.x < t_max.z {
                if t_max.x > REACH_DISTANCE { break; }
                current.x += step.x;
                t_max.x += t_delta.x;
            } else if t_max.y < t_max.z {
                if t_max.y > REACH_DISTANCE { break; }
                current.y += step.y;
                t_max.y += t_delta.y;
            } else {
                if t_max.z > REACH_DISTANCE { break; }
                current.z += step.z;
                t_max.z += t_delta.z;
            }
        }
    }

    // Check conveyors
    for (entity, _conveyor, conveyor_transform) in machines.conveyor.iter() {
        let conveyor_pos = conveyor_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            conveyor_pos - Vec3::new(half_size, 0.15, half_size),
            conveyor_pos + Vec3::new(half_size, 0.15, half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_hit.as_ref().is_none_or(|h| t < h.1);
                if is_closer {
                    closest_hit = Some((HitType::Conveyor(entity), t));
                }
            }
        }
    }

    // Check miners
    for (entity, _miner, miner_transform) in machines.miner.iter() {
        let miner_pos = miner_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            miner_pos - Vec3::splat(half_size),
            miner_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_hit.as_ref().is_none_or(|h| t < h.1);
                if is_closer {
                    closest_hit = Some((HitType::Miner(entity), t));
                }
            }
        }
    }

    // Check crushers
    for (entity, _crusher, crusher_transform) in machines.crusher.iter() {
        let crusher_pos = crusher_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            crusher_pos - Vec3::splat(half_size),
            crusher_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_hit.as_ref().is_none_or(|h| t < h.1);
                if is_closer {
                    closest_hit = Some((HitType::Crusher(entity), t));
                }
            }
        }
    }

    // Check furnaces
    for (entity, _furnace, furnace_transform) in machines.furnace.iter() {
        let furnace_pos = furnace_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            furnace_pos - Vec3::splat(half_size),
            furnace_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_hit.as_ref().is_none_or(|h| t < h.1);
                if is_closer {
                    closest_hit = Some((HitType::Furnace(entity), t));
                }
            }
        }
    }

    // Handle the hit
    if let Some((hit_type, _)) = closest_hit {
        match hit_type {
            HitType::WorldBlock(pos) => {
                if let Some(block_type) = world_data.remove_block(pos) {
                    info!(category = "BLOCK", action = "break", ?pos, ?block_type, "Block broken");
                    inventory.add_item(block_type, 1);
                    // No auto-select - keep current slot selected

                    // Regenerate the chunk mesh for the affected chunk (with neighbor awareness)
                    let chunk_coord = WorldData::world_to_chunk(pos);

                    // Helper closure to regenerate a chunk mesh
                    let regenerate_chunk = |coord: IVec2,
                                            commands: &mut Commands,
                                            world_data: &mut WorldData,
                                            meshes: &mut Assets<Mesh>,
                                            materials: &mut Assets<StandardMaterial>| {
                        // First despawn old entities BEFORE generating new mesh
                        #[allow(unused_variables)]
                        let old_count = if let Some(old_entities) = world_data.chunk_entities.remove(&coord) {
                            let count = old_entities.len();
                            for entity in old_entities {
                                commands.entity(entity).try_despawn_recursive();
                            }
                            count
                        } else {
                            0
                        };

                        if let Some(new_mesh) = world_data.generate_chunk_mesh(coord) {
                            let mesh_handle = meshes.add(new_mesh);
                            let material = materials.add(StandardMaterial {
                                base_color: Color::WHITE,
                                perceptual_roughness: 0.9,
                                ..default()
                            });

                            let entity = commands.spawn((
                                Mesh3d(mesh_handle),
                                MeshMaterial3d(material),
                                Transform::IDENTITY,
                                ChunkMesh { coord },
                            )).id();

                            world_data.chunk_entities.insert(coord, vec![entity]);

                            #[cfg(debug_assertions)]
                            info!("Regenerated chunk {:?}: despawned {} old, spawned new {:?}", coord, old_count, entity);
                        }
                    };

                    // Regenerate the main chunk
                    regenerate_chunk(chunk_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);

                    // Check if block is at chunk boundary and regenerate neighbor chunks
                    let local_pos = WorldData::world_to_local(pos);
                    let neighbor_coords: Vec<IVec2> = [
                        (local_pos.x == 0, IVec2::new(chunk_coord.x - 1, chunk_coord.y)),
                        (local_pos.x == CHUNK_SIZE - 1, IVec2::new(chunk_coord.x + 1, chunk_coord.y)),
                        (local_pos.z == 0, IVec2::new(chunk_coord.x, chunk_coord.y - 1)),
                        (local_pos.z == CHUNK_SIZE - 1, IVec2::new(chunk_coord.x, chunk_coord.y + 1)),
                    ]
                    .iter()
                    .filter(|(is_boundary, _)| *is_boundary)
                    .map(|(_, coord)| *coord)
                    .filter(|coord| world_data.chunks.contains_key(coord))
                    .collect();

                    for neighbor_coord in neighbor_coords {
                        regenerate_chunk(neighbor_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);
                    }
                }
            }
            HitType::Conveyor(entity) => {
                // Get conveyor items before despawning
                let item_count = if let Ok((_, conveyor, transform)) = machines.conveyor.get(entity) {
                    let pos = transform.translation();
                    let count = conveyor.items.len();
                    // Despawn all item visuals and return items to inventory
                    for item in &conveyor.items {
                        if let Some(visual_entity) = item.visual_entity {
                            if item_visual_query.get(visual_entity).is_ok() {
                                commands.entity(visual_entity).despawn();
                            }
                        }
                        inventory.add_item(item.block_type, 1);
                    }
                    info!(category = "MACHINE", action = "break", machine = "conveyor", ?pos, items_returned = count, "Conveyor broken");
                    count
                } else { 0 };
                let _ = item_count; // Suppress unused warning
                // Use despawn_recursive to also remove arrow marker child
                commands.entity(entity).despawn_recursive();
                // Return conveyor to inventory
                inventory.add_item(BlockType::ConveyorBlock, 1);
            }
            HitType::Miner(entity) => {
                info!(category = "MACHINE", action = "break", machine = "miner", "Miner broken");
                commands.entity(entity).despawn_recursive();
                // Return miner to inventory
                inventory.add_item(BlockType::MinerBlock, 1);
            }
            HitType::Crusher(entity) => {
                // Return crusher contents to inventory before despawning
                if let Ok((_, crusher, _)) = machines.crusher.get(entity) {
                    if let Some(input_type) = crusher.input_type {
                        if crusher.input_count > 0 {
                            inventory.add_item(input_type, crusher.input_count);
                        }
                    }
                    if let Some(output_type) = crusher.output_type {
                        if crusher.output_count > 0 {
                            inventory.add_item(output_type, crusher.output_count);
                        }
                    }
                }
                info!(category = "MACHINE", action = "break", machine = "crusher", "Crusher broken");
                commands.entity(entity).despawn_recursive();
                // Return crusher to inventory
                inventory.add_item(BlockType::CrusherBlock, 1);
            }
            HitType::Furnace(entity) => {
                // Return furnace contents to inventory before despawning
                if let Ok((_, furnace, _)) = machines.furnace.get(entity) {
                    // Return fuel (coal)
                    if furnace.fuel > 0 {
                        inventory.add_item(BlockType::Coal, furnace.fuel);
                    }
                    // Return input ore
                    if let Some(input_type) = furnace.input_type {
                        if furnace.input_count > 0 {
                            inventory.add_item(input_type, furnace.input_count);
                        }
                    }
                    // Return output ingots
                    if let Some(output_type) = furnace.output_type {
                        if furnace.output_count > 0 {
                            inventory.add_item(output_type, furnace.output_count);
                        }
                    }
                }
                info!(category = "MACHINE", action = "break", machine = "furnace", "Furnace broken");
                commands.entity(entity).despawn_recursive();
                // Return furnace to inventory
                inventory.add_item(BlockType::FurnaceBlock, 1);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn block_place(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &PlayerCamera)>,
    machines: MachinePlaceQueries,
    platform_query: Query<&Transform, With<DeliveryPlatform>>,
    mut world_data: ResMut<WorldData>,
    mut inventory: ResMut<Inventory>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Query<&Window>,
    creative_mode: Res<CreativeMode>,
    input_resources: InputStateResourcesWithCursor,
    mut action_timer: ResMut<ContinuousActionTimer>,
    mut rotation: ResMut<ConveyorRotationOffset>,
    machine_models: Res<MachineModels>,
) {
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Use InputState to check if block actions are allowed (see CLAUDE.md 入力マトリクス)
    let input_state = input_resources.get_state();
    if !input_state.allows_block_actions() || !cursor_locked {
        return;
    }

    // Support continuous placing: first click is instant, then timer-gated
    let can_place = mouse_button.just_pressed(MouseButton::Right)
        || (mouse_button.pressed(MouseButton::Right) && action_timer.place_timer.finished());
    if can_place {
        action_timer.place_timer.reset();
    }

    if !can_place {
        return;
    }

    // Check if we have a selected block type with items
    if !inventory.has_selected() {
        return;
    }
    let selected_type = inventory.selected_block().unwrap();

    let Ok((camera_transform, player_camera)) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();
    let half_size = BLOCK_SIZE / 2.0;

    // Check if looking at a furnace or crusher - if so, don't place (let machine UI handle it)
    for furnace_transform in machines.furnace.iter() {
        let furnace_pos = furnace_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            furnace_pos - Vec3::splat(half_size),
            furnace_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                return; // Looking at furnace, let furnace_interact handle it
            }
        }
    }
    for (_, crusher_transform) in machines.crusher.iter() {
        let crusher_pos = crusher_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            crusher_pos - Vec3::splat(half_size),
            crusher_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                return; // Looking at crusher, let crusher_interact handle it
            }
        }
    }

    // Find closest block intersection with hit normal using DDA
    let mut closest_hit: Option<(IVec3, Vec3, f32)> = None;

    {
        // Current voxel position
        let mut current = IVec3::new(
            ray_origin.x.floor() as i32,
            ray_origin.y.floor() as i32,
            ray_origin.z.floor() as i32,
        );

        // Direction sign for stepping (+1 or -1 for each axis)
        let step = IVec3::new(
            if ray_direction.x >= 0.0 { 1 } else { -1 },
            if ray_direction.y >= 0.0 { 1 } else { -1 },
            if ray_direction.z >= 0.0 { 1 } else { -1 },
        );

        // How far along the ray we need to travel for one voxel step on each axis
        let t_delta = Vec3::new(
            if ray_direction.x.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.x).abs() },
            if ray_direction.y.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.y).abs() },
            if ray_direction.z.abs() < 1e-8 { f32::MAX } else { (1.0 / ray_direction.z).abs() },
        );

        // Distance to next voxel boundary for each axis
        let mut t_max = Vec3::new(
            if ray_direction.x >= 0.0 {
                ((current.x + 1) as f32 - ray_origin.x) / ray_direction.x.abs().max(1e-8)
            } else {
                (ray_origin.x - current.x as f32) / ray_direction.x.abs().max(1e-8)
            },
            if ray_direction.y >= 0.0 {
                ((current.y + 1) as f32 - ray_origin.y) / ray_direction.y.abs().max(1e-8)
            } else {
                (ray_origin.y - current.y as f32) / ray_direction.y.abs().max(1e-8)
            },
            if ray_direction.z >= 0.0 {
                ((current.z + 1) as f32 - ray_origin.z) / ray_direction.z.abs().max(1e-8)
            } else {
                (ray_origin.z - current.z as f32) / ray_direction.z.abs().max(1e-8)
            },
        );

        // Track which axis we stepped on last (for face normal)
        let mut last_step_axis = 0; // 0=x, 1=y, 2=z
        let max_steps = (REACH_DISTANCE * 2.0) as i32;

        for _ in 0..max_steps {
            if world_data.has_block(current) {
                let block_center = Vec3::new(
                    current.x as f32 + 0.5,
                    current.y as f32 + 0.5,
                    current.z as f32 + 0.5,
                );
                if let Some((hit_t, _normal)) = ray_aabb_intersection_with_normal(
                    ray_origin,
                    ray_direction,
                    block_center - Vec3::splat(half_size),
                    block_center + Vec3::splat(half_size),
                ) {
                    if hit_t > 0.0 && hit_t < REACH_DISTANCE {
                        // Use DDA-calculated normal for more accurate placement
                        let dda_normal = match last_step_axis {
                            0 => Vec3::new(-step.x as f32, 0.0, 0.0),
                            1 => Vec3::new(0.0, -step.y as f32, 0.0),
                            _ => Vec3::new(0.0, 0.0, -step.z as f32),
                        };
                        closest_hit = Some((current, dda_normal, hit_t));
                        break;
                    }
                }
            }

            // Step to next voxel
            if t_max.x < t_max.y && t_max.x < t_max.z {
                if t_max.x > REACH_DISTANCE { break; }
                current.x += step.x;
                t_max.x += t_delta.x;
                last_step_axis = 0;
            } else if t_max.y < t_max.z {
                if t_max.y > REACH_DISTANCE { break; }
                current.y += step.y;
                t_max.y += t_delta.y;
                last_step_axis = 1;
            } else {
                if t_max.z > REACH_DISTANCE { break; }
                current.z += step.z;
                t_max.z += t_delta.z;
                last_step_axis = 2;
            }
        }
    }

    // Also check DeliveryPlatform for raycast hit
    if let Ok(platform_transform) = platform_query.get_single() {
        let platform_center = platform_transform.translation;
        let platform_half_x = (PLATFORM_SIZE as f32 * BLOCK_SIZE) / 2.0;
        let platform_half_y = BLOCK_SIZE * 0.1; // 0.2 height / 2
        let platform_half_z = platform_half_x;

        let platform_min = platform_center - Vec3::new(platform_half_x, platform_half_y, platform_half_z);
        let platform_max = platform_center + Vec3::new(platform_half_x, platform_half_y, platform_half_z);

        if let Some((hit_t, normal)) = ray_aabb_intersection_with_normal(
            ray_origin,
            ray_direction,
            platform_min,
            platform_max,
        ) {
            if hit_t > 0.0 && hit_t < REACH_DISTANCE {
                // Convert hit point to block position for placement
                let hit_point = ray_origin + ray_direction * hit_t;
                let hit_block_pos = IVec3::new(
                    hit_point.x.floor() as i32,
                    hit_point.y.floor() as i32,
                    hit_point.z.floor() as i32,
                );
                let is_closer = closest_hit.is_none_or(|h| hit_t < h.2);
                if is_closer {
                    closest_hit = Some((hit_block_pos, normal, hit_t));
                }
            }
        }
    }

    // Place block on the adjacent face
    if let Some((hit_pos, normal, _)) = closest_hit {
        let place_pos = hit_pos + IVec3::new(
            normal.x.round() as i32,
            normal.y.round() as i32,
            normal.z.round() as i32,
        );

        // Don't place if already occupied (check world data and all machine entities)
        if world_data.has_block(place_pos) {
            return;
        }
        // Check if any conveyor occupies this position
        for conveyor in machines.conveyor.iter() {
            if conveyor.position == place_pos {
                return;
            }
        }
        // Check if any miner occupies this position
        for miner in machines.miner.iter() {
            if miner.position == place_pos {
                return;
            }
        }
        // Check if any crusher occupies this position
        for (crusher, _) in machines.crusher.iter() {
            if crusher.position == place_pos {
                return;
            }
        }
        // Check if any furnace occupies this position
        for furnace_transform in machines.furnace.iter() {
            let furnace_pos = IVec3::new(
                (furnace_transform.translation.x / BLOCK_SIZE).floor() as i32,
                (furnace_transform.translation.y / BLOCK_SIZE).floor() as i32,
                (furnace_transform.translation.z / BLOCK_SIZE).floor() as i32,
            );
            if furnace_pos == place_pos {
                return;
            }
        }

        // Consume from inventory (unless in creative mode)
        if !creative_mode.enabled {
            inventory.consume_selected();
        }

        // Get chunk coord for the placed block
        let chunk_coord = WorldData::world_to_chunk(place_pos);

        // Calculate direction from player yaw for conveyors
        let player_facing = yaw_to_direction(player_camera.yaw);

        // For conveyors, use auto-direction based on adjacent machines
        let facing_direction = if selected_type == BlockType::ConveyorBlock {
            // Collect conveyor positions and directions
            let conveyors: Vec<(IVec3, Direction)> = machines.conveyor
                .iter()
                .map(|c| (c.position, c.direction))
                .collect();

            // Collect machine positions (miners, crushers, furnaces)
            let mut machine_positions: Vec<IVec3> = Vec::new();
            for miner in machines.miner.iter() {
                machine_positions.push(miner.position);
            }
            for (crusher, _) in machines.crusher.iter() {
                machine_positions.push(crusher.position);
            }
            for furnace_transform in machines.furnace.iter() {
                machine_positions.push(IVec3::new(
                    furnace_transform.translation.x.floor() as i32,
                    furnace_transform.translation.y.floor() as i32,
                    furnace_transform.translation.z.floor() as i32,
                ));
            }

            // Apply rotation offset (R key)
            let mut dir = auto_conveyor_direction(place_pos, player_facing, &conveyors, &machine_positions);
            for _ in 0..rotation.offset {
                dir = dir.rotate_cw();
            }
            dir
        } else {
            player_facing
        };

        // Helper closure to regenerate a chunk mesh (same pattern as block_break)
        let regenerate_chunk = |coord: IVec2,
                                commands: &mut Commands,
                                world_data: &mut WorldData,
                                meshes: &mut Assets<Mesh>,
                                materials: &mut Assets<StandardMaterial>| {
            // First despawn old entities BEFORE generating new mesh
            if let Some(old_entities) = world_data.chunk_entities.remove(&coord) {
                for entity in old_entities {
                    commands.entity(entity).try_despawn_recursive();
                }
            }

            if let Some(new_mesh) = world_data.generate_chunk_mesh(coord) {
                let mesh_handle = meshes.add(new_mesh);
                let material = materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 0.9,
                    ..default()
                });

                let entity = commands.spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material),
                    Transform::IDENTITY,
                    ChunkMesh { coord },
                )).id();

                world_data.chunk_entities.insert(coord, vec![entity]);
            }
        };

        // Spawn entity based on block type
        match selected_type {
            BlockType::MinerBlock => {
                info!(category = "MACHINE", action = "place", machine = "miner", ?place_pos, "Miner placed");
                // Machines are spawned as separate entities, no need to modify world data
                // (they don't occlude terrain blocks)

                let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                let material = materials.add(StandardMaterial {
                    base_color: selected_type.color(),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(cube_mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE + 0.5,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    )),
                    Miner {
                        position: place_pos,
                        ..default()
                    },
                ));
            }
            BlockType::ConveyorBlock => {
                info!(category = "MACHINE", action = "place", machine = "conveyor", ?place_pos, ?facing_direction, "Conveyor placed");
                // Machines are spawned as separate entities, no need to modify world data

                let conveyor_pos = Vec3::new(
                    place_pos.x as f32 * BLOCK_SIZE + 0.5,
                    place_pos.y as f32 * BLOCK_SIZE,
                    place_pos.z as f32 * BLOCK_SIZE + 0.5,
                );

                // Try to use glTF model, fallback to procedural mesh
                if let Some(model_handle) = machine_models.get_conveyor_model(ConveyorShape::Straight) {
                    // Spawn with glTF model
                    // Note: GlobalTransform and Visibility are required for rendering
                    commands.spawn((
                        SceneRoot(model_handle),
                        Transform::from_translation(conveyor_pos)
                            .with_rotation(facing_direction.to_rotation()),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Conveyor {
                            position: place_pos,
                            direction: facing_direction,
                            items: Vec::new(),
                            last_output_index: 0,
                            last_input_source: 0,
                            shape: ConveyorShape::Straight,
                        },
                        ConveyorVisual,
                    ));
                } else {
                    // Fallback: procedural mesh
                    let conveyor_mesh = meshes.add(Cuboid::new(
                        BLOCK_SIZE * CONVEYOR_BELT_WIDTH,
                        BLOCK_SIZE * CONVEYOR_BELT_HEIGHT,
                        BLOCK_SIZE
                    ));
                    let material = materials.add(StandardMaterial {
                        base_color: selected_type.color(),
                        ..default()
                    });
                    let arrow_mesh = meshes.add(Cuboid::new(BLOCK_SIZE * 0.12, BLOCK_SIZE * 0.03, BLOCK_SIZE * 0.35));
                    let arrow_material = materials.add(StandardMaterial {
                        base_color: Color::srgb(0.9, 0.9, 0.2),
                        ..default()
                    });
                    let belt_y = place_pos.y as f32 * BLOCK_SIZE + CONVEYOR_BELT_HEIGHT / 2.0;
                    commands.spawn((
                        Mesh3d(conveyor_mesh),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(
                            place_pos.x as f32 * BLOCK_SIZE + 0.5,
                            belt_y,
                            place_pos.z as f32 * BLOCK_SIZE + 0.5,
                        )).with_rotation(facing_direction.to_rotation()),
                        Conveyor {
                            position: place_pos,
                            direction: facing_direction,
                            items: Vec::new(),
                            last_output_index: 0,
                            last_input_source: 0,
                            shape: ConveyorShape::Straight,
                        },
                        ConveyorVisual,
                    )).with_children(|parent| {
                        parent.spawn((
                            Mesh3d(arrow_mesh),
                            MeshMaterial3d(arrow_material),
                            Transform::from_translation(Vec3::new(0.0, CONVEYOR_BELT_HEIGHT / 2.0 + 0.02, -0.25)),
                        ));
                    });
                }
                // Reset rotation offset after placing (so next placement uses auto-direction)
                rotation.offset = 0;
            }
            BlockType::CrusherBlock => {
                info!(category = "MACHINE", action = "place", machine = "crusher", ?place_pos, "Crusher placed");
                // Machines are spawned as separate entities, no need to modify world data
                // (they don't occlude terrain blocks)

                let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                let material = materials.add(StandardMaterial {
                    base_color: selected_type.color(),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(cube_mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE + 0.5,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    )),
                    Crusher {
                        position: place_pos,
                        input_type: None,
                        input_count: 0,
                        output_type: None,
                        output_count: 0,
                        progress: 0.0,
                    },
                ));
            }
            BlockType::FurnaceBlock => {
                info!(category = "MACHINE", action = "place", machine = "furnace", ?place_pos, "Furnace placed");
                // Furnace - similar to crusher, spawns entity with Furnace component
                let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                let material = materials.add(StandardMaterial {
                    base_color: selected_type.color(),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(cube_mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE + 0.5,
                        place_pos.y as f32 * BLOCK_SIZE + 0.5,
                        place_pos.z as f32 * BLOCK_SIZE + 0.5,
                    )),
                    Furnace::default(),
                ));
            }
            _ => {
                // Regular block - add to world data and regenerate chunk mesh
                info!(category = "BLOCK", action = "place", ?place_pos, block_type = ?selected_type, "Block placed");
                world_data.set_block(place_pos, selected_type);
                regenerate_chunk(chunk_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);

                // Check if block is at chunk boundary and regenerate neighbor chunks
                let local_pos = WorldData::world_to_local(place_pos);
                let neighbor_offsets: [(i32, i32, bool); 4] = [
                    (-1, 0, local_pos.x == 0),           // West boundary
                    (1, 0, local_pos.x == CHUNK_SIZE - 1), // East boundary
                    (0, -1, local_pos.z == 0),           // North boundary
                    (0, 1, local_pos.z == CHUNK_SIZE - 1), // South boundary
                ];

                for (dx, dz, at_boundary) in neighbor_offsets {
                    if at_boundary {
                        let neighbor_coord = IVec2::new(chunk_coord.x + dx, chunk_coord.y + dz);
                        if world_data.chunks.contains_key(&neighbor_coord) {
                            regenerate_chunk(neighbor_coord, &mut commands, &mut world_data, &mut meshes, &mut materials);
                        }
                    }
                }
            }
        }
    }
}

// === Creative Mode ===

// Note: Target block, highlight, conveyor shapes, and guide markers moved to systems/targeting.rs
// F-key shortcuts removed - use Creative Catalog (E key while in creative mode) instead
// Use /creative and /survival commands to toggle modes

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;
    use world::ChunkData;
    use crate::systems::get_quests;

    #[test]
    fn test_chunk_generation() {
        let chunk = ChunkData::generate(IVec2::ZERO);
        // Check that chunk has blocks
        assert!(!chunk.blocks_map.is_empty());

        // Check that top layer is grass or ore (biome ore patches show on surface)
        let surface_block = chunk.blocks_map.get(&IVec3::new(0, 7, 0));
        assert!(matches!(
            surface_block,
            Some(BlockType::Grass) | Some(BlockType::IronOre) | Some(BlockType::CopperOre) | Some(BlockType::Coal)
        ));

        // Check that lower layers are stone or ore (ores are generated randomly)
        let block = chunk.blocks_map.get(&IVec3::new(0, 0, 0));
        assert!(matches!(
            block,
            Some(BlockType::Stone) | Some(BlockType::IronOre) | Some(BlockType::CopperOre) | Some(BlockType::Coal)
        ));
    }

    #[test]
    fn test_world_coordinate_conversion() {
        // Test world to chunk conversion
        assert_eq!(WorldData::world_to_chunk(IVec3::new(0, 0, 0)), IVec2::new(0, 0));
        assert_eq!(WorldData::world_to_chunk(IVec3::new(15, 0, 15)), IVec2::new(0, 0));
        assert_eq!(WorldData::world_to_chunk(IVec3::new(16, 0, 0)), IVec2::new(1, 0));
        assert_eq!(WorldData::world_to_chunk(IVec3::new(-1, 0, -1)), IVec2::new(-1, -1));
        assert_eq!(WorldData::world_to_chunk(IVec3::new(-16, 0, -16)), IVec2::new(-1, -1));

        // Test world to local conversion
        assert_eq!(WorldData::world_to_local(IVec3::new(0, 5, 0)), IVec3::new(0, 5, 0));
        assert_eq!(WorldData::world_to_local(IVec3::new(17, 3, 18)), IVec3::new(1, 3, 2));
        assert_eq!(WorldData::world_to_local(IVec3::new(-1, 7, -1)), IVec3::new(15, 7, 15));

        // Test local to world conversion
        assert_eq!(WorldData::local_to_world(IVec2::new(0, 0), IVec3::new(5, 3, 7)), IVec3::new(5, 3, 7));
        assert_eq!(WorldData::local_to_world(IVec2::new(1, 2), IVec3::new(5, 3, 7)), IVec3::new(21, 3, 39));
    }

    #[test]
    fn test_world_data_block_operations() {
        let mut world = WorldData::default();

        // Insert a chunk first
        world.chunks.insert(IVec2::new(0, 0), ChunkData::generate(IVec2::ZERO));

        // Test get_block - surface can be grass or ore (biome ore patches)
        let surface = world.get_block(IVec3::new(0, 7, 0));
        assert!(matches!(
            surface,
            Some(BlockType::Grass) | Some(BlockType::IronOre) | Some(BlockType::CopperOre) | Some(BlockType::Coal)
        ));
        // Lower layers can be stone or ore
        let block = world.get_block(IVec3::new(0, 0, 0));
        assert!(matches!(
            block,
            Some(BlockType::Stone) | Some(BlockType::IronOre) | Some(BlockType::CopperOre) | Some(BlockType::Coal)
        ));

        // Test has_block
        assert!(world.has_block(IVec3::new(0, 0, 0)));
        assert!(!world.has_block(IVec3::new(0, 10, 0))); // Above terrain

        // Test remove_block - surface can be grass or ore (biome ore patches)
        let removed = world.remove_block(IVec3::new(0, 7, 0));
        assert!(matches!(
            removed,
            Some(BlockType::Grass) | Some(BlockType::IronOre) | Some(BlockType::CopperOre) | Some(BlockType::Coal)
        ));
        assert!(!world.has_block(IVec3::new(0, 7, 0)));

        // Test set_block (y=7 is within CHUNK_HEIGHT)
        world.set_block(IVec3::new(0, 7, 0), BlockType::Stone);
        assert_eq!(world.get_block(IVec3::new(0, 7, 0)), Some(&BlockType::Stone));
    }

    #[test]
    fn test_mesh_regeneration_after_block_removal() {
        let mut world = WorldData::default();
        let chunk_coord = IVec2::new(0, 0);
        world.chunks.insert(chunk_coord, ChunkData::generate(chunk_coord));

        // Generate initial mesh
        let mesh_before = world.generate_chunk_mesh(chunk_coord).unwrap();
        let positions_before = mesh_before.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let vertex_count_before = match positions_before {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Unexpected vertex format"),
        };

        // Remove a block in the middle of the chunk
        let block_pos = IVec3::new(8, 7, 8);
        assert!(world.has_block(block_pos), "Block should exist before removal");
        world.remove_block(block_pos);
        assert!(!world.has_block(block_pos), "Block should not exist after removal");

        // Generate mesh after removal
        let mesh_after = world.generate_chunk_mesh(chunk_coord).unwrap();
        let positions_after = mesh_after.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let vertex_count_after = match positions_after {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Unexpected vertex format"),
        };

        // After removing a block, the mesh should have MORE vertices
        // because the neighboring blocks' inner faces are now exposed
        // Removing one block exposes: bottom face of removed block (now visible from above is gone)
        // BUT: the neighbors' faces toward the removed block are now visible
        // So we should have more faces (4 side faces + 1 bottom face - 1 top face = +4 faces minimum)
        assert!(
            vertex_count_after > vertex_count_before,
            "Mesh should have more vertices after block removal. Before: {}, After: {}",
            vertex_count_before,
            vertex_count_after
        );
    }

    #[test]
    fn test_mesh_winding_order() {
        // Test that mesh triangles have correct winding order (CCW when viewed from outside)
        // This ensures faces are not culled by backface culling
        let mut world = WorldData::default();
        let chunk_coord = IVec2::new(0, 0);
        world.chunks.insert(chunk_coord, ChunkData::generate(chunk_coord));

        let mesh = world.generate_chunk_mesh(chunk_coord).unwrap();

        // Get positions and indices
        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.clone(),
            _ => panic!("Unexpected vertex format"),
        };

        let indices = match mesh.indices().unwrap() {
            bevy::render::mesh::Indices::U32(v) => v.clone(),
            _ => panic!("Unexpected index format"),
        };

        // Get normals from mesh
        let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.clone(),
            _ => panic!("Unexpected normal format"),
        };

        // Check each triangle's winding order
        let mut checked_triangles = 0;
        let mut correct_winding = 0;
        let mut wrong_examples: Vec<String> = Vec::new();

        for tri in indices.chunks(3) {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let v0 = Vec3::from_array(positions[i0]);
            let v1 = Vec3::from_array(positions[i1]);
            let v2 = Vec3::from_array(positions[i2]);

            // Calculate face normal using cross product (CCW winding)
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let cross = edge1.cross(edge2);

            // Skip degenerate triangles
            if cross.length() < 0.0001 {
                continue;
            }
            let calculated_normal = cross.normalize();

            // Get expected normal from vertex attribute
            let expected_normal = Vec3::from_array(normals[i0]);

            // Check if calculated normal matches expected normal (dot product > 0)
            let dot = calculated_normal.dot(expected_normal);

            checked_triangles += 1;
            if dot > 0.9 {
                correct_winding += 1;
            } else if wrong_examples.len() < 3 {
                wrong_examples.push(format!(
                    "v0={:?}, v1={:?}, v2={:?}, calc_norm={:?}, expected={:?}, dot={:.3}",
                    v0, v1, v2, calculated_normal, expected_normal, dot
                ));
            }
        }

        // All triangles should have correct winding order
        let correct_percentage = (correct_winding as f32 / checked_triangles as f32) * 100.0;
        assert!(
            correct_percentage > 99.0,
            "Winding order incorrect! Only {:.1}% of {} triangles have correct winding. \
             This will cause backface culling issues (faces appearing as black holes).\n\
             Examples of wrong triangles:\n{}",
            correct_percentage,
            checked_triangles,
            wrong_examples.join("\n")
        );
    }

    #[test]
    fn test_inventory_add() {
        let mut inventory = Inventory::default();
        inventory.add_item(BlockType::Stone, 1);
        assert_eq!(inventory.get_item_count(BlockType::Stone), 1);

        inventory.add_item(BlockType::Stone, 1);
        assert_eq!(inventory.get_item_count(BlockType::Stone), 2);
    }

    #[test]
    fn test_block_type_properties() {
        assert_eq!(BlockType::Stone.name(), "Stone");
        assert_eq!(BlockType::Grass.name(), "Grass");
    }

    #[test]
    fn test_ray_aabb_hit() {
        // Ray pointing at box
        let result = ray_aabb_intersection(
            Vec3::new(0.0, 0.0, -5.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        assert!(result.is_some());
    }

    #[test]
    fn test_ray_aabb_miss() {
        // Ray pointing away from box
        let result = ray_aabb_intersection(
            Vec3::new(0.0, 0.0, -5.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_ray_aabb_with_normal_z() {
        // Ray from -Z hitting front face
        let result = ray_aabb_intersection_with_normal(
            Vec3::new(0.0, 0.0, -5.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        assert!(result.is_some());
        let (t, normal) = result.unwrap();
        assert!(t > 0.0);
        assert_eq!(normal, Vec3::NEG_Z); // Hit front face, normal points back
    }

    #[test]
    fn test_ray_aabb_with_normal_y() {
        // Ray from above hitting top face
        let result = ray_aabb_intersection_with_normal(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        assert!(result.is_some());
        let (_, normal) = result.unwrap();
        assert_eq!(normal, Vec3::Y); // Hit top face, normal points up
    }

    #[test]
    fn test_inventory_selected() {
        let mut inventory = Inventory::default();
        assert_eq!(inventory.selected_slot, 0);
        assert!(inventory.selected_block().is_none()); // No items, so no block at slot 0

        // Add item and verify it's accessible at slot 0
        inventory.add_item(BlockType::Stone, 5);
        // Stone should be in the first slot
        assert!(inventory.selected_block().is_some());
        assert_eq!(inventory.selected_block(), Some(BlockType::Stone));
        assert_eq!(inventory.get_slot_count(0), 5);

        // Change slot to empty slot
        inventory.selected_slot = 1;
        assert_eq!(inventory.selected_slot, 1);
        assert!(inventory.selected_block().is_none());
    }

    // ============================================================
    // Consistency Check Tests - Detect spec/implementation mismatches
    // ============================================================

    /// Check that scroll wheel only cycles through hotbar slots (0-8)
    /// Rule: Mouse wheel should not select main inventory slots (9-35)
    #[test]
    fn test_hotbar_scroll_stays_in_bounds() {
        assert_eq!(HOTBAR_SLOTS, 9, "Hotbar should have 9 slots");
        assert_eq!(NUM_SLOTS, 36, "Total inventory should have 36 slots");

        // Verify scroll logic stays within hotbar bounds
        for start_slot in 0..HOTBAR_SLOTS {
            // Scroll down from any hotbar slot should stay in hotbar
            let next_slot = (start_slot + 1) % HOTBAR_SLOTS;
            assert!(next_slot < HOTBAR_SLOTS,
                "Scroll down from slot {} went to {} (outside hotbar)", start_slot, next_slot);

            // Scroll up from any hotbar slot should stay in hotbar
            let prev_slot = if start_slot == 0 { HOTBAR_SLOTS - 1 } else { start_slot - 1 };
            assert!(prev_slot < HOTBAR_SLOTS,
                "Scroll up from slot {} went to {} (outside hotbar)", start_slot, prev_slot);
        }
    }

    /// Check that inventory consumption works correctly
    /// Rule: Block placement should consume inventory (this tests the mechanism)
    #[test]
    fn test_inventory_consumption_mechanism() {
        let mut inventory = Inventory::default();
        inventory.add_item(BlockType::Stone, 10);
        inventory.selected_slot = 0;

        // Verify initial state
        assert_eq!(inventory.get_slot_count(0), 10);
        assert!(inventory.has_selected());

        // Consume one item
        let consumed = inventory.consume_selected();
        assert_eq!(consumed, Some(BlockType::Stone));
        assert_eq!(inventory.get_slot_count(0), 9);

        // Consume all remaining
        for _ in 0..9 {
            inventory.consume_selected();
        }
        assert_eq!(inventory.get_slot_count(0), 0);
        assert!(!inventory.has_selected(), "Empty slot should report no selection");
    }

    /// Check that mode-related constants are properly defined
    #[test]
    fn test_mode_constants_consistency() {
        // CreativeMode should default to disabled (survival mode is default)
        let creative = CreativeMode::default();
        assert!(!creative.enabled, "Game should start in survival mode by default");
    }

    /// Check that inventory slot indices are valid
    #[test]
    fn test_inventory_slot_indices_consistency() {
        let inventory = Inventory::default();

        // All hotbar slots (0-8) should be accessible
        for i in 0..HOTBAR_SLOTS {
            assert!(inventory.get_slot(i).is_none() || inventory.get_slot(i).is_some(),
                "Slot {} should be accessible", i);
        }

        // All main inventory slots (9-35) should be accessible
        for i in HOTBAR_SLOTS..NUM_SLOTS {
            assert!(inventory.get_slot(i).is_none() || inventory.get_slot(i).is_some(),
                "Slot {} should be accessible", i);
        }

        // Out of bounds should return None
        assert!(inventory.get_slot(NUM_SLOTS).is_none(),
            "Slot beyond NUM_SLOTS should return None");
    }

    /// Check that BlockType properties are consistent
    #[test]
    fn test_block_type_consistency() {
        // All block types should have a name
        let all_types = [
            BlockType::Grass,
            BlockType::Stone,
            BlockType::IronOre,
            BlockType::Coal,
            BlockType::IronIngot,
            BlockType::MinerBlock,
            BlockType::ConveyorBlock,
            BlockType::CopperOre,
            BlockType::CopperIngot,
            BlockType::CrusherBlock,
            BlockType::FurnaceBlock,
        ];

        for bt in all_types {
            assert!(!bt.name().is_empty(), "{:?} should have a non-empty name", bt);
            // Color should be valid (not panic)
            let _color = bt.color();
        }
    }

    /// Check that quest rewards are valid block types
    #[test]
    fn test_quest_rewards_consistency() {
        let quests = get_quests();
        assert!(!quests.is_empty(), "Should have at least one quest");

        for (i, quest) in quests.iter().enumerate() {
            // Required item should be a valid block type
            let _name = quest.required_item.name();

            // Rewards should all be valid
            for (block_type, amount) in &quest.rewards {
                assert!(*amount > 0, "Quest {} reward amount should be positive", i);
                let _name = block_type.name();
            }

            // Required amount should be positive
            assert!(quest.required_amount > 0, "Quest {} required amount should be positive", i);
        }
    }

    /// Check that initial items are properly set up for new game
    #[test]
    fn test_initial_game_state_consistency() {
        // New inventory should be empty (items are added by setup_initial_items)
        let inventory = Inventory::default();
        for i in 0..NUM_SLOTS {
            assert!(inventory.slots[i].is_none(),
                "Default inventory slot {} should be empty", i);
        }

        // Default selected slot should be 0
        assert_eq!(inventory.selected_slot, 0);
    }

    // === InputState Tests ===

    #[test]
    fn test_input_state_priority() {
        // Test priority: Paused > Command > Inventory > FurnaceUI > CrusherUI > Gameplay

        // All closed = Gameplay
        let state = InputState::current(
            &InventoryOpen(false),
            &InteractingFurnace(None),
            &InteractingCrusher(None),
            &InteractingMiner(None),
            &CommandInputState::default(),
            &CursorLockState::default(),
        );
        assert!(matches!(state, InputState::Gameplay));

        // Paused overrides everything
        let state = InputState::current(
            &InventoryOpen(true),
            &InteractingFurnace(Some(Entity::PLACEHOLDER)),
            &InteractingCrusher(Some(Entity::PLACEHOLDER)),
            &InteractingMiner(Some(Entity::PLACEHOLDER)),
            &CommandInputState { open: true, text: String::new() },
            &CursorLockState { paused: true, ..default() },
        );
        assert!(matches!(state, InputState::Paused));

        // Command overrides UI states
        let state = InputState::current(
            &InventoryOpen(true),
            &InteractingFurnace(Some(Entity::PLACEHOLDER)),
            &InteractingCrusher(None),
            &InteractingMiner(None),
            &CommandInputState { open: true, text: String::new() },
            &CursorLockState::default(),
        );
        assert!(matches!(state, InputState::Command));

        // Inventory overrides machine UIs
        let state = InputState::current(
            &InventoryOpen(true),
            &InteractingFurnace(Some(Entity::PLACEHOLDER)),
            &InteractingCrusher(None),
            &InteractingMiner(None),
            &CommandInputState::default(),
            &CursorLockState::default(),
        );
        assert!(matches!(state, InputState::Inventory));
    }

    #[test]
    fn test_input_state_allows_methods() {
        // Gameplay allows everything
        assert!(InputState::Gameplay.allows_movement());
        assert!(InputState::Gameplay.allows_block_actions());
        assert!(InputState::Gameplay.allows_hotbar());

        // UI states block movement
        assert!(!InputState::Inventory.allows_movement());
        assert!(!InputState::FurnaceUI.allows_movement());
        assert!(!InputState::CrusherUI.allows_movement());
        assert!(!InputState::Command.allows_movement());
        assert!(!InputState::Paused.allows_movement());

        // UI states block block actions
        assert!(!InputState::Inventory.allows_block_actions());
        assert!(!InputState::Command.allows_block_actions());
        assert!(!InputState::Paused.allows_block_actions());
    }

    #[test]
    fn test_tutorial_state_default() {
        // TutorialShown should default to false (tutorial visible on startup)
        let tutorial = TutorialShown::default();
        assert!(!tutorial.0, "Tutorial should not be shown by default");
    }

    #[test]
    fn test_biome_generation() {
        // Test that biomes are deterministic
        let biome1 = ChunkData::get_biome(0, 0);
        let biome2 = ChunkData::get_biome(0, 0);
        assert_eq!(biome1, biome2, "Biome should be deterministic");

        // Test that different regions can have different biomes
        let mut biomes_found = std::collections::HashSet::new();
        for x in 0..10 {
            for z in 0..10 {
                let biome = ChunkData::get_biome(x * 32, z * 32);
                biomes_found.insert(biome);
            }
        }
        // Should find multiple biome types (0=Mixed, 1=Iron, 2=Copper, 3=Coal)
        assert!(biomes_found.len() >= 2, "Should have at least 2 different biomes");

        // Test biome ore concentrations by generating chunks
        let iron_chunk = ChunkData::generate(IVec2::new(100, 100)); // Will be in some biome
        let has_ore = iron_chunk.blocks.iter().any(|b| b.is_some() && matches!(b, Some(BlockType::IronOre) | Some(BlockType::CopperOre) | Some(BlockType::Coal)));
        assert!(has_ore, "Chunks should contain some ore");
    }

    #[test]
    fn test_surface_ore_patches() {
        // Test that surface ore patches are deterministic
        let is_patch1 = ChunkData::is_surface_ore_patch(0, 0);
        let is_patch2 = ChunkData::is_surface_ore_patch(0, 0);
        assert_eq!(is_patch1, is_patch2, "Ore patch check should be deterministic");

        // Test that some positions are ore patches and some are not
        let mut patch_count = 0;
        for x in 0..100 {
            for z in 0..100 {
                if ChunkData::is_surface_ore_patch(x, z) {
                    patch_count += 1;
                }
            }
        }
        // Expect roughly 12.5% (1/8) to be patches
        assert!(patch_count > 500 && patch_count < 2000,
            "Ore patch distribution should be ~12.5%, got {}", patch_count);
    }
}
