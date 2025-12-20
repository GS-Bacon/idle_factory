// src/gameplay/weather.rs
//! 天候・昼夜サイクルシステム
//! - 昼夜サイクル
//! - 天候（晴れ、雨、嵐）
//! - 天候効果（機械効率、植物成長など）

use bevy::prelude::*;
use std::f32::consts::PI;

/// 昼夜サイクルの時刻
#[derive(Resource)]
pub struct DayNightCycle {
    /// 現在の時刻（0.0 = 真夜中、0.5 = 正午、1.0 = 次の真夜中）
    pub time_of_day: f32,
    /// 1日の長さ（秒）
    pub day_length: f32,
    /// 経過日数
    pub day_count: u32,
    /// 時間の進行速度倍率
    pub time_scale: f32,
    /// 一時停止フラグ
    pub paused: bool,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time_of_day: 0.25, // 朝6時スタート
            day_length: 600.0, // 10分で1日
            day_count: 0,
            time_scale: 1.0,
            paused: false,
        }
    }
}

impl DayNightCycle {
    /// 時刻を時間形式で取得（0-24）
    pub fn get_hour(&self) -> f32 {
        self.time_of_day * 24.0
    }

    /// 昼間かどうか（6:00-18:00）
    pub fn is_daytime(&self) -> bool {
        let hour = self.get_hour();
        (6.0..18.0).contains(&hour)
    }

    /// 太陽の高度（0.0-1.0、正午で最大）
    pub fn sun_intensity(&self) -> f32 {
        let hour = self.get_hour();
        if (6.0..=18.0).contains(&hour) {
            // 6時から18時：sin曲線で0->1->0
            let progress = (hour - 6.0) / 12.0;
            (progress * PI).sin()
        } else {
            0.0
        }
    }

    /// 月の高度（0.0-1.0、真夜中で最大）
    pub fn moon_intensity(&self) -> f32 {
        let hour = self.get_hour();
        if !(6.0..18.0).contains(&hour) {
            // 18時から6時：sin曲線で0->1->0
            let progress = if hour >= 18.0 {
                (hour - 18.0) / 12.0
            } else {
                (hour + 6.0) / 12.0
            };
            (progress * PI).sin() * 0.3 // 月は太陽の30%の明るさ
        } else {
            0.0
        }
    }

    /// 環境光の強度
    pub fn ambient_light(&self) -> f32 {
        (self.sun_intensity() + self.moon_intensity()).clamp(0.1, 1.0)
    }
}

/// 天候の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherType {
    #[default]
    Clear,  // 晴れ
    Cloudy, // 曇り
    Rain,   // 雨
    Storm,  // 嵐
}

/// 天候状態
#[derive(Resource)]
pub struct Weather {
    pub current: WeatherType,
    pub intensity: f32,           // 0.0-1.0
    pub duration_remaining: f32,  // 残り時間（秒）
    pub transition_timer: f32,    // 遷移中のタイマー
    pub next_weather: Option<WeatherType>,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            current: WeatherType::Clear,
            intensity: 0.0,
            duration_remaining: 300.0, // 5分
            transition_timer: 0.0,
            next_weather: None,
        }
    }
}

impl Weather {
    /// 天候を変更
    pub fn change_weather(&mut self, weather: WeatherType, duration: f32) {
        self.next_weather = Some(weather);
        self.transition_timer = 30.0; // 30秒かけて遷移
        self.duration_remaining = duration;
    }

    /// 雨が降っているか
    pub fn is_raining(&self) -> bool {
        matches!(self.current, WeatherType::Rain | WeatherType::Storm)
    }

    /// 嵐かどうか
    pub fn is_stormy(&self) -> bool {
        self.current == WeatherType::Storm
    }
}

/// 天候効果の設定
#[derive(Resource)]
pub struct WeatherEffects {
    /// 水車効率ボーナス（雨）
    pub waterwheel_bonus: f32,
    /// 植物成長ボーナス（雨）
    pub plant_growth_bonus: f32,
    /// 野ざらし機械の効率低下（雨）
    pub exposed_machine_penalty: f32,
    /// 野ざらし機械の破損確率/秒（嵐）
    pub machine_damage_chance: f32,
}

impl Default for WeatherEffects {
    fn default() -> Self {
        Self {
            waterwheel_bonus: 0.5,        // 50%効率UP
            plant_growth_bonus: 0.3,      // 30%成長促進
            exposed_machine_penalty: 0.2, // 20%効率低下
            machine_damage_chance: 0.001, // 0.1%/秒の破損確率
        }
    }
}

/// 屋根の下かどうかを示すマーカー
#[derive(Component)]
pub struct UnderRoof;

/// 屋外機械（天候の影響を受ける）
#[derive(Component)]
pub struct ExposedMachine {
    pub weather_resistance: f32, // 0.0-1.0（1.0で完全耐性）
}

impl Default for ExposedMachine {
    fn default() -> Self {
        Self {
            weather_resistance: 0.0,
        }
    }
}

/// 天候変更イベント
#[derive(Event)]
pub struct WeatherChangedEvent {
    pub previous: WeatherType,
    pub current: WeatherType,
}

/// 天候システムプラグイン
pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DayNightCycle>()
            .init_resource::<Weather>()
            .init_resource::<WeatherEffects>()
            .add_event::<WeatherChangedEvent>()
            .add_systems(
                Update,
                (
                    update_day_night_cycle,
                    update_weather,
                    update_ambient_light,
                    apply_weather_effects,
                ),
            );
    }
}

