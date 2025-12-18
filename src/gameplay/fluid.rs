// src/gameplay/fluid.rs
//! 液体・気体システム
//! - パイプネットワーク
//! - タンク
//! - 液体流動

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// 液体の種類
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FluidType {
    pub id: String,
    pub is_gas: bool,
    pub is_hot: bool,       // 高温液体（溶岩など）
    pub viscosity: u32,     // 粘度（0-100、高いほど遅い）
    pub temperature: i32,   // 温度（℃）
}

impl FluidType {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            is_gas: false,
            is_hot: false,
            viscosity: 10,
            temperature: 20,
        }
    }

    pub fn water() -> Self {
        Self::new("water")
    }

    pub fn lava() -> Self {
        Self {
            id: "lava".to_string(),
            is_gas: false,
            is_hot: true,
            viscosity: 80,
            temperature: 1200,
        }
    }

    pub fn steam() -> Self {
        Self {
            id: "steam".to_string(),
            is_gas: true,
            is_hot: true,
            viscosity: 1,
            temperature: 100,
        }
    }
}

/// 液体の量（ミリバケツ単位）
pub type FluidAmount = u32;

/// 液体スタック
#[derive(Debug, Clone)]
pub struct FluidStack {
    pub fluid_type: FluidType,
    pub amount: FluidAmount,
}

impl FluidStack {
    pub fn new(fluid_type: FluidType, amount: FluidAmount) -> Self {
        Self { fluid_type, amount }
    }

    pub fn is_empty(&self) -> bool {
        self.amount == 0
    }
}

/// パイプコンポーネント
#[derive(Component)]
pub struct Pipe {
    pub tier: u8,                    // 上位パイプほど流量多い
    pub fluid: Option<FluidStack>,   // 含まれている液体
    pub max_flow_rate: u32,          // 最大流量 (mb/tick)
    pub network_id: Option<u32>,     // 所属ネットワークID
}

impl Default for Pipe {
    fn default() -> Self {
        Self {
            tier: 1,
            fluid: None,
            max_flow_rate: 100, // 100mb/tick
            network_id: None,
        }
    }
}

impl Pipe {
    pub fn new(tier: u8) -> Self {
        let max_flow_rate = match tier {
            1 => 100,
            2 => 500,
            3 => 2000,
            _ => 100,
        };

        Self {
            tier,
            max_flow_rate,
            ..default()
        }
    }

    /// 液体を受け入れ可能か
    pub fn can_accept(&self, fluid_type: &FluidType) -> bool {
        match &self.fluid {
            None => true,
            Some(stack) => stack.fluid_type.id == fluid_type.id,
        }
    }

    /// 空かどうか
    pub fn is_empty(&self) -> bool {
        self.fluid.is_none() || self.fluid.as_ref().is_some_and(|s| s.is_empty())
    }
}

/// タンクコンポーネント
#[derive(Component)]
pub struct Tank {
    pub capacity: FluidAmount,       // 容量 (mb)
    pub fluid: Option<FluidStack>,   // 含まれている液体
    pub is_high_temp: bool,          // 高温対応タンクか
    pub input_sides: u8,             // 入力可能な面（ビットマスク）
    pub output_sides: u8,            // 出力可能な面（ビットマスク）
}

impl Default for Tank {
    fn default() -> Self {
        Self {
            capacity: 10_000, // 10バケツ
            fluid: None,
            is_high_temp: false,
            input_sides: 0b111111,  // 全面入力可
            output_sides: 0b000001, // 下面のみ出力
        }
    }
}

impl Tank {
    pub fn new(capacity: FluidAmount) -> Self {
        Self {
            capacity,
            ..default()
        }
    }

    pub fn high_temp(capacity: FluidAmount) -> Self {
        Self {
            capacity,
            is_high_temp: true,
            ..default()
        }
    }

    /// 液体を追加
    pub fn fill(&mut self, fluid_type: FluidType, amount: FluidAmount) -> FluidAmount {
        // 異種液体は混合不可
        if let Some(ref stack) = self.fluid {
            if stack.fluid_type.id != fluid_type.id {
                return amount; // 全量返却
            }
        }

        // 高温液体チェック
        if fluid_type.is_hot && !self.is_high_temp {
            // 高温液体を通常タンクに入れると破損（将来実装）
            return amount;
        }

        let current_amount = self.fluid.as_ref().map_or(0, |s| s.amount);
        let space = self.capacity.saturating_sub(current_amount);
        let to_fill = amount.min(space);

        if to_fill > 0 {
            if let Some(ref mut stack) = self.fluid {
                stack.amount += to_fill;
            } else {
                self.fluid = Some(FluidStack::new(fluid_type, to_fill));
            }
        }

        amount - to_fill // 余った量を返却
    }

