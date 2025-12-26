//! Idle Factory - Milestone 1: Minimal Voxel Game
//! Goal: Walk, mine blocks, collect in inventory

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::window::{CursorGrabMode, PresentMode};
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
    App::new()
        // Disable pipelined rendering to reduce input lag (1 frame delay reduction)
        // Trade-off: ~10-30% lower framerate, but much better input responsiveness
        .add_plugins((
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
        ))
        .init_resource::<Inventory>()
        .init_resource::<WorldData>()
        .init_resource::<CursorLockState>()
        .add_systems(Startup, (setup_lighting, setup_player, setup_ui))
        .add_systems(
            Update,
            (
                chunk_loading,
                toggle_cursor_lock,
                player_look,
                player_move,
                block_break,
                block_place,
                select_block_type,
                update_inventory_ui,
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
    fn generate(_chunk_coord: IVec2) -> Self {
        let mut blocks = HashMap::new();
        // Generate a 16x16x8 chunk of blocks
        // Bottom layers are stone, top layer is grass
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..8 {
                    let block_type = if y == 7 {
                        BlockType::Grass
                    } else {
                        BlockType::Stone
                    };
                    blocks.insert(IVec3::new(x, y, z), block_type);
                }
            }
        }
        Self { blocks }
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
}

impl BlockType {
    fn color(&self) -> Color {
        match self {
            BlockType::Stone => Color::srgb(0.5, 0.5, 0.5),
            BlockType::Grass => Color::srgb(0.2, 0.8, 0.2),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            BlockType::Stone => "Stone",
            BlockType::Grass => "Grass",
        }
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
) {
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
) {
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
    mut world_data: ResMut<WorldData>,
    mut inventory: ResMut<Inventory>,
    windows: Query<&Window>,
) {
    // Only break blocks when cursor is locked (to distinguish from lock-click)
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    if !cursor_locked || !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((camera_transform, _camera)) = camera_query.get_single() else {
        return;
    };

    // Calculate ray from camera using its actual transform
    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Find closest block intersection
    let mut closest_hit: Option<(Entity, IVec3, f32)> = None;

    for (entity, block, block_transform) in block_query.iter() {
        let block_pos = block_transform.translation();
        let half_size = BLOCK_SIZE / 2.0;

        // Simple AABB ray intersection
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            block_pos - Vec3::splat(half_size),
            block_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0
                && t < REACH_DISTANCE
                && (closest_hit.is_none() || t < closest_hit.unwrap().2)
            {
                closest_hit = Some((entity, block.position, t));
            }
        }
    }

    // Break the closest block
    if let Some((entity, pos, _)) = closest_hit {
        if let Some(block_type) = world_data.remove_block(pos) {
            commands.entity(entity).despawn();
            *inventory.items.entry(block_type).or_insert(0) += 1;
            // Auto-select the block type if nothing selected
            if inventory.selected.is_none() {
                inventory.selected = Some(block_type);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn block_place(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
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

    let Ok(camera_transform) = camera_query.get_single() else {
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

        // Add to world data
        world_data.set_block(place_pos, selected_type);

        // Get chunk coord for the placed block
        let chunk_coord = WorldData::world_to_chunk(place_pos);

        // Spawn block entity
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

    let hint = "LClick:Break | RClick:Place | 1-2:Select | WASD:Move";

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

fn update_window_title_fps(diagnostics: Res<DiagnosticsStore>, mut windows: Query<&mut Window>) {
    if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            if let Ok(mut window) = windows.get_single_mut() {
                window.title = format!("Idle Factory - FPS: {:.0}", value);
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
        assert!(!chunk.blocks.is_empty());

        // Check that top layer is grass (local coordinates)
        assert_eq!(chunk.blocks.get(&IVec3::new(0, 7, 0)), Some(&BlockType::Grass));

        // Check that lower layers are stone
        assert_eq!(chunk.blocks.get(&IVec3::new(0, 0, 0)), Some(&BlockType::Stone));
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
        assert_eq!(world.get_block(IVec3::new(0, 0, 0)), Some(&BlockType::Stone));

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
