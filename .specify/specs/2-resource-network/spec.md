# Feature Specification: Resource Networks

**Feature Branch**: `2-resource-network`
**Created**: 2026-01-30
**Status**: Draft
**Input**: User description: "リソースネットワークの機能追加をしたい。電力・流体・信号の3つのネットワークシステムを実装。現在のプロジェクトの内容把握して実装するためのステップを定義。Speckitのルールに従って計画ファイルを保存。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Power Network (Priority: P1)

Player places a generator (water wheel or coal generator) and connects it to machines using power wire blocks. Machines connected to powered networks operate normally, while machines without power stop functioning.

**Why this priority**: This is core functionality that enables entire power system. Without basic power generation and distribution, no other power-related features have value.

**Independent Test**: Can be fully tested by placing a generator, connecting it to a machine with power wires, and observing machine operates when powered and stops when unpowered.

**Acceptance Scenarios**:

1. **Given** a generator is placed, **When** power wires connect it to a machine, **Then** machine receives power and operates normally
2. **Given** a powered machine, **When** generator is removed, **Then** machine loses power and stops operating
3. **Given** two separate power networks, **When** machines are connected to different networks, **Then** power states are independent (one can be unpowered while other is powered)

---

### User Story 2 - Basic Fluid Network (Priority: P1)

Player places a fluid source (pump), connects pipes, and creates a tank or fluid-consuming machine. Fluids flow through pipes to reach their destination, with tanks storing intermediate amounts.

**Why this priority**: Fluids are a core resource type for many machines (cooling, fueling, processing). Without fluid distribution, these machines cannot function.

**Independent Test**: Can be fully tested by placing a pump, connecting pipes to a tank, and observing fluid transfer from source to tank.

**Acceptance Scenarios**:

1. **Given** a fluid source and pipe network, **When** pipes connect to a tank, **Then** fluid flows from source to tank
2. **Given** a full tank, **When** a fluid-consuming machine connects, **Then** fluid flows from tank to machine
3. **Given** disconnected pipe networks, **When** fluids are pumped, **Then** fluids remain in their respective networks (no cross-network flow)

---

### User Story 3 - Basic Signal Network (Priority: P1)

Player places a signal emitter (sensor) and connects signal wires to receivers or logic gates. Signals propagate through wires with attenuation, and receivers activate based on signal strength and threshold.

**Why this priority**: Signals enable automated control of machines based on conditions (inventory levels, power status, timers). Without signals, factories require manual intervention for complex operations.

**Independent Test**: Can be fully tested by placing a sensor, connecting signal wire to a machine, and observing machine activation/deactivation based on sensor condition.

**Acceptance Scenarios**:

1. **Given** a signal emitter with condition satisfied, **When** signal wires connect to a receiver, **Then** receiver activates when signal strength exceeds threshold
2. **Given** signal wires, **When** signal propagates through wires, **Then** signal strength decreases by 1 per block distance
3. **Given** a logic gate (AND/OR/NOT), **When** inputs meet gate condition, **Then** output activates with correct signal strength

---

### User Story 4 - Multiple Independent Networks (Priority: P2)

Players can create and manage multiple separate networks for power, fluids, and signals. Each network type maintains independent grids that don't interfere with each other.

**Why this priority**: Large factories need isolation for reliability and organization. Mixing networks across different systems would cause confusion and incorrect behavior.

**Independent Test**: Can be fully tested by creating separate power, fluid, and signal networks and verifying they operate independently.

**Acceptance Scenarios**:

1. **Given** a power network and fluid network, **When** power is lost, **Then** fluid system continues operating unaffected
2. **Given** two power networks close to each other, **When** power wires are placed but not connected, **Then** networks remain separate (no power transfer)
3. **Given** a signal network controlling machines, **When** signal is cut, **Then** power and fluid networks continue operating independently

---

### User Story 5 - Network UI Feedback (Priority: P2)

Players can visually identify network states through UI, including power status, fluid levels, and signal strengths. Network statistics (generation, consumption, flow rates) are displayed for each network.

**Why this priority**: Visual feedback is critical for player understanding. Without clear indicators, players cannot debug why machines aren't working or optimize their networks.

