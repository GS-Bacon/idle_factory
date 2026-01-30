# System Contracts: M3 Power System

**Feature**: M3電力システム
**Date**: 2026-01-30

## Overview

本契約は、電力システムと他のゲームシステム（機械システム、UIシステム、イベントシステム）間のインターフェースを定義する。

---

## Contract 1: Power Grid Calculation System

**Responsibility**: 電力ネットワークを検出・管理し、グリッド情報を提供する

### Input Events

| Event | Trigger | Data |
|--------|----------|-------|
| `BlockPlaced` | 電線ブロック配置 | `pos: IVec3`, `block: ItemId` |
| `BlockBroken` | 電線ブロック破壊 | `pos: IVec3`, `block: ItemId` |
| `MachineSpawned` | 発電機/消費機械配置 | `entity: Entity`, `pos: IVec3` |
| `MachineDespawned` | 発電機/消費機械破壊 | `entity: Entity` |

### Output Events

| Event | Trigger | Data |
|--------|----------|-------|
| `PowerGridChanged` | グリッド変化時 | `grid_id: u64`, `change_type: PowerGridChangeType` |

### Query Requirements

```rust
Query<(
    Entity,
    &PowerProducer,
    &Transform,
)>

Query<(
    Entity,
    &PowerConsumer,
    &Transform,
)>

Query<(
    Entity,
    &PowerWire,
    &Transform,
)>

ResMut<PowerGrids>
```

### Output

- `PowerGrids.grids: HashMap<u64, PowerGrid>` - 全電力グリッド
- 全ての`PowerConsumer.is_powered`が更新される

### Performance Constraints

- 1グリッド100台以上の機械で50ms以内に計算完了
- FixedUpdate(20Hz)で毎チック実行

---

## Contract 2: Generator Tick System

**Responsibility**: 発電機の燃料消費と運転状態を管理する

### Dependencies

```rust
ResMut<PowerGrids>
Query<(
    Entity,
    &mut Machine,
    &mut PowerProducer,
)>

Time<Fixed>
```

### Output

- `PowerProducer.is_operational`の更新
- `PowerProducer.fuel_slot.fuel`の更新
- 燃料が尽きた時: `PowerGridChanged(GeneratorRemoved)`イベント発行

### Behavior

```rust
for each (entity, mut machine, mut producer) in query.iter_mut() {
    if producer.is_operational {
        if let Some(consumption_rate) = machine.spec.fuel_consumption_rate {
            // 燃料消費（既存のmachine.fuelを使用）
            machine.fuel = (machine.fuel as f32 - consumption_rate).max(0.0) as u32;

            if machine.fuel == 0 {
                producer.is_operational = false;
                emit(PowerGridChanged {
                    grid_id: get_grid_id(entity),
                    change_type: PowerGridChangeType::GeneratorRemoved,
                });
            }
        }
    } else {
        // 起動タイマー減算（燃料がある場合）
        if let Some(startup_delay) = machine.spec.startup_delay {
            if machine.fuel > 0 {
                producer.startup_timer = (producer.startup_timer - delta).max(0.0);
                if producer.startup_timer == 0.0 {
                    producer.is_operational = true;
                    emit(PowerGridChanged {
                        grid_id: get_grid_id(entity),
                        change_type: PowerGridChangeType::GeneratorAdded,
                    });
                }
            }
        } else {
            // 水車など燃料不要な発電機は即時起動
            producer.is_operational = true;
            emit(PowerGridChanged {
                grid_id: get_grid_id(entity),
                change_type: PowerGridChangeType::GeneratorAdded,
            });
        }
    }
}
```

---

## Contract 3: Consumer Tick System

**Responsibility**: 電力を消費する機械の動作/停止を制御する

### Dependencies

```rust
Query<(
    Entity,
    &PowerConsumer,
    &Machine,
)>

Res<PowerGrids>
```

### Output

- 無電力時: 機械のレシピ処理を停止（早期リターン）
- 電力供給時: 機械は既存の処理を続行

### Behavior

