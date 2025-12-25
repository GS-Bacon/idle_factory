//! プレイヤー物理システム
//! Minecraft風のサバイバルモード移動を実装
//! - 重力・ジャンプ
//! - 全方向衝突判定
//! - スニーク（端落ち防止）
//! - 水泳・はしご

use bevy::prelude::*;
use crate::core::input::KeyBindings;
use crate::core::registry::BlockRegistry;
use crate::gameplay::commands::GameMode;
use crate::gameplay::player_stats::{DamageEvent, DamageSource, FallTracker};
use crate::rendering::chunk::{Chunk, CHUNK_SIZE};
use crate::ui::command_ui::CommandUiState;
use crate::ui::inventory_ui::InventoryUiState;
use crate::ui::main_menu::AppState;
use crate::ui::settings_ui::SettingsUiState;

/// 物理定数
#[derive(Resource)]
pub struct PhysicsConstants {
    pub gravity: f32,
    pub terminal_velocity: f32,
    pub jump_velocity: f32,
    pub walk_speed: f32,
    pub sprint_speed: f32,
    pub sneak_speed: f32,
    pub swim_speed: f32,
    pub ladder_speed: f32,
    pub water_gravity: f32,
    pub air_resistance: f32,
    pub coyote_duration: f32,
    pub jump_buffer_duration: f32,
}

impl Default for PhysicsConstants {
    fn default() -> Self {
        Self {
            gravity: 32.0,
            terminal_velocity: 78.4,
            jump_velocity: 9.0,
            walk_speed: 4.317,
            sprint_speed: 5.612,
            sneak_speed: 1.31,
            swim_speed: 2.0,
            ladder_speed: 2.35,
            water_gravity: 2.0,
            air_resistance: 0.91,
            coyote_duration: 0.1,
            jump_buffer_duration: 0.1,
        }
    }
}

/// プレイヤーの物理状態
#[derive(Component)]
pub struct PlayerPhysics {
    pub velocity: Vec3,
    pub is_on_ground: bool,
    pub is_in_water: bool,
    pub is_on_ladder: bool,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub coyote_time: f32,
    pub jump_buffer: f32,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            is_on_ground: false,
            is_in_water: false,
            is_on_ladder: false,
            is_sneaking: false,
            is_sprinting: false,
            coyote_time: 0.0,
            jump_buffer: 0.0,
        }
    }
}

/// プレイヤーの当たり判定
#[derive(Component)]
pub struct PlayerCollider {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
}

impl Default for PlayerCollider {
    fn default() -> Self {
        Self {
            width: 0.5,
            height: 1.6,
            eye_height: 1.5,
        }
    }
}


/// 物理プラグイン
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsConstants>()
            .add_systems(
                Update,
                (
                    process_movement_input,
                    check_environment_state,
                    apply_physics,
                    move_and_resolve_collisions,
                    update_fall_tracking,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InventoryUiState::Closed))
                    .run_if(in_state(CommandUiState::Closed))
                    .run_if(in_state(SettingsUiState::Closed))
                    .run_if(resource_equals(GameMode::Survival)),
            );
    }
}

/// 入力処理
fn process_movement_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    keybinds: Res<KeyBindings>,
    mut query: Query<(&Transform, &mut PlayerPhysics)>,
    constants: Res<PhysicsConstants>,
) {
    for (transform, mut physics) in query.iter_mut() {
        // スニーク判定
        physics.is_sneaking = keyboard.pressed(keybinds.descend);

        // スプリント判定
        physics.is_sprinting = keyboard.pressed(keybinds.sprint) && !physics.is_sneaking;

        // ジャンプバッファ更新
        if keyboard.just_pressed(keybinds.jump) {
            physics.jump_buffer = constants.jump_buffer_duration;
        }

        // 移動方向計算
        let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw_rot = Quat::from_rotation_y(yaw);
        let forward = yaw_rot * Vec3::NEG_Z;
        let right = yaw_rot * Vec3::X;

        let mut move_dir = Vec3::ZERO;
        if keyboard.pressed(keybinds.forward) {
            move_dir += forward;
        }
        if keyboard.pressed(keybinds.backward) {
            move_dir -= forward;
        }
        if keyboard.pressed(keybinds.right) {
            move_dir += right;
        }
        if keyboard.pressed(keybinds.left) {
            move_dir -= right;
        }

        // 正規化
        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
        }

        // 速度決定
        let speed = if physics.is_sneaking {
            constants.sneak_speed
        } else if physics.is_sprinting {
            constants.sprint_speed
        } else {
            constants.walk_speed
        };

        // 水平速度を設定
        physics.velocity.x = move_dir.x * speed;
        physics.velocity.z = move_dir.z * speed;
    }
}

