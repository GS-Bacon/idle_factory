# Data Model: M3 Power System

**Feature**: M3電力システム
**Date**: 2026-01-30

## Entities

### PowerProducer

発電機を表すComponent。既存の`MachineSlots.fuel`を活用し、新規`PowerFuelSlot` Componentは実装しない。

| Field | Type | Description | Validation |
|--------|------|-------------|-------------|
| `is_operational` | `bool` | 運転状態（燃料がある/起動完了） | - |
| `startup_timer` | `f32` | 起動タイマー（秒） | >= 0 |

**State Transitions**:
```
起動待ち (startup_timer > 0)
    ↓ startup_timer reaches 0
運転中 (is_operational = true, Machine.fuel > 0)
    ↓ fuel runs out
停止中 (is_operational = false)
    ↓ fuel added + startup delay
起動待ち
```

**Note**: 燃料管理は既存の`Machine.fuel`フィールドを使用

---

### PowerFuelSlot

燃料ベース発電機の燃料スロット。既存の`MachineSlots.fuel`を活用するため、個別のComponentとしては実装しない。

**使用方法**: 既存の`Machine.fuel`フィールドを活用

| 既存フィールド | Type | 用途 |
|-------------|------|------|
| `MachineSlots.fuel` | `u32` | 燃料個数 |

| 追加フィールド | Type | 用途 |
|-------------|------|------|
| `MachineSpec.fuel_consumption_rate` | `f32` | 燃料消費率（単位/チック） |
| `MachineSpec.startup_delay` | `f32` | 起動遅延（秒） |

| 新規Component | 用途 |
|-------------|------|
| `PowerProducer.startup_timer` | `f32` | 起動タイマー（秒） |

---

### PowerConsumer

電力を消費する機械を表すComponent。

| Field | Type | Description | Validation |
|--------|------|-------------|-------------|
| `required_power` | `f32` | 必要電力（ワット/チック） | >= 0 |
| `is_powered` | `bool` | 現在電力が供給されているか | - |

**Validation Rules**:
- `required_power = 0` の場合、常に `is_powered = true` （消費しない機械）

---

### PowerWire

電線ブロックを表すComponent。

| Field | Type | Description | Validation |
|--------|------|-------------|-------------|
| `connections` | `Vec<IVec3>` | 隣接する接続位置 | 最大6面（上下左右前後） |
| `grid_id` | `u64` | 関連するグリッドID | - |

---

### PowerGrids (Resource)

全電力グリッドを管理するResource。

| Field | Type | Description |
|--------|------|-------------|
| `grids` | `HashMap<u64, PowerGrid>` | 全電力グリッド（キー: grid_id） |
| `next_id` | `u64` | 次に使用するグリッドID |

---

### PowerGrid

接続された電力ネットワークを表す構造体。

| Field | Type | Description |
|--------|------|-------------|
| `id` | `u64` | グリッドID |
| `total_generation` | `u32` | 総発電量（全発電機の合計） |
| `total_consumption` | `u32` | 総消費量（全消費機械の合計） |
| `producers` | `Vec<Entity>` | 接続された発電機のEntity |
| `consumers` | `Vec<Entity>` | 接続された消費機械のEntity |
| `wires` | `Vec<Entity>` | 接続された電線のEntity |

**Derived Properties**:
- `has_power()`: `total_generation >= total_consumption`
- `surplus()`: `total_generation as i32 - total_consumption as i32`

---

## MachineSpec拡張

既存の`MachineSpec`に電力関連フィールドを追加。

| 追加フィールド | Type | 用途 |
|-------------|------|------|
| `power_output` | `Option<u32>` | 発電機の出力電力（ワット/チック） |
| `power_consumption` | `Option<u32>` | 消費機械の消費電力（ワット/チック） |
| `fuel_consumption_rate` | `Option<f32>` | 燃料ベース発電機の燃料消費率（単位/チック） |
| `startup_delay` | `Option<f32>` | 燃料ベース発電機の起動遅延（秒） |

