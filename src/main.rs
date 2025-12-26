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
const CHUNK_SIZE: usize = 16;
const BLOCK_SIZE: f32 = 1.0;
const PLAYER_SPEED: f32 = 5.0;
const REACH_DISTANCE: f32 = 5.0;

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
        .init_resource::<ChunkData>()
        .add_systems(Startup, (setup_world, setup_player, setup_ui))
        .add_systems(
            Update,
            (
                toggle_cursor_lock,
                player_look,
                player_move,
                block_break,
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


#[derive(Component)]
struct Block {
    position: IVec3,
}

#[derive(Component)]
struct InventoryUI;

#[derive(Component)]
struct InventoryText;


// === Resources ===

#[derive(Resource, Default)]
struct Inventory {
    items: HashMap<BlockType, u32>,
}

#[derive(Resource)]
struct ChunkData {
    blocks: HashMap<IVec3, BlockType>,
}

impl Default for ChunkData {
    fn default() -> Self {
        let mut blocks = HashMap::new();
        // Generate a 16x16x16 chunk of blocks
        // Bottom half is stone, top layer is grass
        for x in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
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

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk_data: Res<ChunkData>,
) {
    // Spawn blocks
    let cube_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));

    for (&pos, &block_type) in chunk_data.blocks.iter() {
        let material = materials.add(StandardMaterial {
            base_color: block_type.color(),
            ..default()
        });

        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::new(
                pos.x as f32 * BLOCK_SIZE,
                pos.y as f32 * BLOCK_SIZE,
                pos.z as f32 * BLOCK_SIZE,
            )),
            Block { position: pos },
        ));
    }

    // Light
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
    windows: Query<&Window>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
) {
    let window = windows.single();
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

    // --- Mouse motion using AccumulatedMouseMotion (Bevy best practice) ---
    // No smoothing, no delta_time multiplication - raw 1:1 input
    if cursor_locked {
        let delta = accumulated_mouse_motion.delta;
        camera.yaw -= delta.x * MOUSE_SENSITIVITY;
        camera.pitch -= delta.y * MOUSE_SENSITIVITY;
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
    mut chunk_data: ResMut<ChunkData>,
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
        if let Some(block_type) = chunk_data.blocks.remove(&pos) {
            commands.entity(entity).despawn();
            *inventory.items.entry(block_type).or_insert(0) += 1;
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

fn update_inventory_ui(inventory: Res<Inventory>, mut query: Query<&mut Text, With<InventoryText>>) {
    if !inventory.is_changed() {
        return;
    }

    let Ok(mut text) = query.get_single_mut() else {
        return;
    };

    let hint = "Click:Lock | Esc:Unlock | WASD:Move | Mouse/Arrow:Look";

    if inventory.items.is_empty() {
        **text = format!("Inventory: Empty\n{}", hint);
    } else {
        let items: Vec<String> = inventory
            .items
            .iter()
            .map(|(block_type, count)| format!("{}: {}", block_type.name(), count))
            .collect();
        **text = format!("Inventory:\n{}\n{}", items.join("\n"), hint);
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
        let chunk = ChunkData::default();
        // Check that chunk has blocks
        assert!(!chunk.blocks.is_empty());

        // Check that top layer is grass
        assert_eq!(chunk.blocks.get(&IVec3::new(0, 7, 0)), Some(&BlockType::Grass));

        // Check that lower layers are stone
        assert_eq!(chunk.blocks.get(&IVec3::new(0, 0, 0)), Some(&BlockType::Stone));
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
}