/// 環境状態チェック（水中・はしご）
fn check_environment_state(
    mut query: Query<(&Transform, &PlayerCollider, &mut PlayerPhysics)>,
    chunks: Query<&Chunk>,
    block_registry: Res<BlockRegistry>,
) {
    for (transform, collider, mut physics) in query.iter_mut() {
        let pos = transform.translation;

        // 体の中心位置のブロック
        let body_block_pos = IVec3::new(
            pos.x.floor() as i32,
            (pos.y + collider.height / 2.0).floor() as i32,
            pos.z.floor() as i32,
        );

        // 水中判定
        physics.is_in_water = is_liquid_at(&chunks, &block_registry, body_block_pos);

        // はしご判定（プレイヤーAABBと重なるはしごブロック）
        physics.is_on_ladder = check_ladder_collision(
            pos,
            collider,
            &chunks,
            &block_registry,
        );
    }
}

/// 物理適用（重力・ジャンプ・水泳・はしご）
fn apply_physics(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    keybinds: Res<KeyBindings>,
    constants: Res<PhysicsConstants>,
    mut query: Query<&mut PlayerPhysics>,
) {
    let delta = time.delta_secs();

    for mut physics in query.iter_mut() {
        // コヨーテタイム更新
        if physics.is_on_ground {
            physics.coyote_time = constants.coyote_duration;
        } else {
            physics.coyote_time = (physics.coyote_time - delta).max(0.0);
        }

        // ジャンプバッファ減少
        physics.jump_buffer = (physics.jump_buffer - delta).max(0.0);

        // 水中処理
        if physics.is_in_water {
            // 水中移動
            if keyboard.pressed(keybinds.jump) {
                physics.velocity.y = constants.swim_speed;
            } else if keyboard.pressed(keybinds.descend) {
                physics.velocity.y = -constants.swim_speed;
            } else {
                // 浮力（ゆっくり浮上）
                physics.velocity.y = (physics.velocity.y + 0.5 * delta).min(0.5);
            }
            // 水中では水平速度も減衰
            physics.velocity.x *= 0.8;
            physics.velocity.z *= 0.8;
        }
        // はしご処理
        else if physics.is_on_ladder {
            // 落下速度制限
            if physics.velocity.y < -constants.ladder_speed {
                physics.velocity.y = -constants.ladder_speed;
            }

            // 上下移動
            if keyboard.pressed(keybinds.jump) {
                physics.velocity.y = constants.ladder_speed;
            } else if keyboard.pressed(keybinds.descend) {
                physics.velocity.y = -constants.ladder_speed;
            } else if physics.is_sneaking {
                // スニーク中は静止
                physics.velocity.y = 0.0;
            }

            // はしご上では水平速度も減衰
            physics.velocity.x *= 0.5;
            physics.velocity.z *= 0.5;
        }
        // 通常の重力処理
        else {
            // ジャンプ判定
            let can_jump = physics.is_on_ground || physics.coyote_time > 0.0;
            let wants_jump = physics.jump_buffer > 0.0 || keyboard.just_pressed(keybinds.jump);

            if can_jump && wants_jump && physics.velocity.y <= 0.0 {
                physics.velocity.y = constants.jump_velocity;
                physics.coyote_time = 0.0;
                physics.jump_buffer = 0.0;
            }

            // 重力適用
            if !physics.is_on_ground {
                physics.velocity.y -= constants.gravity * delta;
                physics.velocity.y = physics.velocity.y.max(-constants.terminal_velocity);
            }
        }
    }
}

