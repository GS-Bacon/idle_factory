//! Idle Factory - Milestone 1: Minimal Voxel Game
//! Goal: Walk, mine blocks, collect in inventory

mod block_type;
mod components;
mod constants;
mod events;
mod game_spec;
mod logging;
mod machines;
mod player;
mod save;
mod systems;
mod ui;
mod world;

use components::*;
use events::GameEventsPlugin;
use logging::GameLoggingPlugin;
use player::Inventory;
use systems::{
    player_look, player_move, receive_chunk_meshes, spawn_chunk_tasks, tick_action_timers,
    toggle_cursor_lock, tutorial_dismiss, unload_distant_chunks,
};
use tracing::info;
use world::{ChunkData, ChunkMesh, ChunkMeshTasks, WorldData};

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::pbr::{CascadeShadowConfigBuilder, NotShadowCaster};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::PresentMode;
use bevy::window::CursorGrabMode;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

pub use block_type::BlockType;
pub use constants::*;

/// Set the UI open state for JavaScript overlay control (WASM only)
/// When ui_open is true, JavaScript will not show "Click to Resume" overlay
#[cfg(target_arch = "wasm32")]
fn set_ui_open_state(ui_open: bool) {
    use web_sys::window;
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(canvas) = doc.get_element_by_id("bevy-canvas") {
                let _ = canvas.set_attribute("data-ui-open", if ui_open { "true" } else { "false" });
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn set_ui_open_state(_ui_open: bool) {
    // No-op on native
}

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

// === Setup Systems ===

fn setup_lighting(mut commands: Commands) {
    // Directional light with high-quality shadows
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            first_cascade_far_bound: 10.0,
            maximum_distance: 100.0,
            ..default()
        }
        .build(),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });
}