**Independent Test**: Can be fully tested by building networks with mixed states and verifying UI displays match actual network conditions.

**Acceptance Scenarios**:

1. **Given** a power network exists, **When** player opens machine UI, **Then** power status is clearly displayed (powered/unpowered)
2. **Given** a fluid network exists, **When** player views tank UI, **Then** fluid type and amount are displayed
3. **Given** a signal network exists, **When** player views signal wire, **Then** current signal strength is displayed

---

### User Story 6 - Fuel-Based Power Generation (Priority: P3)

Some generators (coal generator) consume fuel items to produce power. Power generation stops when fuel runs out, and players must refuel to restore operation.

**Why this priority**: Fuel-based generators add resource management complexity and create gameplay loops. Without this, power generation is too simple and doesn't create interesting logistical challenges.

**Independent Test**: Can be fully tested by placing a coal generator, loading fuel, observing power generation, waiting for fuel depletion, and refueling to restore generation.

**Acceptance Scenarios**:

1. **Given** a coal generator with fuel, **When** generator operates, **Then** fuel is consumed over time and power is produced
2. **Given** a coal generator runs out of fuel, **When** no fuel remains, **Then** power generation stops and connected machines lose power
3. **Given** a coal generator without fuel, **When** player adds fuel to generator, **Then** power generation resumes after a short startup delay

---

### User Story 7 - Fluid Temperature and Viscosity (Priority: P4)

Fluids have properties like temperature (affects processing) and viscosity (affects flow speed through pipes). Some machines require specific fluid temperatures for operation.

**Why this priority**: Fluid properties add depth to factory design and enable complex processing chains (e.g., heating water for steam, cooling lava).

**Independent Test**: Can be fully tested by pumping fluids with different viscosities and observing flow rates, and heating fluids to required temperatures.

**Acceptance Scenarios**:

1. **Given** a high-viscosity fluid, **When** pumped through pipes, **Then** flow rate is slower than low-viscosity fluid
2. **Given** a fluid heater, **When** fluid passes through, **Then** fluid temperature increases according to heater settings
3. **Given** a machine requiring hot fluid, **When** cold fluid is supplied, **Then** machine does not operate until fluid reaches required temperature

---

### User Story 8 - Complex Signal Logic (Priority: P4)

Players can create complex control systems using logic gates (AND, OR, NOT, XOR) and sensors with various conditions (inventory full/empty, power low, timer). This enables automated factory management.

**Why this priority**: Complex signal logic is essential for advanced automation. Without it, players cannot create sophisticated factory behaviors like conditional operation or emergency shutdowns.

**Independent Test**: Can be fully tested by building a logic circuit with multiple gates and verifying correct output for all input combinations.

**Acceptance Scenarios**:

1. **Given** an AND gate with two inputs, **When** both inputs are active, **Then** output activates with combined signal strength
2. **Given** a NOT gate, **When** input is active, **Then** output is inactive, and vice versa
3. **Given** multiple sensors connected to logic gates, **When** sensors change state, **Then** network recalculates and outputs update within one tick

---

### Edge Cases

- What happens when a power wire connects two previously separate power grids with different power levels?
- What happens when a pipe connects two fluid networks with different fluid types?
- What happens when a signal wire connects networks with overlapping signal sources?
- How does system handle cycles in networks (e.g., wires forming loops)?
- What happens when a generator/source is destroyed while machines/consumers are actively using resources?
- How does system handle edge cases like a generator with 0 output or a pipe with 0 capacity?
- What happens when game is saved while networks are in an intermediate state (fluid flowing, signals propagating)?
- How does system handle power/pipe/signal placement on top of other blocks?
- What happens when fuel is added to a generator that's already full?
- How does system handle mixed fluid types in same pipe network?
- What happens when signal strength drops below threshold due to long wire distance?
- How does system handle rapid network changes (add/remove wires within same tick)?

## Functional Requirements

### FR-001: Power Network Generation

**Description**: Generators produce power that can be distributed through power wires to machines.

**Acceptance Criteria**:
- [ ] Generators (water wheel, coal generator) can be placed
- [ ] Generators produce power output (watts) as specified in MachineSpec
- [ ] Power is available immediately after generator placement
- [ ] Power output is constant for water wheel, fuel-dependent for coal generator

