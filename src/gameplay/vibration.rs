// src/gameplay/vibration.rs
//! 振動システム
//! - 機械の振動レベル
//! - 振動伝播（3x3x3範囲）
//! - 近隣機械への性能影響

use bevy::prelude::*;
use std::collections::HashMap;

/// 振動レベル（0-100）
pub type VibrationLevel = f32;

/// 振動を発生させるコンポーネント
#[derive(Component)]
pub struct VibrationSource {
    pub base_level: VibrationLevel,  // 基本振動レベル
    pub current_level: VibrationLevel, // 現在の振動レベル（動作中かどうかで変化）
    pub is_active: bool,
}

impl Default for VibrationSource {
    fn default() -> Self {
        Self {
            base_level: 10.0,
            current_level: 0.0,
            is_active: false,
        }
    }
}

impl VibrationSource {
    pub fn new(level: VibrationLevel) -> Self {
        Self {
            base_level: level,
            ..default()
        }
    }

    /// 静音アップグレードで振動を減少
    pub fn with_noise_reduction(mut self, reduction_percent: f32) -> Self {
        self.base_level *= 1.0 - (reduction_percent / 100.0);
        self
    }
}

/// 振動の影響を受けるコンポーネント
#[derive(Component)]
pub struct VibrationReceiver {
    pub received_vibration: VibrationLevel,
    pub sensitivity: f32,           // 振動感度（1.0が標準）
    pub efficiency_loss_rate: f32,  // 振動1ポイントあたりの効率低下率
}

impl Default for VibrationReceiver {
    fn default() -> Self {
        Self {
            received_vibration: 0.0,
            sensitivity: 1.0,
            efficiency_loss_rate: 0.01, // 1%/振動レベル
        }
    }
}

impl VibrationReceiver {
    /// 振動による効率低下を計算（0.0-1.0）
    pub fn efficiency_modifier(&self) -> f32 {
        let loss = self.received_vibration * self.efficiency_loss_rate * self.sensitivity;
        (1.0 - loss).clamp(0.1, 1.0) // 最低10%効率は維持
    }

    /// 精密機械用（振動に敏感）
    pub fn precision() -> Self {
        Self {
            sensitivity: 2.0,
            efficiency_loss_rate: 0.02,
            ..default()
        }
    }
}

/// 防振台コンポーネント
#[derive(Component)]
pub struct VibrationDamper {
    pub reduction_percent: f32,  // 振動減衰率
    pub range: i32,              // 効果範囲
}

impl Default for VibrationDamper {
    fn default() -> Self {
        Self {
            reduction_percent: 50.0,
            range: 1,
        }
    }
}

/// 振動マップ（位置 -> 振動レベル）
#[derive(Resource, Default)]
pub struct VibrationMap {
    pub levels: HashMap<IVec3, VibrationLevel>,
    pub dampers: HashMap<IVec3, (f32, i32)>, // 位置 -> (減衰率, 範囲)
}

impl VibrationMap {
    /// 指定位置の振動レベルを取得
    pub fn get_vibration(&self, pos: IVec3) -> VibrationLevel {
        *self.levels.get(&pos).unwrap_or(&0.0)
    }

    /// 振動レベルを更新（すべてクリアして再計算）
    pub fn clear(&mut self) {
        self.levels.clear();
    }

    /// 振動源から振動を伝播
    pub fn propagate_from(&mut self, source_pos: IVec3, level: VibrationLevel) {
        // 3x3x3範囲に振動を伝播
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let pos = source_pos + IVec3::new(dx, dy, dz);
                    let distance = (dx.abs() + dy.abs() + dz.abs()) as f32;

                    // 距離による減衰
                    let attenuated = level / (1.0 + distance * 0.5);

                    // 防振台による減衰
                    let damped = self.apply_damping(pos, attenuated);

                    // 既存の振動と加算
                    *self.levels.entry(pos).or_insert(0.0) += damped;
                }
            }
        }
    }

    /// 防振台による減衰を適用
    fn apply_damping(&self, pos: IVec3, vibration: VibrationLevel) -> VibrationLevel {
        let mut result = vibration;

        for (damper_pos, (reduction, range)) in &self.dampers {
            let distance = (pos.x - damper_pos.x).abs()
                + (pos.y - damper_pos.y).abs()
                + (pos.z - damper_pos.z).abs();

            if distance <= *range {
                result *= 1.0 - (reduction / 100.0);
            }
        }

        result
    }
}

/// 振動警告イベント
#[derive(Event)]
pub struct HighVibrationWarningEvent {
    pub entity: Entity,
    pub position: IVec3,
    pub vibration_level: VibrationLevel,
    pub efficiency_loss: f32,
}