**使用例**:
```rust
// 発電機
pub const WATER_WHEEL_GENERATOR: MachineSpec = MachineSpec {
    // 既存フィールド...
    power_output: Some(100),
    power_consumption: None,
    fuel_consumption_rate: None,
    startup_delay: None,
    // ...
};

// 消費機械
pub const FURNACE: MachineSpec = MachineSpec {
    // 既存フィールド...
    power_output: None,
    power_consumption: Some(10),
    fuel_consumption_rate: None,
    startup_delay: None,
    // ...
};

// 石炭発電機
pub const COAL_GENERATOR: MachineSpec = MachineSpec {
    // 既存フィールド...
    power_output: Some(200),
    power_consumption: None,
    fuel_consumption_rate: Some(0.1),
    startup_delay: Some(1.0),
    // ...
};
```

## Relationships

```
PowerProducer (Component)
    └─> uses Machine.fuel (existing field)

PowerConsumer (Component)
    └─> なし

PowerWire (Component)
    └─> PowerGrids (Resource via grid_id)

PowerGrid (struct)
    ├─> PowerProducer entities
    ├─> PowerConsumer entities
    └─> PowerWire entities

Machine (existing Component)
    ├─> uses Machine.fuel (existing field, for generators)
    ├─> PowerProducer (added for power-generating machines)
    ├─> PowerConsumer (added for power-requiring machines)
    └─> (existing: Crafter, MachineInventory, etc.)

MachineSpec (existing struct, extended)
    └─> power_output, power_consumption, fuel_consumption_rate, startup_delay (new fields)
```

---

## Data Flow

### 1. グリッド計算

```
1. 全てのPowerProducer, PowerConsumer, PowerWire Entityを収集
2. Union-Findで連結成分を計算
3. 各連結成分ごとにPowerGridを作成
4. total_generation = sum(producer.output_watts)
5. total_consumption = sum(consumer.required_power)
6. 全てのconsumer.is_powered = grid.has_power()
```

### 2. 燃料消費

```
毎チック:
  for each (entity, machine, mut producer) in PowerProducer query:
    if is_operational:
      if machine.spec.fuel_consumption_rate.is_some():
        // 既存のmachine.fuelを使用
        machine.fuel -= machine.spec.fuel_consumption_rate
        if machine.fuel <= 0:
          is_operational = false
          fire PowerGridChanged(GeneratorRemoved)
```

### 3. 機械動作

```
毎チック:
  for each Machine with PowerConsumer:
    if !power_consumer.is_powered:
      return  // 早期リターン、機械停止
    // 既存のレシピ処理を続行
```

---

## Save/Load

### Save Format

```toml
# PowerProducer Componentとして保存
[[power_producer]]
entity_id = "xxx"
is_operational = true
startup_timer = 0.0

# PowerConsumer Componentとして保存
[[power_consumer]]
entity_id = "yyy"
required_power = 10.0
is_powered = true

# PowerWire Componentとして保存
[[power_wire]]
entity_id = "zzz"
connections = [[1, 2, 3], [1, 2, 4]]
grid_id = 0

# Machineのfuelフィールドは既存のMachineセーブで保存済み
[[machines]]
entity_id = "yyy"
fuel = 50  # 燃料個数（発電機用）
# ... 既存の他のフィールド
```

### Load Process

1. 全てのPowerComponentを読み込み
2. EntityIDで関連付け
3. グリッド計算システムが起動時に再計算
4. 不明なItemIdがあった場合: 警告 + デフォルト値フォールバック

---

## Constraints & Invariants

### 性能制約
- 1グリッド100台以上の機械で50ms以内に計算完了
- Union-Findのamortized cost: O(α(n)) ≈ O(1)

### 整合性制約
- グリッド内の全Entityのgrid_idは一致
- `is_powered`はグリッドの`has_power()`と一致
- `total_generation >= 0`, `total_consumption >= 0`

### 業務ルール
- 燃料ベース発電機: 燃料がないと停止
- 無電力機械: 完全停止（部分動作なし）
- グリッド分割: 新しいgrid_idを割り当て
- グリッド統合: 小さいIDのグリッドに統合