/// 衝突判定と位置更新
fn move_and_resolve_collisions(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &PlayerCollider, &mut PlayerPhysics)>,
    chunks: Query<&Chunk>,
    block_registry: Res<BlockRegistry>,
) {
    let delta = time.delta_secs();

    for (mut transform, collider, mut physics) in query.iter_mut() {
        let mut pos = transform.translation;
        let mut vel = physics.velocity;

        // スニーク時の端判定用に元の位置を保存
        let original_pos = pos;

        // Y軸処理（重力方向優先）
        let new_y = pos.y + vel.y * delta;
        if check_collision_y(&pos, new_y, collider, &chunks, &block_registry) {
            if vel.y < 0.0 {
                // 着地
                physics.is_on_ground = true;
                pos.y = snap_to_ground(&pos, collider, &chunks, &block_registry);
            } else {
                // 天井衝突
                pos.y = snap_to_ceiling(&pos, collider, &chunks, &block_registry);
            }
            vel.y = 0.0;
        } else {
            pos.y = new_y;
            physics.is_on_ground = false;
        }

        // X軸処理
        let new_x = pos.x + vel.x * delta;
        if check_collision_x(&pos, new_x, collider, &chunks, &block_registry) {
            vel.x = 0.0;
        } else {
            pos.x = new_x;
        }

        // Z軸処理
        let new_z = pos.z + vel.z * delta;
        if check_collision_z(&pos, new_z, collider, &chunks, &block_registry) {
            vel.z = 0.0;
        } else {
            pos.z = new_z;
        }

        // スニーク時の端判定
        if physics.is_sneaking
            && physics.is_on_ground
            && !has_ground_support(&pos, collider, &chunks, &block_registry)
        {
            // 端に来たら戻す
            pos.x = original_pos.x;
            pos.z = original_pos.z;
            vel.x = 0.0;
            vel.z = 0.0;
        }

        transform.translation = pos;
        physics.velocity = vel;
    }
}

/// 落下ダメージ追跡
fn update_fall_tracking(
    mut query: Query<(Entity, &Transform, &PlayerPhysics, &mut FallTracker)>,
    mut ev_damage: EventWriter<DamageEvent>,
) {
    for (entity, transform, physics, mut tracker) in query.iter_mut() {
        let current_y = transform.translation.y;

        // 上昇中は最高点を更新
        if current_y > tracker.highest_y {
            tracker.highest_y = current_y;
        }

        // 着地判定
        if physics.is_on_ground && !tracker.is_on_ground {
            let fall_distance = tracker.highest_y - current_y;

            // 3ブロック以上で落下ダメージ
            if fall_distance > 3.0 {
                let damage = (fall_distance - 3.0) * 0.5;
                ev_damage.send(DamageEvent {
                    entity,
                    amount: damage,
                    source: DamageSource::Fall(fall_distance),
                });
            }

            tracker.highest_y = current_y;
        }

        tracker.is_on_ground = physics.is_on_ground;
    }
}

// === ヘルパー関数 ===

/// 指定位置のブロックを取得
fn get_block_at<'a>(
    chunks: &'a Query<&Chunk>,
    x: i32,
    y: i32,
    z: i32,
) -> Option<&'a String> {
    // 現在は単一チャンク(0,0,0)のみ
    if x < 0 || y < 0 || z < 0 {
        return None;
    }
    if x >= CHUNK_SIZE as i32 || y >= CHUNK_SIZE as i32 || z >= CHUNK_SIZE as i32 {
        return None;
    }

    for chunk in chunks.iter() {
        if chunk.position == IVec3::ZERO {
            return chunk.get_block(x as usize, y as usize, z as usize);
        }
    }
    None
}

/// 液体ブロック判定
fn is_liquid_at(
    chunks: &Query<&Chunk>,
    block_registry: &BlockRegistry,
    pos: IVec3,
) -> bool {
    if let Some(block_id) = get_block_at(chunks, pos.x, pos.y, pos.z) {
        if let Some(prop) = block_registry.map.get(block_id) {
            return prop.is_liquid;
        }
    }
    false
}