/// 振動システムプラグイン
pub struct VibrationPlugin;

impl Plugin for VibrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VibrationMap>()
            .add_event::<HighVibrationWarningEvent>()
            .add_systems(FixedUpdate, (
                update_vibration_sources,
                update_dampers,
                propagate_vibration,
                apply_vibration_to_receivers,
                check_high_vibration,
            ).chain());
    }
}

/// 振動源の更新
fn update_vibration_sources(
    mut sources: Query<&mut VibrationSource>,
) {
    for mut source in sources.iter_mut() {
        source.current_level = if source.is_active {
            source.base_level
        } else {
            0.0
        };
    }
}

/// 防振台の登録
fn update_dampers(
    mut vibration_map: ResMut<VibrationMap>,
    dampers: Query<(&VibrationDamper, &Transform)>,
) {
    vibration_map.dampers.clear();

    for (damper, transform) in dampers.iter() {
        let pos = transform.translation.as_ivec3();
        vibration_map.dampers.insert(pos, (damper.reduction_percent, damper.range));
    }
}

/// 振動の伝播
fn propagate_vibration(
    mut vibration_map: ResMut<VibrationMap>,
    sources: Query<(&VibrationSource, &Transform)>,
) {
    vibration_map.clear();

    for (source, transform) in sources.iter() {
        if source.current_level > 0.0 {
            let pos = transform.translation.as_ivec3();
            vibration_map.propagate_from(pos, source.current_level);
        }
    }
}

/// 振動受信コンポーネントへの適用
fn apply_vibration_to_receivers(
    vibration_map: Res<VibrationMap>,
    mut receivers: Query<(&mut VibrationReceiver, &Transform)>,
) {
    for (mut receiver, transform) in receivers.iter_mut() {
        let pos = transform.translation.as_ivec3();
        receiver.received_vibration = vibration_map.get_vibration(pos);
    }
}

/// 高振動警告
fn check_high_vibration(
    mut ev_warning: EventWriter<HighVibrationWarningEvent>,
    receivers: Query<(Entity, &VibrationReceiver, &Transform)>,
) {
    const WARNING_THRESHOLD: VibrationLevel = 30.0;

    for (entity, receiver, transform) in receivers.iter() {
        if receiver.received_vibration > WARNING_THRESHOLD {
            ev_warning.send(HighVibrationWarningEvent {
                entity,
                position: transform.translation.as_ivec3(),
                vibration_level: receiver.received_vibration,
                efficiency_loss: 1.0 - receiver.efficiency_modifier(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vibration_source() {
        let mut source = VibrationSource::new(50.0);
        assert_eq!(source.current_level, 0.0);

        source.is_active = true;
        // update_vibration_sourcesで更新される
        source.current_level = source.base_level;
        assert_eq!(source.current_level, 50.0);
    }

    #[test]
    fn test_vibration_reduction() {
        let source = VibrationSource::new(100.0).with_noise_reduction(30.0);
        assert_eq!(source.base_level, 70.0);
    }

    #[test]
    fn test_receiver_efficiency() {
        let mut receiver = VibrationReceiver::default();
        assert_eq!(receiver.efficiency_modifier(), 1.0);

        receiver.received_vibration = 20.0;
        let eff = receiver.efficiency_modifier();
        assert!((eff - 0.8).abs() < 0.01); // 20 * 0.01 = 20%低下
    }

    #[test]
    fn test_precision_receiver() {
        let mut receiver = VibrationReceiver::precision();
        receiver.received_vibration = 20.0;
        let eff = receiver.efficiency_modifier();
        // 20 * 0.02 * 2.0 = 80%低下 -> 20%効率
        assert!((eff - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_vibration_propagation() {
        let mut map = VibrationMap::default();
        map.propagate_from(IVec3::ZERO, 100.0);

        // 中心は最大
        let center = map.get_vibration(IVec3::ZERO);
        assert!(center > 50.0);

        // 隣接は減衰
        let adjacent = map.get_vibration(IVec3::X);
        assert!(adjacent < center);
        assert!(adjacent > 0.0);

        // 範囲外はゼロ
        let far = map.get_vibration(IVec3::new(5, 0, 0));
        assert_eq!(far, 0.0);
    }

    #[test]
    fn test_damper_effect() {
        let mut map = VibrationMap::default();
        map.dampers.insert(IVec3::ZERO, (50.0, 1));
        map.propagate_from(IVec3::new(2, 0, 0), 100.0);

        // 防振台の範囲内は減衰
        let damped = map.get_vibration(IVec3::ZERO);
        let undamped = map.get_vibration(IVec3::new(3, 0, 0));

        // 同距離でも防振台の有無で差がある
        assert!(damped < undamped * 0.8);
    }
}