    /// 液体を排出
    pub fn drain(&mut self, max_amount: FluidAmount) -> Option<FluidStack> {
        if let Some(ref mut stack) = self.fluid {
            let to_drain = stack.amount.min(max_amount);
            stack.amount -= to_drain;

            let result = FluidStack::new(stack.fluid_type.clone(), to_drain);

            if stack.amount == 0 {
                self.fluid = None;
            }

            Some(result)
        } else {
            None
        }
    }

    /// 液面レベル（0.0-1.0）
    pub fn fill_level(&self) -> f32 {
        self.fluid.as_ref().map_or(0.0, |s| s.amount as f32 / self.capacity as f32)
    }

    /// 空きスペース
    pub fn free_space(&self) -> FluidAmount {
        let current = self.fluid.as_ref().map_or(0, |s| s.amount);
        self.capacity.saturating_sub(current)
    }
}

/// パイプネットワーク
#[derive(Resource, Default)]
pub struct PipeNetwork {
    pub networks: HashMap<u32, HashSet<IVec3>>,
    pub pipe_to_network: HashMap<IVec3, u32>,
    pub next_network_id: u32,
}

impl PipeNetwork {
    /// パイプを追加
    pub fn add_pipe(&mut self, pos: IVec3) {
        // 隣接パイプを探す
        let neighbors: Vec<u32> = [
            IVec3::X,
            IVec3::NEG_X,
            IVec3::Y,
            IVec3::NEG_Y,
            IVec3::Z,
            IVec3::NEG_Z,
        ]
        .iter()
        .filter_map(|&offset| self.pipe_to_network.get(&(pos + offset)).copied())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

        match neighbors.len() {
            0 => {
                // 新しいネットワーク
                let id = self.next_network_id;
                self.next_network_id += 1;

                let mut network = HashSet::new();
                network.insert(pos);
                self.networks.insert(id, network);
                self.pipe_to_network.insert(pos, id);
            }
            1 => {
                // 既存のネットワークに追加
                let network_id = neighbors[0];
                if let Some(network) = self.networks.get_mut(&network_id) {
                    network.insert(pos);
                }
                self.pipe_to_network.insert(pos, network_id);
            }
            _ => {
                // 複数ネットワークを統合
                let target_id = neighbors[0];

                for &other_id in &neighbors[1..] {
                    if let Some(other_network) = self.networks.remove(&other_id) {
                        let pipes_to_move: Vec<IVec3> = other_network.iter().copied().collect();
                        for p in &pipes_to_move {
                            self.pipe_to_network.insert(*p, target_id);
                        }
                        if let Some(target_network) = self.networks.get_mut(&target_id) {
                            target_network.extend(pipes_to_move);
                        }
                    }
                }

                if let Some(network) = self.networks.get_mut(&target_id) {
                    network.insert(pos);
                }
                self.pipe_to_network.insert(pos, target_id);
            }
        }
    }

    /// パイプを削除
    pub fn remove_pipe(&mut self, pos: IVec3) {
        if let Some(network_id) = self.pipe_to_network.remove(&pos) {
            if let Some(network) = self.networks.get_mut(&network_id) {
                network.remove(&pos);

                // ネットワークが分断されたかチェック（簡易実装：再構築）
                if !network.is_empty() {
                    // 全パイプを削除して再追加
                    let pipes: Vec<IVec3> = network.iter().copied().collect();
                    self.networks.remove(&network_id);

                    for p in &pipes {
                        self.pipe_to_network.remove(p);
                    }

                    for p in pipes {
                        self.add_pipe(p);
                    }
                } else {
                    self.networks.remove(&network_id);
                }
            }
        }
    }
}

/// ポンプコンポーネント
#[derive(Component)]
pub struct Pump {
    pub flow_rate: u32,    // 流量 (mb/tick)
    pub is_active: bool,
}

impl Default for Pump {
    fn default() -> Self {
        Self {
            flow_rate: 500,
            is_active: false,
        }
    }
}

/// ドレインバルブ
#[derive(Component)]
pub struct DrainValve {
    pub is_open: bool,
}

/// 液体システムプラグイン
pub struct FluidPlugin;

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PipeNetwork>()
            .add_systems(FixedUpdate, (
                tick_pumps,
                flow_fluids,
            ));
    }
}

/// ポンプの処理
fn tick_pumps(
    mut pumps: Query<(&Pump, &Transform)>,
    mut tanks: Query<(&mut Tank, &Transform)>,
    mut pipes: Query<(&mut Pipe, &Transform)>,
) {
    for (pump, pump_transform) in pumps.iter_mut() {
        if !pump.is_active {
            continue;
        }

        let pump_pos = pump_transform.translation.as_ivec3();

        // 入力側のタンクを探す
        let input_pos = pump_pos + IVec3::NEG_Y; // 下から吸い上げ

        // 出力側のパイプを探す
        let output_pos = pump_pos + IVec3::Y; // 上に送る

        // 入力タンクから液体を取得
        let mut drained: Option<FluidStack> = None;

        for (mut tank, tank_transform) in tanks.iter_mut() {
            if tank_transform.translation.as_ivec3() == input_pos {
                drained = tank.drain(pump.flow_rate);
                break;
            }
        }

        // 出力パイプに液体を送る
        if let Some(fluid) = drained {
            for (mut pipe, pipe_transform) in pipes.iter_mut() {
                if pipe_transform.translation.as_ivec3() == output_pos {
                    if pipe.can_accept(&fluid.fluid_type) {
                        pipe.fluid = Some(fluid);
                    }
                    break;
                }
            }
        }
    }
}

