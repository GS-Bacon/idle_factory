# Quickstart: M3 Power System

**Feature**: M3電力システム
**Date**: 2026-01-30

## Overview

M3電力システムは、発電機→電線→機械の電力供給を管理する。複数の独立した電力グリッドをサポートし、電力不足時に機械が停止する。

## Prerequisites

- Rust 1.75+
- Bevy 0.14+
- 既存の`src/machines/`、`src/logistics/`、`src/events/`構造の理解
- Union-Findアルゴリズムの基礎知識

## Component Architecture

### 新規追加

| Component | Path | 説明 |
|-----------|------|--------|
| `PowerProducer` | `src/components/power.rs` | 発電機（水車・石炭発電機） |
| `PowerWire` | `src/components/power.rs` | 電線ブロック |
| `PowerGrids` | `src/components/power.rs` (Resource) | 全電力グリッド管理 |

### 既存統合

| Component/Struct | 変更 | 説明 |
|-----------------|------|--------|
| `PowerConsumer` | `src/components/machines/ports.rs`（既存） | 電力消費機械に追加 |
| `MachineSpec` | `src/game_spec/machines.rs`（拡張） | 電力関連フィールド追加 |

---

## System Architecture

### 新規追加

| System | Path | 説明 |
|--------|------|--------|
| `grid_calculation_system` | `src/systems/power/grid_calc.rs` | 電力グリッド計算（Union-Find） |
| `generator_tick_system` | `src/systems/power/generator_tick.rs` | 発電機tick処理（燃料消費） |
| `consumer_tick_system` | `src/systems/power/consumer_tick.rs` | 消費機械tick制御 |

### 既存変更

| System | 変更 | 説明 |
|--------|------|--------|
| `generic_machine_tick` | `src/machines/generic/tick.rs` | 冒頭で電力チェック追加 |
| `machine_ui` | `src/systems/ui/machine_ui.rs` | 電力情報表示追加 |

---

## Implementation Steps

### Step 1: Component定義

```bash
# 1. src/components/power.rsを作成
touch src/components/power.rs

# 2. src/components/mod.rsに追加
pub mod power;
pub use power::{PowerProducer, PowerWire, PowerGrids, PowerGrid};
```

**実装内容**:
- `PowerProducer` Component（`is_operational`, `startup_timer`のみ）
- `PowerWire` Component
- `PowerGrids` Resource
- `PowerGrid` struct

**注意**: `PowerFuelSlot` Componentは実装しない。既存の`Machine.fuel`を使用する。

---

### Step 1.5: MachineSpec拡張

```bash
# src/game_spec/machines.rsを変更
```

**実装内容**:
```rust
pub struct MachineSpec {
    // 既存フィールド...
    pub power_output: Option<u32>,          // 発電機の出力電力
    pub power_consumption: Option<u32>,        // 消費機械の消費電力
    pub fuel_consumption_rate: Option<f32>,     // 燃料消費率
    pub startup_delay: Option<f32>,           // 起動遅延
    // ...
}
```

---

### Step 2: Event定義

```bash
# src/events/game_events.rsに追加
```

**実装内容**:
- `PowerGridChanged`イベント
- `PowerGridChangeType` enum

---

### Step 3: System実装

#### 3.1 Power Grid Calculation System

```bash
# src/systems/power/ディレクトリ作成
mkdir -p src/systems/power

# src/systems/power/mod.rsを作成
touch src/systems/power/mod.rs

# src/systems/power/grid_calc.rsを作成
touch src/systems/power/grid_calc.rs
```

**実装内容**:
- Union-Findデータ構造
- 全ての`PowerProducer`, `PowerConsumer`, `PowerWire` Entityを収集
- Union-Findで連結成分を計算
- 各連結成分ごとに`PowerGrid`を作成
- 全ての`PowerConsumer.is_powered`を更新
- `PowerGrids` Resourceを更新

**Run Schedule**:
```rust
app.add_systems(
    (
        calculate_power_grids,
    )
        .in_set(OnUpdate(GameState::InGame))
        .in_set(FixedUpdate),
);
```

---

#### 3.2 Generator Tick System

```bash
# src/systems/power/generator_tick.rsを作成
touch src/systems/power/generator_tick.rs
```

**実装内容**:
- 全ての`PowerProducer`を反復
- 燃料消費処理（`PowerFuelSlot`がある場合）
- `is_operational`状態更新
- 起動遅延タイマー減算
- `PowerGridChanged`イベント発行

**Run Schedule**:
```rust
app.add_systems(
    (
        generator_tick,
    )
        .in_set(OnUpdate(GameState::InGame))
        .in_set(FixedUpdate),
);
```

---

#### 3.3 Consumer Tick System

```bash
# src/systems/power/consumer_tick.rsを作成
touch src/systems/power/consumer_tick.rs
```

**実装内容**:
- 実装自体は最小（`PowerConsumer.is_powered`チェック）
- 実際の制御は`generic_machine_tick()`で実施

**Run Schedule**:
```rust
app.add_systems(
    (
        consumer_tick,
    )
        .in_set(OnUpdate(GameState::InGame))
        .in_set(FixedUpdate),
);
```

---

### Step 4: 既存System統合

```bash
# src/machines/generic/tick.rsを変更
```