**Testable**: Yes - Place generator, measure power output in UI

---

### FR-002: Power Network Distribution

**Description**: Power wires connect generators and machines into power networks. Power flows from generators to consumers.

**Acceptance Criteria**:
- [ ] Power wires can be placed between adjacent blocks
- [ ] Wires form connected power networks (using NetworkGraph)
- [ ] Power flows from generators to consumers in same network
- [ ] Networks update automatically when wires are added/removed

**Testable**: Yes - Build wire network, verify connectivity and power flow

---

### FR-003: Power Consumption

**Description**: Machines consume power to operate. Machines without power stop functioning.

**Acceptance Criteria**:
- [ ] Machines have power consumption specified in MachineSpec
- [ ] Machines operate only when powered (is_powered = true)
- [ ] Unpowered machines stop processing recipes
- [ ] Power state is visible in machine UI

**Testable**: Yes - Remove power, observe machine stops; restore power, observe machine resumes

---

### FR-004: Multiple Power Grids

**Description**: Multiple independent power networks can exist. Each has its own generation and consumption.

**Acceptance Criteria**:
- [ ] Separate wire networks don't share power
- [ ] Each network maintains independent power state
- [ ] Network statistics are displayed per-network
- [ ] Networks can be split/merged by wire changes

**Testable**: Yes - Build two separate networks, verify independence

---

### FR-005: Fuel-Based Power Generation

**Description**: Coal generators consume fuel items to produce power. Generation stops when fuel runs out.

**Acceptance Criteria**:
- [ ] Coal generators have fuel slot in MachineSpec
- [ ] Fuel consumption rate is specified in MachineSpec
- [ ] Power generation continues while fuel is available
- [ ] Generation stops when fuel is empty
- [ ] Refueling resumes generation after startup delay

**Testable**: Yes - Load fuel, observe generation stops when fuel depleted

---

### FR-006: Fluid Source and Pumping

**Description**: Fluid sources (pumps) extract fluids and inject them into pipe networks.

**Acceptance Criteria**:
- [ ] Pumps can be placed adjacent to fluid sources
- [ ] Pumps extract fluids at specified rate
- [ ] Fluids are injected into connected pipe network
- [ ] Pumping continues as long as fluid source has fluid

**Testable**: Yes - Place pump, connect pipe, observe fluid in tank

---

### FR-007: Fluid Pipe Network

**Description**: Pipes transport fluids between sources, tanks, and consumers. Fluids flow based on pressure/viscosity.

**Acceptance Criteria**:
- [ ] Pipes can be placed in adjacent blocks
- [ ] Pipes form connected fluid networks (using NetworkGraph)
- [ ] Fluids flow from high pressure to low pressure
- [ ] Flow rate is affected by fluid viscosity
- [ ] Networks update automatically when pipes are added/removed

**Testable**: Yes - Build pipe network, verify fluid flow

---

### FR-008: Fluid Tanks and Storage

**Description**: Tanks store fluids with capacity limits. They act as buffers in pipe networks.

**Acceptance Criteria**:
- [ ] Tanks can be placed
- [ ] Tanks have capacity specified in MachineSpec
- [ ] Tanks store fluid type and amount
- [ ] Tanks display fluid level in UI
- [ ] Full tanks block incoming flow, empty tanks allow flow

**Testable**: Yes - Fill tank, observe capacity limit; drain tank, observe empty state

---

### FR-009: Fluid Consumption

**Description**: Fluid-consuming machines draw fluids from connected pipes. Machines without fluid stop operating.

**Acceptance Criteria**:
- [ ] Machines have fluid consumption specified in MachineSpec
- [ ] Machines consume fluids at specified rate
- [ ] Machines operate only when fluid is available
- [ ] Fluid consumption is visible in machine UI
- [ ] Machines stop when fluid is unavailable

**Testable**: Yes - Cut fluid supply, observe machine stops; restore fluid, observe machine resumes

---

### FR-010: Fluid Properties

**Description**: Fluids have properties (temperature, viscosity) that affect behavior.

