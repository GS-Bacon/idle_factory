//! Idle Factory - Milestone 1: Minimal Voxel Game
//! Goal: Walk, mine blocks, collect in inventory

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::PresentMode;
use bevy::window::CursorGrabMode;
use std::collections::HashMap;
use std::f32::consts::PI;

// === Constants ===
const CHUNK_SIZE: i32 = 16;
const BLOCK_SIZE: f32 = 1.0;
const PLAYER_SPEED: f32 = 5.0;
const REACH_DISTANCE: f32 = 5.0;
const VIEW_DISTANCE: i32 = 2; // How many chunks to load around player (radius)

// Camera settings
const MOUSE_SENSITIVITY: f32 = 0.002; // Balanced sensitivity
const KEY_ROTATION_SPEED: f32 = 2.0; // radians per second for arrow keys

fn main() {
    let mut app = App::new();

    // Configure plugins based on platform
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Native: Disable pipelined rendering for lower input lag
        app.add_plugins((
            DefaultPlugins
                .build()
                .disable::<PipelinedRenderingPlugin>()
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
        // WASM: Use default plugins (no pipelined rendering available)
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Idle Factory".into(),
                    // WASM uses default present mode
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin,
        ));
    }

    app
        .init_resource::<Inventory>()
        .init_resource::<WorldData>()
        .init_resource::<CursorLockState>()
        .init_resource::<InteractingFurnace>()
        .init_resource::<CurrentQuest>()
        .init_resource::<GameFont>()
        .add_systems(Startup, (setup_lighting, setup_player, setup_ui, setup_initial_items, setup_delivery_platform))
        .add_systems(
            Update,
            (
                // Core gameplay systems
                chunk_loading,
                toggle_cursor_lock,
                player_look,
                player_move,
                block_break,
                block_place,
                select_block_type,
                furnace_interact,
                furnace_ui_input,
                furnace_smelting,
            ),
        )
        .add_systems(
            Update,
            (
                // Machine systems
                miner_mining,
                miner_output,
                crusher_processing,
                crusher_output,
                conveyor_transfer,
                update_conveyor_item_visuals,
                delivery_platform_receive,
                quest_progress_check,
                quest_claim_rewards,
                // UI update systems
                update_inventory_ui,
                update_furnace_ui,
                update_delivery_ui,
                update_quest_ui,
                update_window_title_fps,
            ),
        )
        .run();
}

// === Components ===

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerCamera {
    /// Pitch (vertical rotation) in radians
    pitch: f32,
    /// Yaw (horizontal rotation) in radians
    yaw: f32,
}

/// Tracks cursor lock state and handles mouse input for both local and RDP environments
#[derive(Resource, Default)]
struct CursorLockState {
    was_locked: bool,
    skip_frames: u8,
    /// Last mouse position for calculating delta in RDP/absolute mode
    last_mouse_pos: Option<Vec2>,
}

/// Font resource for UI text
#[derive(Resource)]
struct GameFont(Handle<Font>);

impl FromWorld for GameFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        GameFont(asset_server.load("fonts/NotoSansJP-Regular.ttf"))
    }
}


#[derive(Component)]
struct Block {
    /// World position of this block
    position: IVec3,
}

#[derive(Component)]
struct ChunkRef {
    /// Which chunk this entity belongs to
    coord: IVec2,
}

#[derive(Component)]
struct InventoryUI;

#[derive(Component)]
struct InventoryText;

/// Furnace component for smelting
#[derive(Component, Default)]
struct Furnace {
    /// Fuel slot (coal)
    fuel: u32,
    /// Input slot - stores ore type and count
    input_type: Option<BlockType>,
    input_count: u32,
    /// Output slot - stores ingot type and count
    output_type: Option<BlockType>,
    output_count: u32,
    /// Smelting progress (0.0-1.0)
    progress: f32,
}

impl Furnace {
    /// Get smelt output for an ore type
    fn get_smelt_output(ore: BlockType) -> Option<BlockType> {
        match ore {
            BlockType::IronOre => Some(BlockType::IronIngot),
            BlockType::CopperOre => Some(BlockType::CopperIngot),
            _ => None,
        }
    }

    /// Check if this ore type can be added to input (same type or empty)
    fn can_add_input(&self, ore: BlockType) -> bool {
        self.input_type.is_none() || self.input_type == Some(ore)
    }
}

/// Marker for furnace UI
#[derive(Component)]
struct FurnaceUI;

/// Marker for furnace UI text
#[derive(Component)]
struct FurnaceUIText;

/// Currently interacting furnace entity
#[derive(Resource, Default)]
struct InteractingFurnace(Option<Entity>);

/// Miner component - automatically mines blocks below
#[derive(Component)]
struct Miner {
    /// World position of this miner
    position: IVec3,
    /// Mining progress (0.0-1.0)
    progress: f32,
    /// Buffer of mined items (block type, count)
    buffer: Option<(BlockType, u32)>,
}

impl Default for Miner {
    fn default() -> Self {
        Self {
            position: IVec3::ZERO,
            progress: 0.0,
            buffer: None,
        }
    }
}

/// Direction for conveyor belts
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Direction {
    North, // -Z
    South, // +Z
    East,  // +X
    West,  // -X
}

impl Direction {
    fn to_ivec3(self) -> IVec3 {
        match self {
            Direction::North => IVec3::new(0, 0, -1),
            Direction::South => IVec3::new(0, 0, 1),
            Direction::East => IVec3::new(1, 0, 0),
            Direction::West => IVec3::new(-1, 0, 0),
        }
    }

    fn to_rotation(self) -> Quat {
        match self {
            Direction::North => Quat::from_rotation_y(0.0),
            Direction::South => Quat::from_rotation_y(PI),
            Direction::East => Quat::from_rotation_y(-PI / 2.0),
            Direction::West => Quat::from_rotation_y(PI / 2.0),
        }
    }
}

/// Conveyor belt component - moves items in a direction
#[derive(Component)]
struct Conveyor {
    /// World position of this conveyor
    position: IVec3,
    /// Direction items move
    direction: Direction,
    /// Item currently on this conveyor (block type)
    item: Option<BlockType>,
    /// Transfer progress (0.0-1.0)
    progress: f32,
    /// Entity for the visual item on conveyor
    item_visual: Option<Entity>,
}

/// Marker for conveyor item visual
#[derive(Component)]
struct ConveyorItemVisual;

/// Crusher component - doubles ore output
#[derive(Component)]
struct Crusher {
    /// World position of this crusher
    position: IVec3,
    /// Input ore type and count
    input_type: Option<BlockType>,
    input_count: u32,
    /// Output ore type and count (doubled)
    output_type: Option<BlockType>,
    output_count: u32,
    /// Processing progress (0.0-1.0)
    progress: f32,
}

impl Crusher {
    /// Check if this ore can be crushed
    fn can_crush(ore: BlockType) -> bool {
        matches!(ore, BlockType::IronOre | BlockType::CopperOre)
    }

    /// Check if this ore type can be added to input (same type or empty)
    fn can_add_input(&self, ore: BlockType) -> bool {
        Self::can_crush(ore) && (self.input_type.is_none() || self.input_type == Some(ore))
    }
}

/// Delivery platform - accepts items for delivery quests
#[derive(Component, Default)]
struct DeliveryPlatform {
    /// Total items delivered (by type)
    delivered: HashMap<BlockType, u32>,
}

