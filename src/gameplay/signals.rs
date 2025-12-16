// src/gameplay/signals.rs
//! シグナルシステム
//! - ワイヤーベースの信号伝送
//! - ロジックゲート（AND, OR, NOT, XOR）
//! - 数値信号処理（加算、乗算、比較）

use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use super::scripting::SignalValue;

/// シグナルシステムプラグイン
pub struct SignalPlugin;

impl Plugin for SignalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SignalNetwork>()
            .add_systems(FixedUpdate, (
                propagate_signals,
                tick_logic_gates,
            ).chain());
    }
}

/// シグナルネットワーク（グローバルな信号グラフ）
#[derive(Resource, Default)]
pub struct SignalNetwork {
    /// ワイヤー接続（position -> connected positions）
    pub connections: HashMap<IVec3, HashSet<IVec3>>,
    /// 現在の信号値（position, channel -> value）
    pub values: HashMap<(IVec3, String), SignalValue>,
}

impl SignalNetwork {
    /// ワイヤー接続を追加
    pub fn connect(&mut self, from: IVec3, to: IVec3) {
        self.connections.entry(from).or_default().insert(to);
        self.connections.entry(to).or_default().insert(from);
    }

    /// ワイヤー接続を削除
    pub fn disconnect(&mut self, from: IVec3, to: IVec3) {
        if let Some(set) = self.connections.get_mut(&from) {
            set.remove(&to);
        }
        if let Some(set) = self.connections.get_mut(&to) {
            set.remove(&from);
        }
    }

    /// 信号値を設定
    pub fn set_signal(&mut self, pos: IVec3, channel: &str, value: SignalValue) {
        self.values.insert((pos, channel.to_string()), value);
    }

    /// 信号値を取得
    pub fn get_signal(&self, pos: IVec3, channel: &str) -> Option<&SignalValue> {
        self.values.get(&(pos, channel.to_string()))
    }

    /// 接続されているノードを取得
    pub fn get_connected(&self, pos: IVec3) -> Vec<IVec3> {
        self.connections.get(&pos).map(|s| s.iter().copied().collect()).unwrap_or_default()
    }

    /// BFSでネットワークグループを検出
    pub fn find_network_group(&self, start: IVec3) -> HashSet<IVec3> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = self.connections.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        visited
    }
}

/// ワイヤーコンポーネント
#[derive(Component)]
pub struct Wire {
    pub channel: String,
}

impl Default for Wire {
    fn default() -> Self {
        Self {
            channel: "default".to_string(),
        }
    }
}

/// シグナル発信コンポーネント
#[derive(Component)]
pub struct SignalEmitter {
    pub outputs: HashMap<String, SignalValue>,
}

impl Default for SignalEmitter {
    fn default() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }
}

impl SignalEmitter {
    pub fn set(&mut self, channel: &str, value: SignalValue) {
        self.outputs.insert(channel.to_string(), value);
    }
}

/// シグナル受信コンポーネント
#[derive(Component)]
pub struct SignalReceiver {
    pub inputs: HashMap<String, SignalValue>,
}

impl Default for SignalReceiver {
    fn default() -> Self {
        Self {
            inputs: HashMap::new(),
        }
    }
}

impl SignalReceiver {
    pub fn get(&self, channel: &str) -> Option<&SignalValue> {
        self.inputs.get(channel)
    }

    pub fn get_number(&self, channel: &str) -> Option<f64> {
        self.inputs.get(channel).and_then(|v| v.as_number())
    }

    pub fn get_bool(&self, channel: &str) -> Option<bool> {
        self.inputs.get(channel).and_then(|v| v.as_bool())
    }
}

/// ロジックゲートの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicGateType {
    And,
    Or,
    Not,
    Xor,
    Nand,
    Nor,
    // 数値処理
    Add,
    Subtract,
    Multiply,
    Divide,
    Compare,  // a > b
    Equal,    // a == b
}

/// ロジックゲートコンポーネント
#[derive(Component)]
pub struct LogicGate {
    pub gate_type: LogicGateType,
    pub input_a: String,
    pub input_b: String,
    pub output: String,
}