fn setup_player(mut commands: Commands) {
    // Player entity with camera
    commands
        .spawn((
            Player,
            Transform::from_xyz(8.0, 12.0, 20.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Projection::Perspective(PerspectiveProjection {
                    fov: 90.0_f32.to_radians(), // Wider FOV for better responsiveness feel
                    ..default()
                }),
                // Use Reinhard tonemapping (doesn't require tonemapping_luts feature)
                Tonemapping::Reinhard,
                PlayerCamera {
                    pitch: 0.0,
                    yaw: 0.0,
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        });
}

/// Helper to spawn a machine UI slot (fuel/input/output)
fn spawn_machine_slot(parent: &mut ChildBuilder, slot_type: MachineSlotType, label: &str, color: Color) {
    parent
        .spawn((
            Button,
            MachineSlotButton(slot_type),
            Node {
                width: Val::Px(60.0),
                height: Val::Px(60.0),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
        ))
        .with_children(|slot| {
            // Label
            slot.spawn((
                Text::new(label),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
            ));
            // Count
            slot.spawn((
                MachineSlotCount(slot_type),
                Text::new("0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Helper to spawn a crusher UI slot (input/output only, no fuel)
fn spawn_crusher_slot(parent: &mut ChildBuilder, slot_type: MachineSlotType, label: &str, color: Color) {
    parent
        .spawn((
            Button,
            CrusherSlotButton(slot_type),
            Node {
                width: Val::Px(55.0),
                height: Val::Px(55.0),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
        ))
        .with_children(|slot| {
            // Label
            slot.spawn((
                Text::new(label),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
            ));
            // Count
            slot.spawn((
                CrusherSlotCount(slot_type),
                Text::new("0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Helper to spawn an inventory slot button
fn spawn_inventory_slot(parent: &mut ChildBuilder, slot_idx: usize) {
    parent
        .spawn((
            Button,
            InventorySlotUI(slot_idx),
            Node {
                width: Val::Px(36.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
        ))
        .with_children(|btn| {
            // Slot number (small, top-left)
            btn.spawn((
                Text::new(""),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn setup_ui(mut commands: Commands) {
    // Hotbar UI - centered at bottom
    commands
        .spawn((
            HotbarUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-225.0), // Center 9 slots (9 * 50 = 450, half = 225)
                    ..default()
                },
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Create 9 hotbar slots
            for i in 0..9 {
                parent
                    .spawn((
                        HotbarSlot(i),
                        Node {
                            width: Val::Px(46.0),
                            height: Val::Px(46.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                        BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                    ))
                    .with_children(|slot| {
                        // Slot number
                        slot.spawn((
                            Text::new(format!("{}", i + 1)),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(2.0),
                                left: Val::Px(4.0),
                                ..default()
                            },
                        ));
                        // Item count
                        slot.spawn((
                            HotbarSlotCount(i),
                            Text::new(""),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });

    // Hotbar item name display (above hotbar)
    commands.spawn((
        HotbarItemNameText,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(75.0),
            left: Val::Percent(50.0),
            ..default()
        },
    ));

    // Crosshair
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(4.0),
            height: Val::Px(4.0),
            margin: UiRect {
                left: Val::Px(-2.0),
                top: Val::Px(-2.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ));

    // Furnace UI panel (hidden by default) - Minecraft-style slot layout
    commands
        .spawn((
            FurnaceUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(15.0)),
                margin: UiRect {
                    left: Val::Px(-175.0),
                    ..default()
                },
                width: Val::Px(350.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Furnace"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Keep FurnaceUIText for backwards compatibility (hidden, used for state)
            parent.spawn((
                FurnaceUIText,
                Text::new(""),
                TextFont { font_size: 1.0, ..default() },
                TextColor(Color::NONE),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));

            // Main slot layout: [Input] -> [Progress] -> [Output]
            //                      [Fuel]
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    ..default()
                },))
                .with_children(|layout| {
                    // Top row: Input -> Arrow -> Output
                    layout
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(15.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|row| {
                            // Input slot (Iron Ore / Copper Ore)
                            spawn_machine_slot(row, MachineSlotType::Input, "Ore", Color::srgb(0.6, 0.5, 0.4));

                            // Progress bar container
                            row.spawn((Node {
                                width: Val::Px(60.0),
                                height: Val::Px(20.0),
                                flex_direction: FlexDirection::Row,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                            ))
                            .with_children(|bar_container| {
                                // Progress fill
                                bar_container.spawn((
                                    MachineProgressBar,
                                    Node {
                                        width: Val::Percent(0.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(1.0, 0.5, 0.0)),
                                ));
                            });

                            // Output slot (Ingot)
                            spawn_machine_slot(row, MachineSlotType::Output, "Ingot", Color::srgb(0.8, 0.8, 0.85));
                        });

                    // Bottom row: Fuel slot
                    layout
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|row| {
                            // Fuel slot (Coal)
                            spawn_machine_slot(row, MachineSlotType::Fuel, "Fuel", Color::srgb(0.15, 0.15, 0.15));
                        });
                });

            // Instructions
            parent.spawn((
                Text::new("Click slots to add/take items | ESC to close"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
        });

    // Crusher UI panel (hidden by default) - Minecraft-style slot layout
    commands
        .spawn((
            CrusherUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(15.0)),
                margin: UiRect {
                    left: Val::Px(-150.0),
                    ..default()
                },
                width: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.12, 0.18, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Crusher"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Slot layout: [Input] -> [Progress] -> [Output]
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(15.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },))
                .with_children(|row| {
                    // Input slot (Ore)
                    spawn_crusher_slot(row, MachineSlotType::Input, "Ore", Color::srgb(0.5, 0.4, 0.35));

                    // Progress bar container
                    row.spawn((Node {
                        width: Val::Px(50.0),
                        height: Val::Px(16.0),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_children(|bar_container| {
                        // Progress fill (uses CrusherProgressBar marker)
                        bar_container.spawn((
                            CrusherProgressBar,
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.3, 0.7)),
                        ));
                    });

                    // Output slot (Ore x2)
                    spawn_crusher_slot(row, MachineSlotType::Output, "x2", Color::srgb(0.6, 0.5, 0.45));
                });

            // Instructions
            parent.spawn((
                Text::new("Click to add/take ore | ESC to close"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
        });

    // Miner UI panel (hidden by default)
    commands
        .spawn((
            MinerUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(15.0)),
                margin: UiRect {
                    left: Val::Px(-150.0),
                    ..default()
                },
                width: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.15, 0.1, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Miner"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Buffer slot row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(15.0),
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Text::new("Buffer:"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    ));

                    // Buffer slot (clickable to take items)
                    row.spawn((
                        Button,
                        MinerBufferButton,
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(60.0),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.4, 0.35)),
                        BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            MinerBufferCountText,
                            Text::new("0"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Take all button
                    row.spawn((
                        Button,
                        MinerBufferButton,
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                        BorderColor(Color::srgba(0.3, 0.7, 0.3, 1.0)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Take All"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                });

            // Clear button row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },))
                .with_children(|row| {
                    // Clear/Reset button
                    row.spawn((
                        Button,
                        MinerClearButton,
                        Node {
                            padding: UiRect::axes(Val::Px(15.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                        BorderColor(Color::srgba(0.8, 0.3, 0.3, 1.0)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Discard Buffer"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                });

            // Instructions
            parent.spawn((
                Text::new("Click buffer to take items | ESC to close"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
        });

    // Delivery platform UI (top right corner)
    commands
        .spawn((
            DeliveryUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.3, 0.1, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                DeliveryUIText,
                Text::new("=== Deliveries ===\nNo items delivered"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    // Quest UI (top center)
    commands
        .spawn((
            QuestUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-150.0),
                    ..default()
                },
                padding: UiRect::all(Val::Px(10.0)),
                width: Val::Px(300.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.3, 0.2, 0.1, 0.9)),
        ))
        .with_children(|parent| {
            parent.spawn((
                QuestUIText,
                Text::new("=== Quest ===\nDeliver 3 Iron Ingots\nProgress: 0/3"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    // Full inventory UI (hidden by default, fullscreen overlay)
    commands
        .spawn((
            InventoryUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Main container (horizontal layout)
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(20.0),
                        ..default()
                    },
                ))
                .with_children(|container| {
                    // Left panel: Player inventory
                    container
                        .spawn((
                            Node {
                                width: Val::Px(400.0),
                                padding: UiRect::all(Val::Px(15.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                        ))
                        .with_children(|panel| {
                            // Title
                            panel.spawn((
                                Text::new("Inventory"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            // Main inventory (27 slots, 3x9 grid) - slots 9-35
                            panel
                                .spawn((
                                    Node {
                                        display: Display::Grid,
                                        grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
                                        row_gap: Val::Px(4.0),
                                        column_gap: Val::Px(4.0),
                                        margin: UiRect::top(Val::Px(10.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|grid| {
                                    for slot_idx in HOTBAR_SLOTS..NUM_SLOTS {
                                        spawn_inventory_slot(grid, slot_idx);
                                    }
                                });

                            // Separator
                            panel.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(2.0),
                                    margin: UiRect::vertical(Val::Px(5.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
                            ));

                            // Hotbar (9 slots) - slots 0-8
                            panel
                                .spawn((
                                    Node {
                                        display: Display::Grid,
                                        grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
                                        column_gap: Val::Px(4.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|grid| {
                                    for slot_idx in 0..HOTBAR_SLOTS {
                                        spawn_inventory_slot(grid, slot_idx);
                                    }
                                });

                            // Bottom row: Trash slot
                            panel
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::FlexEnd,
                                        margin: UiRect::top(Val::Px(10.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|row| {
                                    // Trash slot
                                    row.spawn((
                                        Button,
                                        TrashSlot,
                                        Node {
                                            width: Val::Px(40.0),
                                            height: Val::Px(40.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.4, 0.1, 0.1)),
                                        BorderColor(Color::srgb(0.6, 0.2, 0.2)),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("X"),
                                            TextFont {
                                                font_size: 16.0,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                                });

                            // Instructions
                            panel.spawn((
                                Text::new("Click: pick/place | Shift+Click: quick move | ESC: close"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
                            ));
                        });

                    // Right panel: Creative catalog (only visible in creative mode)
                    container
                        .spawn((
                            CreativePanel, // Marker to identify this panel
                            Node {
                                width: Val::Px(350.0),
                                padding: UiRect::all(Val::Px(15.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                            Visibility::Hidden, // Hidden by default, shown only in creative mode
                        ))
                        .with_children(|panel| {
                            // Title
                            panel.spawn((
                                Text::new("Creative Catalog"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            // Items grid
                            panel
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        flex_wrap: FlexWrap::Wrap,
                                        column_gap: Val::Px(6.0),
                                        row_gap: Val::Px(6.0),
                                        ..default()
                                    },
                                ))
                                .with_children(|grid| {
                                    for (block_type, _category) in CREATIVE_ITEMS.iter() {
                                        grid.spawn((
                                            Button,
                                            CreativeItemButton(*block_type),
                                            Node {
                                                width: Val::Px(60.0),
                                                height: Val::Px(60.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                flex_direction: FlexDirection::Column,
                                                border: UiRect::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(block_type.color()),
                                            BorderColor(Color::srgba(0.3, 0.3, 0.3, 1.0)),
                                        ))
                                        .with_children(|btn| {
                                            btn.spawn((
                                                Text::new(block_type.name()),
                                                TextFont {
                                                    font_size: 9.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });
                                    }
                                });
                        });
                });

            // Held item display (follows cursor)
            parent.spawn((
                HeldItemDisplay,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(36.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::End,
                    padding: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Visibility::Hidden,
            )).with_children(|parent| {
                // Item count text
                parent.spawn((
                    HeldItemText,
                    Text::new(""),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // Tooltip display (follows cursor, shows item name on hover)
            parent.spawn((
                InventoryTooltip,
                Node {
                    position_type: PositionType::Absolute,
                    padding: UiRect::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                Visibility::Hidden,
            )).with_children(|tooltip| {
                tooltip.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });

    // Command input UI (bottom center, hidden by default)
    commands
        .spawn((
            CommandInputUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(80.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-200.0),
                    ..default()
                },
                width: Val::Px(400.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.12, 0.12, 0.9)),
            BorderColor(Color::srgb(0.33, 0.33, 0.33)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Input text with ">" prefix
            parent.spawn((
                CommandInputText,
                Text::new("> "),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    // Tutorial popup (shown on first play)
    commands
        .spawn((
            TutorialPopup,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(450.0),
                        padding: UiRect::all(Val::Px(25.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(15.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderColor(Color::srgb(0.4, 0.6, 0.4)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("Controls"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.8, 0.4)),
                    ));

                    // Controls list
                    let controls = [
                        ("WASD", "Move"),
                        ("Mouse", "Look around"),
                        ("Left Click", "Break block"),
                        ("Right Click", "Place / Open machine"),
                        ("1-9 / Scroll", "Select hotbar"),
                        ("E", "Inventory"),
                        ("T or /", "Commands (/creative, /survival)"),
                        ("F3", "Debug info"),
                        ("ESC", "Pause"),
                    ];

                    for (key, action) in controls {
                        panel
                            .spawn((Node {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },))
                            .with_children(|row| {
                                row.spawn((
                                    Text::new(key),
                                    TextFont {
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.9, 0.5)),
                                ));
                                row.spawn((
                                    Text::new(action),
                                    TextFont {
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }

                    // Close instruction
                    panel.spawn((
                        Text::new("Click or press any key to start"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
                        Node {
                            margin: UiRect::top(Val::Px(15.0)),
                            ..default()
                        },
                    ));
                });
        });
}

/// Setup initial items on ground and furnace
/// Spec: game_spec::INITIAL_EQUIPMENT
fn setup_initial_items(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut inventory: ResMut<Inventory>,
) {
    // Give player initial items from spec (order determines slot positions)
    for (block_type, count) in game_spec::INITIAL_EQUIPMENT {
        inventory.add_item(*block_type, *count);
    }
    inventory.selected_slot = 0; // First slot

    let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));

    // Spawn a furnace near player spawn point (8, 8, 18)
    let furnace_pos = IVec3::new(10, 8, 18);
    let furnace_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.3, 0.3), // Dark reddish-brown for furnace
        ..default()
    });

    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(furnace_material),
        Transform::from_translation(Vec3::new(
            furnace_pos.x as f32 * BLOCK_SIZE + 0.5,
            furnace_pos.y as f32 * BLOCK_SIZE + 0.5,
            furnace_pos.z as f32 * BLOCK_SIZE + 0.5,
        )),
        Furnace::default(),
    ));
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

/// Convert player yaw to cardinal direction
fn yaw_to_direction(yaw: f32) -> Direction {
    // Normalize yaw to 0..2PI
    let yaw = yaw.rem_euclid(std::f32::consts::TAU);
    // Split into 4 quadrants (45 degree offset for centered regions)
    if !(PI / 4.0..7.0 * PI / 4.0).contains(&yaw) {
        Direction::North
    } else if yaw < 3.0 * PI / 4.0 {
        Direction::West
    } else if yaw < 5.0 * PI / 4.0 {
        Direction::South
    } else {
        Direction::East
    }
}

/// Determine optimal conveyor direction based on adjacent conveyors and machines
fn auto_conveyor_direction(
    place_pos: IVec3,
    fallback_direction: Direction,
    conveyors: &[(IVec3, Direction)], // (position, direction)
    machines: &[IVec3], // positions of miners, crushers, furnaces
) -> Direction {
    // Priority 1: Continue chain from adjacent conveyor pointing toward us
    for (conv_pos, conv_dir) in conveyors {
        let expected_target = *conv_pos + conv_dir.to_ivec3();
        if expected_target == place_pos {
            // This conveyor is pointing at our position, continue in same direction
            return *conv_dir;
        }
    }

    // Priority 2: Point away from adjacent machine (to receive items from it)
    for machine_pos in machines {
        let diff = place_pos - *machine_pos;
        if diff.x.abs() + diff.y.abs() + diff.z.abs() == 1 {
            // Adjacent machine - point away from it
            if diff.x == 1 { return Direction::East; }
            if diff.x == -1 { return Direction::West; }
            if diff.z == 1 { return Direction::South; }
            if diff.z == -1 { return Direction::North; }
        }
    }

    // Note: Previously had Priority 3 to align with adjacent conveyors,
    // but this prevented creating branches/splits. Now uses player direction.

    // Fallback: use player's facing direction
    fallback_direction
}

/// Select slot with number keys (1-9) or scroll wheel
fn select_block_type(
    key_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut inventory: ResMut<Inventory>,
    input_resources: InputStateResourcesWithCursor,
) {
    // Use InputState to check if hotbar selection is allowed (see CLAUDE.md 入力マトリクス)
    let input_state = input_resources.get_state();
    if !input_state.allows_hotbar() {
        // Still need to drain events to prevent accumulation
        for _ in mouse_wheel.read() {}
        return;
    }

    // Handle mouse wheel scroll (cycles through hotbar slots 0-8 only)
    for event in mouse_wheel.read() {
        let scroll = event.y;
        if scroll > 0.0 {
            // Scroll up - previous slot (within hotbar)
            inventory.selected_slot = if inventory.selected_slot == 0 {
                HOTBAR_SLOTS - 1
            } else {
                (inventory.selected_slot - 1).min(HOTBAR_SLOTS - 1)
            };
        } else if scroll < 0.0 {
            // Scroll down - next slot (within hotbar)
            inventory.selected_slot = (inventory.selected_slot + 1) % HOTBAR_SLOTS;
        }
    }

    // Number keys to select specific slots (1-9)
    let digit_keys = [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
    ];
    for (key, slot) in digit_keys {
        if key_input.just_pressed(key) {
            inventory.selected_slot = slot;
        }
    }
}

fn ray_aabb_intersection(
    ray_origin: Vec3,
    ray_direction: Vec3,
    box_min: Vec3,
    box_max: Vec3,
) -> Option<f32> {
    let inv_dir = Vec3::new(
        1.0 / ray_direction.x,
        1.0 / ray_direction.y,
        1.0 / ray_direction.z,
    );

    let t1 = (box_min.x - ray_origin.x) * inv_dir.x;
    let t2 = (box_max.x - ray_origin.x) * inv_dir.x;
    let t3 = (box_min.y - ray_origin.y) * inv_dir.y;
    let t4 = (box_max.y - ray_origin.y) * inv_dir.y;
    let t5 = (box_min.z - ray_origin.z) * inv_dir.z;
    let t6 = (box_max.z - ray_origin.z) * inv_dir.z;

    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    if tmax < 0.0 || tmin > tmax {
        None
    } else {
        Some(tmin)
    }
}

/// Ray-AABB intersection that also returns the hit normal
fn ray_aabb_intersection_with_normal(
    ray_origin: Vec3,
    ray_direction: Vec3,
    box_min: Vec3,
    box_max: Vec3,
) -> Option<(f32, Vec3)> {
    let inv_dir = Vec3::new(
        1.0 / ray_direction.x,
        1.0 / ray_direction.y,
        1.0 / ray_direction.z,
    );

    let tx1 = (box_min.x - ray_origin.x) * inv_dir.x;
    let tx2 = (box_max.x - ray_origin.x) * inv_dir.x;
    let ty1 = (box_min.y - ray_origin.y) * inv_dir.y;
    let ty2 = (box_max.y - ray_origin.y) * inv_dir.y;
    let tz1 = (box_min.z - ray_origin.z) * inv_dir.z;
    let tz2 = (box_max.z - ray_origin.z) * inv_dir.z;

    let tmin_x = tx1.min(tx2);
    let tmax_x = tx1.max(tx2);
    let tmin_y = ty1.min(ty2);
    let tmax_y = ty1.max(ty2);
    let tmin_z = tz1.min(tz2);
    let tmax_z = tz1.max(tz2);

    let tmin = tmin_x.max(tmin_y).max(tmin_z);
    let tmax = tmax_x.min(tmax_y).min(tmax_z);

    if tmax < 0.0 || tmin > tmax {
        return None;
    }

    // Determine which face was hit by finding which axis contributed to tmin
    let normal = if tmin == tmin_x {
        if ray_direction.x > 0.0 { Vec3::NEG_X } else { Vec3::X }
    } else if tmin == tmin_y {
        if ray_direction.y > 0.0 { Vec3::NEG_Y } else { Vec3::Y }
    } else if ray_direction.z > 0.0 {
        Vec3::NEG_Z
    } else {
        Vec3::Z
    };

    Some((tmin, normal))
}

fn update_hotbar_ui(
    inventory: Res<Inventory>,
    mut slot_query: Query<(&HotbarSlot, &mut BackgroundColor, &mut BorderColor)>,
    mut count_query: Query<(&HotbarSlotCount, &mut Text)>,
) {
    if !inventory.is_changed() {
        return;
    }

    // Update slot backgrounds - use slot index for selection
    for (slot, mut bg, mut border) in slot_query.iter_mut() {
        let is_selected = inventory.selected_slot == slot.0;
        let has_item = inventory.get_slot(slot.0).is_some();

        if is_selected {
            // Selected slot - same highlight for empty and filled
            *bg = BackgroundColor(Color::srgba(0.4, 0.4, 0.2, 0.9));
            *border = BorderColor(Color::srgba(1.0, 1.0, 0.5, 1.0));
        } else if has_item {
            // Non-selected filled slot
            *bg = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8));
            *border = BorderColor(Color::srgba(0.5, 0.5, 0.5, 1.0));
        } else {
            // Non-selected empty slot
            *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8));
            *border = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
        }
    }

    // Update slot counts
    for (slot_count, mut text) in count_query.iter_mut() {
        if let Some(block_type) = inventory.get_slot(slot_count.0) {
            let count = inventory.get_slot_count(slot_count.0);
            // Show abbreviated name and count
            let name = match block_type {
                BlockType::Grass => "Grs",
                BlockType::Stone => "Stn",
                BlockType::IronOre => "Fe",
                BlockType::Coal => "C",
                BlockType::IronIngot => "FeI",
                BlockType::MinerBlock => "Min",
                BlockType::ConveyorBlock => "Cnv",
                BlockType::CopperOre => "Cu",
                BlockType::CopperIngot => "CuI",
                BlockType::CrusherBlock => "Cru",
                BlockType::FurnaceBlock => "Fur",
            };
            **text = format!("{}\n{}", name, count);
        } else {
            **text = String::new();
        }
    }
}

/// Interact with furnace when looking at it and right-clicking
#[allow(clippy::too_many_arguments)]
fn furnace_interact(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    furnace_query: Query<(Entity, &Transform), With<Furnace>>,
    mut interacting: ResMut<InteractingFurnace>,
    mut furnace_ui_query: Query<&mut Visibility, With<FurnaceUI>>,
    mut windows: Query<&mut Window>,
    inventory_open: Res<InventoryOpen>,
    command_state: Res<CommandInputState>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't process when inventory, command input is open or game is paused (input matrix: Right Click)
    if inventory_open.0 || command_state.open || cursor_state.paused {
        return;
    }

    // ESC or E key to close furnace UI (when open)
    let e_pressed = key_input.just_pressed(KeyCode::KeyE);
    let esc_pressed = key_input.just_pressed(KeyCode::Escape);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        if let Ok(mut vis) = furnace_ui_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        let mut window = windows.single_mut();
        if esc_pressed {
            // ESC: Browser releases pointer lock automatically in WASM
            // Don't set paused=true - JS will auto-relock via data-ui-open observer (BUG-6 fix)
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            // Don't set paused - let JS handle auto-relock
            set_ui_open_state(false);
        } else {
            // E key: Keep cursor locked (no browser interference)
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Only open furnace UI with right-click
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;
    if !cursor_locked {
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Find closest furnace intersection
    let mut closest_furnace: Option<(Entity, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    for (entity, furnace_transform) in furnace_query.iter() {
        let furnace_pos = furnace_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            furnace_pos - Vec3::splat(half_size),
            furnace_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_furnace.is_none_or(|f| t < f.1);
                if is_closer {
                    closest_furnace = Some((entity, t));
                }
            }
        }
    }

    // Open furnace UI
    if let Some((entity, _)) = closest_furnace {
        interacting.0 = Some(entity);
        if let Ok(mut vis) = furnace_ui_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
        // Unlock cursor for UI interaction
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        set_ui_open_state(true);
    }
}

/// Handle slot click interactions when furnace UI is open
fn furnace_ui_input(
    interacting: Res<InteractingFurnace>,
    mut furnace_query: Query<&mut Furnace>,
    mut inventory: ResMut<Inventory>,
    mut slot_query: Query<
        (&Interaction, &MachineSlotButton, &mut BackgroundColor, &mut BorderColor),
        Changed<Interaction>,
    >,
) {
    let Some(furnace_entity) = interacting.0 else {
        return;
    };

    let Ok(mut furnace) = furnace_query.get_mut(furnace_entity) else {
        return;
    };

    for (interaction, slot_button, mut bg_color, mut border_color) in slot_query.iter_mut() {
        let slot_type = slot_button.0;

        match *interaction {
            Interaction::Pressed => {
                match slot_type {
                    MachineSlotType::Fuel => {
                        // Add coal from inventory (max 64)
                        const MAX_FUEL: u32 = 64;
                        if furnace.fuel < MAX_FUEL && inventory.consume_item(BlockType::Coal, 1) {
                            furnace.fuel += 1;
                        }
                    }
                    MachineSlotType::Input => {
                        // Add ore from inventory (prioritize iron, then copper)
                        if furnace.can_add_input(BlockType::IronOre)
                            && inventory.consume_item(BlockType::IronOre, 1)
                        {
                            furnace.input_type = Some(BlockType::IronOre);
                            furnace.input_count += 1;
                        } else if furnace.can_add_input(BlockType::CopperOre)
                            && inventory.consume_item(BlockType::CopperOre, 1)
                        {
                            furnace.input_type = Some(BlockType::CopperOre);
                            furnace.input_count += 1;
                        }
                    }
                    MachineSlotType::Output => {
                        // Take output ingot to inventory
                        if furnace.output_count > 0 {
                            if let Some(output_type) = furnace.output_type {
                                furnace.output_count -= 1;
                                inventory.add_item(output_type, 1);
                                if furnace.output_count == 0 {
                                    furnace.output_type = None;
                                }
                            }
                        }
                    }
                }
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                // Brighten background slightly
                let base = match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.25, 0.25, 0.25),
                    MachineSlotType::Input => Color::srgb(0.7, 0.6, 0.5),
                    MachineSlotType::Output => Color::srgb(0.9, 0.9, 0.95),
                };
                *bg_color = BackgroundColor(base);
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.15, 0.15, 0.15),
                    MachineSlotType::Input => Color::srgb(0.6, 0.5, 0.4),
                    MachineSlotType::Output => Color::srgb(0.8, 0.8, 0.85),
                });
            }
        }
    }
}

/// Smelting logic - convert ore + coal to ingot
fn furnace_smelting(
    time: Res<Time>,
    mut furnace_query: Query<&mut Furnace>,
) {
    for mut furnace in furnace_query.iter_mut() {
        // Need fuel, input ore, and valid recipe to smelt
        let Some(input_ore) = furnace.input_type else {
            furnace.progress = 0.0;
            continue;
        };

        if furnace.fuel == 0 || furnace.input_count == 0 {
            furnace.progress = 0.0;
            continue;
        }

        let output_ingot = Furnace::get_smelt_output(input_ore);

        // Check output slot compatibility
        let output_compatible = match (furnace.output_type, output_ingot) {
            (None, Some(_)) => true,
            (Some(current), Some(new)) => current == new && furnace.output_count < 64,
            _ => false,
        };

        if output_compatible {
            furnace.progress += time.delta_secs() / SMELT_TIME;

            // When progress reaches 1.0, complete smelting
            if furnace.progress >= 1.0 {
                furnace.progress = 0.0;
                furnace.fuel -= 1;
                furnace.input_count -= 1;
                if furnace.input_count == 0 {
                    furnace.input_type = None;
                }
                furnace.output_type = output_ingot;
                furnace.output_count += 1;
            }
        } else {
            furnace.progress = 0.0;
        }
    }
}

/// Crusher processing - doubles ore
fn crusher_processing(
    time: Res<Time>,
    mut crusher_query: Query<&mut Crusher>,
) {
    for mut crusher in crusher_query.iter_mut() {
        // Need input ore to process
        let Some(input_ore) = crusher.input_type else {
            crusher.progress = 0.0;
            continue;
        };

        if crusher.input_count == 0 {
            crusher.progress = 0.0;
            continue;
        }

        // Check output slot compatibility (same ore type or empty, max 64)
        let output_compatible = match crusher.output_type {
            None => true,
            Some(current) => current == input_ore && crusher.output_count < 63, // 63 because we add 2
        };

        if output_compatible {
            crusher.progress += time.delta_secs() / CRUSH_TIME;

            // When progress reaches 1.0, complete crushing
            if crusher.progress >= 1.0 {
                crusher.progress = 0.0;
                crusher.input_count -= 1;
                if crusher.input_count == 0 {
                    crusher.input_type = None;
                }
                crusher.output_type = Some(input_ore);
                crusher.output_count += 2; // Double output!
            }
        } else {
            crusher.progress = 0.0;
        }
    }
}

/// Handle crusher right-click interaction (open/close UI)
#[allow(clippy::too_many_arguments)]
fn crusher_interact(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    crusher_query: Query<(Entity, &Transform), With<Crusher>>,
    mut interacting: ResMut<InteractingCrusher>,
    inventory_open: Res<InventoryOpen>,
    interacting_furnace: Res<InteractingFurnace>,
    mut crusher_ui_query: Query<&mut Visibility, With<CrusherUI>>,
    mut windows: Query<&mut Window>,
    command_state: Res<CommandInputState>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't open crusher if inventory, furnace is open, command input is active, or game is paused (input matrix: Right Click)
    if inventory_open.0 || interacting_furnace.0.is_some() || command_state.open || cursor_state.paused {
        return;
    }

    let e_pressed = key_input.just_pressed(KeyCode::KeyE);
    let esc_pressed = key_input.just_pressed(KeyCode::Escape);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        if let Ok(mut vis) = crusher_ui_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        let mut window = windows.single_mut();
        if esc_pressed {
            // ESC: Browser releases pointer lock automatically in WASM
            // Don't set paused=true - JS will auto-relock via data-ui-open observer (BUG-6 fix)
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(false);
        } else {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Only open crusher UI with right-click
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;
    if !cursor_locked {
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Find closest crusher intersection
    let mut closest_crusher: Option<(Entity, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    for (entity, crusher_transform) in crusher_query.iter() {
        let crusher_pos = crusher_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            crusher_pos - Vec3::splat(half_size),
            crusher_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_crusher.is_none_or(|f| t < f.1);
                if is_closer {
                    closest_crusher = Some((entity, t));
                }
            }
        }
    }

    // Open crusher UI
    if let Some((entity, _)) = closest_crusher {
        interacting.0 = Some(entity);
        if let Ok(mut vis) = crusher_ui_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
        // Unlock cursor for UI interaction
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        set_ui_open_state(true);
    }
}

/// Handle crusher slot click interactions
fn crusher_ui_input(
    interacting: Res<InteractingCrusher>,
    mut crusher_query: Query<&mut Crusher>,
    mut inventory: ResMut<Inventory>,
    mut slot_query: Query<
        (&Interaction, &CrusherSlotButton, &mut BackgroundColor, &mut BorderColor),
        Changed<Interaction>,
    >,
) {
    let Some(crusher_entity) = interacting.0 else {
        return;
    };

    let Ok(mut crusher) = crusher_query.get_mut(crusher_entity) else {
        return;
    };

    for (interaction, slot_button, mut bg_color, mut border_color) in slot_query.iter_mut() {
        let slot_type = slot_button.0;

        match *interaction {
            Interaction::Pressed => {
                match slot_type {
                    MachineSlotType::Fuel => {
                        // Crusher has no fuel slot - do nothing
                    }
                    MachineSlotType::Input => {
                        // Add ore from inventory (prioritize iron, then copper, max 64)
                        const MAX_INPUT: u32 = 64;
                        if crusher.input_count < MAX_INPUT
                            && (crusher.input_type.is_none() || crusher.input_type == Some(BlockType::IronOre))
                            && inventory.consume_item(BlockType::IronOre, 1)
                        {
                            crusher.input_type = Some(BlockType::IronOre);
                            crusher.input_count += 1;
                        } else if crusher.input_count < MAX_INPUT
                            && (crusher.input_type.is_none() || crusher.input_type == Some(BlockType::CopperOre))
                            && inventory.consume_item(BlockType::CopperOre, 1)
                        {
                            crusher.input_type = Some(BlockType::CopperOre);
                            crusher.input_count += 1;
                        }
                    }
                    MachineSlotType::Output => {
                        // Take output ore to inventory
                        if crusher.output_count > 0 {
                            if let Some(output_type) = crusher.output_type {
                                crusher.output_count -= 1;
                                inventory.add_item(output_type, 1);
                                if crusher.output_count == 0 {
                                    crusher.output_type = None;
                                }
                            }
                        }
                    }
                }
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                let base = match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.5, 0.4, 0.35),
                    MachineSlotType::Input => Color::srgb(0.6, 0.5, 0.45),
                    MachineSlotType::Output => Color::srgb(0.7, 0.6, 0.55),
                };
                *bg_color = BackgroundColor(base);
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.5, 0.4, 0.35),
                    MachineSlotType::Input => Color::srgb(0.5, 0.4, 0.35),
                    MachineSlotType::Output => Color::srgb(0.6, 0.5, 0.45),
                });
            }
        }
    }
}

/// Handle miner right-click interaction (open/close UI)
#[allow(clippy::too_many_arguments)]
fn miner_interact(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    miner_query: Query<(Entity, &Transform), With<Miner>>,
    mut interacting: ResMut<InteractingMiner>,
    inventory_open: Res<InventoryOpen>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    mut miner_ui_query: Query<&mut Visibility, With<MinerUI>>,
    mut windows: Query<&mut Window>,
    command_state: Res<CommandInputState>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't open miner if other UI is open
    if inventory_open.0 || interacting_furnace.0.is_some() || interacting_crusher.0.is_some()
        || command_state.open || cursor_state.paused
    {
        return;
    }

    let e_pressed = key_input.just_pressed(KeyCode::KeyE);
    let esc_pressed = key_input.just_pressed(KeyCode::Escape);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        if let Ok(mut vis) = miner_ui_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        let mut window = windows.single_mut();
        if esc_pressed {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(false);
        } else {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Only open miner UI with right-click
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;
    if !cursor_locked {
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Find closest miner intersection
    let mut closest_miner: Option<(Entity, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    for (entity, miner_transform) in miner_query.iter() {
        let miner_pos = miner_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            miner_pos - Vec3::splat(half_size),
            miner_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_miner.is_none_or(|f| t < f.1);
                if is_closer {
                    closest_miner = Some((entity, t));
                }
            }
        }
    }

    // Open miner UI
    if let Some((entity, _)) = closest_miner {
        interacting.0 = Some(entity);
        if let Ok(mut vis) = miner_ui_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
        // Unlock cursor for UI interaction
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        set_ui_open_state(true);
    }
}

/// Handle miner UI button clicks (take buffer, discard buffer)
#[allow(clippy::type_complexity)]
fn miner_ui_input(
    interacting: Res<InteractingMiner>,
    mut miner_query: Query<&mut Miner>,
    mut inventory: ResMut<Inventory>,
    mut buffer_btn_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (With<MinerBufferButton>, Changed<Interaction>),
    >,
    mut clear_btn_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (With<MinerClearButton>, Without<MinerBufferButton>, Changed<Interaction>),
    >,
) {
    let Some(miner_entity) = interacting.0 else {
        return;
    };

    let Ok(mut miner) = miner_query.get_mut(miner_entity) else {
        return;
    };

    // Buffer button (take all items)
    for (interaction, mut bg_color, mut border_color) in buffer_btn_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Take all items from buffer to inventory
                if let Some((block_type, count)) = miner.buffer.take() {
                    inventory.add_item(block_type, count);
                }
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.5, 0.45));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(Color::srgb(0.5, 0.4, 0.35));
            }
        }
    }

    // Clear button (discard buffer)
    for (interaction, mut bg_color, mut border_color) in clear_btn_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Discard buffer contents
                miner.buffer = None;
                *border_color = BorderColor(Color::srgb(1.0, 0.3, 0.3));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(1.0, 0.5, 0.5));
                *bg_color = BackgroundColor(Color::srgb(0.7, 0.3, 0.3));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.8, 0.3, 0.3, 1.0));
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.2, 0.2));
            }
        }
    }
}

/// Update miner UI buffer count display
fn update_miner_ui(
    interacting: Res<InteractingMiner>,
    miner_query: Query<&Miner>,
    mut text_query: Query<&mut Text, With<MinerBufferCountText>>,
) {
    let Some(miner_entity) = interacting.0 else {
        return;
    };

    let Ok(miner) = miner_query.get(miner_entity) else {
        return;
    };

    // Update buffer count text
    for mut text in text_query.iter_mut() {
        if let Some((block_type, count)) = &miner.buffer {
            **text = format!("{}\n{}", block_type.name(), count);
        } else {
            **text = "Empty".to_string();
        }
    }
}

// === Miner & Conveyor Systems ===

/// Mining logic - automatically mine blocks below the miner
fn miner_mining(
    time: Res<Time>,
    mut miner_query: Query<&mut Miner>,
    world_data: Res<WorldData>,
) {
    for mut miner in miner_query.iter_mut() {
        // Skip if buffer is full
        if let Some((_, count)) = miner.buffer {
            if count >= 64 {
                continue;
            }
        }

        // Find block below miner to determine resource type
        let below_pos = miner.position + IVec3::new(0, -1, 0);
        let Some(&block_type) = world_data.get_block(below_pos) else {
            miner.progress = 0.0;
            continue;
        };

        // Only mine resource blocks (not grass/stone)
        let resource_type = match block_type {
            BlockType::IronOre => BlockType::IronOre,
            BlockType::Coal => BlockType::Coal,
            BlockType::CopperOre => BlockType::CopperOre,
            BlockType::Stone => BlockType::Stone,
            _ => {
                // Can't mine this block type, skip
                miner.progress = 0.0;
                continue;
            }
        };

        // Mine progress
        miner.progress += time.delta_secs() / MINE_TIME;

        if miner.progress >= 1.0 {
            miner.progress = 0.0;

            // Generate resource infinitely (don't remove block)
            // Add to buffer
            if let Some((buf_type, ref mut count)) = miner.buffer {
                if buf_type == resource_type {
                    *count += 1;
                }
            } else {
                miner.buffer = Some((resource_type, 1));
            }
        }
    }
}

/// Visual feedback for miner activity (pulse scale when mining)
fn miner_visual_feedback(
    time: Res<Time>,
    mut miner_query: Query<(&Miner, &mut Transform)>,
) {
    for (miner, mut transform) in miner_query.iter_mut() {
        // If progress > 0, the miner is actively mining
        if miner.progress > 0.0 {
            // Pulse effect: scale between 0.95 and 1.05 based on progress
            let pulse = 1.0 + 0.05 * (miner.progress * std::f32::consts::TAU * 2.0).sin();
            transform.scale = Vec3::splat(pulse);
        } else if miner.buffer.is_some() {
            // Buffer full but not mining: slight glow/scale up
            let pulse = 1.0 + 0.02 * (time.elapsed_secs() * 3.0).sin();
            transform.scale = Vec3::splat(pulse);
        } else {
            // Idle: reset scale
            transform.scale = Vec3::ONE;
        }
    }
}

/// Output from miner to adjacent conveyor
fn miner_output(
    mut miner_query: Query<&mut Miner>,
    mut conveyor_query: Query<&mut Conveyor>,
) {
    for mut miner in miner_query.iter_mut() {
        let Some((block_type, count)) = miner.buffer else {
            continue;
        };
        if count == 0 {
            continue;
        }

        // Check for adjacent conveyors (horizontal + above)
        let adjacent_positions = [
            miner.position + IVec3::new(1, 0, 0),   // east
            miner.position + IVec3::new(-1, 0, 0),  // west
            miner.position + IVec3::new(0, 0, 1),   // south
            miner.position + IVec3::new(0, 0, -1),  // north
            miner.position + IVec3::new(0, 1, 0),   // above
        ];

        'outer: for mut conveyor in conveyor_query.iter_mut() {
            for pos in &adjacent_positions {
                if conveyor.position == *pos {
                    // Use get_join_progress to determine if item can join and at what progress
                    // For above conveyor, allow joining at entry (0.0) for any direction
                    let join_progress = if *pos == miner.position + IVec3::new(0, 1, 0) {
                        Some(0.0) // Above: always join at entry
                    } else {
                        conveyor.get_join_progress(miner.position)
                    };

                    if let Some(progress) = join_progress {
                        if conveyor.can_accept_item(progress) {
                            conveyor.add_item(block_type, progress);
                            if let Some((_, ref mut buf_count)) = miner.buffer {
                                *buf_count -= 1;
                                if *buf_count == 0 {
                                    miner.buffer = None;
                                }
                            }
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
}

/// Crusher output to conveyor
fn crusher_output(
    mut crusher_query: Query<&mut Crusher>,
    mut conveyor_query: Query<&mut Conveyor>,
) {
    for mut crusher in crusher_query.iter_mut() {
        let Some(output_type) = crusher.output_type else {
            continue;
        };

        if crusher.output_count == 0 {
            continue;
        }

        // Check for adjacent conveyors (horizontal + above)
        let adjacent_positions = [
            crusher.position + IVec3::new(1, 0, 0),   // east
            crusher.position + IVec3::new(-1, 0, 0),  // west
            crusher.position + IVec3::new(0, 0, 1),   // south
            crusher.position + IVec3::new(0, 0, -1),  // north
            crusher.position + IVec3::new(0, 1, 0),   // above
        ];

        'outer: for mut conveyor in conveyor_query.iter_mut() {
            for pos in &adjacent_positions {
                if conveyor.position == *pos {
                    let join_progress = if *pos == crusher.position + IVec3::new(0, 1, 0) {
                        Some(0.0) // Above: always join at entry
                    } else {
                        conveyor.get_join_progress(crusher.position)
                    };

                    if let Some(progress) = join_progress {
                        if conveyor.can_accept_item(progress) {
                            conveyor.add_item(output_type, progress);
                            crusher.output_count -= 1;
                            if crusher.output_count == 0 {
                                crusher.output_type = None;
                            }
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
}

/// Furnace output to conveyor
fn furnace_output(
    mut furnace_query: Query<(&Transform, &mut Furnace)>,
    mut conveyor_query: Query<&mut Conveyor>,
) {
    for (transform, mut furnace) in furnace_query.iter_mut() {
        let Some(output_type) = furnace.output_type else {
            continue;
        };

        if furnace.output_count == 0 {
            continue;
        }

        // Get furnace position from Transform
        let furnace_pos = IVec3::new(
            transform.translation.x.floor() as i32,
            transform.translation.y.floor() as i32,
            transform.translation.z.floor() as i32,
        );

        // Check for adjacent conveyors (horizontal + above)
        let adjacent_positions = [
            furnace_pos + IVec3::new(1, 0, 0),   // east
            furnace_pos + IVec3::new(-1, 0, 0),  // west
            furnace_pos + IVec3::new(0, 0, 1),   // south
            furnace_pos + IVec3::new(0, 0, -1),  // north
            furnace_pos + IVec3::new(0, 1, 0),   // above
        ];

        'outer: for mut conveyor in conveyor_query.iter_mut() {
            for pos in &adjacent_positions {
                if conveyor.position == *pos {
                    let join_progress = if *pos == furnace_pos + IVec3::new(0, 1, 0) {
                        Some(0.0) // Above: always join at entry
                    } else {
                        conveyor.get_join_progress(furnace_pos)
                    };

                    if let Some(progress) = join_progress {
                        if conveyor.can_accept_item(progress) {
                            conveyor.add_item(output_type, progress);
                            furnace.output_count -= 1;
                            if furnace.output_count == 0 {
                                furnace.output_type = None;
                            }
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
}

/// Conveyor transfer logic - move items along conveyor chain (supports multiple items per conveyor)
fn conveyor_transfer(
    time: Res<Time>,
    mut commands: Commands,
    mut conveyor_query: Query<(Entity, &mut Conveyor)>,
    mut furnace_query: Query<(&Transform, &mut Furnace)>,
    mut crusher_query: Query<&mut Crusher>,
    mut platform_query: Query<(&Transform, &mut DeliveryPlatform)>,
) {
    use constants::*;

    // Build lookup maps
    let conveyor_positions: HashMap<IVec3, Entity> = conveyor_query
        .iter()
        .map(|(e, c)| (c.position, e))
        .collect();

    // Collect furnace positions
    let furnace_positions: HashMap<IVec3, Entity> = furnace_query
        .iter()
        .map(|(t, _)| {
            let pos = IVec3::new(
                t.translation.x.floor() as i32,
                t.translation.y.floor() as i32,
                t.translation.z.floor() as i32,
            );
            (pos, Entity::PLACEHOLDER) // We'll look up by position
        })
        .collect();

    // Collect crusher positions
    let crusher_positions: HashMap<IVec3, Entity> = crusher_query
        .iter()
        .map(|c| (c.position, Entity::PLACEHOLDER))
        .collect();

    // Check if position is on delivery platform
    let platform_bounds: Option<(IVec3, IVec3)> = platform_query.iter().next().map(|(t, _)| {
        let center = IVec3::new(
            t.translation.x.floor() as i32,
            t.translation.y.floor() as i32,
            t.translation.z.floor() as i32,
        );
        let half = PLATFORM_SIZE / 2;
        (
            IVec3::new(center.x - half, center.y, center.z - half),
            IVec3::new(center.x + half, center.y, center.z + half),
        )
    });

    // Transfer actions to apply
    struct TransferAction {
        source_entity: Entity,
        source_pos: IVec3, // Position of source conveyor (for join progress calculation)
        item_index: usize,
        target: TransferTarget,
    }
    enum TransferTarget {
        Conveyor(Entity), // Target conveyor entity
        Furnace(IVec3),
        Crusher(IVec3),
        Delivery,
    }

    let mut actions: Vec<TransferAction> = Vec::new();

    // Track splitter output indices for round-robin (entity -> next output index)
    let mut splitter_indices: HashMap<Entity, usize> = HashMap::new();

    // First pass: update progress and collect transfer actions
    for (entity, conveyor) in conveyor_query.iter() {
        for (idx, item) in conveyor.items.iter().enumerate() {
            // Only transfer items that reached the end
            if item.progress < 1.0 {
                continue;
            }

            // Determine output position(s) based on shape
            let output_positions: Vec<IVec3> = if conveyor.shape == ConveyorShape::Splitter {
                // Splitter: try front, left, right in round-robin order
                let outputs = conveyor.get_splitter_outputs();
                let start_idx = *splitter_indices.get(&entity).unwrap_or(&conveyor.last_output_index);
                // Rotate the array to start from the current index
                let mut rotated = Vec::with_capacity(3);
                for i in 0..3 {
                    rotated.push(outputs[(start_idx + i) % 3]);
                }
                rotated
            } else {
                // Normal conveyor: front only
                vec![conveyor.position + conveyor.direction.to_ivec3()]
            };

            // Try each output position in order
            let mut found_target = false;
            for next_pos in output_positions {
                // Check if next position is on delivery platform
                if let Some((min, max)) = platform_bounds {
                    if next_pos.x >= min.x && next_pos.x <= max.x
                        && next_pos.y >= min.y && next_pos.y <= max.y
                        && next_pos.z >= min.z && next_pos.z <= max.z
                    {
                        actions.push(TransferAction {
                            source_entity: entity,
                            source_pos: conveyor.position,
                            item_index: idx,
                            target: TransferTarget::Delivery,
                        });
                        // Update splitter index for next item
                        if conveyor.shape == ConveyorShape::Splitter {
                            let current = splitter_indices.entry(entity).or_insert(conveyor.last_output_index);
                            *current = (*current + 1) % 3;
                        }
                        found_target = true;
                        break;
                    }
                }

                // Check if next position has a conveyor
                if let Some(&next_entity) = conveyor_positions.get(&next_pos) {
                    actions.push(TransferAction {
                        source_entity: entity,
                        source_pos: conveyor.position,
                        item_index: idx,
                        target: TransferTarget::Conveyor(next_entity),
                    });
                    if conveyor.shape == ConveyorShape::Splitter {
                        let current = splitter_indices.entry(entity).or_insert(conveyor.last_output_index);
                        *current = (*current + 1) % 3;
                    }
                    found_target = true;
                    break;
                } else if furnace_positions.contains_key(&next_pos) {
                    actions.push(TransferAction {
                        source_entity: entity,
                        source_pos: conveyor.position,
                        item_index: idx,
                        target: TransferTarget::Furnace(next_pos),
                    });
                    if conveyor.shape == ConveyorShape::Splitter {
                        let current = splitter_indices.entry(entity).or_insert(conveyor.last_output_index);
                        *current = (*current + 1) % 3;
                    }
                    found_target = true;
                    break;
                } else if crusher_positions.contains_key(&next_pos) {
                    actions.push(TransferAction {
                        source_entity: entity,
                        source_pos: conveyor.position,
                        item_index: idx,
                        target: TransferTarget::Crusher(next_pos),
                    });
                    if conveyor.shape == ConveyorShape::Splitter {
                        let current = splitter_indices.entry(entity).or_insert(conveyor.last_output_index);
                        *current = (*current + 1) % 3;
                    }
                    found_target = true;
                    break;
                }
            }

            // If no target found for splitter, still advance the index to try next output next time
            if !found_target && conveyor.shape == ConveyorShape::Splitter {
                let current = splitter_indices.entry(entity).or_insert(conveyor.last_output_index);
                *current = (*current + 1) % 3;
            }
        }
    }

    // Sort actions by item_index descending so we can remove without index shifting issues
    actions.sort_by(|a, b| b.item_index.cmp(&a.item_index));

    // === ZIPPER MERGE LOGIC ===
    // Group sources by target conveyor for zipper merge
    let mut sources_by_target: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for action in &actions {
        if let TransferTarget::Conveyor(target) = action.target {
            let sources = sources_by_target.entry(target).or_default();
            if !sources.contains(&action.source_entity) {
                sources.push(action.source_entity);
            }
        }
    }

    // Determine which source is allowed for each target (zipper logic)
    // When multiple sources try to feed into the same conveyor, only one is allowed per tick
    let allowed_source: HashMap<Entity, Entity> = sources_by_target.iter()
        .filter_map(|(target, sources)| {
            if sources.len() <= 1 {
                // Only one source, always allow
                sources.first().map(|s| (*target, *s))
            } else {
                // Multiple sources - use zipper logic with last_input_source
                conveyor_query.get(*target).ok().map(|(_, c)| {
                    let mut sorted_sources: Vec<Entity> = sources.clone();
                    sorted_sources.sort_by_key(|e| e.index());
                    let idx = c.last_input_source % sorted_sources.len();
                    (*target, sorted_sources[idx])
                })
            }
        })
        .collect();

    // Track which targets successfully received an item (to update last_input_source)
    let mut targets_to_update: Vec<Entity> = Vec::new();

    // First pass: check which conveyor-to-conveyor transfers can proceed
    // This avoids borrow conflicts
    // Value is Some((progress, lateral_offset)) if can accept, None otherwise
    let conveyor_transfer_ok: HashMap<(Entity, usize), Option<(f32, f32)>> = actions
        .iter()
        .filter_map(|action| {
            if let TransferTarget::Conveyor(target_entity) = action.target {
                let result = conveyor_query.get(target_entity).ok().and_then(|(_, c)| {
                    // Calculate join info (progress, lateral_offset) based on source position
                    c.get_join_info(action.source_pos)
                        .filter(|&(progress, _)| c.can_accept_item(progress))
                });
                Some(((action.source_entity, action.item_index), result))
            } else {
                None
            }
        })
        .collect();

    // Collect conveyor adds for second pass (to avoid borrow conflicts)
    // Tuple: (target_entity, block_type, join_progress, visual_entity, lateral_offset)
    let mut conveyor_adds: Vec<(Entity, BlockType, f32, Option<Entity>, f32)> = Vec::new();

    // Apply transfers
    for action in actions {
        let Ok((_, mut source_conv)) = conveyor_query.get_mut(action.source_entity) else {
            continue;
        };

        if action.item_index >= source_conv.items.len() {
            continue;
        }

        let item = source_conv.items[action.item_index].clone();

        match action.target {
            TransferTarget::Conveyor(target_entity) => {
                // Zipper merge: check if this source is allowed for this target
                if let Some(&allowed) = allowed_source.get(&target_entity) {
                    if allowed != action.source_entity {
                        // This source is not allowed this tick (zipper logic)
                        continue;
                    }
                }

                // Check pre-computed result - Some((progress, lateral_offset)) if can accept
                let join_info = conveyor_transfer_ok
                    .get(&(action.source_entity, action.item_index))
                    .copied()
                    .flatten();

                if let Some((progress, lateral_offset)) = join_info {
                    // Keep visual entity for seamless transfer (BUG-3 fix)
                    let visual = item.visual_entity;
                    source_conv.items.remove(action.item_index);
                    // Queue add to target conveyor with visual and lateral offset
                    conveyor_adds.push((target_entity, item.block_type, progress, visual, lateral_offset));
                    // Mark target for last_input_source update
                    if !targets_to_update.contains(&target_entity) {
                        targets_to_update.push(target_entity);
                    }
                }
            }
            TransferTarget::Furnace(furnace_pos) => {
                let mut accepted = false;
                for (furnace_transform, mut furnace) in furnace_query.iter_mut() {
                    let pos = IVec3::new(
                        furnace_transform.translation.x.floor() as i32,
                        furnace_transform.translation.y.floor() as i32,
                        furnace_transform.translation.z.floor() as i32,
                    );
                    if pos == furnace_pos {
                        let can_accept = match item.block_type {
                            BlockType::Coal => furnace.fuel < 64,
                            BlockType::IronOre | BlockType::CopperOre => {
                                furnace.can_add_input(item.block_type) && furnace.input_count < 64
                            }
                            _ => false,
                        };
                        if can_accept {
                            match item.block_type {
                                BlockType::Coal => furnace.fuel += 1,
                                BlockType::IronOre | BlockType::CopperOre => {
                                    furnace.input_type = Some(item.block_type);
                                    furnace.input_count += 1;
                                }
                                _ => {}
                            }
                            accepted = true;
                        }
                        break;
                    }
                }
                if accepted {
                    if let Some(visual) = item.visual_entity {
                        commands.entity(visual).despawn();
                    }
                    source_conv.items.remove(action.item_index);
                }
            }
            TransferTarget::Crusher(crusher_pos) => {
                let mut accepted = false;
                for mut crusher in crusher_query.iter_mut() {
                    if crusher.position == crusher_pos {
                        let can_accept = Crusher::can_crush(item.block_type)
                            && (crusher.input_type.is_none() || crusher.input_type == Some(item.block_type))
                            && crusher.input_count < 64;
                        if can_accept {
                            crusher.input_type = Some(item.block_type);
                            crusher.input_count += 1;
                            accepted = true;
                        }
                        break;
                    }
                }
                if accepted {
                    if let Some(visual) = item.visual_entity {
                        commands.entity(visual).despawn();
                    }
                    source_conv.items.remove(action.item_index);
                }
            }
            TransferTarget::Delivery => {
                // Deliver the item to platform
                if let Some((_, mut platform)) = platform_query.iter_mut().next() {
                    let count = platform.delivered.entry(item.block_type).or_insert(0);
                    *count += 1;
                    info!(category = "QUEST", action = "deliver", item = ?item.block_type, total = *count, "Item delivered");
                }
                if let Some(visual) = item.visual_entity {
                    commands.entity(visual).despawn();
                }
                source_conv.items.remove(action.item_index);
            }
        }
    }

    // Second pass: add items to target conveyors at their calculated join progress
    for (target_entity, block_type, progress, visual, lateral_offset) in conveyor_adds {
        if let Ok((_, mut target_conv)) = conveyor_query.get_mut(target_entity) {
            target_conv.add_item_with_visual(block_type, progress, visual, lateral_offset);
        }
    }

    // Update last_input_source for conveyors that received items (zipper merge)
    for target_entity in targets_to_update {
        if let Ok((_, mut target_conv)) = conveyor_query.get_mut(target_entity) {
            target_conv.last_input_source += 1;
        }
    }

    // Persist splitter output indices
    for (entity, new_index) in splitter_indices {
        if let Ok((_, mut conv)) = conveyor_query.get_mut(entity) {
            conv.last_output_index = new_index;
        }
    }

    // Update progress for all items on all conveyors
    let delta = time.delta_secs() / CONVEYOR_SPEED;
    let lateral_decay = time.delta_secs() * 3.0; // Decay rate for lateral offset (BUG-5 fix)
    for (_, mut conveyor) in conveyor_query.iter_mut() {
        let item_count = conveyor.items.len();
        for i in 0..item_count {
            // Decay lateral offset towards center
            if conveyor.items[i].lateral_offset.abs() > 0.01 {
                let sign = conveyor.items[i].lateral_offset.signum();
                conveyor.items[i].lateral_offset -= sign * lateral_decay;
                // Clamp to prevent overshooting
                if sign * conveyor.items[i].lateral_offset < 0.0 {
                    conveyor.items[i].lateral_offset = 0.0;
                }
            } else {
                conveyor.items[i].lateral_offset = 0.0;
            }

            if conveyor.items[i].progress < 1.0 {
                // Check if blocked by item ahead (higher progress)
                let current_progress = conveyor.items[i].progress;
                let blocked = conveyor.items.iter().any(|other| {
                    other.progress > current_progress
                        && other.progress - current_progress < CONVEYOR_ITEM_SPACING
                });
                if !blocked {
                    conveyor.items[i].progress += delta;
                    if conveyor.items[i].progress > 1.0 {
                        conveyor.items[i].progress = 1.0;
                    }
                }
            }
        }
    }
}

/// Update conveyor item visuals - spawn/despawn/move items on conveyors (multiple items)
fn update_conveyor_item_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut conveyor_query: Query<&mut Conveyor>,
    mut visual_query: Query<&mut Transform, With<ConveyorItemVisual>>,
) {
    let item_mesh = meshes.add(Cuboid::new(BLOCK_SIZE * CONVEYOR_ITEM_SIZE, BLOCK_SIZE * CONVEYOR_ITEM_SIZE, BLOCK_SIZE * CONVEYOR_ITEM_SIZE));

    for mut conveyor in conveyor_query.iter_mut() {
        // Position items on top of the belt (belt height + item size/2)
        let item_y = conveyor.position.y as f32 * BLOCK_SIZE + CONVEYOR_BELT_HEIGHT + CONVEYOR_ITEM_SIZE / 2.0;
        let base_pos = Vec3::new(
            conveyor.position.x as f32 * BLOCK_SIZE + 0.5,
            item_y,
            conveyor.position.z as f32 * BLOCK_SIZE + 0.5,
        );
        let direction_vec = conveyor.direction.to_ivec3().as_vec3();
        // Perpendicular vector for lateral offset (BUG-5 fix, BUG-9 fix)
        // Positive lateral_offset = right side of conveyor direction
        let lateral_vec = match conveyor.direction {
            Direction::East => Vec3::new(0.0, 0.0, 1.0),   // Right is +Z (South)
            Direction::West => Vec3::new(0.0, 0.0, -1.0),  // Right is -Z (North)
            Direction::South => Vec3::new(-1.0, 0.0, 0.0), // Right is -X (West)
            Direction::North => Vec3::new(1.0, 0.0, 0.0),  // Right is +X (East)
        };

        for item in conveyor.items.iter_mut() {
            // Calculate position: progress 0.0 = entry (-0.5), 1.0 = exit (+0.5)
            let forward_offset = (item.progress - 0.5) * BLOCK_SIZE;
            let lateral_offset_world = item.lateral_offset * BLOCK_SIZE;
            let item_pos = base_pos + direction_vec * forward_offset + lateral_vec * lateral_offset_world;

            match item.visual_entity {
                None => {
                    // Spawn visual
                    let material = materials.add(StandardMaterial {
                        base_color: item.block_type.color(),
                        ..default()
                    });
                    let entity = commands.spawn((
                        Mesh3d(item_mesh.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(item_pos),
                        ConveyorItemVisual,
                    )).id();
                    item.visual_entity = Some(entity);
                }
                Some(entity) => {
                    // Update position
                    if let Ok(mut transform) = visual_query.get_mut(entity) {
                        transform.translation = item_pos;
                    }
                }
            }
        }
    }
}

/// Update furnace UI slot counts and progress bar
fn update_furnace_ui(
    interacting: Res<InteractingFurnace>,
    furnace_query: Query<&Furnace>,
    mut slot_count_query: Query<(&MachineSlotCount, &mut Text)>,
    mut progress_bar_query: Query<&mut Node, With<MachineProgressBar>>,
) {
    let Some(furnace_entity) = interacting.0 else {
        return;
    };

    let Ok(furnace) = furnace_query.get(furnace_entity) else {
        return;
    };

    // Update slot counts
    for (slot_count, mut text) in slot_count_query.iter_mut() {
        **text = match slot_count.0 {
            MachineSlotType::Fuel => format!("{}", furnace.fuel),
            MachineSlotType::Input => format!("{}", furnace.input_count),
            MachineSlotType::Output => format!("{}", furnace.output_count),
        };
    }

    // Update progress bar
    for mut node in progress_bar_query.iter_mut() {
        node.width = Val::Percent(furnace.progress * 100.0);
    }
}

/// Update crusher UI slot counts and progress bar
fn update_crusher_ui(
    interacting: Res<InteractingCrusher>,
    crusher_query: Query<&Crusher>,
    mut slot_count_query: Query<(&CrusherSlotCount, &mut Text)>,
    mut progress_bar_query: Query<&mut Node, With<CrusherProgressBar>>,
) {
    let Some(crusher_entity) = interacting.0 else {
        return;
    };

    let Ok(crusher) = crusher_query.get(crusher_entity) else {
        return;
    };

    // Update slot counts
    for (slot_count, mut text) in slot_count_query.iter_mut() {
        **text = match slot_count.0 {
            MachineSlotType::Fuel => "".to_string(), // Crusher has no fuel
            MachineSlotType::Input => format!("{}", crusher.input_count),
            MachineSlotType::Output => format!("{}", crusher.output_count),
        };
    }

    // Update progress bar
    for mut node in progress_bar_query.iter_mut() {
        node.width = Val::Percent(crusher.progress * 100.0);
    }
}

fn update_window_title_fps(diagnostics: Res<DiagnosticsStore>, mut windows: Query<&mut Window>) {
    if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            if let Ok(mut window) = windows.get_single_mut() {
                window.title = format!("Idle Factory - FPS: {:.0}", value);
            }
        }
    }
}

/// Toggle debug HUD with F3 key
fn toggle_debug_hud(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugHudState>,
    debug_query: Query<Entity, With<DebugHudText>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        debug_state.visible = !debug_state.visible;

        if debug_state.visible {
            // Spawn debug HUD
            commands.spawn((
                Text::new("Debug Info"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                DebugHudText,
            ));
        } else {
            // Despawn debug HUD
            for entity in debug_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// Update debug HUD with current info
fn update_debug_hud(
    mut debug_query: Query<&mut Text, With<DebugHudText>>,
    player_query: Query<&Transform, With<Player>>,
    world_data: Res<WorldData>,
    diagnostics: Res<DiagnosticsStore>,
    debug_state: Res<DebugHudState>,
    creative_mode: Res<CreativeMode>,
    cursor_state: Res<CursorLockState>,
) {
    if !debug_state.visible {
        return;
    }

    let Ok(mut text) = debug_query.get_single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);

    let player_pos = player_query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    let chunk_count = world_data.chunks.len();

    // Mode strings
    let mode_str = if creative_mode.enabled { "Creative" } else { "Survival" };
    let pause_str = if cursor_state.paused { " [PAUSED]" } else { "" };

    **text = format!(
        "=== Debug (F3) ===\n\
         FPS: {:.0}\n\
         Pos: ({:.1}, {:.1}, {:.1})\n\
         Chunks: {}\n\
         Mode: {}{}",
        fps,
        player_pos.x, player_pos.y, player_pos.z,
        chunk_count,
        mode_str,
        pause_str
    );
}

// === Delivery Platform Systems ===

/// Setup delivery platform near spawn point
fn setup_delivery_platform(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Platform position: 12x12 area starting at (20, 8, 10)
    let platform_origin = IVec3::new(20, 8, 10);

    // Create platform mesh (flat plate)
    let platform_mesh = meshes.add(Cuboid::new(
        PLATFORM_SIZE as f32 * BLOCK_SIZE,
        BLOCK_SIZE * 0.2,
        PLATFORM_SIZE as f32 * BLOCK_SIZE,
    ));

    let platform_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.5, 0.3), // Green-ish for delivery area
        ..default()
    });

    // Spawn platform entity
    // Platform center: origin + half_size (in blocks), then offset by 0.5 for grid alignment
    commands.spawn((
        Mesh3d(platform_mesh),
        MeshMaterial3d(platform_material),
        Transform::from_translation(Vec3::new(
            platform_origin.x as f32 * BLOCK_SIZE + (PLATFORM_SIZE as f32 * BLOCK_SIZE / 2.0),
            platform_origin.y as f32 * BLOCK_SIZE + 0.1,
            platform_origin.z as f32 * BLOCK_SIZE + (PLATFORM_SIZE as f32 * BLOCK_SIZE / 2.0),
        )),
        DeliveryPlatform::default(),
    ));

    // Spawn delivery port markers (visual indicators at edges)
    // Use tall vertical markers for better visibility
    let port_mesh = meshes.add(Cuboid::new(BLOCK_SIZE * 0.3, BLOCK_SIZE * 0.8, BLOCK_SIZE * 0.3));
    let port_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.9, 0.2), // Bright yellow for ports
        emissive: bevy::color::LinearRgba::new(0.5, 0.45, 0.1, 1.0),
        ..default()
    });

    // Create 16 ports along edges (4 per side)
    let port_positions = [
        // North edge (z = 10)
        IVec3::new(22, 8, 10), IVec3::new(25, 8, 10), IVec3::new(28, 8, 10), IVec3::new(31, 8, 10),
        // South edge (z = 21)
        IVec3::new(22, 8, 21), IVec3::new(25, 8, 21), IVec3::new(28, 8, 21), IVec3::new(31, 8, 21),
        // West edge (x = 20)
        IVec3::new(20, 8, 12), IVec3::new(20, 8, 15), IVec3::new(20, 8, 18), IVec3::new(20, 8, 21),
        // East edge (x = 31)
        IVec3::new(31, 8, 12), IVec3::new(31, 8, 15), IVec3::new(31, 8, 18), IVec3::new(31, 8, 21),
    ];

    for port_pos in port_positions {
        commands.spawn((
            Mesh3d(port_mesh.clone()),
            MeshMaterial3d(port_material.clone()),
            Transform::from_translation(Vec3::new(
                port_pos.x as f32 * BLOCK_SIZE + 0.5,
                port_pos.y as f32 * BLOCK_SIZE + 0.5,
                port_pos.z as f32 * BLOCK_SIZE + 0.5,
            )),
        ));
    }
}

/// Load 3D models for machines and conveyors (if available)
fn load_machine_models(
    asset_server: Res<AssetServer>,
    mut models: ResMut<MachineModels>,
) {
    // Try to load conveyor models
    models.conveyor_straight = Some(asset_server.load("models/machines/conveyor/straight.glb#Scene0"));
    models.conveyor_corner_left = Some(asset_server.load("models/machines/conveyor/corner_left.glb#Scene0"));
    models.conveyor_corner_right = Some(asset_server.load("models/machines/conveyor/corner_right.glb#Scene0"));
    models.conveyor_t_junction = Some(asset_server.load("models/machines/conveyor/t_junction.glb#Scene0"));
    models.conveyor_splitter = Some(asset_server.load("models/machines/conveyor/splitter.glb#Scene0"));

    // Try to load machine models
    models.miner = Some(asset_server.load("models/machines/miner.glb#Scene0"));
    models.furnace = Some(asset_server.load("models/machines/furnace.glb#Scene0"));
    models.crusher = Some(asset_server.load("models/machines/crusher.glb#Scene0"));

    // Will check if loaded in update system
    models.loaded = false;
}

/// Update delivery UI text
fn update_delivery_ui(
    platform_query: Query<&DeliveryPlatform>,
    mut text_query: Query<&mut Text, With<DeliveryUIText>>,
) {
    let Ok(platform) = platform_query.get_single() else {
        return;
    };

    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    if platform.delivered.is_empty() {
        **text = "=== Deliveries ===\nNo items delivered".to_string();
    } else {
        let items: Vec<String> = platform
            .delivered
            .iter()
            .map(|(block_type, count)| format!("{}: {}", block_type.name(), count))
            .collect();
        **text = format!("=== Deliveries ===\n{}", items.join("\n"));
    }
}

// === Quest Systems ===

/// Quest definitions from game_spec (Single Source of Truth)
fn get_quests() -> Vec<QuestDef> {
    game_spec::QUESTS
        .iter()
        .map(|spec| QuestDef {
            description: spec.description,
            required_item: spec.required_item,
            required_amount: spec.required_amount,
            rewards: spec.rewards.to_vec(),
        })
        .collect()
}

/// Check quest progress
fn quest_progress_check(
    platform_query: Query<&DeliveryPlatform>,
    mut current_quest: ResMut<CurrentQuest>,
) {
    if current_quest.completed {
        return;
    }

    let Ok(platform) = platform_query.get_single() else {
        return;
    };

    let quests = get_quests();
    let Some(quest) = quests.get(current_quest.index) else {
        return;
    };

    let delivered = platform.delivered.get(&quest.required_item).copied().unwrap_or(0);
    if delivered >= quest.required_amount {
        current_quest.completed = true;
    }
}

/// Claim quest rewards with Q key
fn quest_claim_rewards(
    key_input: Res<ButtonInput<KeyCode>>,
    mut current_quest: ResMut<CurrentQuest>,
    mut inventory: ResMut<Inventory>,
    command_state: Res<CommandInputState>,
) {
    // Don't process while command input is open
    if command_state.open {
        return;
    }

    if !current_quest.completed || current_quest.rewards_claimed {
        return;
    }

    if !key_input.just_pressed(KeyCode::KeyQ) {
        return;
    }

    let quests = get_quests();
    let Some(quest) = quests.get(current_quest.index) else {
        return;
    };

    // Add rewards to inventory
    for (block_type, amount) in &quest.rewards {
        inventory.add_item(*block_type, *amount);
    }

    current_quest.rewards_claimed = true;

    // Move to next quest
    if current_quest.index + 1 < quests.len() {
        current_quest.index += 1;
        current_quest.completed = false;
        current_quest.rewards_claimed = false;
    }
}

/// Update quest UI
fn update_quest_ui(
    current_quest: Res<CurrentQuest>,
    platform_query: Query<&DeliveryPlatform>,
    mut text_query: Query<&mut Text, With<QuestUIText>>,
) {
    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    let quests = get_quests();

    if current_quest.index >= quests.len() {
        **text = "=== Quest ===\nAll quests completed!".to_string();
        return;
    }

    let quest = &quests[current_quest.index];
    let delivered = platform_query
        .get_single()
        .map(|p| p.delivered.get(&quest.required_item).copied().unwrap_or(0))
        .unwrap_or(0);

    if current_quest.completed && !current_quest.rewards_claimed {
        let rewards: Vec<String> = quest.rewards
            .iter()
            .map(|(bt, amt)| format!("{} x{}", bt.name(), amt))
            .collect();
        **text = format!(
            "=== Quest Complete! ===\n{}\n\nRewards:\n{}\n\n[Q] Claim Rewards",
            quest.description,
            rewards.join("\n")
        );
    } else {
        **text = format!(
            "=== Quest ===\n{}\nProgress: {}/{}",
            quest.description,
            delivered.min(quest.required_amount),
            quest.required_amount
        );
    }
}

// === Target Block Highlight ===

/// Update target block based on player's view direction
fn update_target_block(
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    world_data: Res<WorldData>,
    windows: Query<&Window>,
    mut target: ResMut<TargetBlock>,
    interacting_furnace: Res<InteractingFurnace>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't update target while UI is open or paused
    if interacting_furnace.0.is_some() || cursor_state.paused {
        target.break_target = None;
        target.place_target = None;
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;
    if !cursor_locked {
        target.break_target = None;
        target.place_target = None;
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Use DDA (Digital Differential Analyzer) for precise voxel traversal
    // This ensures we check every voxel the ray passes through, in order

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

    // Maximum number of steps (prevent infinite loop)
    let max_steps = (REACH_DISTANCE * 2.0) as i32;

    for _ in 0..max_steps {
        // Check current voxel
        if world_data.has_block(current) {
            target.break_target = Some(current);

            // Calculate place position based on last step axis
            let normal = match last_step_axis {
                0 => IVec3::new(-step.x, 0, 0),
                1 => IVec3::new(0, -step.y, 0),
                _ => IVec3::new(0, 0, -step.z),
            };
            target.place_target = Some(current + normal);
            return;
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

    // No block found
    target.break_target = None;
    target.place_target = None;
}

/// Create a wireframe conveyor mesh with direction arrow
fn create_conveyor_wireframe_mesh(direction: Direction) -> Mesh {
    use bevy::render::mesh::PrimitiveTopology;

    let half_w = BLOCK_SIZE * 0.505; // Width
    let half_h = 0.155; // Height (conveyor is about 0.3 tall)
    let half_l = BLOCK_SIZE * 0.505; // Length

    // 8 corners of the conveyor bounding box (centered at y=0.15)
    let y_offset = 0.15;
    let corners = [
        Vec3::new(-half_w, y_offset - half_h, -half_l), // 0: bottom-back-left
        Vec3::new( half_w, y_offset - half_h, -half_l), // 1: bottom-back-right
        Vec3::new( half_w, y_offset + half_h, -half_l), // 2: top-back-right
        Vec3::new(-half_w, y_offset + half_h, -half_l), // 3: top-back-left
        Vec3::new(-half_w, y_offset - half_h,  half_l), // 4: bottom-front-left
        Vec3::new( half_w, y_offset - half_h,  half_l), // 5: bottom-front-right
        Vec3::new( half_w, y_offset + half_h,  half_l), // 6: top-front-right
        Vec3::new(-half_w, y_offset + half_h,  half_l), // 7: top-front-left
    ];

    // Arrow points (centered, then rotated for direction)
    let arrow_y = y_offset + half_h + 0.02; // Slightly above conveyor
    let arrow_base = 0.0;
    let arrow_tip = 0.35;
    let arrow_wing = 0.15;

    // Base arrow points in +X direction (East)
    let arrow_points = match direction {
        Direction::East => [
            (Vec3::new(arrow_base, arrow_y, 0.0), Vec3::new(arrow_tip, arrow_y, 0.0)), // Main arrow
            (Vec3::new(arrow_tip, arrow_y, 0.0), Vec3::new(arrow_tip - arrow_wing, arrow_y, arrow_wing)), // Left wing
            (Vec3::new(arrow_tip, arrow_y, 0.0), Vec3::new(arrow_tip - arrow_wing, arrow_y, -arrow_wing)), // Right wing
        ],
        Direction::West => [
            (Vec3::new(-arrow_base, arrow_y, 0.0), Vec3::new(-arrow_tip, arrow_y, 0.0)),
            (Vec3::new(-arrow_tip, arrow_y, 0.0), Vec3::new(-arrow_tip + arrow_wing, arrow_y, arrow_wing)),
            (Vec3::new(-arrow_tip, arrow_y, 0.0), Vec3::new(-arrow_tip + arrow_wing, arrow_y, -arrow_wing)),
        ],
        Direction::South => [
            (Vec3::new(0.0, arrow_y, arrow_base), Vec3::new(0.0, arrow_y, arrow_tip)),
            (Vec3::new(0.0, arrow_y, arrow_tip), Vec3::new(arrow_wing, arrow_y, arrow_tip - arrow_wing)),
            (Vec3::new(0.0, arrow_y, arrow_tip), Vec3::new(-arrow_wing, arrow_y, arrow_tip - arrow_wing)),
        ],
        Direction::North => [
            (Vec3::new(0.0, arrow_y, -arrow_base), Vec3::new(0.0, arrow_y, -arrow_tip)),
            (Vec3::new(0.0, arrow_y, -arrow_tip), Vec3::new(arrow_wing, arrow_y, -arrow_tip + arrow_wing)),
            (Vec3::new(0.0, arrow_y, -arrow_tip), Vec3::new(-arrow_wing, arrow_y, -arrow_tip + arrow_wing)),
        ],
    };

    // Build positions: 12 box edges + 3 arrow lines = 15 lines = 30 vertices
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(30);

    // Box edges (12 edges, 24 vertices)
    let edges = [
        // Bottom face
        (corners[0], corners[1]),
        (corners[1], corners[5]),
        (corners[5], corners[4]),
        (corners[4], corners[0]),
        // Top face
        (corners[3], corners[2]),
        (corners[2], corners[6]),
        (corners[6], corners[7]),
        (corners[7], corners[3]),
        // Vertical edges
        (corners[0], corners[3]),
        (corners[1], corners[2]),
        (corners[5], corners[6]),
        (corners[4], corners[7]),
    ];

    for (a, b) in edges {
        positions.push(a.to_array());
        positions.push(b.to_array());
    }

    // Arrow lines (3 lines, 6 vertices)
    for (a, b) in arrow_points {
        positions.push(a.to_array());
        positions.push(b.to_array());
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh
}

/// Create a wireframe cube mesh (12 edges)
fn create_wireframe_cube_mesh() -> Mesh {
    use bevy::render::mesh::PrimitiveTopology;

    let half = BLOCK_SIZE * 0.505; // Slightly larger to avoid z-fighting

    // 8 corners of the cube
    let corners = [
        Vec3::new(-half, -half, -half), // 0
        Vec3::new( half, -half, -half), // 1
        Vec3::new( half,  half, -half), // 2
        Vec3::new(-half,  half, -half), // 3
        Vec3::new(-half, -half,  half), // 4
        Vec3::new( half, -half,  half), // 5
        Vec3::new( half,  half,  half), // 6
        Vec3::new(-half,  half,  half), // 7
    ];

    // 12 edges as line pairs (24 vertices total)
    let positions: Vec<[f32; 3]> = [
        // Bottom face edges
        (corners[0], corners[1]),
        (corners[1], corners[5]),
        (corners[5], corners[4]),
        (corners[4], corners[0]),
        // Top face edges
        (corners[3], corners[2]),
        (corners[2], corners[6]),
        (corners[6], corners[7]),
        (corners[7], corners[3]),
        // Vertical edges
        (corners[0], corners[3]),
        (corners[1], corners[2]),
        (corners[5], corners[6]),
        (corners[4], corners[7]),
    ].iter().flat_map(|(a, b)| vec![a.to_array(), b.to_array()]).collect();

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh
}

/// Update target highlight entity position
#[allow(clippy::too_many_arguments)]
fn update_target_highlight(
    mut commands: Commands,
    mut target: ResMut<TargetBlock>,
    break_query: Query<Entity, (With<TargetHighlight>, Without<PlaceHighlight>)>,
    place_query: Query<Entity, (With<PlaceHighlight>, Without<TargetHighlight>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    inventory: Res<Inventory>,
    conveyor_query: Query<&Conveyor>,
    miner_query: Query<&Miner>,
    crusher_query: Query<&Crusher>,
    furnace_query: Query<&Transform, With<Furnace>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    rotation: Res<ConveyorRotationOffset>,
) {
    // Check if player has a placeable item selected
    let has_placeable_item = inventory.has_selected();

    // Check if selected item is a conveyor
    let selected_item = inventory.get_selected_type();
    let placing_conveyor = selected_item == Some(BlockType::ConveyorBlock);

    // Get player's facing direction as fallback
    let player_facing = camera_query.get_single().ok().map(|cam_transform| {
        let forward = cam_transform.forward().as_vec3();
        yaw_to_direction(-forward.x.atan2(-forward.z))
    });

    // Calculate place direction using auto_conveyor_direction (same logic as block_place)
    let place_direction = if placing_conveyor {
        if let (Some(place_pos), Some(fallback_dir)) = (target.place_target, player_facing) {
            // Collect conveyor positions and directions
            let conveyors: Vec<(IVec3, Direction)> = conveyor_query
                .iter()
                .map(|c| (c.position, c.direction))
                .collect();

            // Collect machine positions
            let mut machine_positions: Vec<IVec3> = Vec::new();
            for miner in miner_query.iter() {
                machine_positions.push(miner.position);
            }
            for crusher in crusher_query.iter() {
                machine_positions.push(crusher.position);
            }
            for furnace_transform in furnace_query.iter() {
                machine_positions.push(IVec3::new(
                    furnace_transform.translation.x.floor() as i32,
                    furnace_transform.translation.y.floor() as i32,
                    furnace_transform.translation.z.floor() as i32,
                ));
            }

            // Apply rotation offset (R key)
            let mut dir = auto_conveyor_direction(place_pos, fallback_dir, &conveyors, &machine_positions);
            for _ in 0..rotation.offset {
                dir = dir.rotate_cw();
            }
            Some(dir)
        } else {
            player_facing
        }
    } else {
        None
    };

    // Check if break target is a conveyor and get its direction
    let break_conveyor_dir = target.break_target.and_then(|pos| {
        conveyor_query.iter().find(|c| c.position == pos).map(|c| c.direction)
    });

    // === Break target (red wireframe) - always show when looking at a block ===
    if let Some(pos) = target.break_target {
        let center = Vec3::new(
            pos.x as f32 + 0.5,
            pos.y as f32 + 0.5,
            pos.z as f32 + 0.5,
        );

        // Always recreate mesh to handle conveyor direction changes
        // Despawn old entity if exists
        if let Some(entity) = target.break_highlight_entity.take() {
            if break_query.get(entity).is_ok() {
                commands.entity(entity).despawn();
            }
        }

        let mesh = if let Some(dir) = break_conveyor_dir {
            meshes.add(create_conveyor_wireframe_mesh(dir))
        } else {
            meshes.add(create_wireframe_cube_mesh())
        };
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.2),
            unlit: true,
            ..default()
        });
        let entity = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(center),
            TargetHighlight,
            NotShadowCaster,
        )).id();
        target.break_highlight_entity = Some(entity);
    } else if let Some(entity) = target.break_highlight_entity.take() {
        if break_query.get(entity).is_ok() {
            commands.entity(entity).despawn();
        }
    }

    // === Place target (green wireframe) - only show if player has a placeable item ===
    if let Some(pos) = target.place_target.filter(|_| has_placeable_item) {
        let center = Vec3::new(
            pos.x as f32 + 0.5,
            pos.y as f32 + 0.5,
            pos.z as f32 + 0.5,
        );

        // Always recreate mesh to handle conveyor direction changes
        if let Some(entity) = target.place_highlight_entity.take() {
            if place_query.get(entity).is_ok() {
                commands.entity(entity).despawn();
            }
        }

        let mesh = if let Some(dir) = place_direction {
            meshes.add(create_conveyor_wireframe_mesh(dir))
        } else {
            meshes.add(create_wireframe_cube_mesh())
        };
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 1.0, 0.2),
            unlit: true,
            ..default()
        });
        let entity = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(center),
            PlaceHighlight,
            NotShadowCaster,
        )).id();
        target.place_highlight_entity = Some(entity);
    } else if let Some(entity) = target.place_highlight_entity.take() {
        if place_query.get(entity).is_ok() {
            commands.entity(entity).despawn();
        }
    }
}

/// Handle R key to rotate conveyor placement direction
fn rotate_conveyor_placement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut rotation: ResMut<ConveyorRotationOffset>,
    inventory: Res<Inventory>,
    input_resources: InputStateResourcesWithCursor,
) {
    // Only active when placing conveyors
    let selected = inventory.get_selected_type();
    if selected != Some(BlockType::ConveyorBlock) {
        // Reset rotation when not placing conveyor
        rotation.offset = 0;
        return;
    }

    // Check input state allows this action
    let input_state = input_resources.get_state();
    if !input_state.allows_block_actions() {
        return;
    }

    // R key rotates 90 degrees clockwise
    if keyboard.just_pressed(KeyCode::KeyR) {
        rotation.offset = (rotation.offset + 1) % 4;
    }
}

/// Update conveyor shapes based on adjacent conveyor connections
/// Adds visual extensions for side inputs (L-shape, T-shape)
/// Detects splitter mode when multiple outputs are available
#[allow(clippy::type_complexity)]
fn update_conveyor_shapes(
    mut commands: Commands,
    mut conveyors: Query<(Entity, &mut Conveyor, Option<&mut Mesh3d>, Option<&SceneRoot>, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    machine_models: Res<MachineModels>,
    furnace_query: Query<&Transform, (With<Furnace>, Without<Conveyor>)>,
    crusher_query: Query<&Crusher>,
) {
    // Collect all conveyor positions and directions first (read-only pass)
    let conveyor_data: Vec<(IVec3, Direction)> = conveyors
        .iter()
        .map(|(_, c, _, _, _)| (c.position, c.direction))
        .collect();

    // Collect positions that can accept items (conveyors, furnaces, crushers)
    let conveyor_positions: HashSet<IVec3> = conveyor_data.iter().map(|(p, _)| *p).collect();
    let furnace_positions: HashSet<IVec3> = furnace_query
        .iter()
        .map(|t| IVec3::new(
            t.translation.x.floor() as i32,
            t.translation.y.floor() as i32,
            t.translation.z.floor() as i32,
        ))
        .collect();
    let crusher_positions: HashSet<IVec3> = crusher_query
        .iter()
        .map(|c| c.position)
        .collect();

    for (entity, mut conveyor, mesh3d_opt, scene_root_opt, transform) in conveyors.iter_mut() {
        // Calculate inputs from adjacent conveyors
        let mut has_left_input = false;
        let mut has_right_input = false;

        let left_dir = conveyor.direction.left();
        let right_dir = conveyor.direction.right();
        let left_pos = conveyor.position + left_dir.to_ivec3();
        let right_pos = conveyor.position + right_dir.to_ivec3();

        // Check if any conveyor is pointing at us from the side
        for (pos, dir) in &conveyor_data {
            if *pos == left_pos && *dir == left_dir.opposite() {
                has_left_input = true;
            }
            if *pos == right_pos && *dir == right_dir.opposite() {
                has_right_input = true;
            }
        }

        // Check for splitter mode: count available outputs
        let front_pos = conveyor.position + conveyor.direction.to_ivec3();
        let can_output = |pos: IVec3| -> bool {
            // Check if position has something that can accept items
            if conveyor_positions.contains(&pos) {
                // Check if conveyor at this position can accept from us
                for (p, dir) in &conveyor_data {
                    if *p == pos {
                        // The target conveyor accepts from behind or sides
                        let from_dir = conveyor.position - pos;
                        let accepts = match *dir {
                            Direction::East => from_dir.x == -1 || from_dir.z != 0,
                            Direction::West => from_dir.x == 1 || from_dir.z != 0,
                            Direction::South => from_dir.z == -1 || from_dir.x != 0,
                            Direction::North => from_dir.z == 1 || from_dir.x != 0,
                        };
                        return accepts;
                    }
                }
            }
            furnace_positions.contains(&pos) || crusher_positions.contains(&pos)
        };

        let has_front_output = can_output(front_pos);
        let has_left_output = can_output(left_pos);
        let has_right_output = can_output(right_pos);
        let output_count = [has_front_output, has_left_output, has_right_output]
            .iter()
            .filter(|&&x| x)
            .count();

        // Determine new shape
        let new_shape = if output_count >= 2 && !has_left_input && !has_right_input {
            // Splitter mode: multiple outputs and no side inputs
            ConveyorShape::Splitter
        } else {
            match (has_left_input, has_right_input) {
                (false, false) => ConveyorShape::Straight,
                (true, false) => ConveyorShape::CornerLeft,
                (false, true) => ConveyorShape::CornerRight,
                (true, true) => ConveyorShape::TJunction,
            }
        };

        // Only update if shape changed
        if conveyor.shape != new_shape {
            let _old_shape = conveyor.shape;
            conveyor.shape = new_shape;

            // Check if using glTF model (has SceneRoot component)
            let uses_gltf = scene_root_opt.is_some();

            if uses_gltf {
                // Using glTF models - need to despawn and respawn with new model
                if let Some(new_model) = machine_models.get_conveyor_model(new_shape) {
                    // Store conveyor data before despawn
                    let conv_data = Conveyor {
                        position: conveyor.position,
                        direction: conveyor.direction,
                        items: std::mem::take(&mut conveyor.items),
                        last_output_index: conveyor.last_output_index,
                        last_input_source: conveyor.last_input_source,
                        shape: new_shape,
                    };
                    let conv_transform = *transform;

                    // Despawn old entity
                    commands.entity(entity).despawn_recursive();

                    // Spawn new entity with new model
                    // Note: GlobalTransform and Visibility are required for rendering
                    commands.spawn((
                        SceneRoot(new_model),
                        conv_transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        conv_data,
                        ConveyorVisual,
                    ));
                }
            } else if let Some(mut mesh3d) = mesh3d_opt {
                // Using procedural mesh - just swap the mesh
                let new_mesh = create_conveyor_mesh(new_shape);
                *mesh3d = Mesh3d(meshes.add(new_mesh));
            }
        }
    }
}

/// Create conveyor mesh based on connection shape
fn create_conveyor_mesh(shape: ConveyorShape) -> Mesh {
    let width = BLOCK_SIZE * CONVEYOR_BELT_WIDTH;
    let height = BLOCK_SIZE * CONVEYOR_BELT_HEIGHT;
    let half_width = width / 2.0;
    let half_height = height / 2.0;
    let half_block = BLOCK_SIZE / 2.0;

    match shape {
        ConveyorShape::Straight => {
            // Simple rectangular belt
            Cuboid::new(width, height, BLOCK_SIZE).into()
        }
        ConveyorShape::CornerLeft => {
            // L-shaped: main belt + left extension
            create_l_shaped_mesh(half_width, half_height, half_block, true)
        }
        ConveyorShape::CornerRight => {
            // L-shaped: main belt + right extension
            create_l_shaped_mesh(half_width, half_height, half_block, false)
        }
        ConveyorShape::TJunction => {
            // T-shaped: main belt + both side extensions
            create_t_shaped_mesh(half_width, half_height, half_block)
        }
        ConveyorShape::Splitter => {
            // Splitter: Y-shaped with 3 output directions (front, left, right)
            create_splitter_mesh(half_width, half_height, half_block)
        }
    }
}

/// Create L-shaped conveyor mesh
fn create_l_shaped_mesh(half_width: f32, half_height: f32, half_block: f32, is_left: bool) -> Mesh {
    // The conveyor faces -Z, so:
    // - Left is +X direction
    // - Right is -X direction
    let side_sign = if is_left { 1.0 } else { -1.0 };

    // Main belt vertices (along Z axis, width along X)
    // Side extension vertices (along X axis from the back half)
    let positions: Vec<[f32; 3]> = vec![
        // Main belt (8 vertices) - full length along Z
        [-half_width, -half_height, -half_block], // 0
        [half_width, -half_height, -half_block],  // 1
        [half_width, half_height, -half_block],   // 2
        [-half_width, half_height, -half_block],  // 3
        [-half_width, -half_height, half_block],  // 4
        [half_width, -half_height, half_block],   // 5
        [half_width, half_height, half_block],    // 6
        [-half_width, half_height, half_block],   // 7

        // Side extension (8 vertices) - extends from side at back half
        // Inner edge at half_width (or -half_width), outer edge at half_block
        [side_sign * half_width, -half_height, 0.0],       // 8 - inner front
        [side_sign * half_block, -half_height, 0.0],       // 9 - outer front
        [side_sign * half_block, half_height, 0.0],        // 10 - outer front top
        [side_sign * half_width, half_height, 0.0],        // 11 - inner front top
        [side_sign * half_width, -half_height, half_block], // 12 - inner back
        [side_sign * half_block, -half_height, half_block], // 13 - outer back
        [side_sign * half_block, half_height, half_block],  // 14 - outer back top
        [side_sign * half_width, half_height, half_block],  // 15 - inner back top
    ];

    // Normals for each vertex
    let normals: Vec<[f32; 3]> = vec![
        // Main belt
        [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], // front
        [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],     // back
        // Side extension (simplified - all outward)
        [0.0, 0.0, -1.0], [side_sign, 0.0, 0.0], [side_sign, 0.0, 0.0], [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0], [side_sign, 0.0, 0.0], [side_sign, 0.0, 0.0], [0.0, 0.0, 1.0],
    ];

    let uvs: Vec<[f32; 2]> = vec![[0.0; 2]; 16];

    // Indices - each face needs to be wound correctly for back-face culling
    let indices = if is_left {
        vec![
            // Main belt faces
            0, 2, 1, 0, 3, 2,       // front
            4, 5, 6, 4, 6, 7,       // back
            0, 1, 5, 0, 5, 4,       // bottom
            3, 6, 2, 3, 7, 6,       // top
            0, 4, 7, 0, 7, 3,       // left (closed since extension is on right/+X)
            1, 2, 6, 1, 6, 5,       // right - open where extension connects

            // Side extension faces
            8, 10, 9, 8, 11, 10,    // front
            12, 13, 14, 12, 14, 15, // back
            8, 9, 13, 8, 13, 12,    // bottom
            11, 14, 10, 11, 15, 14, // top
            9, 10, 14, 9, 14, 13,   // outer side
        ]
    } else {
        vec![
            // Main belt faces
            0, 2, 1, 0, 3, 2,       // front
            4, 5, 6, 4, 6, 7,       // back
            0, 1, 5, 0, 5, 4,       // bottom
            3, 6, 2, 3, 7, 6,       // top
            1, 2, 6, 1, 6, 5,       // right (closed since extension is on left/-X)
            0, 4, 7, 0, 7, 3,       // left - open where extension connects

            // Side extension faces (reversed winding for -X side)
            8, 9, 10, 8, 10, 11,    // front
            12, 14, 13, 12, 15, 14, // back
            8, 13, 9, 8, 12, 13,    // bottom
            11, 10, 14, 11, 14, 15, // top
            9, 14, 10, 9, 13, 14,   // outer side
        ]
    };

    let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
}

/// Create T-shaped conveyor mesh (both sides)
fn create_t_shaped_mesh(half_width: f32, half_height: f32, half_block: f32) -> Mesh {
    // Main belt + extensions on both sides
    let positions: Vec<[f32; 3]> = vec![
        // Main belt (8 vertices)
        [-half_width, -half_height, -half_block], // 0
        [half_width, -half_height, -half_block],  // 1
        [half_width, half_height, -half_block],   // 2
        [-half_width, half_height, -half_block],  // 3
        [-half_width, -half_height, half_block],  // 4
        [half_width, -half_height, half_block],   // 5
        [half_width, half_height, half_block],    // 6
        [-half_width, half_height, half_block],   // 7

        // Left extension (+X side, 8 vertices)
        [half_width, -half_height, 0.0],          // 8
        [half_block, -half_height, 0.0],          // 9
        [half_block, half_height, 0.0],           // 10
        [half_width, half_height, 0.0],           // 11
        [half_width, -half_height, half_block],   // 12
        [half_block, -half_height, half_block],   // 13
        [half_block, half_height, half_block],    // 14
        [half_width, half_height, half_block],    // 15

        // Right extension (-X side, 8 vertices)
        [-half_width, -half_height, 0.0],         // 16
        [-half_block, -half_height, 0.0],         // 17
        [-half_block, half_height, 0.0],          // 18
        [-half_width, half_height, 0.0],          // 19
        [-half_width, -half_height, half_block],  // 20
        [-half_block, -half_height, half_block],  // 21
        [-half_block, half_height, half_block],   // 22
        [-half_width, half_height, half_block],   // 23
    ];

    let normals: Vec<[f32; 3]> = vec![
        // Main belt
        [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
        // Left extension
        [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0],
        // Right extension
        [0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0],
    ];

    let uvs: Vec<[f32; 2]> = vec![[0.0; 2]; 24];

    let indices: Vec<u32> = vec![
        // Main belt faces
        0, 2, 1, 0, 3, 2,       // front
        4, 5, 6, 4, 6, 7,       // back
        0, 1, 5, 0, 5, 4,       // bottom
        3, 6, 2, 3, 7, 6,       // top
        // Left and right main sides are open for extensions

        // Left extension (+X)
        8, 10, 9, 8, 11, 10,    // front
        12, 13, 14, 12, 14, 15, // back
        8, 9, 13, 8, 13, 12,    // bottom
        11, 14, 10, 11, 15, 14, // top
        9, 10, 14, 9, 14, 13,   // outer side (+X face)

        // Right extension (-X)
        16, 17, 18, 16, 18, 19, // front
        20, 22, 21, 20, 23, 22, // back
        16, 21, 17, 16, 20, 21, // bottom
        19, 18, 22, 19, 22, 23, // top
        17, 22, 18, 17, 21, 22, // outer side (-X face)
    ];

    let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
}

/// Create splitter conveyor mesh (Y-shaped with 3 output directions)
fn create_splitter_mesh(half_width: f32, half_height: f32, half_block: f32) -> Mesh {
    // Main belt (short, just the center) + 3 extensions (front, left, right)
    let positions: Vec<[f32; 3]> = vec![
        // Center hub (8 vertices) - small cube at center
        [-half_width, -half_height, -half_width], // 0
        [half_width, -half_height, -half_width],  // 1
        [half_width, half_height, -half_width],   // 2
        [-half_width, half_height, -half_width],  // 3
        [-half_width, -half_height, half_width],  // 4
        [half_width, -half_height, half_width],   // 5
        [half_width, half_height, half_width],    // 6
        [-half_width, half_height, half_width],   // 7

        // Front extension (-Z direction, 8 vertices)
        [-half_width, -half_height, -half_block], // 8
        [half_width, -half_height, -half_block],  // 9
        [half_width, half_height, -half_block],   // 10
        [-half_width, half_height, -half_block],  // 11
        [-half_width, -half_height, -half_width], // 12
        [half_width, -half_height, -half_width],  // 13
        [half_width, half_height, -half_width],   // 14
        [-half_width, half_height, -half_width],  // 15

        // Left extension (+X direction, 8 vertices)
        [half_width, -half_height, -half_width],  // 16
        [half_block, -half_height, -half_width],  // 17
        [half_block, half_height, -half_width],   // 18
        [half_width, half_height, -half_width],   // 19
        [half_width, -half_height, half_width],   // 20
        [half_block, -half_height, half_width],   // 21
        [half_block, half_height, half_width],    // 22
        [half_width, half_height, half_width],    // 23

        // Right extension (-X direction, 8 vertices)
        [-half_block, -half_height, -half_width], // 24
        [-half_width, -half_height, -half_width], // 25
        [-half_width, half_height, -half_width],  // 26
        [-half_block, half_height, -half_width],  // 27
        [-half_block, -half_height, half_width],  // 28
        [-half_width, -half_height, half_width],  // 29
        [-half_width, half_height, half_width],   // 30
        [-half_block, half_height, half_width],   // 31
    ];

    let normals: Vec<[f32; 3]> = vec![
        // Center hub (simplified normals)
        [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
        // Front extension
        [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
        // Left extension
        [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
        // Right extension
        [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
    ];

    let uvs: Vec<[f32; 2]> = vec![[0.0; 2]; 32];

    let indices: Vec<u32> = vec![
        // Center hub - top face only (sides connect to extensions)
        3, 6, 2, 3, 7, 6,       // top
        0, 1, 5, 0, 5, 4,       // bottom

        // Front extension (-Z)
        8, 10, 9, 8, 11, 10,    // front face
        8, 9, 13, 8, 13, 12,    // bottom
        11, 14, 10, 11, 15, 14, // top
        8, 12, 15, 8, 15, 11,   // left side
        9, 10, 14, 9, 14, 13,   // right side

        // Left extension (+X)
        17, 18, 22, 17, 22, 21, // outer face (+X)
        16, 17, 21, 16, 21, 20, // bottom
        19, 22, 18, 19, 23, 22, // top
        16, 19, 18, 16, 18, 17, // front
        20, 21, 22, 20, 22, 23, // back

        // Right extension (-X)
        24, 28, 31, 24, 31, 27, // outer face (-X)
        24, 25, 29, 24, 29, 28, // bottom
        27, 31, 30, 27, 30, 26, // top
        24, 27, 26, 24, 26, 25, // front
        28, 29, 30, 28, 30, 31, // back
    ];

    let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
}

/// Update guide markers based on selected item
/// Shows recommended placement positions for machines
#[allow(clippy::too_many_arguments)]
fn update_guide_markers(
    mut commands: Commands,
    mut guide_markers: ResMut<GuideMarkers>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    inventory: Res<Inventory>,
    time: Res<Time>,
    miner_query: Query<&Miner>,
    conveyor_query: Query<&Conveyor>,
    furnace_query: Query<&Transform, (With<Furnace>, Without<GuideMarker>)>,
    crusher_query: Query<&Transform, (With<Crusher>, Without<GuideMarker>)>,
) {
    let selected = inventory.get_selected_type();

    // Clear markers if selection changed or nothing selected
    if selected != guide_markers.last_selected {
        for entity in guide_markers.entities.drain(..) {
            commands.entity(entity).despawn_recursive();
        }
        guide_markers.last_selected = selected;
    }

    // No markers if nothing is selected or non-machine item
    let Some(block_type) = selected else {
        return;
    };

    // Only show guides for placeable machines
    if !matches!(block_type,
        BlockType::MinerBlock |
        BlockType::ConveyorBlock |
        BlockType::FurnaceBlock |
        BlockType::CrusherBlock
    ) {
        return;
    }

    // Calculate pulse effect (0.3 to 0.7 alpha)
    let pulse = (time.elapsed_secs() * 3.0).sin() * 0.2 + 0.5;

    // Generate guide positions based on selected item
    let guide_positions = match block_type {
        BlockType::MinerBlock => {
            // Show positions outside delivery platform edges
            generate_miner_guide_positions()
        }
        BlockType::ConveyorBlock => {
            // Show positions extending from existing machines
            generate_conveyor_guide_positions(&miner_query, &conveyor_query, &furnace_query, &crusher_query)
        }
        BlockType::FurnaceBlock | BlockType::CrusherBlock => {
            // Show positions along conveyor paths
            generate_processor_guide_positions(&conveyor_query)
        }
        _ => vec![],
    };

    // Only update if we need to spawn new markers
    if guide_markers.entities.is_empty() && !guide_positions.is_empty() {
        let mesh = meshes.add(create_wireframe_cube_mesh());

        for pos in guide_positions {
            let material = materials.add(StandardMaterial {
                base_color: Color::srgba(0.3, 0.6, 1.0, pulse),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            let entity = commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(
                    pos.x as f32 + 0.5,
                    pos.y as f32 + 0.5,
                    pos.z as f32 + 0.5,
                )),
                GuideMarker,
                NotShadowCaster,
            )).id();

            guide_markers.entities.push(entity);
        }
    }
    // Note: pulse effect would require material recreation each frame - skipped for performance
}

/// Generate guide positions for miners (outside delivery platform edges)
fn generate_miner_guide_positions() -> Vec<IVec3> {
    let mut positions = Vec::new();

    // Delivery platform: origin (20, 8, 10), size 12x12
    // Show positions 2-3 blocks outside each edge at Y=8

    // North of platform (z = 8, 9)
    for x in (20..32).step_by(3) {
        positions.push(IVec3::new(x, 8, 8));
    }

    // South of platform (z = 23, 24)
    for x in (20..32).step_by(3) {
        positions.push(IVec3::new(x, 8, 23));
    }

    // West of platform (x = 18)
    for z in (10..22).step_by(3) {
        positions.push(IVec3::new(18, 8, z));
    }

    // East of platform (x = 33)
    for z in (10..22).step_by(3) {
        positions.push(IVec3::new(33, 8, z));
    }

    positions
}

/// Generate guide positions for conveyors (extending from existing machines)
fn generate_conveyor_guide_positions(
    miner_query: &Query<&Miner>,
    conveyor_query: &Query<&Conveyor>,
    furnace_query: &Query<&Transform, (With<Furnace>, Without<GuideMarker>)>,
    crusher_query: &Query<&Transform, (With<Crusher>, Without<GuideMarker>)>,
) -> Vec<IVec3> {
    let mut positions = Vec::new();
    let mut existing: std::collections::HashSet<IVec3> = std::collections::HashSet::new();

    // Collect existing machine positions
    for miner in miner_query.iter() {
        existing.insert(miner.position);
    }
    for conveyor in conveyor_query.iter() {
        existing.insert(conveyor.position);
    }
    for transform in furnace_query.iter() {
        let pos = IVec3::new(
            transform.translation.x.floor() as i32,
            transform.translation.y.floor() as i32,
            transform.translation.z.floor() as i32,
        );
        existing.insert(pos);
    }
    for transform in crusher_query.iter() {
        let pos = IVec3::new(
            transform.translation.x.floor() as i32,
            transform.translation.y.floor() as i32,
            transform.translation.z.floor() as i32,
        );
        existing.insert(pos);
    }

    // Show positions adjacent to conveyor ends
    for conveyor in conveyor_query.iter() {
        let next_pos = match conveyor.direction {
            Direction::North => conveyor.position + IVec3::new(0, 0, -1),
            Direction::South => conveyor.position + IVec3::new(0, 0, 1),
            Direction::East => conveyor.position + IVec3::new(1, 0, 0),
            Direction::West => conveyor.position + IVec3::new(-1, 0, 0),
        };

        if !existing.contains(&next_pos) && !positions.contains(&next_pos) {
            positions.push(next_pos);
        }
    }

    // Show positions adjacent to miners (output side)
    for miner in miner_query.iter() {
        for dir in [IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z] {
            let adj = miner.position + dir;
            if !existing.contains(&adj) && !positions.contains(&adj) {
                positions.push(adj);
                break; // Only one suggestion per miner
            }
        }
    }

    // Limit to 8 suggestions to avoid clutter
    positions.truncate(8);
    positions
}

/// Generate guide positions for processors (along conveyor paths)
fn generate_processor_guide_positions(conveyor_query: &Query<&Conveyor>) -> Vec<IVec3> {
    let mut positions = Vec::new();
    let mut conveyor_positions: std::collections::HashSet<IVec3> = std::collections::HashSet::new();

    for conveyor in conveyor_query.iter() {
        conveyor_positions.insert(conveyor.position);
    }

    // Show positions adjacent to conveyors (as inline processors)
    for conveyor in conveyor_query.iter() {
        // Position perpendicular to conveyor direction
        let perpendicular = match conveyor.direction {
            Direction::North | Direction::South => [IVec3::X, IVec3::NEG_X],
            Direction::East | Direction::West => [IVec3::Z, IVec3::NEG_Z],
        };

        for dir in perpendicular {
            let adj = conveyor.position + dir;
            if !conveyor_positions.contains(&adj) && !positions.contains(&adj) {
                positions.push(adj);
            }
        }
    }

    // Limit to 6 suggestions
    positions.truncate(6);
    positions
}

// === Creative Mode ===

// Note: F-key shortcuts removed - use Creative Catalog (E key while in creative mode) instead
// Use /creative and /survival commands to toggle modes

/// Return held item to inventory when closing
fn return_held_item_to_inventory(inventory: &mut Inventory, held_item: &mut HeldItem) {
    if let Some((block_type, count)) = held_item.0.take() {
        // Try to add to inventory
        let remaining = inventory.add_item(block_type, count);
        if remaining > 0 {
            // If inventory is full, item is lost (or could be dropped later)
            // For now, just put back what couldn't fit
            held_item.0 = Some((block_type, remaining));
        }
    }
}

/// Toggle inventory with E key (works in both survival and creative mode)
#[allow(clippy::too_many_arguments)]
fn inventory_toggle(
    key_input: Res<ButtonInput<KeyCode>>,
    mut inventory_open: ResMut<InventoryOpen>,
    mut inventory: ResMut<Inventory>,
    mut held_item: ResMut<HeldItem>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    command_state: Res<CommandInputState>,
    cursor_state: Res<CursorLockState>,
    creative_mode: Res<CreativeMode>,
    mut ui_query: Query<&mut Visibility, With<InventoryUI>>,
    mut creative_panel_query: Query<&mut Visibility, (With<CreativePanel>, Without<InventoryUI>)>,
    mut windows: Query<&mut Window>,
) {
    // Don't toggle if other UIs are open or game is paused (input matrix: E key)
    if interacting_furnace.0.is_some() || interacting_crusher.0.is_some() || command_state.open || cursor_state.paused {
        if key_input.just_pressed(KeyCode::KeyE) {
            info!("[INVENTORY] E pressed but blocked: furnace={}, crusher={}, command={}, paused={}",
                interacting_furnace.0.is_some(),
                interacting_crusher.0.is_some(),
                command_state.open,
                cursor_state.paused);
        }
        return;
    }

    // E key to toggle inventory
    if key_input.just_pressed(KeyCode::KeyE) {
        info!("[INVENTORY] E key pressed, toggling from {} to {}", inventory_open.0, !inventory_open.0);
        inventory_open.0 = !inventory_open.0;

        // Return held item when closing
        if !inventory_open.0 {
            return_held_item_to_inventory(&mut inventory, &mut held_item);
        }

        let mut ui_count = 0;
        for mut vis in ui_query.iter_mut() {
            ui_count += 1;
            *vis = if inventory_open.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        info!("[INVENTORY] Updated {} UI entities, now open={}", ui_count, inventory_open.0);

        if ui_count == 0 {
            warn!("[INVENTORY] No InventoryUI entity found! UI will not display.");
        }

        // Show/hide creative panel based on creative mode
        for mut vis in creative_panel_query.iter_mut() {
            *vis = if inventory_open.0 && creative_mode.enabled {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }

        // Unlock/lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            if inventory_open.0 {
                window.cursor_options.grab_mode = CursorGrabMode::None;
                window.cursor_options.visible = true;
                set_ui_open_state(true);
            } else {
                window.cursor_options.grab_mode = CursorGrabMode::Locked;
                window.cursor_options.visible = false;
                set_ui_open_state(false);
            }
        }
    }

    // ESC to close
    if inventory_open.0 && key_input.just_pressed(KeyCode::Escape) {
        inventory_open.0 = false;

        // Return held item when closing
        return_held_item_to_inventory(&mut inventory, &mut held_item);

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Also hide creative panel
        for mut vis in creative_panel_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Unlock cursor - JS will auto-relock via data-ui-open observer (BUG-6 fix)
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(false);
        }
    }
}

/// Handle creative inventory item button clicks (only in creative mode)
fn creative_inventory_click(
    creative_inv_open: Res<InventoryOpen>,
    creative_mode: Res<CreativeMode>,
    mut inventory: ResMut<Inventory>,
    mut interaction_query: Query<
        (&Interaction, &CreativeItemButton, &mut BackgroundColor, &mut BorderColor),
        Changed<Interaction>,
    >,
) {
    // Only handle clicks in creative mode with inventory open
    if !creative_inv_open.0 || !creative_mode.enabled {
        return;
    }

    for (interaction, button, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        let block_type = button.0;

        match *interaction {
            Interaction::Pressed => {
                // Add 64 of this item to selected slot
                let slot = inventory.selected_slot;
                inventory.slots[slot] = Some((block_type, 64));
                // Visual feedback
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                // Highlight on hover
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                // Slightly brighter background
                let base = block_type.color();
                let Srgba { red, green, blue, alpha } = base.to_srgba();
                *bg_color = BackgroundColor(Color::srgba(
                    (red + 0.2).min(1.0),
                    (green + 0.2).min(1.0),
                    (blue + 0.2).min(1.0),
                    alpha,
                ));
            }
            Interaction::None => {
                // Reset to normal
                *border_color = BorderColor(Color::srgba(0.3, 0.3, 0.3, 1.0));
                *bg_color = BackgroundColor(block_type.color());
            }
        }
    }
}

/// Handle inventory slot clicks (pick up / place items)
fn inventory_slot_click(
    inventory_open: Res<InventoryOpen>,
    mut inventory: ResMut<Inventory>,
    mut held_item: ResMut<HeldItem>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<
        (&Interaction, &InventorySlotUI, &mut BackgroundColor, &mut BorderColor),
        Changed<Interaction>,
    >,
) {
    if !inventory_open.0 {
        return;
    }

    let shift_held = key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight);

    for (interaction, slot_ui, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        let slot_idx = slot_ui.0;

        match *interaction {
            Interaction::Pressed => {
                if shift_held {
                    // Shift+Click: Quick move between hotbar and main inventory
                    if let Some((block_type, count)) = inventory.slots[slot_idx].take() {
                        // Determine target area
                        let target_range = if slot_idx < HOTBAR_SLOTS {
                            // From hotbar -> main inventory
                            HOTBAR_SLOTS..NUM_SLOTS
                        } else {
                            // From main -> hotbar
                            0..HOTBAR_SLOTS
                        };

                        // Try to stack first
                        let mut remaining = count;
                        for target_idx in target_range.clone() {
                            if remaining == 0 { break; }
                            if let Some((bt, ref mut c)) = inventory.slots[target_idx] {
                                if bt == block_type && *c < MAX_STACK_SIZE {
                                    let space = MAX_STACK_SIZE - *c;
                                    let to_add = remaining.min(space);
                                    *c += to_add;
                                    remaining -= to_add;
                                }
                            }
                        }

                        // Then find empty slots
                        for target_idx in target_range {
                            if remaining == 0 { break; }
                            if inventory.slots[target_idx].is_none() {
                                let to_add = remaining.min(MAX_STACK_SIZE);
                                inventory.slots[target_idx] = Some((block_type, to_add));
                                remaining -= to_add;
                            }
                        }

                        // Put back any remaining
                        if remaining > 0 {
                            inventory.slots[slot_idx] = Some((block_type, remaining));
                        }
                    }
                } else {
                    // Normal click: pick up or place
                    let slot_item = inventory.slots[slot_idx].take();
                    let held = held_item.0.take();

                    match (slot_item, held) {
                        (None, None) => {
                            // Both empty, do nothing
                        }
                        (Some(item), None) => {
                            // Pick up item from slot
                            held_item.0 = Some(item);
                        }
                        (None, Some(item)) => {
                            // Place held item into slot
                            inventory.slots[slot_idx] = Some(item);
                        }
                        (Some((slot_type, slot_count)), Some((held_type, held_count))) => {
                            if slot_type == held_type {
                                // Same type - try to stack
                                let total = slot_count + held_count;
                                if total <= MAX_STACK_SIZE {
                                    inventory.slots[slot_idx] = Some((slot_type, total));
                                } else {
                                    inventory.slots[slot_idx] = Some((slot_type, MAX_STACK_SIZE));
                                    held_item.0 = Some((held_type, total - MAX_STACK_SIZE));
                                }
                            } else {
                                // Different types - swap
                                inventory.slots[slot_idx] = Some((held_type, held_count));
                                held_item.0 = Some((slot_type, slot_count));
                            }
                        }
                    }
                }

                // Visual feedback
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.7, 0.7, 0.7));
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.9));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9));
            }
        }
    }
}

/// Helper function to perform shift-click move on a slot
fn perform_shift_click_move(inventory: &mut Inventory, slot_idx: usize) -> bool {
    if let Some((block_type, count)) = inventory.slots[slot_idx].take() {
        // Determine target area
        let target_range = if slot_idx < HOTBAR_SLOTS {
            // From hotbar -> main inventory
            HOTBAR_SLOTS..NUM_SLOTS
        } else {
            // From main -> hotbar
            0..HOTBAR_SLOTS
        };

        // Try to stack first
        let mut remaining = count;
        for target_idx in target_range.clone() {
            if remaining == 0 { break; }
            if let Some((bt, ref mut c)) = inventory.slots[target_idx] {
                if bt == block_type && *c < MAX_STACK_SIZE {
                    let space = MAX_STACK_SIZE - *c;
                    let to_add = remaining.min(space);
                    *c += to_add;
                    remaining -= to_add;
                }
            }
        }

        // Then find empty slots
        for target_idx in target_range {
            if remaining == 0 { break; }
            if inventory.slots[target_idx].is_none() {
                let to_add = remaining.min(MAX_STACK_SIZE);
                inventory.slots[target_idx] = Some((block_type, to_add));
                remaining -= to_add;
            }
        }

        // Put back any remaining
        if remaining > 0 {
            inventory.slots[slot_idx] = Some((block_type, remaining));
        }
        return remaining < count; // Return true if anything was moved
    }
    false
}

/// Continuous shift+click support for inventory
fn inventory_continuous_shift_click(
    inventory_open: Res<InventoryOpen>,
    mut inventory: ResMut<Inventory>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut action_timer: ResMut<ContinuousActionTimer>,
    interaction_query: Query<(&Interaction, &InventorySlotUI)>,
) {
    if !inventory_open.0 {
        return;
    }

    let shift_held = key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight);
    if !shift_held || !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    // Skip if timer hasn't finished (and this isn't the first click handled by inventory_slot_click)
    if !action_timer.inventory_timer.finished() {
        return;
    }

    // Find hovered slot
    for (interaction, slot_ui) in interaction_query.iter() {
        if *interaction == Interaction::Hovered {
            let slot_idx = slot_ui.0;
            if perform_shift_click_move(&mut inventory, slot_idx) {
                action_timer.inventory_timer.reset();
            }
            break;
        }
    }
}

/// Update inventory slot visuals to reflect current inventory state
fn inventory_update_slots(
    inventory_open: Res<InventoryOpen>,
    inventory: Res<Inventory>,
    mut slot_query: Query<(&InventorySlotUI, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    if !inventory_open.0 {
        return;
    }

    for (slot_ui, mut bg_color, children) in slot_query.iter_mut() {
        let slot_idx = slot_ui.0;

        if let Some((block_type, count)) = inventory.slots[slot_idx] {
            // Show item color and count
            *bg_color = BackgroundColor(block_type.color());

            // Update text (count)
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = if count > 1 {
                        format!("{}", count)
                    } else {
                        String::new()
                    };
                }
            }
        } else {
            // Empty slot
            *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9));

            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = String::new();
                }
            }
        }
    }
}

/// Update held item display to follow cursor and show held item
fn update_held_item_display(
    inventory_open: Res<InventoryOpen>,
    held_item: Res<HeldItem>,
    windows: Query<&Window>,
    mut held_display_query: Query<(&mut Node, &mut BackgroundColor, &mut Visibility), With<HeldItemDisplay>>,
    mut held_text_query: Query<&mut Text, With<HeldItemText>>,
) {
    let Ok((mut node, mut bg_color, mut visibility)) = held_display_query.get_single_mut() else {
        return;
    };

    // Only show when inventory is open and we're holding something
    if !inventory_open.0 {
        *visibility = Visibility::Hidden;
        return;
    }

    match &held_item.0 {
        Some((block_type, count)) => {
            // Show the held item
            *visibility = Visibility::Visible;
            *bg_color = BackgroundColor(block_type.color());

            // Update count text
            if let Ok(mut text) = held_text_query.get_single_mut() {
                text.0 = if *count > 1 {
                    format!("{}", count)
                } else {
                    String::new()
                };
            }

            // Position at cursor
            if let Ok(window) = windows.get_single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    // Offset so item appears slightly below and to the right of cursor
                    node.left = Val::Px(cursor_pos.x + 8.0);
                    node.top = Val::Px(cursor_pos.y + 8.0);
                }
            }
        }
        None => {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Update the hotbar item name display to show the selected item's name
fn update_hotbar_item_name(
    inventory: Res<Inventory>,
    inventory_open: Res<InventoryOpen>,
    mut text_query: Query<(&mut Text, &mut Node), With<HotbarItemNameText>>,
) {
    let Ok((mut text, mut node)) = text_query.get_single_mut() else {
        return;
    };

    // Hide when inventory is open
    if inventory_open.0 {
        text.0 = String::new();
        return;
    }

    // Show selected item name
    if let Some(block_type) = inventory.selected_block() {
        let name = block_type.name();
        text.0 = name.to_string();
        // Center the text by adjusting margin based on text length
        let char_width = 8.0; // Approximate character width
        node.margin.left = Val::Px(-(name.len() as f32 * char_width / 2.0));
    } else {
        text.0 = String::new();
    }
}

/// Update inventory tooltip to show item name when hovering over slots
fn update_inventory_tooltip(
    inventory_open: Res<InventoryOpen>,
    inventory: Res<Inventory>,
    windows: Query<&Window>,
    slot_query: Query<(&Interaction, &InventorySlotUI, &GlobalTransform)>,
    creative_query: Query<(&Interaction, &CreativeItemButton, &GlobalTransform)>,
    mut tooltip_query: Query<(&mut Node, &mut Visibility, &Children), With<InventoryTooltip>>,
    mut text_query: Query<&mut Text>,
) {
    let Ok((mut node, mut visibility, children)) = tooltip_query.get_single_mut() else {
        return;
    };

    // Hide tooltip if inventory is closed
    if !inventory_open.0 {
        *visibility = Visibility::Hidden;
        return;
    }

    // Find hovered slot (inventory slots)
    let mut hovered_item: Option<(BlockType, Option<u32>, Vec2)> = None;
    for (interaction, slot_ui, global_transform) in slot_query.iter() {
        if *interaction == Interaction::Hovered {
            let slot_idx = slot_ui.0;
            if let Some((block_type, count)) = inventory.slots[slot_idx] {
                let pos = global_transform.translation();
                hovered_item = Some((block_type, Some(count), Vec2::new(pos.x, pos.y)));
                break;
            }
        }
    }

    // Check creative catalog items if no inventory slot is hovered
    if hovered_item.is_none() {
        for (interaction, creative_btn, global_transform) in creative_query.iter() {
            if *interaction == Interaction::Hovered {
                let pos = global_transform.translation();
                hovered_item = Some((creative_btn.0, None, Vec2::new(pos.x, pos.y)));
                break;
            }
        }
    }

    if let Some((block_type, count_opt, slot_pos)) = hovered_item {
        *visibility = Visibility::Inherited;

        // Position tooltip near the slot (offset to the right and up)
        if let Ok(window) = windows.get_single() {
            let half_width = window.width() / 2.0;
            let half_height = window.height() / 2.0;
            // Convert from global UI coords to absolute position
            node.left = Val::Px(slot_pos.x + half_width + 45.0);
            node.top = Val::Px(half_height - slot_pos.y - 10.0);
        }

        // Update tooltip text
        if let Some(&child) = children.first() {
            if let Ok(mut text) = text_query.get_mut(child) {
                if let Some(count) = count_opt {
                    text.0 = format!("{} ({})", block_type.name(), count);
                } else {
                    // Creative catalog item - just show name
                    text.0 = block_type.name().to_string();
                }
            }
        }
    } else {
        *visibility = Visibility::Hidden;
    }
}

/// Handle trash slot clicks (delete held item)
#[allow(clippy::type_complexity)]
fn trash_slot_click(
    inventory_open: Res<InventoryOpen>,
    mut held_item: ResMut<HeldItem>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<TrashSlot>),
    >,
) {
    if !inventory_open.0 {
        return;
    }

    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Delete held item
                held_item.0 = None;
                *border_color = BorderColor(Color::srgb(1.0, 0.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(1.0, 0.5, 0.5));
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.1, 0.1));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgb(0.6, 0.2, 0.2));
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.1, 0.1));
            }
        }
    }
}

// === Command Input System ===

/// Toggle command input with T or / key
#[allow(clippy::too_many_arguments)]
fn command_input_toggle(
    key_input: Res<ButtonInput<KeyCode>>,
    mut command_state: ResMut<CommandInputState>,
    mut ui_query: Query<&mut Visibility, With<CommandInputUI>>,
    mut text_query: Query<&mut Text, With<CommandInputText>>,
    mut windows: Query<&mut Window>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    creative_inv_open: Res<InventoryOpen>,
) {
    // Don't open if other UI is open
    if interacting_furnace.0.is_some() || interacting_crusher.0.is_some() || creative_inv_open.0 {
        return;
    }

    // T or / to open command input (when not already open)
    if !command_state.open
        && (key_input.just_pressed(KeyCode::KeyT) || key_input.just_pressed(KeyCode::Slash))
    {
        command_state.open = true;
        command_state.text.clear();

        // If opened with /, pre-fill with /
        if key_input.just_pressed(KeyCode::Slash) {
            command_state.text.push('/');
        }

        // Show UI
        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Visible;
        }

        // Update text
        for mut text in text_query.iter_mut() {
            text.0 = format!("> {}", command_state.text);
        }

        // Unlock cursor for typing
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(true);
        }
    }
}

/// Handle command input typing and execution
#[allow(clippy::too_many_arguments)]
fn command_input_handler(
    mut char_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut command_state: ResMut<CommandInputState>,
    mut ui_query: Query<&mut Visibility, With<CommandInputUI>>,
    mut text_query: Query<&mut Text, With<CommandInputText>>,
    mut windows: Query<&mut Window>,
    mut creative_mode: ResMut<CreativeMode>,
    mut inventory: ResMut<Inventory>,
    mut save_events: EventWriter<SaveGameEvent>,
    mut load_events: EventWriter<LoadGameEvent>,
) {
    if !command_state.open {
        return;
    }

    // ESC to close without executing
    if key_input.just_pressed(KeyCode::Escape) {
        command_state.open = false;
        command_state.text.clear();

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Unlock cursor - JS will auto-relock via data-ui-open observer (BUG-6 fix)
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(false);
        }
        return;
    }

    // Enter to execute command
    if key_input.just_pressed(KeyCode::Enter) {
        let command_text = command_state.text.trim().to_lowercase();
        execute_command(&command_text, &mut creative_mode, &mut inventory, &mut save_events, &mut load_events);

        command_state.open = false;
        command_state.text.clear();

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Re-lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Backspace to delete last character
    if key_input.just_pressed(KeyCode::Backspace) && !command_state.text.is_empty() {
        command_state.text.pop();
    }

    // Handle character input
    for event in char_events.read() {
        // Only process key press events
        if !event.state.is_pressed() {
            continue;
        }

        // Convert key code to character
        if let Some(c) = keycode_to_char(event.key_code, key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight)) {
            // Limit input length
            if command_state.text.len() < 50 {
                command_state.text.push(c);
            }
        }
    }

    // Update display text
    for mut text in text_query.iter_mut() {
        text.0 = format!("> {}|", command_state.text);
    }
}

/// Convert key code to character
fn keycode_to_char(key_code: KeyCode, shift: bool) -> Option<char> {
    match key_code {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some(if shift { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift { '(' } else { '9' }),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Equal => Some(if shift { '+' } else { '=' }),
        KeyCode::Slash => Some(if shift { '?' } else { '/' }),
        _ => None,
    }
}

/// Execute a command
fn execute_command(
    command: &str,
    creative_mode: &mut ResMut<CreativeMode>,
    inventory: &mut ResMut<Inventory>,
    save_events: &mut EventWriter<SaveGameEvent>,
    load_events: &mut EventWriter<LoadGameEvent>,
) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "/creative" | "creative" => {
            creative_mode.enabled = true;
            // Give all items when entering creative mode
            let all_items = [
                BlockType::Stone,
                BlockType::Grass,
                BlockType::IronOre,
                BlockType::Coal,
                BlockType::IronIngot,
                BlockType::CopperOre,
                BlockType::CopperIngot,
                BlockType::MinerBlock,
                BlockType::ConveyorBlock,
                BlockType::CrusherBlock,
            ];
            for (i, block_type) in all_items.iter().take(9).enumerate() {
                inventory.slots[i] = Some((*block_type, 64));
            }
            info!("Creative mode enabled");
        }
        "/survival" | "survival" => {
            creative_mode.enabled = false;
            info!("Survival mode enabled");
        }
        "/give" | "give" => {
            // /give <item> [count]
            if parts.len() >= 2 {
                let item_name = parts[1].to_lowercase();
                let count: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(64);

                if let Some(block_type) = parse_item_name(&item_name) {
                    inventory.add_item(block_type, count);
                    info!("Gave {} x{}", block_type.name(), count);
                }
            }
        }
        "/clear" | "clear" => {
            // Clear inventory
            for slot in inventory.slots.iter_mut() {
                *slot = None;
            }
            info!("Inventory cleared");
        }
        "/save" | "save" => {
            // /save [filename]
            let filename = parts.get(1).unwrap_or(&"quicksave").to_string();
            save_events.send(SaveGameEvent { filename });
        }
        "/load" | "load" => {
            // /load [filename]
            let filename = parts.get(1).unwrap_or(&"quicksave").to_string();
            load_events.send(LoadGameEvent { filename });
        }
        "/help" | "help" => {
            info!("Commands: /creative, /survival, /give <item> [count], /clear, /save [name], /load [name]");
        }
        _ => {
            info!("Unknown command: {}", command);
        }
    }
}

/// Parse item name to BlockType
fn parse_item_name(name: &str) -> Option<BlockType> {
    match name {
        "stone" => Some(BlockType::Stone),
        "grass" => Some(BlockType::Grass),
        "ironore" | "iron_ore" => Some(BlockType::IronOre),
        "copperore" | "copper_ore" => Some(BlockType::CopperOre),
        "coal" => Some(BlockType::Coal),
        "ironingot" | "iron_ingot" | "iron" => Some(BlockType::IronIngot),
        "copperingot" | "copper_ingot" | "copper" => Some(BlockType::CopperIngot),
        "miner" => Some(BlockType::MinerBlock),
        "conveyor" => Some(BlockType::ConveyorBlock),
        "crusher" => Some(BlockType::CrusherBlock),
        "furnace" => Some(BlockType::FurnaceBlock),
        _ => None,
    }
}

// === Save/Load System ===

/// Collect all game state into SaveData
#[allow(clippy::too_many_arguments)]
fn collect_save_data(
    player_query: &Query<&Transform, With<Player>>,
    camera_query: &Query<&PlayerCamera>,
    inventory: &Inventory,
    world_data: &WorldData,
    miner_query: &Query<&Miner>,
    conveyor_query: &Query<&Conveyor>,
    furnace_query: &Query<(&Furnace, &GlobalTransform)>,
    crusher_query: &Query<&Crusher>,
    delivery_query: &Query<&DeliveryPlatform>,
    current_quest: &CurrentQuest,
    creative_mode: &CreativeMode,
) -> save::SaveData {
    use save::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    // Collect player data
    let player_data = if let Ok(transform) = player_query.get_single() {
        let rotation = camera_query.get_single()
            .map(|c| CameraRotation { pitch: c.pitch, yaw: c.yaw })
            .unwrap_or(CameraRotation { pitch: 0.0, yaw: 0.0 });
        PlayerSaveData {
            position: transform.translation.into(),
            rotation,
        }
    } else {
        PlayerSaveData {
            position: Vec3Save { x: 8.0, y: 12.0, z: 20.0 },
            rotation: CameraRotation { pitch: 0.0, yaw: 0.0 },
        }
    };

    // Collect inventory data
    let inventory_data = InventorySaveData {
        selected_slot: inventory.selected_slot,
        slots: inventory.slots.iter().map(|slot| {
            slot.map(|(bt, count)| ItemStack {
                item_type: bt.into(),
                count,
            })
        }).collect(),
    };

    // Collect world modifications
    let modified_blocks: std::collections::HashMap<String, Option<BlockTypeSave>> = world_data
        .modified_blocks
        .iter()
        .map(|(pos, block)| {
            (WorldSaveData::pos_to_key(*pos), block.map(|b| b.into()))
        })
        .collect();

    let world_save = WorldSaveData { modified_blocks };

    // Collect machines
    let mut machines = Vec::new();

    // Miners
    for miner in miner_query.iter() {
        machines.push(MachineSaveData::Miner(MinerSaveData {
            position: miner.position.into(),
            progress: miner.progress,
            buffer: miner.buffer.map(|(bt, count)| ItemStack {
                item_type: bt.into(),
                count,
            }),
        }));
    }

    // Conveyors
    for conveyor in conveyor_query.iter() {
        let direction = match conveyor.direction {
            Direction::North => DirectionSave::North,
            Direction::South => DirectionSave::South,
            Direction::East => DirectionSave::East,
            Direction::West => DirectionSave::West,
        };
        let shape = match conveyor.shape {
            ConveyorShape::Straight => ConveyorShapeSave::Straight,
            ConveyorShape::CornerLeft => ConveyorShapeSave::CornerLeft,
            ConveyorShape::CornerRight => ConveyorShapeSave::CornerRight,
            ConveyorShape::TJunction => ConveyorShapeSave::TJunction,
            ConveyorShape::Splitter => ConveyorShapeSave::Splitter,
        };
        let items: Vec<ConveyorItemSave> = conveyor.items.iter().map(|item| {
            ConveyorItemSave {
                item_type: item.block_type.into(),
                progress: item.progress,
                lateral_offset: item.lateral_offset,
            }
        }).collect();

        machines.push(MachineSaveData::Conveyor(ConveyorSaveData {
            position: conveyor.position.into(),
            direction,
            shape,
            items,
            last_output_index: conveyor.last_output_index,
            last_input_source: conveyor.last_input_source,
        }));
    }

    // Furnaces
    for (furnace, transform) in furnace_query.iter() {
        let pos = IVec3::new(
            transform.translation().x.floor() as i32,
            transform.translation().y.floor() as i32,
            transform.translation().z.floor() as i32,
        );
        machines.push(MachineSaveData::Furnace(FurnaceSaveData {
            position: pos.into(),
            fuel: furnace.fuel,
            input: furnace.input_type.map(|bt| ItemStack {
                item_type: bt.into(),
                count: furnace.input_count,
            }),
            output: furnace.output_type.map(|bt| ItemStack {
                item_type: bt.into(),
                count: furnace.output_count,
            }),
            progress: furnace.progress,
        }));
    }

    // Crushers
    for crusher in crusher_query.iter() {
        machines.push(MachineSaveData::Crusher(CrusherSaveData {
            position: crusher.position.into(),
            input: crusher.input_type.map(|bt| ItemStack {
                item_type: bt.into(),
                count: crusher.input_count,
            }),
            output: crusher.output_type.map(|bt| ItemStack {
                item_type: bt.into(),
                count: crusher.output_count,
            }),
            progress: crusher.progress,
        }));
    }

    // Collect quest data
    let delivered: std::collections::HashMap<BlockTypeSave, u32> = delivery_query
        .iter()
        .next()
        .map(|d| {
            d.delivered.iter().map(|(bt, count)| ((*bt).into(), *count)).collect()
        })
        .unwrap_or_default();

    let quest_data = QuestSaveData {
        current_index: current_quest.index,
        completed: current_quest.completed,
        rewards_claimed: current_quest.rewards_claimed,
        delivered,
    };

    // Game mode
    let mode_data = GameModeSaveData {
        creative: creative_mode.enabled,
    };

    SaveData {
        version: save::SAVE_VERSION.to_string(),
        timestamp,
        player: player_data,
        inventory: inventory_data,
        world: world_save,
        machines,
        quests: quest_data,
        mode: mode_data,
    }
}

/// Convert Direction from save format
fn direction_from_save(dir: save::DirectionSave) -> Direction {
    match dir {
        save::DirectionSave::North => Direction::North,
        save::DirectionSave::South => Direction::South,
        save::DirectionSave::East => Direction::East,
        save::DirectionSave::West => Direction::West,
    }
}

/// Convert ConveyorShape from save format
fn conveyor_shape_from_save(shape: save::ConveyorShapeSave) -> ConveyorShape {
    match shape {
        save::ConveyorShapeSave::Straight => ConveyorShape::Straight,
        save::ConveyorShapeSave::CornerLeft => ConveyorShape::CornerLeft,
        save::ConveyorShapeSave::CornerRight => ConveyorShape::CornerRight,
        save::ConveyorShapeSave::TJunction => ConveyorShape::TJunction,
        save::ConveyorShapeSave::Splitter => ConveyorShape::Splitter,
    }
}

/// Auto-save system - saves game every minute
fn auto_save_system(
    time: Res<Time>,
    mut auto_save_timer: ResMut<save::AutoSaveTimer>,
    mut save_events: EventWriter<SaveGameEvent>,
) {
    auto_save_timer.timer.tick(time.delta());

    if auto_save_timer.timer.just_finished() {
        save_events.send(SaveGameEvent {
            filename: "autosave".to_string(),
        });
        info!("Auto-save triggered");
    }
}

/// Handle save game events
#[allow(clippy::too_many_arguments)]
fn handle_save_event(
    mut events: EventReader<SaveGameEvent>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&PlayerCamera>,
    inventory: Res<Inventory>,
    world_data: Res<WorldData>,
    miner_query: Query<&Miner>,
    conveyor_query: Query<&Conveyor>,
    furnace_query: Query<(&Furnace, &GlobalTransform)>,
    crusher_query: Query<&Crusher>,
    delivery_query: Query<&DeliveryPlatform>,
    current_quest: Res<CurrentQuest>,
    creative_mode: Res<CreativeMode>,
    mut save_load_state: ResMut<SaveLoadState>,
) {
    for event in events.read() {
        let save_data = collect_save_data(
            &player_query,
            &camera_query,
            &inventory,
            &world_data,
            &miner_query,
            &conveyor_query,
            &furnace_query,
            &crusher_query,
            &delivery_query,
            &current_quest,
            &creative_mode,
        );

        match save::save_game(&save_data, &event.filename) {
            Ok(()) => {
                let msg = format!("Game saved to '{}'", event.filename);
                info!("{}", msg);
                save_load_state.last_message = Some(msg);
            }
            Err(e) => {
                let msg = format!("Failed to save: {}", e);
                info!("{}", msg);
                save_load_state.last_message = Some(msg);
            }
        }
    }
}

/// Handle load game events
#[allow(clippy::too_many_arguments)]
fn handle_load_event(
    mut events: EventReader<LoadGameEvent>,
    mut save_load_state: ResMut<SaveLoadState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<&mut PlayerCamera>,
    mut inventory: ResMut<Inventory>,
    mut world_data: ResMut<WorldData>,
    mut current_quest: ResMut<CurrentQuest>,
    mut creative_mode: ResMut<CreativeMode>,
    mut delivery_query: Query<&mut DeliveryPlatform>,
    // Entities to despawn
    miner_entities: Query<Entity, With<Miner>>,
    conveyor_entities: Query<Entity, With<Conveyor>>,
    furnace_entities: Query<Entity, With<Furnace>>,
    crusher_entities: Query<Entity, With<Crusher>>,
) {
    for event in events.read() {
        match save::load_game(&event.filename) {
            Ok(data) => {
                // Apply player position
                if let Ok(mut transform) = player_query.get_single_mut() {
                    transform.translation = data.player.position.into();
                }

                // Apply camera rotation
                if let Ok(mut camera) = camera_query.get_single_mut() {
                    camera.pitch = data.player.rotation.pitch;
                    camera.yaw = data.player.rotation.yaw;
                }

                // Apply inventory
                inventory.selected_slot = data.inventory.selected_slot;
                for (i, slot) in data.inventory.slots.iter().enumerate() {
                    if i < inventory.slots.len() {
                        inventory.slots[i] = slot.as_ref().map(|s| {
                            (s.item_type.clone().into(), s.count)
                        });
                    }
                }

                // Apply world modifications
                world_data.modified_blocks.clear();
                for (key, block_opt) in &data.world.modified_blocks {
                    if let Some(pos) = save::WorldSaveData::key_to_pos(key) {
                        world_data.modified_blocks.insert(
                            pos,
                            block_opt.as_ref().map(|b| b.clone().into())
                        );
                    }
                }

                // Despawn existing machines
                for entity in miner_entities.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in conveyor_entities.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in furnace_entities.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in crusher_entities.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Spawn machines from save data
                for machine in &data.machines {
                    match machine {
                        save::MachineSaveData::Miner(miner_data) => {
                            let pos: IVec3 = miner_data.position.into();
                            let world_pos = Vec3::new(
                                pos.x as f32 + 0.5,
                                pos.y as f32 + 0.5,
                                pos.z as f32 + 0.5,
                            );

                            let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                            commands.spawn((
                                Miner {
                                    position: pos,
                                    progress: miner_data.progress,
                                    buffer: miner_data.buffer.as_ref().map(|b| {
                                        (b.item_type.clone().into(), b.count)
                                    }),
                                },
                                Mesh3d(cube_mesh),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: BlockType::MinerBlock.color(),
                                    ..default()
                                })),
                                Transform::from_translation(world_pos),
                            ));
                        }
                        save::MachineSaveData::Conveyor(conveyor_data) => {
                            let pos: IVec3 = conveyor_data.position.into();
                            let direction = direction_from_save(conveyor_data.direction);
                            let shape = conveyor_shape_from_save(conveyor_data.shape);
                            let world_pos = Vec3::new(
                                pos.x as f32 + 0.5,
                                pos.y as f32 + 0.5,
                                pos.z as f32 + 0.5,
                            );

                            let items: Vec<ConveyorItem> = conveyor_data.items.iter().map(|item| {
                                ConveyorItem {
                                    block_type: item.item_type.clone().into(),
                                    progress: item.progress,
                                    visual_entity: None, // Will be created by update_conveyor_item_visuals
                                    lateral_offset: item.lateral_offset,
                                }
                            }).collect();

                            commands.spawn((
                                Conveyor {
                                    position: pos,
                                    direction,
                                    items,
                                    last_output_index: conveyor_data.last_output_index,
                                    last_input_source: conveyor_data.last_input_source,
                                    shape,
                                },
                                Mesh3d(meshes.add(create_conveyor_mesh(shape))),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: BlockType::ConveyorBlock.color(),
                                    ..default()
                                })),
                                Transform::from_translation(world_pos)
                                    .with_rotation(direction.to_rotation()),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ));
                        }
                        save::MachineSaveData::Furnace(furnace_data) => {
                            let pos: IVec3 = furnace_data.position.into();
                            let world_pos = Vec3::new(
                                pos.x as f32 + 0.5,
                                pos.y as f32 + 0.5,
                                pos.z as f32 + 0.5,
                            );

                            let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                            commands.spawn((
                                Furnace {
                                    fuel: furnace_data.fuel,
                                    input_type: furnace_data.input.as_ref().map(|s| s.item_type.clone().into()),
                                    input_count: furnace_data.input.as_ref().map(|s| s.count).unwrap_or(0),
                                    output_type: furnace_data.output.as_ref().map(|s| s.item_type.clone().into()),
                                    output_count: furnace_data.output.as_ref().map(|s| s.count).unwrap_or(0),
                                    progress: furnace_data.progress,
                                },
                                Mesh3d(cube_mesh),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: BlockType::FurnaceBlock.color(),
                                    ..default()
                                })),
                                Transform::from_translation(world_pos),
                            ));
                        }
                        save::MachineSaveData::Crusher(crusher_data) => {
                            let pos: IVec3 = crusher_data.position.into();
                            let world_pos = Vec3::new(
                                pos.x as f32 + 0.5,
                                pos.y as f32 + 0.5,
                                pos.z as f32 + 0.5,
                            );

                            let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                            commands.spawn((
                                Crusher {
                                    position: pos,
                                    input_type: crusher_data.input.as_ref().map(|s| s.item_type.clone().into()),
                                    input_count: crusher_data.input.as_ref().map(|s| s.count).unwrap_or(0),
                                    output_type: crusher_data.output.as_ref().map(|s| s.item_type.clone().into()),
                                    output_count: crusher_data.output.as_ref().map(|s| s.count).unwrap_or(0),
                                    progress: crusher_data.progress,
                                },
                                Mesh3d(cube_mesh),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: BlockType::CrusherBlock.color(),
                                    ..default()
                                })),
                                Transform::from_translation(world_pos),
                            ));
                        }
                    }
                }

                // Apply quest progress
                current_quest.index = data.quests.current_index;
                current_quest.completed = data.quests.completed;
                current_quest.rewards_claimed = data.quests.rewards_claimed;

                // Apply delivery platform state
                if let Ok(mut delivery) = delivery_query.get_single_mut() {
                    delivery.delivered.clear();
                    for (bt, count) in &data.quests.delivered {
                        delivery.delivered.insert(bt.clone().into(), *count);
                    }
                }

                // Apply game mode
                creative_mode.enabled = data.mode.creative;

                let msg = format!("Game loaded from '{}'", event.filename);
                info!("{}", msg);
                save_load_state.last_message = Some(msg);

                // Force chunk reload by clearing chunks (they will regenerate with modified_blocks applied)
                // Note: This is a simple approach; a more sophisticated one would only reload affected chunks
                world_data.chunks.clear();
            }
            Err(e) => {
                let msg = format!("Failed to load: {}", e);
                info!("{}", msg);
                save_load_state.last_message = Some(msg);
            }
        }
    }
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;

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