/// Marker for delivery platform UI
#[derive(Component)]
struct DeliveryUI;

/// Marker for delivery UI text
#[derive(Component)]
struct DeliveryUIText;

/// Quest definition
#[derive(Clone, Debug)]
struct QuestDef {
    /// Quest description
    description: &'static str,
    /// Required item type
    required_item: BlockType,
    /// Required amount
    required_amount: u32,
    /// Rewards: (BlockType, amount)
    rewards: Vec<(BlockType, u32)>,
}

/// Current quest state
#[derive(Resource, Default)]
struct CurrentQuest {
    /// Index of current quest (0-based)
    index: usize,
    /// Whether the quest is completed
    completed: bool,
    /// Whether rewards were claimed
    rewards_claimed: bool,
}

/// Marker for quest UI
#[derive(Component)]
struct QuestUI;

/// Marker for quest UI text
#[derive(Component)]
struct QuestUIText;


// === Resources ===

#[derive(Resource, Default)]
struct Inventory {
    items: HashMap<BlockType, u32>,
    /// Currently selected block type for placement
    selected: Option<BlockType>,
}

/// Single chunk data - blocks stored with local coordinates (0..CHUNK_SIZE)
#[derive(Clone)]
struct ChunkData {
    blocks: HashMap<IVec3, BlockType>,
}

impl ChunkData {
    /// Generate a chunk at the given chunk coordinate
    fn generate(chunk_coord: IVec2) -> Self {
        let mut blocks = HashMap::new();
        // Generate a 16x16x8 chunk of blocks
        // Bottom layers are stone with ore veins, top layer is grass
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..8 {
                    let block_type = if y == 7 {
                        BlockType::Grass
                    } else {
                        // Use simple hash for ore distribution
                        let world_x = chunk_coord.x * CHUNK_SIZE + x;
                        let world_z = chunk_coord.y * CHUNK_SIZE + z;
                        let hash = Self::simple_hash(world_x, y, world_z);

                        if y <= 4 && hash % 20 == 0 {
                            // Iron ore: 5% chance at y=0-4
                            BlockType::IronOre
                        } else if y <= 3 && hash % 25 == 1 {
                            // Copper ore: 4% chance at y=0-3
                            BlockType::CopperOre
                        } else if y <= 5 && hash % 15 == 2 {
                            // Coal: ~7% chance at y=0-5
                            BlockType::Coal
                        } else {
                            BlockType::Stone
                        }
                    };
                    blocks.insert(IVec3::new(x, y, z), block_type);
                }
            }
        }
        Self { blocks }
    }

    /// Simple hash function for deterministic ore generation
    fn simple_hash(x: i32, y: i32, z: i32) -> u32 {
        let mut h = (x as u32).wrapping_mul(374761393);
        h = h.wrapping_add((y as u32).wrapping_mul(668265263));
        h = h.wrapping_add((z as u32).wrapping_mul(2147483647));
        h ^= h >> 13;
        h = h.wrapping_mul(1274126177);
        h ^= h >> 16;
        h
    }
}

/// World data - manages multiple chunks
#[derive(Resource, Default)]
struct WorldData {
    /// Loaded chunks indexed by chunk coordinate
    chunks: HashMap<IVec2, ChunkData>,
    /// Block entities for each chunk (for despawning)
    chunk_entities: HashMap<IVec2, Vec<Entity>>,
}

impl WorldData {
    /// Convert world position to chunk coordinate
    fn world_to_chunk(world_pos: IVec3) -> IVec2 {
        IVec2::new(
            world_pos.x.div_euclid(CHUNK_SIZE),
            world_pos.z.div_euclid(CHUNK_SIZE),
        )
    }

    /// Convert world position to local chunk position
    fn world_to_local(world_pos: IVec3) -> IVec3 {
        IVec3::new(
            world_pos.x.rem_euclid(CHUNK_SIZE),
            world_pos.y,
            world_pos.z.rem_euclid(CHUNK_SIZE),
        )
    }

    /// Convert chunk coord + local pos to world position
    fn local_to_world(chunk_coord: IVec2, local_pos: IVec3) -> IVec3 {
        IVec3::new(
            chunk_coord.x * CHUNK_SIZE + local_pos.x,
            local_pos.y,
            chunk_coord.y * CHUNK_SIZE + local_pos.z,
        )
    }

    /// Get block at world position
    fn get_block(&self, world_pos: IVec3) -> Option<&BlockType> {
        let chunk_coord = Self::world_to_chunk(world_pos);
        let local_pos = Self::world_to_local(world_pos);
        self.chunks.get(&chunk_coord)?.blocks.get(&local_pos)
    }

    /// Set block at world position
    fn set_block(&mut self, world_pos: IVec3, block_type: BlockType) {
        let chunk_coord = Self::world_to_chunk(world_pos);
        let local_pos = Self::world_to_local(world_pos);
        if let Some(chunk) = self.chunks.get_mut(&chunk_coord) {
            chunk.blocks.insert(local_pos, block_type);
        }
    }

    /// Remove block at world position, returns the removed block type
    fn remove_block(&mut self, world_pos: IVec3) -> Option<BlockType> {
        let chunk_coord = Self::world_to_chunk(world_pos);
        let local_pos = Self::world_to_local(world_pos);
        self.chunks.get_mut(&chunk_coord)?.blocks.remove(&local_pos)
    }

    /// Check if block exists at world position
    fn has_block(&self, world_pos: IVec3) -> bool {
        self.get_block(world_pos).is_some()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum BlockType {
    Stone,
    Grass,
    IronOre,
    Coal,
    IronIngot,
    MinerBlock,
    ConveyorBlock,
    CopperOre,
    CopperIngot,
    CrusherBlock,
}

impl BlockType {
    fn color(&self) -> Color {
        match self {
            BlockType::Stone => Color::srgb(0.5, 0.5, 0.5),
            BlockType::Grass => Color::srgb(0.2, 0.8, 0.2),
            BlockType::IronOre => Color::srgb(0.6, 0.5, 0.4),
            BlockType::Coal => Color::srgb(0.15, 0.15, 0.15),
            BlockType::IronIngot => Color::srgb(0.8, 0.8, 0.85),
            BlockType::MinerBlock => Color::srgb(0.8, 0.6, 0.2),
            BlockType::ConveyorBlock => Color::srgb(0.3, 0.3, 0.35),
            BlockType::CopperOre => Color::srgb(0.7, 0.4, 0.3),
            BlockType::CopperIngot => Color::srgb(0.9, 0.5, 0.3),
            BlockType::CrusherBlock => Color::srgb(0.4, 0.3, 0.5),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            BlockType::Stone => "Stone",
            BlockType::Grass => "Grass",
            BlockType::IronOre => "Iron Ore",
            BlockType::Coal => "Coal",
            BlockType::IronIngot => "Iron Ingot",
            BlockType::MinerBlock => "Miner",
            BlockType::ConveyorBlock => "Conveyor",
            BlockType::CopperOre => "Copper Ore",
            BlockType::CopperIngot => "Copper Ingot",
            BlockType::CrusherBlock => "Crusher",
        }
    }

    /// Returns true if this block type is a machine (not a regular block)
    fn is_machine(&self) -> bool {
        matches!(self, BlockType::MinerBlock | BlockType::ConveyorBlock | BlockType::CrusherBlock)
    }
}

// === Setup Systems ===

fn setup_lighting(mut commands: Commands) {
    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });
}