**実装内容**:
```rust
fn generic_machine_tick(
    mut commands: Commands,
    mut machines: Query<(Entity, &mut Machine, Option<&PowerConsumer>), ...>,
    // ...
) {
    for (entity, mut machine, power_consumer) in machines.iter_mut() {
        // 電力チェック（新規）
        if let Some(pc) = power_consumer {
            if !pc.is_powered {
                return; // 無電力時は早期リターン
            }
        }

        // 既存のレシピ処理
        // ...
    }
}
```

---

### Step 5: UI実装

```bash
# src/systems/ui/machine_ui.rsを変更
```

**実装内容**:
```rust
fn spawn_machine_ui(..., power_consumer: Option<&PowerConsumer>, ...) {
    // 既存のUI要素...

    // 電力セクション（新規）
    if let Some(pc) = power_consumer {
        spawn_section_header(panel, font, "電力");
        spawn_text(panel, font, &format!("状態: {}", if pc.is_powered { "稼働中" } else { "停止中" }));
        spawn_text(panel, font, &format!("消費電力: {}W", pc.required_power));
    }
}
```

---

### Step 6: Plugin登録

```bash
# src/main.rsまたは適切なPluginファイルに追加
```

**実装内容**:
```rust
app.add_plugins((
    // 既存のplugins...
    PowerSystemPlugin::default(),
));
```

**Plugin定義** (`src/systems/power/mod.rs`):
```rust
pub struct PowerSystemPlugin;

impl Plugin for PowerSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PowerGrids>()
            .add_event::<PowerGridChanged>()
            .add_systems(
                (
                    calculate_power_grids,
                    generator_tick,
                    consumer_tick,
                )
                    .in_set(OnUpdate(GameState::InGame))
                    .in_set(FixedUpdate),
            );
    }
}
```

---

## Testing

### Unit Tests

```bash
# src/components/power.rsのtests moduleに追加
cargo test
```

**テスト項目**:
- `PowerGrid.has_power()`が正しく動作
- `PowerGrid.surplus()`が正しく計算
- Union-Findの正確性

---

### Scenario Tests

```bash
# tests/scenarios/power_system.tomlを作成
node scripts/run-scenario.js power_system.toml
```

**シナリオ例**:
```toml
name = "Basic power network"

[[actions]]
type = "place_block"
pos = [0, 0, 0]
block = "base:water_wheel_generator"

[[actions]]
type = "place_block"
pos = [1, 0, 0]
block = "base:power_wire"

[[actions]]
type = "place_block"
pos = [2, 0, 0]
block = "base:furnace"

[[verifications]]
type = "entity_has_component"
entity_type = "furnace"
component = "PowerConsumer"
field = "is_powered"
expected = true
```

---

## Data Files

### mods/base/items.toml

```toml
[[items]]
id = "base:power_wire"
name = "電線"
color = [0.5, 0.5, 1.0]
stack_size = 64

[[items]]
id = "base:coal"
name = "石炭"
color = [0.2, 0.2, 0.2]
stack_size = 64
```

### mods/base/machines.toml

```toml
[[machines]]
id = "base:water_wheel_generator"
name = "水車発電機"
model = "water_wheel"
# MachineSpec拡張フィールド
power_output = 100
power_consumption = null
fuel_consumption_rate = null
startup_delay = null

[[machines]]
id = "base:coal_generator"
name = "石炭発電機"
model = "coal_generator"
# MachineSpec拡張フィールド
power_output = 200
power_consumption = null
fuel_consumption_rate = 0.1
startup_delay = 1.0
```

---

## Debugging

### グリッド計算のデバッグ

```rust
// デバッグモードでグリッド情報をログ出力
if cfg!(debug_assertions) {
    for (id, grid) in power_grids.grids.iter() {
        info!(
            "Grid #{}: {} producers, {} consumers, surplus: {}W",
            id,
            grid.producers.len(),
            grid.consumers.len(),
            grid.surplus()
        );
    }
}
```

### 電力状態の可視化

```bash
# デバッグコマンドで全機械の電力状態を表示
```

---

## Performance Tuning

### Union-Find Optimization

- Path compressionを常に実施
- Union by rankを使用
- 頻繁な再計算を避けるため、変更検出を効率化

### Grid Update Frequency

- 全てのチックで再計算ではなく、変更時のみ再計算（Observerパターン）
- `PowerGridChanged`イベントを検知した時のみ再計算

---

## Common Issues

### Issue 1: 機械が停止しない

**原因**: `PowerConsumer` Componentが追加されていない

**解決**: `mods/base/machines.toml`の`power_consumption`フィールドを確認

---

### Issue 2: グリッドが分割されない

**原因**: Union-Findの実装バグ

**解決**: `find()`と`union()`のロジックを確認

---

### Issue 3: 燃料が消費されない

**原因**: `is_operational`が`false`

**解決**: 起動遅延タイマーを確認、燃料追加後の状態遷移を確認

---

## References

- `.claude/architecture.md` - 電力システム設計
- `specs/001-power-system-spec/research.md` - 設計決定の根拠
- `specs/001-power-system-spec/data-model.md` - データモデル詳細
- `specs/001-power-system-spec/contracts/systems.md` - システム契約