impl LogicGate {
    pub fn new(gate_type: LogicGateType) -> Self {
        Self {
            gate_type,
            input_a: "a".to_string(),
            input_b: "b".to_string(),
            output: "out".to_string(),
        }
    }

    pub fn with_channels(mut self, input_a: &str, input_b: &str, output: &str) -> Self {
        self.input_a = input_a.to_string();
        self.input_b = input_b.to_string();
        self.output = output.to_string();
        self
    }

    pub fn compute(&self, a: Option<&SignalValue>, b: Option<&SignalValue>) -> SignalValue {
        match self.gate_type {
            LogicGateType::And => {
                let a = a.and_then(|v| v.as_bool()).unwrap_or(false);
                let b = b.and_then(|v| v.as_bool()).unwrap_or(false);
                SignalValue::Boolean(a && b)
            }
            LogicGateType::Or => {
                let a = a.and_then(|v| v.as_bool()).unwrap_or(false);
                let b = b.and_then(|v| v.as_bool()).unwrap_or(false);
                SignalValue::Boolean(a || b)
            }
            LogicGateType::Not => {
                let a = a.and_then(|v| v.as_bool()).unwrap_or(false);
                SignalValue::Boolean(!a)
            }
            LogicGateType::Xor => {
                let a = a.and_then(|v| v.as_bool()).unwrap_or(false);
                let b = b.and_then(|v| v.as_bool()).unwrap_or(false);
                SignalValue::Boolean(a ^ b)
            }
            LogicGateType::Nand => {
                let a = a.and_then(|v| v.as_bool()).unwrap_or(false);
                let b = b.and_then(|v| v.as_bool()).unwrap_or(false);
                SignalValue::Boolean(!(a && b))
            }
            LogicGateType::Nor => {
                let a = a.and_then(|v| v.as_bool()).unwrap_or(false);
                let b = b.and_then(|v| v.as_bool()).unwrap_or(false);
                SignalValue::Boolean(!(a || b))
            }
            LogicGateType::Add => {
                let a = a.and_then(|v| v.as_number()).unwrap_or(0.0);
                let b = b.and_then(|v| v.as_number()).unwrap_or(0.0);
                SignalValue::Number(a + b)
            }
            LogicGateType::Subtract => {
                let a = a.and_then(|v| v.as_number()).unwrap_or(0.0);
                let b = b.and_then(|v| v.as_number()).unwrap_or(0.0);
                SignalValue::Number(a - b)
            }
            LogicGateType::Multiply => {
                let a = a.and_then(|v| v.as_number()).unwrap_or(0.0);
                let b = b.and_then(|v| v.as_number()).unwrap_or(0.0);
                SignalValue::Number(a * b)
            }
            LogicGateType::Divide => {
                let a = a.and_then(|v| v.as_number()).unwrap_or(0.0);
                let b = b.and_then(|v| v.as_number()).unwrap_or(1.0);
                if b != 0.0 {
                    SignalValue::Number(a / b)
                } else {
                    SignalValue::Number(0.0)
                }
            }
            LogicGateType::Compare => {
                let a = a.and_then(|v| v.as_number()).unwrap_or(0.0);
                let b = b.and_then(|v| v.as_number()).unwrap_or(0.0);
                SignalValue::Boolean(a > b)
            }
            LogicGateType::Equal => {
                let a = a.and_then(|v| v.as_number()).unwrap_or(0.0);
                let b = b.and_then(|v| v.as_number()).unwrap_or(0.0);
                SignalValue::Boolean((a - b).abs() < f64::EPSILON)
            }
        }
    }
}