/// Load/unload chunks around the player
fn chunk_loading(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_data: ResMut<WorldData>,
    player_query: Query<&Transform, With<Player>>,
    chunk_entities_query: Query<(Entity, &ChunkRef)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    // Get player's chunk coordinate
    let player_world_pos = IVec3::new(
        player_transform.translation.x.floor() as i32,
        0,
        player_transform.translation.z.floor() as i32,
    );
    let player_chunk = WorldData::world_to_chunk(player_world_pos);

    // Calculate which chunks should be loaded
    let mut chunks_to_load: Vec<IVec2> = Vec::new();
    for dx in -VIEW_DISTANCE..=VIEW_DISTANCE {
        for dz in -VIEW_DISTANCE..=VIEW_DISTANCE {
            let chunk_coord = IVec2::new(player_chunk.x + dx, player_chunk.y + dz);
            if !world_data.chunks.contains_key(&chunk_coord) {
                chunks_to_load.push(chunk_coord);
            }
        }
    }

    // Calculate which chunks should be unloaded
    let mut chunks_to_unload: Vec<IVec2> = Vec::new();
    for &chunk_coord in world_data.chunks.keys() {
        let dx = (chunk_coord.x - player_chunk.x).abs();
        let dz = (chunk_coord.y - player_chunk.y).abs();
        if dx > VIEW_DISTANCE || dz > VIEW_DISTANCE {
            chunks_to_unload.push(chunk_coord);
        }
    }

    // Unload chunks
    for chunk_coord in chunks_to_unload {
        // Despawn all entities in this chunk
        for (entity, chunk_ref) in chunk_entities_query.iter() {
            if chunk_ref.coord == chunk_coord {
                commands.entity(entity).despawn();
            }
        }
        world_data.chunks.remove(&chunk_coord);
        world_data.chunk_entities.remove(&chunk_coord);
    }

    // Load chunks (limit to 1 per frame for performance)
    if let Some(chunk_coord) = chunks_to_load.first() {
        let chunk_coord = *chunk_coord;
        let chunk_data = ChunkData::generate(chunk_coord);

        // Spawn blocks for this chunk
        let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
        let mut entities = Vec::new();

        for (&local_pos, &block_type) in chunk_data.blocks.iter() {
            let world_pos = WorldData::local_to_world(chunk_coord, local_pos);
            let material = materials.add(StandardMaterial {
                base_color: block_type.color(),
                ..default()
            });

            let entity = commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(
                    world_pos.x as f32 * BLOCK_SIZE,
                    world_pos.y as f32 * BLOCK_SIZE,
                    world_pos.z as f32 * BLOCK_SIZE,
                )),
                Block { position: world_pos },
                ChunkRef { coord: chunk_coord },
            )).id();

            entities.push(entity);
        }

        world_data.chunks.insert(chunk_coord, chunk_data);
        world_data.chunk_entities.insert(chunk_coord, entities);
    }
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

fn setup_ui(mut commands: Commands) {
    // Inventory UI panel
    commands
        .spawn((
            InventoryUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                InventoryText,
                Text::new("Inventory: Empty\nWASD:Move Mouse:Look"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

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

    // Furnace UI panel (hidden by default)
    commands
        .spawn((
            FurnaceUI,
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
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                FurnaceUIText,
                Text::new("=== Furnace ===\nFuel: 0 Coal\nInput: 0 Iron Ore\nOutput: 0 Iron Ingot\n\n[1] Add Coal | [2] Add Iron Ore\n[3] Take Iron Ingot | [E] Close"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
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
}

/// Setup initial items on ground and furnace
fn setup_initial_items(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut inventory: ResMut<Inventory>,
) {
    // Give player initial items
    inventory.items.insert(BlockType::IronOre, 5);
    inventory.items.insert(BlockType::Coal, 5);
    inventory.items.insert(BlockType::MinerBlock, 3);
    inventory.items.insert(BlockType::ConveyorBlock, 10);
    inventory.items.insert(BlockType::CrusherBlock, 2);
    inventory.selected = Some(BlockType::MinerBlock);

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
            furnace_pos.x as f32 * BLOCK_SIZE,
            furnace_pos.y as f32 * BLOCK_SIZE,
            furnace_pos.z as f32 * BLOCK_SIZE,
        )),
        Furnace::default(),
    ));

    // === Demo: Miner + Conveyor chain ===
    // Layout (top view, Y=8):
    //   [Miner] -> [Conv] -> [Conv] -> [Furnace]
    //   x=5        x=6       x=7       x=8
    //   z=15       z=15      z=15      z=15

    // Spawn Miner at (5, 8, 15) - sits on top of grass, mines stone below
    let miner_pos = IVec3::new(5, 8, 15);
    let miner_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.6, 0.2), // Orange for miner
        ..default()
    });
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(miner_material),
        Transform::from_translation(Vec3::new(
            miner_pos.x as f32 * BLOCK_SIZE,
            miner_pos.y as f32 * BLOCK_SIZE,
            miner_pos.z as f32 * BLOCK_SIZE,
        )),
        Miner {
            position: miner_pos,
            ..default()
        },
    ));

    // Spawn conveyor belt chain (flat boxes)
    let conveyor_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE * 0.3, BLOCK_SIZE));
    let conveyor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.35), // Dark gray for conveyor
        ..default()
    });

    // Conveyor 1: next to miner, heading East
    let conv1_pos = IVec3::new(6, 8, 15);
    commands.spawn((
        Mesh3d(conveyor_mesh.clone()),
        MeshMaterial3d(conveyor_material.clone()),
        Transform::from_translation(Vec3::new(
            conv1_pos.x as f32 * BLOCK_SIZE,
            conv1_pos.y as f32 * BLOCK_SIZE - 0.35, // Slightly lower
            conv1_pos.z as f32 * BLOCK_SIZE,
        )).with_rotation(Direction::East.to_rotation()),
        Conveyor {
            position: conv1_pos,
            direction: Direction::East,
            item: None,
            progress: 0.0,
            item_visual: None,
        },
    ));

    // Conveyor 2: continuing East
    let conv2_pos = IVec3::new(7, 8, 15);
    commands.spawn((
        Mesh3d(conveyor_mesh.clone()),
        MeshMaterial3d(conveyor_material.clone()),
        Transform::from_translation(Vec3::new(
            conv2_pos.x as f32 * BLOCK_SIZE,
            conv2_pos.y as f32 * BLOCK_SIZE - 0.35,
            conv2_pos.z as f32 * BLOCK_SIZE,
        )).with_rotation(Direction::East.to_rotation()),
        Conveyor {
            position: conv2_pos,
            direction: Direction::East,
            item: None,
            progress: 0.0,
            item_visual: None,
        },
    ));

    // Spawn a second furnace at end of conveyor chain
    let furnace2_pos = IVec3::new(8, 8, 15);
    let furnace2_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.3, 0.3),
        ..default()
    });
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(furnace2_material),
        Transform::from_translation(Vec3::new(
            furnace2_pos.x as f32 * BLOCK_SIZE,
            furnace2_pos.y as f32 * BLOCK_SIZE,
            furnace2_pos.z as f32 * BLOCK_SIZE,
        )),
        Furnace::default(),
    ));
}

// === Update Systems ===

