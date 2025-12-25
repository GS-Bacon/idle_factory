// src/gameplay/player_stats.rs
//! プレイヤーステータスシステム
//! - HP、経験値、ダメージ、死亡、リスポーン

use bevy::prelude::*;

/// プレイヤーの健康状態
#[derive(Component)]
pub struct PlayerHealth {
    pub current: f32,
    pub max: f32,
    pub regeneration_rate: f32,     // HP/秒
    pub regeneration_delay: f32,    // ダメージ後の回復開始までの遅延
    pub last_damage_time: f32,      // 最後にダメージを受けた時間
    pub invincibility_timer: f32,   // 無敵時間
}

impl Default for PlayerHealth {
    fn default() -> Self {
        Self {
            current: 10.0,
            max: 10.0,
            regeneration_rate: 0.5,     // 0.5 HP/秒
            regeneration_delay: 5.0,    // 5秒後に回復開始
            last_damage_time: -100.0,   // 初期状態では回復可能
            invincibility_timer: 0.0,
        }
    }
}

impl PlayerHealth {
    pub fn new(max_hp: f32) -> Self {
        Self {
            current: max_hp,
            max: max_hp,
            ..default()
        }
    }

    /// ダメージを受ける
    pub fn take_damage(&mut self, amount: f32, current_time: f32) -> bool {
        if self.invincibility_timer > 0.0 {
            return false;
        }

        self.current = (self.current - amount).max(0.0);
        self.last_damage_time = current_time;
        self.invincibility_timer = 0.5; // 0.5秒の無敵時間

        self.current <= 0.0
    }

    /// HP回復
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// 全回復
    pub fn full_heal(&mut self) {
        self.current = self.max;
    }

    /// 死亡判定
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    /// HPパーセント
    pub fn percentage(&self) -> f32 {
        self.current / self.max
    }
}

/// プレイヤーの経験値
#[derive(Component, Default)]
pub struct PlayerExperience {
    pub current: u32,
    pub level: u32,
    pub total_earned: u32, // 累計獲得経験値
}

impl PlayerExperience {
    /// レベルアップに必要な経験値
    pub fn xp_for_level(level: u32) -> u32 {
        // レベル0->1: 100, レベル1->2: 200, ...
        (level + 1) * 100
    }

    /// 経験値を獲得
    pub fn add_xp(&mut self, amount: u32) {
        self.current += amount;
        self.total_earned += amount;

        // レベルアップ処理
        while self.current >= Self::xp_for_level(self.level) {
            self.current -= Self::xp_for_level(self.level);
            self.level += 1;
        }
    }

    /// エンチャントなどで経験値を消費
    pub fn spend_xp(&mut self, amount: u32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    /// 現在レベルの進捗率
    pub fn level_progress(&self) -> f32 {
        self.current as f32 / Self::xp_for_level(self.level) as f32
    }
}

/// ダメージ原因
#[derive(Debug, Clone, PartialEq)]
pub enum DamageSource {
    Fall(f32),      // 落下高さ
    Lava,           // 溶岩
    Void,           // 奈落
    Drowning,       // 溺れ（将来用）
    Machine,        // 機械ダメージ（将来用）
}

/// ダメージイベント
#[derive(Event)]
pub struct DamageEvent {
    pub entity: Entity,
    pub amount: f32,
    pub source: DamageSource,
}

/// 死亡イベント
#[derive(Event)]
pub struct PlayerDeathEvent {
    pub entity: Entity,
    pub source: DamageSource,
    pub lost_xp: u32,
}

/// リスポーンイベント
#[derive(Event)]
pub struct PlayerRespawnEvent {
    pub entity: Entity,
}

/// スポーンアンカー（リスポーン地点）
#[derive(Resource)]
pub struct SpawnAnchor {
    pub position: Vec3,
    pub is_set: bool,
}

impl Default for SpawnAnchor {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 0.0),
            is_set: false,
        }
    }
}

