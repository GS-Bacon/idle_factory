# Feature Specification: M3 Power System

**Feature Branch**: `001-power-system-spec`
**Created**: 2026-01-30
**Status**: Draft
**Input**: User description: "M3の電力システムの仕様を固める"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Power Network (Priority: P1)

Player places a generator (water wheel or coal generator) and connects it to machines using power wire blocks. Machines connected to powered networks operate normally, while machines without power stop functioning.

**Why this priority**: This is core functionality that enables the entire power system. Without basic power generation and distribution, no other power-related features have value.

**Independent Test**: Can be fully tested by placing a generator, connecting it to a machine with power wires, and observing the machine operates when powered and stops when unpowered.

**Acceptance Scenarios**:

1. **Given** a generator is placed, **When** power wires connect it to a machine, **Then** the machine receives power and operates normally
2. **Given** a powered machine, **When** the generator is removed, **Then** the machine loses power and stops operating
3. **Given** two separate power networks, **When** machines are connected to different networks, **Then** power states are independent (one can be unpowered while the other is powered)

---

### User Story 2 - Power UI Feedback (Priority: P2)

Players can visually identify which machines have power and which don't, and view overall power grid statistics including total generation, total consumption, and surplus/deficit.

**Why this priority**: Visual feedback is critical for player understanding. Without clear power indicators, players cannot debug why machines aren't working or optimize their power network.

**Independent Test**: Can be fully tested by building a network with mixed powered/unpowered machines and verifying visual indicators match actual power state, and checking grid statistics display accurate numbers.

**Acceptance Scenarios**:

1. **Given** a power network exists, **When** player opens machine UI, **Then** power status is clearly displayed (powered/unpowered)
2. **Given** a power network exists, **When** total consumption exceeds generation, **Then** UI shows power deficit and indicates which machines are unpowered
3. **Given** multiple power networks exist, **When** player views power statistics, **Then** statistics are displayed per-network with totals

---

### User Story 3 - Multiple Independent Power Grids (Priority: P3)

Players can create and manage multiple separate power networks that don't connect to each other. Each network has its own generation capacity, consumption, and power state.

**Why this priority**: Large factories often need separate power networks for isolation, redundancy, or different stages of expansion. Single-network limitation would constrain factory design flexibility.

**Independent Test**: Can be fully tested by creating two disconnected power networks with different generators and machines, then modifying one network and verifying the other is unaffected.

**Acceptance Scenarios**:

1. **Given** two separate power networks with no connection, **When** one network loses all generators, **Then** machines on the other network continue operating normally
2. **Given** two power networks close to each other, **When** power wires are placed but not directly connected, **Then** networks remain separate (no power transfer between them)
3. **Given** a machine is connected to one network, **When** the network is split by removing wires, **Then** two separate networks are created with appropriate power states

---

### User Story 4 - Fuel-Based Generators (Priority: P4)

Some generators (coal generator) consume fuel items to produce power. Power generation stops when fuel runs out, and players must refuel to restore operation.

**Why this priority**: Fuel-based generators add resource management complexity and create gameplay loops. Without this, power generation is too simple and doesn't create interesting logistical challenges.

**Independent Test**: Can be fully tested by placing a coal generator, loading fuel, observing power generation, waiting for fuel depletion, and refueling to restore generation.

**Acceptance Scenarios**:

1. **Given** a coal generator with fuel, **When** the generator operates, **Then** fuel is consumed over time and power is produced
2. **Given** a coal generator runs out of fuel, **When** no fuel remains, **Then** power generation stops and connected machines lose power
3. **Given** a coal generator without fuel, **When** player adds fuel to the generator, **Then** power generation resumes after a short startup delay

---

### User Story 5 - Dynamic Power Grid Updates (Priority: P5)

Power grids automatically recalculate when the network changes (wires added/removed, machines placed/destroyed, generators added/removed). Players see updated power states immediately after network modifications.

**Why this priority**: Responsive power grid behavior is essential for player experience. Delayed or incorrect updates cause confusion and make the system feel broken.

**Independent Test**: Can be fully tested by making various network changes and observing that power states update correctly within one game tick (50ms).

**Acceptance Scenarios**:

1. **Given** a powered network, **When** a new machine is connected, **Then** power state updates within one tick showing the new consumption
2. **Given** a power-deficit network, **When** a new generator is added, **Then** power surplus updates immediately and previously unpowered machines resume operation
3. **Given** a connected network, **When** a power wire is removed, **Then** the network splits into separate grids with recalculated power states

---

### Edge Cases