**Acceptance Criteria**:
- [ ] FluidSpec includes temperature and viscosity fields
- [ ] Viscosity affects flow rate through pipes
- [ ] Temperature is preserved in fluid network
- [ ] Fluid heating/cooling modifies temperature
- [ ] Some machines require specific fluid temperatures

**Testable**: Yes - Pump different viscosity fluids, compare flow rates; heat fluid, observe temperature change

---

### FR-011: Signal Emission

**Description**: Signal emitters (sensors) produce signals based on conditions. Signals have strength (0-15).

**Acceptance Criteria**:
- [ ] Sensors can be placed
- [ ] Sensors evaluate conditions (inventory full/empty, power low, timer)
- [ ] Sensors emit signal with strength when condition is met
- [ ] Sensors stop emitting when condition is not met
- [ ] Signal condition can be specified in MachineSpec

**Testable**: Yes - Place sensor, trigger condition, observe signal emission

---

### FR-012: Signal Wire Network

**Description**: Signal wires transport signals from emitters to receivers. Signals attenuate with distance.

**Acceptance Criteria**:
- [ ] Signal wires can be placed in adjacent blocks
- [ ] Wires form connected signal networks (using NetworkGraph)
- [ ] Signals propagate from emitters through wires
- [ ] Signal strength decreases by 1 per block distance
- [ ] Networks update automatically when wires are added/removed

**Testable**: Yes - Build wire network, measure signal strength at different distances

---

### FR-013: Signal Reception

**Description**: Signal receivers activate when signal strength exceeds threshold. Machines can have signal inputs.

**Acceptance Criteria**:
- [ ] Receivers have threshold specified in MachineSpec
- [ ] Receivers activate (is_active = true) when signal >= threshold
- [ ] Receivers deactivate when signal < threshold
- [ ] Machines can have signal_input to control operation
- [ ] Signal state is visible in machine UI

**Testable**: Yes - Send strong signal, observe activation; send weak signal, observe deactivation

---

### FR-014: Logic Gates

**Description**: Logic gates (AND, OR, NOT, XOR) process signals to create complex control logic.

**Acceptance Criteria**:
- [ ] Logic gates can be placed
- [ ] AND gate activates when all inputs are active
- [ ] OR gate activates when any input is active
- [ ] NOT gate inverts input signal
- [ ] XOR gate activates when inputs differ
- [ ] Gates output signal with combined strength

**Testable**: Yes - Build truth tables for each gate, verify outputs

---

### FR-015: Network Independence

**Description**: Power, fluid, and signal networks operate independently. Networks of different types don't interfere with each other.

**Acceptance Criteria**:
- [ ] Power network changes don't affect fluid/signal networks
- [ ] Fluid network changes don't affect power/signal networks
- [ ] Signal network changes don't affect power/fluid networks
- [ ] Each network type maintains separate NetworkGraph instances
- [ ] Networks can coexist in same spatial area

**Testable**: Yes - Build all three network types, modify one, verify others unaffected

---

## Non-Functional Requirements

### NFR-001: Performance

**Description**: Network calculations must complete within one game tick (50ms) for typical factory sizes.

**Acceptance Criteria**:
- [ ] Power grid calculation completes in < 50ms for 100+ machines
- [ ] Fluid network calculation completes in < 50ms for 100+ pipes
- [ ] Signal network calculation completes in < 50ms for 100+ wires

**Testable**: Yes - Benchmark with large networks

---

### NFR-002: Reliability

**Description**: Networks must update correctly after any configuration change (wire/pipe/signal add/remove, machine add/remove).

**Acceptance Criteria**:
- [ ] Network state updates within one tick after any change
- [ ] Network splits/merges are handled correctly
- [ ] Invalid network states (cycles, disconnected components) are resolved

**Testable**: Yes - Make various network changes, verify state correctness

---

### NFR-003: Save/Load

**Description**: Network states must be correctly saved and restored.

**Acceptance Criteria**:
- [ ] All network components are serialized to save file
- [ ] Network state (power levels, fluid amounts, signal strengths) is preserved
- [ ] Networks are correctly reconstructed after load
- [ ] Unknown IDs are handled gracefully (warning + fallback)