/// Toggle cursor lock with Escape key
fn toggle_cursor_lock(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();

    // Escape to unlock cursor
    if key_input.just_pressed(KeyCode::Escape) {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }

    // Click to lock cursor (when not locked)
    if mouse_button.just_pressed(MouseButton::Left)
        && window.cursor_options.grab_mode == CursorGrabMode::None
    {
        // Use Locked mode - it properly captures relative mouse motion
        // Confined mode causes issues where mouse hits window edge and spins
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

fn player_look(
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut windows: Query<&mut Window>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut cursor_lock_state: ResMut<CursorLockState>,
    interacting_furnace: Res<InteractingFurnace>,
) {
    // Don't look around while furnace UI is open
    if interacting_furnace.0.is_some() {
        return;
    }

    let mut window = windows.single_mut();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Get camera component
    let Ok((mut camera_transform, mut camera)) = camera_query.get_single_mut() else {
        return;
    };
    let Ok(mut player_transform) = player_query.get_single_mut() else {
        return;
    };

    // Pitch limit to prevent gimbal lock (±89 degrees)
    const PITCH_LIMIT: f32 = 1.54; // ~88 degrees in radians

    // --- Arrow keys for camera control (always works, time-based) ---
    if key_input.pressed(KeyCode::ArrowLeft) {
        camera.yaw += KEY_ROTATION_SPEED * time.delta_secs();
    }
    if key_input.pressed(KeyCode::ArrowRight) {
        camera.yaw -= KEY_ROTATION_SPEED * time.delta_secs();
    }
    if key_input.pressed(KeyCode::ArrowUp) {
        camera.pitch += KEY_ROTATION_SPEED * time.delta_secs();
    }
    if key_input.pressed(KeyCode::ArrowDown) {
        camera.pitch -= KEY_ROTATION_SPEED * time.delta_secs();
    }

    // --- Track cursor lock state changes ---
    if cursor_locked && !cursor_lock_state.was_locked {
        // Just became locked - reset state
        cursor_lock_state.skip_frames = 2;
        cursor_lock_state.last_mouse_pos = None;
    }
    if !cursor_locked {
        cursor_lock_state.last_mouse_pos = None;
    }
    cursor_lock_state.was_locked = cursor_locked;

    // --- Mouse motion ---
    // Try AccumulatedMouseMotion first (works on local/native)
    // Fall back to cursor position delta (works on RDP)
    if cursor_locked && cursor_lock_state.skip_frames == 0 {
        let raw_delta = accumulated_mouse_motion.delta;

        // Check if AccumulatedMouseMotion gives reasonable values
        // RDP often reports huge values (>1000) due to absolute coordinates
        const MAX_REASONABLE_DELTA: f32 = 200.0;

        if raw_delta.x.abs() < MAX_REASONABLE_DELTA && raw_delta.y.abs() < MAX_REASONABLE_DELTA {
            // Native mode - use raw delta directly
            camera.yaw -= raw_delta.x * MOUSE_SENSITIVITY;
            camera.pitch -= raw_delta.y * MOUSE_SENSITIVITY;
        } else if let Some(cursor_pos) = window.cursor_position() {
            // RDP/Confined mode - calculate delta from cursor position
            let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);

            if let Some(last_pos) = cursor_lock_state.last_mouse_pos {
                let delta = cursor_pos - last_pos;
                // Only apply if delta is reasonable and non-trivial
                if delta.length() < MAX_REASONABLE_DELTA && delta.length() > 0.5 {
                    camera.yaw -= delta.x * MOUSE_SENSITIVITY;
                    camera.pitch -= delta.y * MOUSE_SENSITIVITY;
                }
            }

            // Re-center cursor only when it gets far from center
            // Reduces overhead from constant set_cursor_position calls
            let dist_from_center = (cursor_pos - center).length();
            if dist_from_center > 100.0 {
                window.set_cursor_position(Some(center));
                cursor_lock_state.last_mouse_pos = Some(center);
            } else {
                cursor_lock_state.last_mouse_pos = Some(cursor_pos);
            }
        }
    }

    // Decrement skip counter
    if cursor_lock_state.skip_frames > 0 {
        cursor_lock_state.skip_frames -= 1;
    }

    // Clamp pitch
    camera.pitch = camera.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

    // --- Apply rotation (YXZ order to prevent roll) ---
    // Player rotates horizontally (yaw only)
    player_transform.rotation = Quat::from_rotation_y(camera.yaw);

    // Camera rotates vertically (pitch) relative to player
    camera_transform.rotation = Quat::from_rotation_x(camera.pitch);
}

fn player_move(
    time: Res<Time>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    camera_query: Query<&PlayerCamera>,
    interacting_furnace: Res<InteractingFurnace>,
) {
    // Don't move while furnace UI is open
    if interacting_furnace.0.is_some() {
        return;
    }

    let Ok(mut player_transform) = player_query.get_single_mut() else {
        return;
    };
    let Ok(camera) = camera_query.get_single() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    // Calculate forward/right from yaw (more stable than transform.forward())
    let (sin_yaw, cos_yaw) = camera.yaw.sin_cos();
    let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
    let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);

    if key_input.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if key_input.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if key_input.pressed(KeyCode::KeyA) {
        direction -= right;
    }
    if key_input.pressed(KeyCode::KeyD) {
        direction += right;
    }
    if key_input.pressed(KeyCode::Space) {
        direction.y += 1.0;
    }
    if key_input.pressed(KeyCode::ShiftLeft) {
        direction.y -= 1.0;
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
        player_transform.translation += direction * PLAYER_SPEED * time.delta_secs();
    }
}