- What happens when a power wire connects two previously separate power grids with different power levels?
- What happens when a machine requires more power than any single generator can provide, but total generation is sufficient?
- How does the system handle cycles in the power network (e.g., wires forming loops)?
- What happens when a generator is destroyed while machines are actively consuming power?
- How does the system handle edge cases like a generator with 0 output or a machine with 0 consumption?
- What happens when the game is saved while machines are in an intermediate power state?
- How does the system handle power wire placement on top of other blocks?
- What happens when fuel is added to a generator that's already full?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST automatically detect connected power networks by analyzing adjacency of power wires, generators, and machines
- **FR-002**: System MUST calculate total power generation for each network by summing output from all connected generators
- **FR-003**: System MUST calculate total power consumption for each network by summing required power from all connected machines
- **FR-004**: System MUST determine power state for each machine based on whether network generation meets or exceeds consumption
- **FR-005**: System MUST prevent machines from operating when they are unpowered (stop processing, stop output, stop consuming inputs)
- **FR-006**: System MUST update power grid calculations within one game tick (50ms) after any network change occurs
- **FR-007**: System MUST support multiple independent power networks that don't share power
- **FR-008**: System MUST visually indicate power state (powered/unpowered) on machines through UI elements or visual effects
- **FR-009**: System MUST display power statistics including total generation, total consumption, and surplus/deficit for each network
- **FR-010**: System MUST support fuel-based generators that consume items from a fuel slot to produce power
- **FR-011**: System MUST stop power generation from fuel-based generators when fuel runs out
- **FR-012**: System MUST resume power generation from fuel-based generators after fuel is added, with a short startup delay
- **FR-013**: System MUST allow power wire blocks to be placed on any face of compatible blocks (machines, generators, other wires)
- **FR-014**: System MUST persist power grid state correctly across save/load cycles
- **FR-015**: System MUST handle edge cases gracefully (zero-power generators, zero-consumption machines, network splits) without crashes or incorrect behavior
- **FR-016**: System MUST allow players to view detailed power information (generation/consumption per machine) through machine UI
- **FR-017**: System MUST support at least 100 machines on a single power grid without performance degradation
- **FR-018**: System MUST provide clear feedback when power consumption exceeds generation (visual indicators, UI warnings)
- **FR-019**: System MUST prevent power wire connections between incompatible blocks if specified by game design
- **FR-020**: System MUST support different generator types with different power outputs and fuel requirements

### Key Entities *(include if feature involves data)*

- **PowerProducer**: Represents a generator that produces power. Key attributes: output capacity (watts), fuel type (optional), fuel consumption rate, current fuel amount, operational state
- **PowerConsumer**: Represents a machine that consumes power to operate. Key attributes: power requirement (watts), current power state (powered/unpowered), operational state based on power availability
- **PowerGrid**: Represents a connected network of power producers, consumers, and wires. Key attributes: total generation capacity, total consumption, surplus/deficit, connected entities, network ID
- **PowerWire**: Represents a block that connects power infrastructure. Key attributes: position, connection points (which sides are connected), associated network ID
- **FuelSlot**: Represents a slot in fuel-based generators that holds consumable items. Key attributes: capacity, current item, quantity, item type restrictions

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can create a functional power network with generator and machines within 5 minutes of first attempt
- **SC-002**: Power grid calculations complete within 50ms (one game tick) after any network modification (wire placement/removal, machine placement/removal, generator addition/removal)
- **SC-003**: Players can identify unpowered machines visually within 2 seconds of network change
- **SC-004**: Power statistics UI displays accurate values with less than 1% error compared to actual grid state
- **SC-005**: Power grid state persists correctly across save/load cycles with 100% accuracy (no machines incorrectly powered/unpowered)
- **SC-006**: System supports at least 100 machines on a single power grid with power grid calculations completing within 50ms per tick
- **SC-007**: Fuel-based generators operate for expected duration based on fuel consumption rate (within 5% variance)
- **SC-008**: Multiple independent power grids operate correctly without power transfer between them (100% isolation)
- **SC-009**: Players successfully power up their first machine network within 10 minutes of initial tutorial without requiring external help
- **SC-010**: Power state visual indicators are correctly understood by at least 90% of players on first attempt (measured through playtesting)

## Assumptions *(include if needed)*

- Power flows instantaneously through the network (no propagation delay)
- Power is not a consumable resource like electricity in real physics - it's binary (sufficient or insufficient)
- Power wires have infinite capacity and don't lose power over distance
- Players have basic understanding of factory games and block placement mechanics
- Single-player focus (multiplayer power network synchronization is out of scope for M3)
- Power grid calculations occur in the fixed tick system (20Hz, 50ms per tick)
- Generators have constant output when operational (no ramp-up/down phases)
- Machines either operate at full power or not at all (no partial-power operation)
