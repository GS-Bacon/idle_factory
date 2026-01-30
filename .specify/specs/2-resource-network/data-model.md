# Data Model: Resource Networks

**Feature**: リソースネットワーク（電力・流体・信号）
**Date**: 2026-01-30

## Overview

このドキュメントでは、電力・流体・信号の3つのネットワークシステムで使用するデータモデルを定義する。すべてのモデルは既存の `NetworkGraph<K, V>` 基盤を活用し、ECS Component として実装される。

---

## 共通データモデル

### GridId

```rust
// core/id.rs に追加
pub struct GridId(pub u32);
```

---

### NetworkGraph Extensions

```rust
// core/network.rs に型エイリアスを追加
pub type PowerNetwork = NetworkGraph<u64, f32>;
pub type FluidNetwork = NetworkGraph<u64, f32>;
pub type SignalNetwork = NetworkGraph<u64, u8>;
```

---

## 電力ネットワークモデル

### Components

#### PowerProducer

```rust
// components/power.rs
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct PowerProducer {
    pub output_watts: f32,
    pub fuel_slot: Option<u8>,
    pub startup_delay: f32,
    pub fuel_consumption_rate: Option<f32>,
    pub current_output: f32,
}
```

---

#### PowerConsumer

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct PowerConsumer {
    pub required_watts: f32,
    pub is_powered: bool,
}
```

---

#### PowerWire

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct PowerWire {
    pub network_id: GridId,
}
```

---

#### PowerNetworks (Resource)

```rust
#[derive(Resource, Default)]
pub struct PowerNetworks {
    pub networks: HashMap<GridId, PowerNetwork>,
    pub next_network_id: u32,
}
```

---

## 流体ネットワークモデル

### Components

#### FluidSource

```rust
// components/fluids.rs
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct FluidSource {
    pub fluid_type: FluidId,
    pub extraction_rate: f32,
    pub current_extraction: f32,
}
```

---

#### FluidDrain

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct FluidDrain {
    pub required_fluid: Option<FluidId>,
    pub consumption_rate: f32,
    pub required_temperature: Option<f32>,
    pub current_consumption: f32,
}
```

---

#### Pipe

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Pipe {
    pub network_id: GridId,
    pub fluid_type: Option<FluidId>,
    pub pressure: f32,
    pub temperature: f32,
}
```

---

#### Tank

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Tank {
    pub network_id: GridId,
    pub fluid_type: Option<FluidId>,
    pub capacity: f32,
    pub current_amount: f32,
    pub temperature: f32,
}
```

---

#### FluidNetworks (Resource)

```rust
#[derive(Resource, Default)]
pub struct FluidNetworks {
    pub networks: HashMap<GridId, FluidNetwork>,
    pub next_network_id: u32,
}
```

---

### FluidSpec

```rust
// game_spec/fluids.rs（新規）
#[derive(Clone, Serialize, Deserialize)]
pub struct FluidSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub viscosity: f32,
    pub default_temperature: f32,
    pub color: [f32; 3],
}
```

---

## 信号ネットワークモデル

### Components

#### SignalEmitter

```rust
// components/signals.rs
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct SignalEmitter {
    pub signal_strength: u8,
    pub condition: SignalCondition,
    pub is_emitting: bool,
}
```

---

#### SignalReceiver

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct SignalReceiver {
    pub threshold: u8,
    pub is_active: bool,
    pub received_strength: u8,
}
```

---

#### SignalWire

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct SignalWire {
    pub network_id: GridId,
    pub strength: u8,
}
```

---

#### LogicGate

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub enum LogicGate {
    AND { inputs: Vec<Entity>, output_strength: u8 },
    OR { inputs: Vec<Entity>, output_strength: u8 },
    NOT { input: Entity, output_strength: u8 },
    XOR { inputs: Vec<Entity>, output_strength: u8 },
}
```

---

#### SignalCondition

```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum SignalCondition {
    Always,
    InventoryFull,
    InventoryEmpty,
    HasItem(ItemId),
    PowerLow,
    Timer { interval_secs: f32 },
}
```

---

#### SignalInput

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct SignalInput {
    pub is_active: bool,
}
```

---

#### SignalNetworks (Resource)

```rust
#[derive(Resource, Default)]
pub struct SignalNetworks {
    pub networks: HashMap<GridId, SignalNetwork>,
    pub next_network_id: u32,
}
```

---

## IoPort拡張

```rust
// components/machines/ports.rs
pub enum PortType {
    Item,   // 既存
    Fluid,  // 新規
    Power,  // 新規
    Signal, // 新規
}
```

---

## セーブ/ロードモデル

### PowerNetworkData

```rust
// save/format/v2/power_networks.rs
#[derive(Serialize, Deserialize)]
pub struct PowerNetworkData {
    pub network_id: u32,
    pub nodes: Vec<(u64, PowerNodeData)>,
    pub edges: Vec<(u64, u64)>,
}

#[derive(Serialize, Deserialize)]
pub struct PowerNodeData {
    pub capacity: f32,
    pub current: f32,
    pub production: f32,
}
```

---

*最終更新: 2026-01-30*