fn block_break(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &PlayerCamera)>,
    block_query: Query<(Entity, &Block, &GlobalTransform)>,
    conveyor_query: Query<(Entity, &Conveyor, &GlobalTransform)>,
    miner_query: Query<(Entity, &Miner, &GlobalTransform)>,
    mut world_data: ResMut<WorldData>,
    mut inventory: ResMut<Inventory>,
    windows: Query<&Window>,
    interacting_furnace: Res<InteractingFurnace>,
    item_visual_query: Query<Entity, With<ConveyorItemVisual>>,
) {
    // Only break blocks when cursor is locked (to distinguish from lock-click)
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Don't break blocks while furnace UI is open
    if interacting_furnace.0.is_some() {
        return;
    }

    if !cursor_locked || !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((camera_transform, _camera)) = camera_query.get_single() else {
        return;
    };

    // Calculate ray from camera using its actual transform
    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Track what we hit (block, conveyor, or miner)
    enum HitType {
        Block(Entity, IVec3),
        Conveyor(Entity, Option<BlockType>, Option<Entity>), // entity, item, item_visual
        Miner(Entity),
    }
    let mut closest_hit: Option<(HitType, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    // Check blocks
    for (entity, block, block_transform) in block_query.iter() {
        let block_pos = block_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            block_pos - Vec3::splat(half_size),
            block_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0
                && t < REACH_DISTANCE
                && (closest_hit.is_none() || t < closest_hit.as_ref().unwrap().1)
            {
                closest_hit = Some((HitType::Block(entity, block.position), t));
            }
        }
    }

    // Check conveyors
    for (entity, conveyor, conveyor_transform) in conveyor_query.iter() {
        let conveyor_pos = conveyor_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            conveyor_pos - Vec3::new(half_size, 0.15, half_size),
            conveyor_pos + Vec3::new(half_size, 0.15, half_size),
        ) {
            if t > 0.0
                && t < REACH_DISTANCE
                && (closest_hit.is_none() || t < closest_hit.as_ref().unwrap().1)
            {
                closest_hit = Some((HitType::Conveyor(entity, conveyor.item, conveyor.item_visual), t));
            }
        }
    }

    // Check miners
    for (entity, _miner, miner_transform) in miner_query.iter() {
        let miner_pos = miner_transform.translation();
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            miner_pos - Vec3::splat(half_size),
            miner_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0
                && t < REACH_DISTANCE
                && (closest_hit.is_none() || t < closest_hit.as_ref().unwrap().1)
            {
                closest_hit = Some((HitType::Miner(entity), t));
            }
        }
    }

    // Handle the hit
    if let Some((hit_type, _)) = closest_hit {
        match hit_type {
            HitType::Block(entity, pos) => {
                if let Some(block_type) = world_data.remove_block(pos) {
                    commands.entity(entity).despawn();
                    *inventory.items.entry(block_type).or_insert(0) += 1;
                    if inventory.selected.is_none() {
                        inventory.selected = Some(block_type);
                    }
                }
            }
            HitType::Conveyor(entity, item, item_visual) => {
                commands.entity(entity).despawn();
                // Also despawn item visual if present
                if let Some(visual_entity) = item_visual {
                    if item_visual_query.get(visual_entity).is_ok() {
                        commands.entity(visual_entity).despawn();
                    }
                }
                // Return conveyor to inventory
                *inventory.items.entry(BlockType::ConveyorBlock).or_insert(0) += 1;
                // Also drop any item on the conveyor
                if let Some(item_type) = item {
                    *inventory.items.entry(item_type).or_insert(0) += 1;
                }
            }
            HitType::Miner(entity) => {
                commands.entity(entity).despawn();
                // Return miner to inventory
                *inventory.items.entry(BlockType::MinerBlock).or_insert(0) += 1;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn block_place(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &PlayerCamera)>,
    block_query: Query<(&Block, &GlobalTransform)>,
    mut world_data: ResMut<WorldData>,
    mut inventory: ResMut<Inventory>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    if !cursor_locked || !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    // Check if we have a selected block type with items
    let Some(selected_type) = inventory.selected else {
        return;
    };
    let Some(&count) = inventory.items.get(&selected_type) else {
        return;
    };
    if count == 0 {
        return;
    }

    let Ok((camera_transform, player_camera)) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Find closest block intersection with hit normal
    let mut closest_hit: Option<(IVec3, Vec3, f32)> = None;

    for (block, block_transform) in block_query.iter() {
        let block_pos = block_transform.translation();
        let half_size = BLOCK_SIZE / 2.0;

        if let Some((t, normal)) = ray_aabb_intersection_with_normal(
            ray_origin,
            ray_direction,
            block_pos - Vec3::splat(half_size),
            block_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0
                && t < REACH_DISTANCE
                && (closest_hit.is_none() || t < closest_hit.unwrap().2)
            {
                closest_hit = Some((block.position, normal, t));
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

        // Don't place if already occupied
        if world_data.has_block(place_pos) {
            return;
        }

        // Consume from inventory
        if let Some(count) = inventory.items.get_mut(&selected_type) {
            *count -= 1;
            if *count == 0 {
                inventory.items.remove(&selected_type);
                // Select next available block type
                inventory.selected = inventory.items.keys().next().copied();
            }
        }

        // Add to world data (only for non-machine blocks)
        if !selected_type.is_machine() {
            world_data.set_block(place_pos, selected_type);
        }

        // Get chunk coord for the placed block
        let chunk_coord = WorldData::world_to_chunk(place_pos);

        // Calculate direction from player yaw for conveyors
        let facing_direction = yaw_to_direction(player_camera.yaw);

        // Spawn entity based on block type
        match selected_type {
            BlockType::MinerBlock => {
                let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                let material = materials.add(StandardMaterial {
                    base_color: selected_type.color(),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(cube_mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE,
                        place_pos.y as f32 * BLOCK_SIZE,
                        place_pos.z as f32 * BLOCK_SIZE,
                    )),
                    Miner {
                        position: place_pos,
                        ..default()
                    },
                ));
            }
            BlockType::ConveyorBlock => {
                let conveyor_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE * 0.3, BLOCK_SIZE));
                let material = materials.add(StandardMaterial {
                    base_color: selected_type.color(),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(conveyor_mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE,
                        place_pos.y as f32 * BLOCK_SIZE - 0.35,
                        place_pos.z as f32 * BLOCK_SIZE,
                    )).with_rotation(facing_direction.to_rotation()),
                    Conveyor {
                        position: place_pos,
                        direction: facing_direction,
                        item: None,
                        progress: 0.0,
                        item_visual: None,
                    },
                ));
            }
            BlockType::CrusherBlock => {
                let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                let material = materials.add(StandardMaterial {
                    base_color: selected_type.color(),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(cube_mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE,
                        place_pos.y as f32 * BLOCK_SIZE,
                        place_pos.z as f32 * BLOCK_SIZE,
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
            _ => {
                // Regular block
                let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                let material = materials.add(StandardMaterial {
                    base_color: selected_type.color(),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(cube_mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        place_pos.x as f32 * BLOCK_SIZE,
                        place_pos.y as f32 * BLOCK_SIZE,
                        place_pos.z as f32 * BLOCK_SIZE,
                    )),
                    Block { position: place_pos },
                    ChunkRef { coord: chunk_coord },
                ));
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

/// Select block type with number keys (1, 2) or scroll wheel
fn select_block_type(
    key_input: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
) {
    // Get available block types from inventory
    let available: Vec<BlockType> = inventory.items.keys().copied().collect();
    if available.is_empty() {
        return;
    }

    // Number keys to select specific types
    if key_input.just_pressed(KeyCode::Digit1) {
        if let Some(&block_type) = available.first() {
            inventory.selected = Some(block_type);
        }
    }
    if key_input.just_pressed(KeyCode::Digit2) {
        if let Some(&block_type) = available.get(1) {
            inventory.selected = Some(block_type);
        }
    }

    // Ensure selected is valid
    if let Some(selected) = inventory.selected {
        if !inventory.items.contains_key(&selected) {
            inventory.selected = available.first().copied();
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

fn update_inventory_ui(inventory: Res<Inventory>, mut query: Query<&mut Text, With<InventoryText>>) {
    if !inventory.is_changed() {
        return;
    }

    let Ok(mut text) = query.get_single_mut() else {
        return;
    };

    let hint = "LClick:Break | RClick:Place | 1-4:Select | E:Furnace";

    if inventory.items.is_empty() {
        **text = format!("Inventory: Empty\n{}", hint);
    } else {
        let selected_name = inventory.selected.map(|b| b.name()).unwrap_or("None");
        let items: Vec<String> = inventory
            .items
            .iter()
            .enumerate()
            .map(|(i, (block_type, count))| {
                let marker = if Some(*block_type) == inventory.selected { ">" } else { " " };
                format!("{} [{}] {}: {}", marker, i + 1, block_type.name(), count)
            })
            .collect();
        **text = format!("Selected: {}\n{}\n{}", selected_name, items.join("\n"), hint);
    }
}

/// Interact with furnace when looking at it and pressing E
fn furnace_interact(
    key_input: Res<ButtonInput<KeyCode>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    furnace_query: Query<(Entity, &Transform), With<Furnace>>,
    mut interacting: ResMut<InteractingFurnace>,
    mut furnace_ui_query: Query<&mut Visibility, With<FurnaceUI>>,
    mut windows: Query<&mut Window>,
) {
    // E key or ESC to toggle furnace UI
    let e_pressed = key_input.just_pressed(KeyCode::KeyE);
    let esc_pressed = key_input.just_pressed(KeyCode::Escape);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        if let Ok(mut vis) = furnace_ui_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        // Re-lock cursor when closing UI
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
        return;
    }

    // Only open furnace UI with E key
    if !e_pressed {
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
            if t > 0.0 && t < REACH_DISTANCE && (closest_furnace.is_none() || t < closest_furnace.unwrap().1) {
                closest_furnace = Some((entity, t));
            }
        }
    }

    // Open furnace UI
    if let Some((entity, _)) = closest_furnace {
        interacting.0 = Some(entity);
        if let Ok(mut vis) = furnace_ui_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
    }
}

/// Handle input when furnace UI is open
fn furnace_ui_input(
    key_input: Res<ButtonInput<KeyCode>>,
    interacting: Res<InteractingFurnace>,
    mut furnace_query: Query<&mut Furnace>,
    mut inventory: ResMut<Inventory>,
) {
    let Some(furnace_entity) = interacting.0 else {
        return;
    };

    let Ok(mut furnace) = furnace_query.get_mut(furnace_entity) else {
        return;
    };

    // [1] Add coal to furnace
    if key_input.just_pressed(KeyCode::Digit1) {
        if let Some(count) = inventory.items.get_mut(&BlockType::Coal) {
            if *count > 0 {
                *count -= 1;
                furnace.fuel += 1;
                if *count == 0 {
                    inventory.items.remove(&BlockType::Coal);
                }
            }
        }
    }

    // [2] Add iron ore to furnace
    if key_input.just_pressed(KeyCode::Digit2) {
        if let Some(count) = inventory.items.get_mut(&BlockType::IronOre) {
            if *count > 0 && furnace.can_add_input(BlockType::IronOre) {
                *count -= 1;
                furnace.input_type = Some(BlockType::IronOre);
                furnace.input_count += 1;
                if *count == 0 {
                    inventory.items.remove(&BlockType::IronOre);
                }
            }
        }
    }

    // [3] Add copper ore to furnace
    if key_input.just_pressed(KeyCode::Digit3) {
        if let Some(count) = inventory.items.get_mut(&BlockType::CopperOre) {
            if *count > 0 && furnace.can_add_input(BlockType::CopperOre) {
                *count -= 1;
                furnace.input_type = Some(BlockType::CopperOre);
                furnace.input_count += 1;
                if *count == 0 {
                    inventory.items.remove(&BlockType::CopperOre);
                }
            }
        }
    }

    // [4] Take output from furnace
    if key_input.just_pressed(KeyCode::Digit4) && furnace.output_count > 0 {
        if let Some(output_type) = furnace.output_type {
            furnace.output_count -= 1;
            *inventory.items.entry(output_type).or_insert(0) += 1;
            if furnace.output_count == 0 {
                furnace.output_type = None;
            }
        }
    }
}

/// Smelting logic - convert ore + coal to ingot
const SMELT_TIME: f32 = 3.0; // seconds to smelt one item

fn furnace_smelting(
    time: Res<Time>,
    mut furnace_query: Query<&mut Furnace>,
) {
    for mut furnace in furnace_query.iter_mut() {
        // Need fuel, input ore, and valid recipe to smelt
        let can_smelt = furnace.fuel > 0
            && furnace.input_count > 0
            && furnace.input_type.is_some();

        if can_smelt {
            let input_ore = furnace.input_type.unwrap();
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
        } else {
            // Reset progress if missing fuel or input
            furnace.progress = 0.0;
        }
    }
}

/// Crusher processing - doubles ore
const CRUSH_TIME: f32 = 4.0; // seconds to crush one ore

fn crusher_processing(
    time: Res<Time>,
    mut crusher_query: Query<&mut Crusher>,
) {
    for mut crusher in crusher_query.iter_mut() {
        // Need input ore to process
        if crusher.input_count > 0 && crusher.input_type.is_some() {
            let input_ore = crusher.input_type.unwrap();

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
        } else {
            crusher.progress = 0.0;
        }
    }
}

// === Miner & Conveyor Systems ===

const MINE_TIME: f32 = 5.0; // seconds to mine one block
const CONVEYOR_SPEED: f32 = 1.0; // seconds to transfer item

/// Mining logic - automatically mine blocks below the miner
fn miner_mining(
    time: Res<Time>,
    mut commands: Commands,
    mut miner_query: Query<&mut Miner>,
    mut world_data: ResMut<WorldData>,
    block_query: Query<(Entity, &Block)>,
) {
    for mut miner in miner_query.iter_mut() {
        // Skip if buffer is full
        if let Some((_, count)) = miner.buffer {
            if count >= 64 {
                continue;
            }
        }

        // Find block below miner
        let below_pos = miner.position + IVec3::new(0, -1, 0);
        let Some(&block_type) = world_data.get_block(below_pos) else {
            miner.progress = 0.0;
            continue;
        };

        // Mine progress
        miner.progress += time.delta_secs() / MINE_TIME;

        if miner.progress >= 1.0 {
            miner.progress = 0.0;

            // Remove block from world
            world_data.remove_block(below_pos);

            // Despawn block entity
            for (entity, block) in block_query.iter() {
                if block.position == below_pos {
                    commands.entity(entity).despawn();
                    break;
                }
            }

            // Add to buffer
            if let Some((buf_type, ref mut count)) = miner.buffer {
                if buf_type == block_type {
                    *count += 1;
                }
            } else {
                miner.buffer = Some((block_type, 1));
            }
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

        // Check for adjacent conveyor (on top of miner, or beside it)
        let adjacent_positions = [
            miner.position + IVec3::new(0, 1, 0),  // above
            miner.position + IVec3::new(1, 0, 0),  // east
            miner.position + IVec3::new(-1, 0, 0), // west
            miner.position + IVec3::new(0, 0, 1),  // south
            miner.position + IVec3::new(0, 0, -1), // north
        ];

        for mut conveyor in conveyor_query.iter_mut() {
            if adjacent_positions.contains(&conveyor.position) && conveyor.item.is_none() {
                // Transfer item to conveyor
                conveyor.item = Some(block_type);
                if let Some((_, ref mut buf_count)) = miner.buffer {
                    *buf_count -= 1;
                    if *buf_count == 0 {
                        miner.buffer = None;
                    }
                }
                break;
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
        if crusher.output_count == 0 || crusher.output_type.is_none() {
            continue;
        }

        let output_type = crusher.output_type.unwrap();

        // Check for adjacent conveyor
        let adjacent_positions = [
            crusher.position + IVec3::new(1, 0, 0),  // east
            crusher.position + IVec3::new(-1, 0, 0), // west
            crusher.position + IVec3::new(0, 0, 1),  // south
            crusher.position + IVec3::new(0, 0, -1), // north
            crusher.position + IVec3::new(0, 1, 0),  // above
        ];

        for mut conveyor in conveyor_query.iter_mut() {
            if adjacent_positions.contains(&conveyor.position) && conveyor.item.is_none() {
                // Transfer item to conveyor
                conveyor.item = Some(output_type);
                crusher.output_count -= 1;
                if crusher.output_count == 0 {
                    crusher.output_type = None;
                }
                break;
            }
        }
    }
}

/// Conveyor transfer logic - move items along conveyor chain
fn conveyor_transfer(
    time: Res<Time>,
    mut conveyor_query: Query<(Entity, &mut Conveyor)>,
    mut furnace_query: Query<(&Transform, &mut Furnace)>,
    mut crusher_query: Query<&mut Crusher>,
) {
    // Collect conveyor states first to avoid borrow issues
    let conveyor_states: Vec<(Entity, IVec3, Direction, Option<BlockType>, f32)> = conveyor_query
        .iter()
        .map(|(e, c)| (e, c.position, c.direction, c.item, c.progress))
        .collect();

    // Build a map of conveyor positions for lookup
    let conveyor_positions: HashMap<IVec3, Entity> = conveyor_states
        .iter()
        .map(|(e, pos, _, _, _)| (*pos, *e))
        .collect();

    // Collect crusher positions
    let crusher_states: Vec<(IVec3, Option<BlockType>, u32)> = crusher_query
        .iter()
        .map(|c| (c.position, c.input_type, c.input_count))
        .collect();

    // Collect transfer actions
    struct TransferAction {
        source: Entity,
        target: TransferTarget,
        item: BlockType,
    }
    enum TransferTarget {
        Conveyor(Entity),
        Furnace(IVec3),
        Crusher(IVec3),
    }

    let mut actions: Vec<TransferAction> = Vec::new();

    for (entity, pos, direction, item, progress) in conveyor_states.iter() {
        let Some(block_type) = item else {
            continue;
        };

        // Only transfer when progress is complete
        if *progress < 1.0 {
            continue;
        }

        let next_pos = *pos + direction.to_ivec3();

        // Check if next position has a conveyor
        if let Some(&next_entity) = conveyor_positions.get(&next_pos) {
            // Check if next conveyor is empty
            if let Some((_, _, _, next_item, _)) = conveyor_states.iter().find(|(e, _, _, _, _)| *e == next_entity) {
                if next_item.is_none() {
                    actions.push(TransferAction {
                        source: *entity,
                        target: TransferTarget::Conveyor(next_entity),
                        item: *block_type,
                    });
                }
            }
        } else {
            // Check if next position has a furnace
            let mut found = false;
            for (furnace_transform, furnace) in furnace_query.iter() {
                let furnace_pos = IVec3::new(
                    furnace_transform.translation.x.round() as i32,
                    furnace_transform.translation.y.round() as i32,
                    furnace_transform.translation.z.round() as i32,
                );
                if furnace_pos == next_pos {
                    // Check if furnace can accept this item
                    let can_accept = match block_type {
                        BlockType::Coal => furnace.fuel < 64,
                        BlockType::IronOre | BlockType::CopperOre => {
                            furnace.can_add_input(*block_type) && furnace.input_count < 64
                        }
                        _ => false,
                    };
                    if can_accept {
                        actions.push(TransferAction {
                            source: *entity,
                            target: TransferTarget::Furnace(furnace_pos),
                            item: *block_type,
                        });
                    }
                    found = true;
                    break;
                }
            }

            // Check if next position has a crusher
            if !found {
                for (crusher_pos, input_type, input_count) in crusher_states.iter() {
                    if *crusher_pos == next_pos {
                        // Check if crusher can accept this ore
                        let can_accept = Crusher::can_crush(*block_type)
                            && (input_type.is_none() || *input_type == Some(*block_type))
                            && *input_count < 64;
                        if can_accept {
                            actions.push(TransferAction {
                                source: *entity,
                                target: TransferTarget::Crusher(*crusher_pos),
                                item: *block_type,
                            });
                        }
                        break;
                    }
                }
            }
        }
    }

    // Apply transfers
    for action in actions {
        // Clear source conveyor
        if let Ok((_, mut source_conv)) = conveyor_query.get_mut(action.source) {
            source_conv.item = None;
            source_conv.progress = 0.0;
        }

        match action.target {
            TransferTarget::Conveyor(target_entity) => {
                if let Ok((_, mut target_conv)) = conveyor_query.get_mut(target_entity) {
                    target_conv.item = Some(action.item);
                    target_conv.progress = 0.0;
                }
            }
            TransferTarget::Furnace(furnace_pos) => {
                for (furnace_transform, mut furnace) in furnace_query.iter_mut() {
                    let pos = IVec3::new(
                        furnace_transform.translation.x.round() as i32,
                        furnace_transform.translation.y.round() as i32,
                        furnace_transform.translation.z.round() as i32,
                    );
                    if pos == furnace_pos {
                        match action.item {
                            BlockType::Coal => furnace.fuel += 1,
                            BlockType::IronOre | BlockType::CopperOre => {
                                furnace.input_type = Some(action.item);
                                furnace.input_count += 1;
                            }
                            _ => {}
                        }
                        break;
                    }
                }
            }
            TransferTarget::Crusher(crusher_pos) => {
                for mut crusher in crusher_query.iter_mut() {
                    if crusher.position == crusher_pos {
                        crusher.input_type = Some(action.item);
                        crusher.input_count += 1;
                        break;
                    }
                }
            }
        }
    }

    // Update progress for conveyors with items
    for (_, mut conveyor) in conveyor_query.iter_mut() {
        if conveyor.item.is_some() && conveyor.progress < 1.0 {
            conveyor.progress += time.delta_secs() / CONVEYOR_SPEED;
            if conveyor.progress > 1.0 {
                conveyor.progress = 1.0;
            }
        }
    }
}

/// Update conveyor item visuals - spawn/despawn/move items on conveyors
fn update_conveyor_item_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut conveyor_query: Query<&mut Conveyor>,
    mut visual_query: Query<&mut Transform, With<ConveyorItemVisual>>,
) {
    let item_mesh = meshes.add(Cuboid::new(BLOCK_SIZE * 0.4, BLOCK_SIZE * 0.4, BLOCK_SIZE * 0.4));

    for mut conveyor in conveyor_query.iter_mut() {
        match (conveyor.item, conveyor.item_visual) {
            // Has item but no visual - spawn it
            (Some(block_type), None) => {
                let material = materials.add(StandardMaterial {
                    base_color: block_type.color(),
                    ..default()
                });

                // Calculate position based on progress
                let base_pos = Vec3::new(
                    conveyor.position.x as f32 * BLOCK_SIZE,
                    conveyor.position.y as f32 * BLOCK_SIZE,
                    conveyor.position.z as f32 * BLOCK_SIZE,
                );
                let dir_offset = conveyor.direction.to_ivec3().as_vec3() * (conveyor.progress - 0.5);

                let entity = commands.spawn((
                    Mesh3d(item_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(base_pos + dir_offset * BLOCK_SIZE),
                    ConveyorItemVisual,
                )).id();

                conveyor.item_visual = Some(entity);
            }
            // Has visual but no item - despawn it
            (None, Some(entity)) => {
                commands.entity(entity).despawn();
                conveyor.item_visual = None;
            }
            // Has both - update position
            (Some(_), Some(entity)) => {
                if let Ok(mut transform) = visual_query.get_mut(entity) {
                    let base_pos = Vec3::new(
                        conveyor.position.x as f32 * BLOCK_SIZE,
                        conveyor.position.y as f32 * BLOCK_SIZE,
                        conveyor.position.z as f32 * BLOCK_SIZE,
                    );
                    let dir_offset = conveyor.direction.to_ivec3().as_vec3() * (conveyor.progress - 0.5);
                    transform.translation = base_pos + dir_offset * BLOCK_SIZE;
                }
            }
            // Neither - nothing to do
            (None, None) => {}
        }
    }
}

/// Update furnace UI text
fn update_furnace_ui(
    interacting: Res<InteractingFurnace>,
    furnace_query: Query<&Furnace>,
    mut text_query: Query<&mut Text, With<FurnaceUIText>>,
) {
    let Some(furnace_entity) = interacting.0 else {
        return;
    };

    let Ok(furnace) = furnace_query.get(furnace_entity) else {
        return;
    };

    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    let progress_bar = if furnace.fuel > 0 && furnace.input_count > 0 {
        let filled = (furnace.progress * 10.0) as usize;
        let empty = 10 - filled;
        format!("[{}{}] {:.0}%", "=".repeat(filled), " ".repeat(empty), furnace.progress * 100.0)
    } else {
        "[          ] 0%".to_string()
    };

    let input_name = furnace.input_type.map_or("None", |t| t.name());
    let output_name = furnace.output_type.map_or("None", |t| t.name());

    **text = format!(
        "=== Furnace ===\n\nFuel: {} Coal\nInput: {} {}\nOutput: {} {}\n\nProgress: {}\n\n[1] Coal | [2] Iron Ore | [3] Copper Ore\n[4] Take Output | [E] Close",
        furnace.fuel,
        furnace.input_count, input_name,
        furnace.output_count, output_name,
        progress_bar
    );
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

// === Delivery Platform Systems ===

const PLATFORM_SIZE: i32 = 12;

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
    commands.spawn((
        Mesh3d(platform_mesh),
        MeshMaterial3d(platform_material),
        Transform::from_translation(Vec3::new(
            platform_origin.x as f32 * BLOCK_SIZE + (PLATFORM_SIZE as f32 * BLOCK_SIZE / 2.0) - 0.5,
            platform_origin.y as f32 * BLOCK_SIZE - 0.4,
            platform_origin.z as f32 * BLOCK_SIZE + (PLATFORM_SIZE as f32 * BLOCK_SIZE / 2.0) - 0.5,
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
                port_pos.x as f32 * BLOCK_SIZE,
                port_pos.y as f32 * BLOCK_SIZE + 0.1, // Raised above platform
                port_pos.z as f32 * BLOCK_SIZE,
            )),
        ));
    }
}

/// Receive items from conveyors onto delivery platform
fn delivery_platform_receive(
    mut platform_query: Query<(&Transform, &mut DeliveryPlatform)>,
    mut conveyor_query: Query<&mut Conveyor>,
) {
    let Ok((platform_transform, mut platform)) = platform_query.get_single_mut() else {
        return;
    };

    // Calculate platform bounds
    let platform_center = platform_transform.translation;
    let half_size = (PLATFORM_SIZE as f32 * BLOCK_SIZE) / 2.0;
    let platform_min_x = (platform_center.x - half_size).floor() as i32;
    let platform_max_x = (platform_center.x + half_size).floor() as i32;
    let platform_min_z = (platform_center.z - half_size).floor() as i32;
    let platform_max_z = (platform_center.z + half_size).floor() as i32;

    // Check conveyors pointing into platform
    for mut conveyor in conveyor_query.iter_mut() {
        if conveyor.item.is_none() || conveyor.progress < 1.0 {
            continue;
        }

        let next_pos = conveyor.position + conveyor.direction.to_ivec3();

        // Check if next position is inside platform area
        if next_pos.x >= platform_min_x
            && next_pos.x <= platform_max_x
            && next_pos.z >= platform_min_z
            && next_pos.z <= platform_max_z
        {
            // Accept the item
            if let Some(block_type) = conveyor.item.take() {
                *platform.delivered.entry(block_type).or_insert(0) += 1;
                conveyor.progress = 0.0;
            }
        }
    }
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

/// Quest definitions
fn get_quests() -> Vec<QuestDef> {
    vec![
        QuestDef {
            description: "Deliver 3 Iron Ingots",
            required_item: BlockType::IronIngot,
            required_amount: 3,
            rewards: vec![
                (BlockType::MinerBlock, 2),
                (BlockType::ConveyorBlock, 20),
            ],
        },
        QuestDef {
            description: "Deliver 10 Copper Ingots",
            required_item: BlockType::CopperIngot,
            required_amount: 10,
            rewards: vec![
                (BlockType::CrusherBlock, 2),
                (BlockType::ConveyorBlock, 20),
            ],
        },
        QuestDef {
            description: "Deliver 50 Iron Ingots",
            required_item: BlockType::IronIngot,
            required_amount: 50,
            rewards: vec![
                (BlockType::MinerBlock, 3),
                (BlockType::CrusherBlock, 2),
            ],
        },
        QuestDef {
            description: "Deliver 50 Copper Ingots",
            required_item: BlockType::CopperIngot,
            required_amount: 50,
            rewards: vec![
                (BlockType::MinerBlock, 3),
                (BlockType::ConveyorBlock, 40),
            ],
        },
    ]
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
) {
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
        *inventory.items.entry(*block_type).or_insert(0) += amount;
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

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_generation() {
        let chunk = ChunkData::generate(IVec2::ZERO);
        // Check that chunk has blocks
        assert!(!chunk.blocks.is_empty());

        // Check that top layer is grass (local coordinates)
        assert_eq!(chunk.blocks.get(&IVec3::new(0, 7, 0)), Some(&BlockType::Grass));

        // Check that lower layers are stone or ore (ores are generated randomly)
        let block = chunk.blocks.get(&IVec3::new(0, 0, 0));
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

        // Test get_block
        assert_eq!(world.get_block(IVec3::new(0, 7, 0)), Some(&BlockType::Grass));
        // Lower layers can be stone or ore
        let block = world.get_block(IVec3::new(0, 0, 0));
        assert!(matches!(
            block,
            Some(BlockType::Stone) | Some(BlockType::IronOre) | Some(BlockType::CopperOre) | Some(BlockType::Coal)
        ));

        // Test has_block
        assert!(world.has_block(IVec3::new(0, 0, 0)));
        assert!(!world.has_block(IVec3::new(0, 10, 0))); // Above terrain

        // Test remove_block
        let removed = world.remove_block(IVec3::new(0, 7, 0));
        assert_eq!(removed, Some(BlockType::Grass));
        assert!(!world.has_block(IVec3::new(0, 7, 0)));

        // Test set_block
        world.set_block(IVec3::new(0, 10, 0), BlockType::Stone);
        assert_eq!(world.get_block(IVec3::new(0, 10, 0)), Some(&BlockType::Stone));
    }

    #[test]
    fn test_inventory_add() {
        let mut inventory = Inventory::default();
        *inventory.items.entry(BlockType::Stone).or_insert(0) += 1;
        assert_eq!(inventory.items.get(&BlockType::Stone), Some(&1));

        *inventory.items.entry(BlockType::Stone).or_insert(0) += 1;
        assert_eq!(inventory.items.get(&BlockType::Stone), Some(&2));
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
        assert!(inventory.selected.is_none());

        // Add item and select it
        inventory.items.insert(BlockType::Stone, 5);
        inventory.selected = Some(BlockType::Stone);
        assert_eq!(inventory.selected, Some(BlockType::Stone));
    }
}