```rust
for each (entity, power_consumer, machine) in query.iter() {
    if !power_consumer.is_powered {
        // 無電力時は機械停止
        // 既存のgeneric_machine_tick()で早期リターン
        continue;
    }

    // 電力がある場合は既存の処理を続行
    // このシステム自体は何もしない（制御はgeneric_machine_tick()で行う）
}
```

---

## Contract 4: Machine UI Integration

**Responsibility**: 機械UIに電力情報を表示する

### Dependencies

```rust
Query<(
    Entity,
    &Machine,
    Option<&PowerConsumer>,
    Option<&PowerProducer>,
)>

Res<PowerGrids>
```

### Output

- 機械UIに電力状態を表示
- グリッド統計パネルを表示

### UI Elements

#### 消費機械用
```
電力
  状態: 稼働中 / 停止中
  消費電力: XX W
```

#### 発電機用
```
電力
  状態: 稼働中 / 停止中 / 起動待ち
  出力電力: XX W
  燃料: XX / XX ( Coal )
```

#### グリッド統計パネル
```
電力グリッド統計
  グリッド: #1
  総発電量: XX W
  総消費量: XX W
  余剰/不足: +XX W / -XX W
```

---

## Contract 5: Event System Integration

**Responsibility**: 電力関連イベントをシステム間で配信する

### New Events

```rust
/// 電力グリッド変更イベント
pub struct PowerGridChanged {
    pub grid_id: u64,
    pub change_type: PowerGridChangeType,
}

pub enum PowerGridChangeType {
    GridCreated,
    GridSplit { new_grid_ids: Vec<u64> },
    GridMerged { merged_into_id: u64 },
    GeneratorAdded,
    GeneratorRemoved,
    ConsumerAdded,
    ConsumerRemoved,
    WireAdded,
    WireRemoved,
}
```

### Event Flow

```
[Player Action] → [System] → [Event] → [Observer] → [Reaction]

例: プレイヤーが電線を配置
  → Power Grid Calculation System
  → PowerGridChanged(WireAdded)
  → (Mod System receives event if subscribed)
  → (UI updates if needed)
```

### High-Frequency Events

以下の高頻度イベントは外部Mod通知OFF：
- `PowerGridChanged`の大部分（グリッド計算結果）

`GuardedEventWriter`を使用して、深さチェックを実施（max_depth: 16）

---

## Cross-System Data Flow

```
Player Action (電線配置)
    ↓
BlockPlaced Event
    ↓
Power Grid Calculation System
    ├─> Detects new connection (Union-Find)
    ├─> Updates PowerGrids resource
    ├─> Updates all PowerConsumer.is_powered
    └─> Emits PowerGridChanged
        ↓
Mod System (via WebSocket)
    ↓
Generator Tick System
    ├─> Consumes fuel (if operational)
    └─> Updates PowerProducer.is_operational
        ↓
Consumer Tick System / Generic Machine Tick
    ├─> Checks PowerConsumer.is_powered
    ├─> Stops machine if !is_powered
    └─> Continues processing if is_powered
        ↓
UI System
    └─> Displays power status in machine UI
```

---

## Validation Rules

### 入力検証
- `PowerProducer.output_watts >= 0`
- `PowerConsumer.required_power >= 0`
- `PowerFuelSlot.fuel.count > 0` if `Some`

### 整合性検証
- グリッド内の全`PowerConsumer.is_powered`は`grid.has_power()`と一致
- `PowerProducer.is_operational`が`true`の場合、燃料が存在する

### 業務ルール検証
- 燃料ベース発電機: 燃料がないと`is_operational = false`
- 無電力機械: レシピ処理を停止
- グリッド計算: 50ms以内に完了

---

## Performance SLAs

| Metric | Target | Measurement |
|--------|---------|-------------|
| Grid calculation time | < 50ms | 100 machines per grid |
| Union-Find operations | O(α(n)) amortized | n = number of entities |
| Event delivery latency | < 1 tick (50ms) | High-frequency events excluded |
| UI update responsiveness | < 100ms | User-visible changes |