/// はしご衝突判定
fn check_ladder_collision(
    pos: Vec3,
    collider: &PlayerCollider,
    chunks: &Query<&Chunk>,
    block_registry: &BlockRegistry,
) -> bool {
    let half_width = collider.width / 2.0;
    let min = Vec3::new(pos.x - half_width, pos.y, pos.z - half_width);
    let max = Vec3::new(pos.x + half_width, pos.y + collider.height, pos.z + half_width);

    let block_min = min.floor().as_ivec3();
    let block_max = max.ceil().as_ivec3();

    for y in block_min.y..=block_max.y {
        for z in block_min.z..=block_max.z {
            for x in block_min.x..=block_max.x {
                if let Some(block_id) = get_block_at(chunks, x, y, z) {
                    if let Some(prop) = block_registry.map.get(block_id) {
                        if prop.is_climbable {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Y軸衝突チェック
fn check_collision_y(
    pos: &Vec3,
    new_y: f32,
    collider: &PlayerCollider,
    chunks: &Query<&Chunk>,
    block_registry: &BlockRegistry,
) -> bool {
    let half_width = collider.width / 2.0;
    let test_pos = Vec3::new(pos.x, new_y, pos.z);

    let min = Vec3::new(test_pos.x - half_width, test_pos.y, test_pos.z - half_width);
    let max = Vec3::new(
        test_pos.x + half_width,
        test_pos.y + collider.height,
        test_pos.z + half_width,
    );

    check_aabb_collision(min, max, chunks, block_registry)
}

/// X軸衝突チェック
fn check_collision_x(
    pos: &Vec3,
    new_x: f32,
    collider: &PlayerCollider,
    chunks: &Query<&Chunk>,
    block_registry: &BlockRegistry,
) -> bool {
    let half_width = collider.width / 2.0;
    let test_pos = Vec3::new(new_x, pos.y, pos.z);

    let min = Vec3::new(test_pos.x - half_width, test_pos.y, test_pos.z - half_width);
    let max = Vec3::new(
        test_pos.x + half_width,
        test_pos.y + collider.height,
        test_pos.z + half_width,
    );

    check_aabb_collision(min, max, chunks, block_registry)
}

/// Z軸衝突チェック
fn check_collision_z(
    pos: &Vec3,
    new_z: f32,
    collider: &PlayerCollider,
    chunks: &Query<&Chunk>,
    block_registry: &BlockRegistry,
) -> bool {
    let half_width = collider.width / 2.0;
    let test_pos = Vec3::new(pos.x, pos.y, new_z);

    let min = Vec3::new(test_pos.x - half_width, test_pos.y, test_pos.z - half_width);
    let max = Vec3::new(
        test_pos.x + half_width,
        test_pos.y + collider.height,
        test_pos.z + half_width,
    );

    check_aabb_collision(min, max, chunks, block_registry)
}

/// AABB衝突判定
fn check_aabb_collision(
    player_min: Vec3,
    player_max: Vec3,
    chunks: &Query<&Chunk>,
    block_registry: &BlockRegistry,
) -> bool {
    let block_min = player_min.floor().as_ivec3();
    let block_max = player_max.ceil().as_ivec3();

    for y in block_min.y..=block_max.y {
        for z in block_min.z..=block_max.z {
            for x in block_min.x..=block_max.x {
                if let Some(block_id) = get_block_at(chunks, x, y, z) {
                    if let Some(prop) = block_registry.map.get(block_id) {
                        if prop.is_solid {
                            let block_aabb = get_block_aabb(x, y, z, &prop.collision_box);
                            if aabb_intersects(player_min, player_max, block_aabb.0, block_aabb.1) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// ブロックのAABBを取得
fn get_block_aabb(x: i32, y: i32, z: i32, collision: &[f32; 6]) -> (Vec3, Vec3) {
    let base = Vec3::new(x as f32, y as f32, z as f32);
    let min = base + Vec3::new(collision[0], collision[1], collision[2]);
    let max = base + Vec3::new(collision[3], collision[4], collision[5]);
    (min, max)
}

/// AABB同士の交差判定
fn aabb_intersects(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> bool {
    a_min.x < b_max.x
        && a_max.x > b_min.x
        && a_min.y < b_max.y
        && a_max.y > b_min.y
        && a_min.z < b_max.z
        && a_max.z > b_min.z
}

/// 地面にスナップ
fn snap_to_ground(
    pos: &Vec3,
    collider: &PlayerCollider,
    chunks: &Query<&Chunk>,
    block_registry: &BlockRegistry,
) -> f32 {
    let half_width = collider.width / 2.0;

    // 足元のブロックを探索
    for test_y in (0..=(pos.y.floor() as i32)).rev() {
        let block_min = IVec3::new(
            (pos.x - half_width).floor() as i32,
            test_y,
            (pos.z - half_width).floor() as i32,
        );
        let block_max = IVec3::new(
            (pos.x + half_width).ceil() as i32,
            test_y,
            (pos.z + half_width).ceil() as i32,
        );

        for z in block_min.z..=block_max.z {
            for x in block_min.x..=block_max.x {
                if let Some(block_id) = get_block_at(chunks, x, test_y, z) {
                    if let Some(prop) = block_registry.map.get(block_id) {
                        if prop.is_solid {
                            let block_top = test_y as f32 + prop.collision_box[4];
                            return block_top;
                        }
                    }
                }
            }
        }
    }

    pos.y
}

/// 天井にスナップ
fn snap_to_ceiling(
    pos: &Vec3,
    collider: &PlayerCollider,
    chunks: &Query<&Chunk>,
    block_registry: &BlockRegistry,
) -> f32 {
    let half_width = collider.width / 2.0;
    let head_y = (pos.y + collider.height).ceil() as i32;

    // 頭上のブロックを探索
    let block_min = IVec3::new(
        (pos.x - half_width).floor() as i32,
        head_y,
        (pos.z - half_width).floor() as i32,
    );
    let block_max = IVec3::new(
        (pos.x + half_width).ceil() as i32,
        head_y,
        (pos.z + half_width).ceil() as i32,
    );

    for z in block_min.z..=block_max.z {
        for x in block_min.x..=block_max.x {
            if let Some(block_id) = get_block_at(chunks, x, head_y, z) {
                if let Some(prop) = block_registry.map.get(block_id) {
                    if prop.is_solid {
                        let block_bottom = head_y as f32 + prop.collision_box[1];
                        return block_bottom - collider.height;
                    }
                }
            }
        }
    }

    pos.y
}

/// 足元にサポートがあるか（スニーク用）
fn has_ground_support(
    pos: &Vec3,
    collider: &PlayerCollider,
    chunks: &Query<&Chunk>,
    block_registry: &BlockRegistry,
) -> bool {
    let half_width = collider.width / 2.0 + 0.1; // 少し余裕を持たせる
    let check_y = (pos.y - 0.1).floor() as i32;

    // 四隅のいずれかにブロックがあればOK
    let corners = [
        IVec3::new((pos.x - half_width).floor() as i32, check_y, (pos.z - half_width).floor() as i32),
        IVec3::new((pos.x + half_width).floor() as i32, check_y, (pos.z - half_width).floor() as i32),
        IVec3::new((pos.x - half_width).floor() as i32, check_y, (pos.z + half_width).floor() as i32),
        IVec3::new((pos.x + half_width).floor() as i32, check_y, (pos.z + half_width).floor() as i32),
    ];

    for corner in corners {
        if let Some(block_id) = get_block_at(chunks, corner.x, corner.y, corner.z) {
            if let Some(prop) = block_registry.map.get(block_id) {
                if prop.is_solid {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_intersection() {
        // 交差する場合
        assert!(aabb_intersects(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(1.5, 1.5, 1.5),
        ));

        // 交差しない場合
        assert!(!aabb_intersects(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(2.0, 2.0, 2.0),
            Vec3::new(3.0, 3.0, 3.0),
        ));

        // 接触のみ（交差なし）
        assert!(!aabb_intersects(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 1.0, 1.0),
        ));
    }

    #[test]
    fn test_block_aabb() {
        let collision = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let (min, max) = get_block_aabb(5, 3, 7, &collision);
        assert_eq!(min, Vec3::new(5.0, 3.0, 7.0));
        assert_eq!(max, Vec3::new(6.0, 4.0, 8.0));

        // コンベアのような薄いブロック
        let conveyor_collision = [0.0, 0.0, 0.0, 1.0, 0.2, 1.0];
        let (min, max) = get_block_aabb(0, 0, 0, &conveyor_collision);
        assert_eq!(min, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(max, Vec3::new(1.0, 0.2, 1.0));
    }

    #[test]
    fn test_physics_constants() {
        let constants = PhysicsConstants::default();
        assert_eq!(constants.gravity, 32.0);
        assert_eq!(constants.jump_velocity, 9.0);
        assert_eq!(constants.walk_speed, 4.317);
    }
}
