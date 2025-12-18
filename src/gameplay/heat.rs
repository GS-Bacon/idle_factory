// src/gameplay/heat.rs
//! 熱システム
//! - 熱源（ボイラー、溶岩、電熱器）
//! - 熱伝導（隣接ブロック）
//! - 機械の温度管理（適正温度、過熱）

use bevy::prelude::*;
use std::collections::HashMap;

/// 温度単位（℃）
pub type Temperature = f32;

/// 熱を持つコンポーネント
#[derive(Component)]
pub struct HeatContainer {
    pub temperature: Temperature,  // 現在温度
    pub heat_capacity: f32,        // 熱容量（J/K）
    pub thermal_conductivity: f32, // 熱伝導率（0.0-1.0）
    pub ambient_loss_rate: f32,    // 環境への放熱率
}

impl Default for HeatContainer {
    fn default() -> Self {
        Self {
            temperature: 20.0,         // 室温
            heat_capacity: 1000.0,
            thermal_conductivity: 0.5,
            ambient_loss_rate: 0.01,
        }
    }
}

impl HeatContainer {
    /// 熱を加える
    pub fn add_heat(&mut self, heat_joules: f32) {
        self.temperature += heat_joules / self.heat_capacity;
    }

    /// 熱を奪う
    pub fn remove_heat(&mut self, heat_joules: f32) {
        self.temperature -= heat_joules / self.heat_capacity;
        self.temperature = self.temperature.max(-273.15); // 絶対零度以下にならない
    }
}

/// 熱源コンポーネント
#[derive(Component)]
pub struct HeatSource {
    pub heat_output: f32,          // 出力熱量（J/tick）
    pub max_temperature: Temperature, // 最大温度
    pub is_active: bool,
}

impl Default for HeatSource {
    fn default() -> Self {
        Self {
            heat_output: 100.0,
            max_temperature: 1000.0,
            is_active: false,
        }
    }
}

/// ボイラーコンポーネント
#[derive(Component)]
pub struct Boiler {
    pub fuel_burn_time: f32,       // 残り燃焼時間
    pub water_mb: u32,             // 水量（mb）
    pub steam_mb: u32,             // 蒸気量（mb）
    pub water_capacity: u32,       // 水タンク容量
    pub steam_capacity: u32,       // 蒸気タンク容量
    pub efficiency: f32,           // 効率
}

impl Default for Boiler {
    fn default() -> Self {
        Self {
            fuel_burn_time: 0.0,
            water_mb: 0,
            steam_mb: 0,
            water_capacity: 10_000,
            steam_capacity: 10_000,
            efficiency: 0.8,
        }
    }
}

impl Boiler {
    /// 蒸気生成レート（mb/tick）- 温度依存
    pub fn steam_rate(&self, temperature: Temperature) -> u32 {
        if temperature < 100.0 {
            0
        } else {
            // 100℃以上で蒸気生成開始、300℃で最大効率
            let efficiency = ((temperature - 100.0) / 200.0).clamp(0.0, 1.0);
            (100.0 * efficiency * self.efficiency) as u32
        }
    }

    /// 燃料を追加
    pub fn add_fuel(&mut self, burn_time: f32) {
        self.fuel_burn_time += burn_time;
    }

    /// 水を追加
    pub fn add_water(&mut self, amount: u32) -> u32 {
        let space = self.water_capacity.saturating_sub(self.water_mb);
        let to_add = amount.min(space);
        self.water_mb += to_add;
        amount - to_add
    }

    /// 蒸気を取り出す
    pub fn extract_steam(&mut self, max_amount: u32) -> u32 {
        let to_extract = self.steam_mb.min(max_amount);
        self.steam_mb -= to_extract;
        to_extract
    }
}

/// 機械の温度要件
#[derive(Component)]
pub struct TemperatureRequirement {
    pub min_temp: Temperature,     // 最低動作温度
    pub optimal_min: Temperature,  // 最適温度下限
    pub optimal_max: Temperature,  // 最適温度上限
    pub max_temp: Temperature,     // 最高動作温度（超えると破損）
}