/// 落下追跡コンポーネント
#[derive(Component)]
pub struct FallTracker {
    pub highest_y: f32,
    pub is_on_ground: bool,
}

impl Default for FallTracker {
    fn default() -> Self {
        Self {
            highest_y: 0.0,
            is_on_ground: true,
        }
    }
}

/// プレイヤーステータスプラグイン
pub struct PlayerStatsPlugin;

impl Plugin for PlayerStatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnAnchor>()
            .add_event::<DamageEvent>()
            .add_event::<PlayerDeathEvent>()
            .add_event::<PlayerRespawnEvent>()
            .add_systems(
                Update,
                (
                    update_invincibility,
                    regenerate_health,
                    process_damage_events,
                    handle_death,
                    handle_respawn,
                    track_fall_damage,
                    check_void_death,
                ),
            );
    }
}

/// 無敵時間の更新
fn update_invincibility(time: Res<Time>, mut query: Query<&mut PlayerHealth>) {
    for mut health in query.iter_mut() {
        if health.invincibility_timer > 0.0 {
            health.invincibility_timer -= time.delta_secs();
        }
    }
}

/// 自然回復
fn regenerate_health(time: Res<Time>, mut query: Query<&mut PlayerHealth>) {
    let current_time = time.elapsed_secs();

    for mut health in query.iter_mut() {
        if health.current < health.max {
            // ダメージ後の遅延をチェック
            if current_time - health.last_damage_time > health.regeneration_delay {
                let heal_amount = health.regeneration_rate * time.delta_secs();
                health.heal(heal_amount);
            }
        }
    }
}

/// ダメージイベント処理
fn process_damage_events(
    time: Res<Time>,
    mut ev_damage: EventReader<DamageEvent>,
    mut ev_death: EventWriter<PlayerDeathEvent>,
    mut query: Query<(&mut PlayerHealth, &PlayerExperience)>,
) {
    let current_time = time.elapsed_secs();

    for event in ev_damage.read() {
        if let Ok((mut health, xp)) = query.get_mut(event.entity) {
            let died = health.take_damage(event.amount, current_time);

            if died {
                // 経験値ロスト（30%）
                let lost_xp = (xp.current as f32 * 0.3) as u32;

                ev_death.send(PlayerDeathEvent {
                    entity: event.entity,
                    source: event.source.clone(),
                    lost_xp,
                });
            }
        }
    }
}

/// 死亡処理
fn handle_death(
    mut ev_death: EventReader<PlayerDeathEvent>,
    mut ev_respawn: EventWriter<PlayerRespawnEvent>,
    mut query: Query<&mut PlayerExperience>,
) {
    for event in ev_death.read() {
        info!(
            "Player died! Source: {:?}, Lost XP: {}",
            event.source, event.lost_xp
        );

        // 経験値を減らす
        if let Ok(mut xp) = query.get_mut(event.entity) {
            xp.current = xp.current.saturating_sub(event.lost_xp);
        }

        // リスポーンをトリガー
        ev_respawn.send(PlayerRespawnEvent {
            entity: event.entity,
        });
    }
}

/// リスポーン処理
fn handle_respawn(
    mut ev_respawn: EventReader<PlayerRespawnEvent>,
    spawn_anchor: Res<SpawnAnchor>,
    mut query: Query<(&mut Transform, &mut PlayerHealth, Option<&mut FallTracker>)>,
) {
    for event in ev_respawn.read() {
        if let Ok((mut transform, mut health, fall_tracker)) = query.get_mut(event.entity) {
            // 位置をリセット
            transform.translation = spawn_anchor.position;

            // HP全回復
            health.full_heal();
            health.invincibility_timer = 3.0; // リスポーン後3秒無敵

            // 落下追跡をリセット
            if let Some(mut tracker) = fall_tracker {
                tracker.highest_y = spawn_anchor.position.y;
                tracker.is_on_ground = true;
            }

            info!("Player respawned at {:?}", spawn_anchor.position);
        }
    }
}