/// 昼夜サイクルの更新
fn update_day_night_cycle(time: Res<Time>, mut cycle: ResMut<DayNightCycle>) {
    if cycle.paused {
        return;
    }

    let delta = time.delta_secs() * cycle.time_scale / cycle.day_length;
    cycle.time_of_day += delta;

    // 日付変更
    if cycle.time_of_day >= 1.0 {
        cycle.time_of_day -= 1.0;
        cycle.day_count += 1;
    }
}

/// 天候の更新
fn update_weather(
    time: Res<Time>,
    mut weather: ResMut<Weather>,
    mut ev_changed: EventWriter<WeatherChangedEvent>,
) {
    let dt = time.delta_secs();

    // 天候遷移中
    if let Some(next) = weather.next_weather {
        weather.transition_timer -= dt;
        weather.intensity = 1.0 - (weather.transition_timer / 30.0).clamp(0.0, 1.0);

        if weather.transition_timer <= 0.0 {
            let previous = weather.current;
            weather.current = next;
            weather.next_weather = None;

            ev_changed.send(WeatherChangedEvent {
                previous,
                current: weather.current,
            });
        }
    } else {
        // 天候継続中
        weather.duration_remaining -= dt;

        if weather.duration_remaining <= 0.0 {
            // 次の天候をランダムに選択（簡易実装）
            let next = match weather.current {
                WeatherType::Clear => {
                    if rand_simple() < 0.3 {
                        WeatherType::Cloudy
                    } else {
                        WeatherType::Clear
                    }
                }
                WeatherType::Cloudy => {
                    let r = rand_simple();
                    if r < 0.3 {
                        WeatherType::Rain
                    } else if r < 0.5 {
                        WeatherType::Clear
                    } else {
                        WeatherType::Cloudy
                    }
                }
                WeatherType::Rain => {
                    let r = rand_simple();
                    if r < 0.2 {
                        WeatherType::Storm
                    } else if r < 0.5 {
                        WeatherType::Cloudy
                    } else {
                        WeatherType::Rain
                    }
                }
                WeatherType::Storm => {
                    if rand_simple() < 0.6 {
                        WeatherType::Rain
                    } else {
                        WeatherType::Cloudy
                    }
                }
            };

            let duration = 180.0 + rand_simple() * 300.0; // 3-8分
            weather.change_weather(next, duration);
        }
    }
}

/// 環境光の更新（AmbientLightはResourceなので別途処理）
fn update_ambient_light(
    cycle: Res<DayNightCycle>,
    weather: Res<Weather>,
    mut ambient: Option<ResMut<AmbientLight>>,
) {
    if let Some(ref mut light) = ambient {
        let base_intensity = cycle.ambient_light();

        // 天候による減光
        let weather_modifier = match weather.current {
            WeatherType::Clear => 1.0,
            WeatherType::Cloudy => 0.7,
            WeatherType::Rain => 0.5,
            WeatherType::Storm => 0.3,
        };

        let final_intensity = base_intensity * weather_modifier;

        // 色温度も調整
        let color = if cycle.is_daytime() {
            Color::srgb(1.0, 0.98, 0.95) // 昼間は暖かみのある白
        } else {
            Color::srgb(0.7, 0.75, 0.9) // 夜は青みがかった色
        };

        light.color = color;
        light.brightness = final_intensity * 500.0;
    }
}

/// 天候効果の適用
fn apply_weather_effects(
    weather: Res<Weather>,
    effects: Res<WeatherEffects>,
    exposed_machines: Query<&ExposedMachine, Without<UnderRoof>>,
) {
    if !weather.is_raining() {
        return;
    }

    for machine in exposed_machines.iter() {
        // 天候耐性を考慮した効率低下
        let penalty = effects.exposed_machine_penalty * (1.0 - machine.weather_resistance);

        // ここで実際の機械効率を低下させる
        // 現在はマーカーのみ（実際の効率低下は各機械のtickシステムで処理）
        let _ = penalty;

        // 嵐時の破損チェック
        if weather.is_stormy() {
            let damage_chance = effects.machine_damage_chance * (1.0 - machine.weather_resistance);
            if rand_simple() < damage_chance {
                // 機械破損イベントを発火（将来実装）
                info!("Machine damaged by storm!");
            }
        }
    }
}

/// 簡易乱数（0.0-1.0）
fn rand_simple() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f32 / 1_000_000_000.0).fract()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_night_cycle() {
        let mut cycle = DayNightCycle::default();
        assert!(cycle.is_daytime()); // 朝6時スタート

        cycle.time_of_day = 0.0; // 真夜中
        assert!(!cycle.is_daytime());

        cycle.time_of_day = 0.5; // 正午
        assert!(cycle.is_daytime());
        assert!((cycle.sun_intensity() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_weather_change() {
        let mut weather = Weather::default();
        assert_eq!(weather.current, WeatherType::Clear);

        weather.change_weather(WeatherType::Rain, 300.0);
        assert_eq!(weather.next_weather, Some(WeatherType::Rain));
        assert!(weather.transition_timer > 0.0);
    }

    #[test]
    fn test_weather_effects() {
        let weather = Weather {
            current: WeatherType::Rain,
            ..default()
        };

        assert!(weather.is_raining());
        assert!(!weather.is_stormy());

        let storm_weather = Weather {
            current: WeatherType::Storm,
            ..default()
        };

        assert!(storm_weather.is_raining());
        assert!(storm_weather.is_stormy());
    }

    #[test]
    fn test_ambient_light() {
        let cycle = DayNightCycle {
            time_of_day: 0.5, // 正午
            ..default()
        };

        let light = cycle.ambient_light();
        assert!(light > 0.9); // 正午は明るい

        let night_cycle = DayNightCycle {
            time_of_day: 0.0, // 真夜中
            ..default()
        };

        let night_light = night_cycle.ambient_light();
        assert!(night_light < 0.4); // 夜は暗い
    }
}