/// パイプネットワーク内の流体移動
fn flow_fluids(
    network: Res<PipeNetwork>,
    mut pipes: Query<(&mut Pipe, &Transform)>,
    mut tanks: Query<(&mut Tank, &Transform)>,
) {
    // 各ネットワークで液体を平均化
    for (_network_id, positions) in network.networks.iter() {
        let mut total_fluid: HashMap<String, u32> = HashMap::new();
        let pipe_count = positions.len() as u32;

        // 総液体量を計算
        for (pipe, transform) in pipes.iter() {
            if positions.contains(&transform.translation.as_ivec3()) {
                if let Some(ref stack) = pipe.fluid {
                    *total_fluid.entry(stack.fluid_type.id.clone()).or_insert(0) += stack.amount;
                }
            }
        }

        // 平均化して分配
        for (mut pipe, transform) in pipes.iter_mut() {
            if positions.contains(&transform.translation.as_ivec3()) {
                if let Some(ref mut stack) = pipe.fluid {
                    if let Some(&total) = total_fluid.get(&stack.fluid_type.id) {
                        stack.amount = total / pipe_count;
                    }
                }
            }
        }

        // タンクへの排出
        for pos in positions {
            let below = *pos + IVec3::NEG_Y;

            // パイプの液体を取得
            let mut pipe_fluid: Option<FluidStack> = None;
            for (mut pipe, transform) in pipes.iter_mut() {
                if transform.translation.as_ivec3() == *pos {
                    if let Some(ref mut stack) = pipe.fluid {
                        let drain_amount = stack.amount.min(100);
                        stack.amount -= drain_amount;
                        pipe_fluid = Some(FluidStack::new(stack.fluid_type.clone(), drain_amount));
                    }
                    break;
                }
            }

            // 下のタンクに注入
            if let Some(fluid) = pipe_fluid {
                for (mut tank, transform) in tanks.iter_mut() {
                    if transform.translation.as_ivec3() == below {
                        let _ = tank.fill(fluid.fluid_type, fluid.amount);
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tank_fill() {
        let mut tank = Tank::new(1000);
        assert_eq!(tank.fill_level(), 0.0);

        let remaining = tank.fill(FluidType::water(), 500);
        assert_eq!(remaining, 0);
        assert_eq!(tank.fill_level(), 0.5);

        let remaining = tank.fill(FluidType::water(), 600);
        assert_eq!(remaining, 100); // 100オーバー
        assert_eq!(tank.fill_level(), 1.0);
    }

    #[test]
    fn test_tank_drain() {
        let mut tank = Tank::new(1000);
        tank.fill(FluidType::water(), 500);

        let drained = tank.drain(200).unwrap();
        assert_eq!(drained.amount, 200);
        assert_eq!(tank.fluid.as_ref().unwrap().amount, 300);

        let drained = tank.drain(500).unwrap();
        assert_eq!(drained.amount, 300); // 残り全部
        assert!(tank.fluid.is_none());
    }

    #[test]
    fn test_tank_no_mix() {
        let mut tank = Tank::new(1000);
        tank.fill(FluidType::water(), 500);

        // 異種液体は入らない
        let remaining = tank.fill(FluidType::lava(), 200);
        assert_eq!(remaining, 200);
        assert_eq!(tank.fluid.as_ref().unwrap().fluid_type.id, "water");
    }

    #[test]
    fn test_pipe_network() {
        let mut network = PipeNetwork::default();

        network.add_pipe(IVec3::ZERO);
        network.add_pipe(IVec3::X);
        network.add_pipe(IVec3::new(5, 0, 0)); // 離れた位置

        // 隣接パイプは同じネットワーク
        assert_eq!(
            network.pipe_to_network.get(&IVec3::ZERO),
            network.pipe_to_network.get(&IVec3::X)
        );

        // 離れたパイプは別ネットワーク
        assert_ne!(
            network.pipe_to_network.get(&IVec3::X),
            network.pipe_to_network.get(&IVec3::new(5, 0, 0))
        );
    }

    #[test]
    fn test_pipe_can_accept() {
        let mut pipe = Pipe::default();
        assert!(pipe.can_accept(&FluidType::water()));

        pipe.fluid = Some(FluidStack::new(FluidType::water(), 100));
        assert!(pipe.can_accept(&FluidType::water()));
        assert!(!pipe.can_accept(&FluidType::lava()));
    }
}