/// 落下ダメージ追跡
fn track_fall_damage(
    mut ev_damage: EventWriter<DamageEvent>,
    mut query: Query<(Entity, &Transform, &mut FallTracker)>,
) {
    for (entity, transform, mut tracker) in query.iter_mut() {
        let current_y = transform.translation.y;

        // 上昇中は最高点を更新
        if current_y > tracker.highest_y {
            tracker.highest_y = current_y;
            tracker.is_on_ground = false;
        }

        // 地面判定（簡易: Y座標が一定以下または前フレームから変化なし）
        // 実際のゲームではレイキャストで判定する
        if current_y <= 1.0 || (tracker.is_on_ground && current_y == tracker.highest_y) {
            if !tracker.is_on_ground {
                let fall_distance = tracker.highest_y - current_y;

                // 3ブロック以上の落下でダメージ
                if fall_distance > 3.0 {
                    let damage = (fall_distance - 3.0) * 0.5; // 超過1ブロックごとに0.5ダメージ

                    ev_damage.send(DamageEvent {
                        entity,
                        amount: damage,
                        source: DamageSource::Fall(fall_distance),
                    });
                }
            }

            tracker.is_on_ground = true;
            tracker.highest_y = current_y;
        }
    }
}

/// 奈落死チェック
fn check_void_death(
    mut ev_damage: EventWriter<DamageEvent>,
    query: Query<(Entity, &Transform, &PlayerHealth)>,
) {
    for (entity, transform, health) in query.iter() {
        // Y < -256 で奈落死
        if transform.translation.y < -256.0 && !health.is_dead() {
            ev_damage.send(DamageEvent {
                entity,
                amount: 1000.0, // 即死ダメージ
                source: DamageSource::Void,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_damage() {
        let mut health = PlayerHealth::default();
        assert_eq!(health.current, 10.0);

        health.take_damage(3.0, 0.0);
        assert_eq!(health.current, 7.0);
        assert!(!health.is_dead());

        // 無敵時間をクリアして再度ダメージ
        health.invincibility_timer = 0.0;
        health.take_damage(10.0, 1.0);
        assert_eq!(health.current, 0.0);
        assert!(health.is_dead());
    }

    #[test]
    fn test_health_heal() {
        let mut health = PlayerHealth {
            current: 5.0,
            ..Default::default()
        };

        health.heal(3.0);
        assert_eq!(health.current, 8.0);

        health.heal(10.0);
        assert_eq!(health.current, 10.0); // 最大HPを超えない
    }

    #[test]
    fn test_experience_level_up() {
        let mut xp = PlayerExperience::default();
        assert_eq!(xp.level, 0);

        xp.add_xp(150);
        assert_eq!(xp.level, 1);
        assert_eq!(xp.current, 50); // 100消費して50残り

        xp.add_xp(250);
        assert_eq!(xp.level, 2);
        assert_eq!(xp.current, 100); // 200消費して100残り
    }

    #[test]
    fn test_experience_spend() {
        let mut xp = PlayerExperience {
            current: 100, // 直接設定（add_xpだとレベルアップで消費される）
            ..Default::default()
        };

        assert!(xp.spend_xp(50));
        assert_eq!(xp.current, 50);

        assert!(!xp.spend_xp(100));
        assert_eq!(xp.current, 50); // 足りない場合は消費しない
    }

    #[test]
    fn test_invincibility() {
        let mut health = PlayerHealth::default();

        // 最初のダメージは通る
        assert!(!health.take_damage(3.0, 0.0));
        assert_eq!(health.current, 7.0);

        // 無敵中はダメージを受けない
        assert!(!health.take_damage(3.0, 0.1));
        assert_eq!(health.current, 7.0);

        // 無敵が切れた後
        health.invincibility_timer = 0.0;
        health.take_damage(3.0, 1.0);
        assert_eq!(health.current, 4.0);
    }
}