impl Default for TemperatureRequirement {
    fn default() -> Self {
        Self {
            min_temp: 0.0,
            optimal_min: 20.0,
            optimal_max: 40.0,
            max_temp: 100.0,
        }
    }
}

impl TemperatureRequirement {
    /// 製錬炉用
    pub fn smelting() -> Self {
        Self {
            min_temp: 800.0,
            optimal_min: 1000.0,
            optimal_max: 1500.0,
            max_temp: 2000.0,
        }
    }

    /// 効率を計算（0.0-1.0）
    pub fn efficiency(&self, temp: Temperature) -> f32 {
        if temp < self.min_temp || temp > self.max_temp {
            0.0
        } else if temp >= self.optimal_min && temp <= self.optimal_max {
            1.0
        } else if temp < self.optimal_min {
            (temp - self.min_temp) / (self.optimal_min - self.min_temp)
        } else {
            (self.max_temp - temp) / (self.max_temp - self.optimal_max)
        }
    }

    /// 過熱状態か
    pub fn is_overheating(&self, temp: Temperature) -> bool {
        temp > self.max_temp
    }
}

/// 熱ネットワーク（隣接ブロック間の熱伝導）
#[derive(Resource, Default)]
pub struct HeatNetwork {
    pub positions: HashMap<IVec3, Entity>,
}

/// 機械過熱イベント
#[derive(Event)]
pub struct MachineOverheatEvent {
    pub entity: Entity,
    pub position: IVec3,
    pub temperature: Temperature,
}

/// 熱システムプラグイン
pub struct HeatPlugin;

impl Plugin for HeatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeatNetwork>()
            .add_event::<MachineOverheatEvent>()
            .add_systems(FixedUpdate, (
                tick_heat_sources,
                tick_boilers,
                conduct_heat,
                ambient_cooling,
                check_overheating,
            ));
    }
}

/// 熱源の処理
fn tick_heat_sources(
    mut query: Query<(&HeatSource, &mut HeatContainer)>,
) {
    for (source, mut container) in query.iter_mut() {
        if source.is_active && container.temperature < source.max_temperature {
            container.add_heat(source.heat_output);
        }
    }
}

/// ボイラーの処理
fn tick_boilers(
    time: Res<Time>,
    mut boilers: Query<(&mut Boiler, &mut HeatContainer, Option<&HeatSource>)>,
) {
    let dt = time.delta_secs();

    for (mut boiler, mut container, heat_source) in boilers.iter_mut() {
        // 燃料消費
        if boiler.fuel_burn_time > 0.0 {
            boiler.fuel_burn_time -= dt;

            // 熱源がある場合は自動で熱を加える
            if heat_source.is_some_and(|s| s.is_active) {
                // HeatSourceのtick_heat_sourcesで処理
            }
        }

        // 蒸気生成
        if container.temperature >= 100.0 && boiler.water_mb > 0 {
            let steam_rate = boiler.steam_rate(container.temperature);
            let water_consumed = steam_rate.min(boiler.water_mb);
            let steam_produced = water_consumed; // 1:1変換

            let steam_space = boiler.steam_capacity.saturating_sub(boiler.steam_mb);
            let actual_steam = steam_produced.min(steam_space);

            if actual_steam > 0 {
                boiler.water_mb -= actual_steam;
                boiler.steam_mb += actual_steam;

                // 蒸気生成で熱を消費
                container.remove_heat(actual_steam as f32 * 2.26); // 気化熱
            }
        }
    }
}