**Testable**: Yes - Save with active networks, load, verify state matches

---

## Success Criteria

### Quantitative Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Network calculation time | < 50ms per tick | Benchmark with 100+ nodes |
| Network update responsiveness | < 1 tick (50ms) | Observe state change after wire placement |
| Save/load accuracy | 100% state preservation | Compare pre-save and post-load states |
| Power generation reliability | 100% uptime (when fueled) | Long-running test with fuel monitoring |
| Fluid flow accuracy | ±5% of theoretical flow rate | Measure actual vs calculated flow |
| Signal propagation latency | < 1 tick per 10 blocks | Measure signal arrival time |
| Concurrent network support | 10+ independent networks | Create and verify 10 separate networks |
| Network size capacity | 1000+ nodes per network | Performance test with large networks |

### Qualitative Outcomes

- Players can intuitively understand and build power, fluid, and signal networks
- Network behavior is predictable and follows intuitive rules (power flows from generators, fluids flow from high to low pressure, signals attenuate with distance)
- UI provides clear feedback for debugging and optimization
- Networks enable complex factory automation (conditional machine operation, emergency shutdowns, load balancing)
- Network systems don't interfere with each other or existing gameplay
- Network performance remains acceptable as factories scale to large sizes

---

## Assumptions

1. **Existing NetworkGraph is sufficient**: The `NetworkGraph<K, V>` in `src/core/network.rs` can be adapted for all three network types without major modifications.

2. **Union-Find for grid calculation**: Union-Find algorithm (as researched for power system) will be used for efficient network grid calculation.

3. **Fixed tick rate**: Network updates occur in `FixedUpdate` at 20 tick/second, matching the existing fixed tick system.

4. **Event-based updates**: Network changes (add/remove wires, add/remove machines) trigger recalculation, rather than continuous recalculation.

5. **Component-based design**: Network state is stored in ECS components (PowerProducer, PowerConsumer, etc.), following existing Bevy patterns.

6. **Mod support**: Network specifications (fluid properties, signal conditions) are defined in `MachineSpec` and can be extended by Data Mods.

7. **No resource depletion**: Fluid sources (like water) are infinite for gameplay simplicity, consistent with the existing "resource depletion disabled" policy.

8. **Signal attenuation model**: Signal strength decreases by 1 per block distance (similar to Minecraft redstone), providing a simple predictable model.

9. **Fluid mixing prevention**: Different fluid types cannot mix in the same pipe network. Attempting to mix fluids results in blocked flow or warning.

10. **Power distribution priority**: When total consumption exceeds generation, power is distributed equally among consumers (all receive partial power and operate at reduced efficiency, rather than some being fully powered and others unpowered).

---

## Out of Scope

The following features are explicitly excluded from this implementation:

- [ ] **Wireless power transfer**: Power must be transmitted through physical wires
- [ ] **Fluid mixing/blending**: Different fluid types cannot mix in pipes
- [ ] **Advanced signal features**: Comparator, repeater, or more complex signal devices beyond basic logic gates
- [ ] **Network visualization tools**: Specialized tools for viewing network topology (beyond basic UI feedback)
- [ ] **Multiplayer synchronization**: Network state is not synchronized across multiplayer (this will be handled separately)
- [ ] **Custom network controllers**: Programmable controllers (robots, computers) are out of scope (robot system is separate feature)
- [ ] **Fluid phase changes**: Fluids cannot change state (liquid ↔ gas) within networks
- [ ] **Power storage devices**: Batteries or capacitors are not included (future enhancement)
- [ ] **Network simulation accuracy**: Physical accuracy for pressure drop, flow rates is not required (gameplay-focused simulation)

---

## References

- [Architecture Design](../../.claude/architecture.md) - Overall system architecture, power/fluid/signal network designs
- [Core Network Implementation](../../src/core/network.rs) - NetworkGraph generic structure
- [Network Components](../../src/components/network.rs) - NetworkId, EntityMap for multiplayer
- [Existing Machines](../../src/game_spec/machines.rs) - MachineSpec structure
- [Power System Experiment](../2026-01-30-glm-power-system/specs/spec.md) - Reference implementation for power system

---

*Last Updated: 2026-01-30*