/// 信号伝播システム
fn propagate_signals(
    mut network: ResMut<SignalNetwork>,
    emitters: Query<(&Transform, &SignalEmitter)>,
    mut receivers: Query<(&Transform, &mut SignalReceiver)>,
) {
    // エミッターからの信号をネットワークに設定
    for (transform, emitter) in &emitters {
        let pos = IVec3::new(
            transform.translation.x as i32,
            transform.translation.y as i32,
            transform.translation.z as i32,
        );

        for (channel, value) in &emitter.outputs {
            network.set_signal(pos, channel, value.clone());
        }
    }

    // レシーバーに信号を伝播
    for (transform, mut receiver) in &mut receivers {
        let pos = IVec3::new(
            transform.translation.x as i32,
            transform.translation.y as i32,
            transform.translation.z as i32,
        );

        // 接続されているノードから信号を収集
        let connected = network.get_connected(pos);
        for neighbor_pos in connected {
            // すべてのチャンネルをチェック
            let keys: Vec<_> = network.values.keys()
                .filter(|(p, _)| *p == neighbor_pos)
                .cloned()
                .collect();

            for (_, channel) in keys {
                if let Some(value) = network.get_signal(neighbor_pos, &channel) {
                    receiver.inputs.insert(channel.clone(), value.clone());
                }
            }
        }
    }
}

/// ロジックゲート処理システム
fn tick_logic_gates(
    mut query: Query<(&SignalReceiver, &LogicGate, &mut SignalEmitter)>,
) {
    for (receiver, gate, mut emitter) in &mut query {
        let a = receiver.inputs.get(&gate.input_a);
        let b = receiver.inputs.get(&gate.input_b);

        let result = gate.compute(a, b);
        emitter.set(&gate.output, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_gate_and() {
        let gate = LogicGate::new(LogicGateType::And);

        assert_eq!(
            gate.compute(Some(&SignalValue::Boolean(true)), Some(&SignalValue::Boolean(true))),
            SignalValue::Boolean(true)
        );
        assert_eq!(
            gate.compute(Some(&SignalValue::Boolean(true)), Some(&SignalValue::Boolean(false))),
            SignalValue::Boolean(false)
        );
    }

    #[test]
    fn test_logic_gate_or() {
        let gate = LogicGate::new(LogicGateType::Or);

        assert_eq!(
            gate.compute(Some(&SignalValue::Boolean(false)), Some(&SignalValue::Boolean(true))),
            SignalValue::Boolean(true)
        );
        assert_eq!(
            gate.compute(Some(&SignalValue::Boolean(false)), Some(&SignalValue::Boolean(false))),
            SignalValue::Boolean(false)
        );
    }

    #[test]
    fn test_logic_gate_not() {
        let gate = LogicGate::new(LogicGateType::Not);

        assert_eq!(
            gate.compute(Some(&SignalValue::Boolean(true)), None),
            SignalValue::Boolean(false)
        );
        assert_eq!(
            gate.compute(Some(&SignalValue::Boolean(false)), None),
            SignalValue::Boolean(true)
        );
    }

    #[test]
    fn test_logic_gate_add() {
        let gate = LogicGate::new(LogicGateType::Add);

        assert_eq!(
            gate.compute(Some(&SignalValue::Number(10.0)), Some(&SignalValue::Number(5.0))),
            SignalValue::Number(15.0)
        );
    }

    #[test]
    fn test_logic_gate_compare() {
        let gate = LogicGate::new(LogicGateType::Compare);

        assert_eq!(
            gate.compute(Some(&SignalValue::Number(10.0)), Some(&SignalValue::Number(5.0))),
            SignalValue::Boolean(true)
        );
        assert_eq!(
            gate.compute(Some(&SignalValue::Number(3.0)), Some(&SignalValue::Number(5.0))),
            SignalValue::Boolean(false)
        );
    }

    #[test]
    fn test_signal_network_connect() {
        let mut network = SignalNetwork::default();

        network.connect(IVec3::new(0, 0, 0), IVec3::new(1, 0, 0));
        network.connect(IVec3::new(1, 0, 0), IVec3::new(2, 0, 0));

        let connected = network.get_connected(IVec3::new(1, 0, 0));
        assert_eq!(connected.len(), 2);
    }

    #[test]
    fn test_signal_network_find_group() {
        let mut network = SignalNetwork::default();

        network.connect(IVec3::new(0, 0, 0), IVec3::new(1, 0, 0));
        network.connect(IVec3::new(1, 0, 0), IVec3::new(2, 0, 0));
        network.connect(IVec3::new(2, 0, 0), IVec3::new(3, 0, 0));

        let group = network.find_network_group(IVec3::new(0, 0, 0));
        assert_eq!(group.len(), 4);
    }
}