/// 熱伝導
fn conduct_heat(
    network: Res<HeatNetwork>,
    mut containers: Query<(&mut HeatContainer, &Transform)>,
) {
    // 位置と温度のマップを作成
    let positions: HashMap<IVec3, (Entity, Temperature, f32)> = containers
        .iter()
        .map(|(container, transform)| {
            let pos = transform.translation.as_ivec3();
            (pos, (Entity::PLACEHOLDER, container.temperature, container.thermal_conductivity))
        })
        .collect();

    // 熱伝導計算
    let mut heat_transfers: HashMap<IVec3, f32> = HashMap::new();

    for (pos, (_entity, temp, conductivity)) in positions.iter() {
        let neighbors = [
            IVec3::X,
            IVec3::NEG_X,
            IVec3::Y,
            IVec3::NEG_Y,
            IVec3::Z,
            IVec3::NEG_Z,
        ];

        for offset in neighbors {
            let neighbor_pos = *pos + offset;
            if let Some((_n_entity, n_temp, n_conductivity)) = positions.get(&neighbor_pos) {
                let temp_diff = n_temp - temp;
                let avg_conductivity = (conductivity + n_conductivity) / 2.0;
                let heat_flow = temp_diff * avg_conductivity * 0.1;

                *heat_transfers.entry(*pos).or_insert(0.0) += heat_flow;
            }
        }
    }

    // 熱を適用
    for (mut container, transform) in containers.iter_mut() {
        let pos = transform.translation.as_ivec3();
        if let Some(&heat) = heat_transfers.get(&pos) {
            container.add_heat(heat);
        }
    }

    let _ = network; // 将来のネットワーク最適化用
}

/// 環境への放熱
fn ambient_cooling(
    mut containers: Query<&mut HeatContainer>,
) {
    const AMBIENT_TEMP: f32 = 20.0;

    for mut container in containers.iter_mut() {
        let temp_diff = container.temperature - AMBIENT_TEMP;
        let heat_loss = temp_diff * container.ambient_loss_rate;
        container.remove_heat(heat_loss);
    }
}

/// 過熱チェック
fn check_overheating(
    mut ev_overheat: EventWriter<MachineOverheatEvent>,
    query: Query<(Entity, &HeatContainer, &TemperatureRequirement, &Transform)>,
) {
    for (entity, container, requirement, transform) in query.iter() {
        if requirement.is_overheating(container.temperature) {
            ev_overheat.send(MachineOverheatEvent {
                entity,
                position: transform.translation.as_ivec3(),
                temperature: container.temperature,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heat_container() {
        let mut container = HeatContainer::default();
        assert_eq!(container.temperature, 20.0);

        container.add_heat(1000.0);
        assert!((container.temperature - 21.0).abs() < 0.01);

        container.remove_heat(500.0);
        assert!((container.temperature - 20.5).abs() < 0.01);
    }

    #[test]
    fn test_boiler_steam_generation() {
        let boiler = Boiler::default();

        // 低温では蒸気生成なし
        assert_eq!(boiler.steam_rate(50.0), 0);
        assert_eq!(boiler.steam_rate(100.0), 0); // 100℃ちょうどは効率0

        // 100℃より上で生成開始
        assert!(boiler.steam_rate(150.0) > 0);

        // 高温で効率UP
        assert!(boiler.steam_rate(300.0) > boiler.steam_rate(150.0));
    }

    #[test]
    fn test_temperature_requirement() {
        let req = TemperatureRequirement::default();

        // 範囲外は効率0
        assert_eq!(req.efficiency(-10.0), 0.0);
        assert_eq!(req.efficiency(150.0), 0.0);

        // 最適範囲は効率1.0
        assert_eq!(req.efficiency(30.0), 1.0);

        // 中間範囲は部分効率
        let eff = req.efficiency(10.0);
        assert!(eff > 0.0 && eff < 1.0);
    }

    #[test]
    fn test_smelting_requirement() {
        let req = TemperatureRequirement::smelting();

        // 低温では動作しない
        assert_eq!(req.efficiency(500.0), 0.0);

        // 最適温度で効率100%
        assert_eq!(req.efficiency(1200.0), 1.0);

        // 過熱チェック
        assert!(!req.is_overheating(1500.0));
        assert!(req.is_overheating(2100.0));
    }
}
