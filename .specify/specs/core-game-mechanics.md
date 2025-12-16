# Feature Specification: Core Game Mechanics

**Branch:** `master`
**Date:** 2025-12-16
**Status:** Implemented (Phase 1-3 Complete)

---

## Summary

Infinite Voxel Factoryは、Factorio、Satisfactory、Minecraft(Create Mod)から影響を受けた3Dボクセル工場シミュレーションゲームです。敵やサバイバル要素を排除し、純粋な「自動化」と「建築」に特化したゲーム体験を提供します。

**技術スタック:** Rust + Bevy Engine 0.15
**アーキテクチャ:** ECS (Entity Component System)
**データ駆動:** YAML定義によるブロック/アイテム/レシピ

---

## User Scenarios & Testing

### [P1] Player Movement and Camera Control
**Priority:** P1 (Critical - Core Gameplay)
**Status:** ✅ Implemented

**User Story:**
As a player, I want to freely navigate the 3D voxel world with intuitive FPS controls, so that I can explore and build my factory efficiently.

**Acceptance Criteria:**
- **GIVEN** the game is running
- **WHEN** I press WASD keys
- **THEN** the player moves in the corresponding direction
- **AND** pressing Ctrl increases movement speed (sprint)
- **AND** pressing Space/Shift moves the player up/down

**Controls:**
| Action | Input | Speed |
|--------|-------|-------|
| Forward | W | 5.0 units/s (10.0 with sprint) |
| Backward | S | 5.0 units/s |
| Left | A | 5.0 units/s |
| Right | D | 5.0 units/s |
| Up | Space | 5.0 units/s |
| Down | Shift | 5.0 units/s |
| Sprint | Ctrl | 2x multiplier |
| Look | Mouse | Sensitivity: 0.003 |

**Technical Implementation:**
- `src/gameplay/player.rs`: Player component with yaw/pitch
- Camera at +1.8y offset (eye level)
- Cursor lock on left-click, release on Escape

---

### [P1] Machine Placement and Building
**Priority:** P1 (Critical - Core Gameplay)
**Status:** ✅ Implemented

**User Story:**
As a player, I want to place machines (conveyors, miners, assemblers) in the world with correct orientation, so that I can build automated production lines.

**Acceptance Criteria:**
- **GIVEN** I have selected a machine type (1/2/3 key)
- **WHEN** I left-click on a valid location
- **THEN** the machine is placed facing away from me (output toward player)
- **AND** the machine is added to the simulation grid
- **AND** visual feedback confirms placement

**Controls:**
| Key | Machine | Description |
|-----|---------|-------------|
| 1 | Conveyor | Transport items |
| 2 | Miner | Generate raw_ore infinitely |
| 3 | Assembler | Process items via recipes |
| Left Click | Place | Place selected machine |
| Right Click | Interact | Open machine UI |

**Technical Implementation:**
- `src/gameplay/building.rs`: Raycast-based placement
- `SimulationGrid`: HashMap<IVec3, MachineInstance>
- Direction-based orientation (North/South/East/West)

---

### [P1] Item Transport via Conveyors
**Priority:** P1 (Critical - Core Gameplay)
**Status:** ✅ Implemented

**User Story:**
As a player, I want items to automatically move along conveyor belts, so that I can create efficient logistics systems.

**Acceptance Criteria:**
- **GIVEN** a conveyor belt with items on it
- **WHEN** the game updates
- **THEN** items move at 1.0 units/second
- **AND** items maintain 0.25 spacing between each other
- **AND** items transfer to the next machine when progress >= 1.0

**Specifications:**
| Parameter | Value |
|-----------|-------|
| Transport Speed | 1.0 units/s |
| Max Items per Belt | 4 |
| Item Spacing | 0.25 (1/4 of belt length) |
| Collision Box | 1.0 x 0.2 x 1.0 |

**Data Structure:**
```rust
pub struct ItemSlot {
    pub item_id: String,
    pub count: u32,
    pub progress: f32,           // 0.0-1.0 position on conveyor
    pub unique_id: u64,
    pub from_direction: Option<Direction>,
}
```

**Technical Implementation:**
- `src/gameplay/machines/conveyor.rs`: tick_conveyors system
- Progress-based positioning (no physics engine)
- Automatic transfer to adjacent machines

---

### [P2] Recipe-Based Crafting in Assemblers
**Priority:** P2 (Important - Automation)
**Status:** ✅ Implemented

**User Story:**
As a player, I want to configure assemblers with recipes and watch them automatically craft items, so that I can automate production chains.

**Acceptance Criteria:**
- **GIVEN** an assembler with a selected recipe
- **WHEN** required inputs arrive from the front
- **THEN** the assembler begins crafting
- **AND** outputs are ejected to the back when complete
- **AND** the UI shows real-time inventory and progress

**Specifications:**
| Parameter | Value |
|-----------|-------|
| Input Inventory | 10 slots max |
| Output Inventory | 10 slots max |
| Input Direction | Front (opposite of orientation) |
| Output Direction | Back (orientation direction) |
| Collision Box | 1.0 x 1.0 x 1.0 |

**Recipe Format (YAML):**
```yaml
- id: "ore_to_ingot"
  name: "Iron Ingot"
  inputs:
    - item: "raw_ore"
      count: 1
  outputs:
    - item: "ingot"
      count: 1
  craft_time: 2.0
```

**Technical Implementation:**
- `src/gameplay/machines/assembler.rs`: tick_assemblers system
- `src/ui/machine_ui.rs`: Recipe selection UI
- Event-driven UI state management

---

### [P2] Power Network System
**Priority:** P2 (Important - Advanced Mechanics)
**Status:** ✅ Implemented

**User Story:**
As a player, I want machines to be connected by a power network with stress/speed mechanics, so that I must balance power generation and consumption.

**Acceptance Criteria:**
- **GIVEN** multiple machines connected via shafts
- **WHEN** the network is overstressed
- **THEN** all consumers in the network stop functioning
- **AND** visual indicators show the overstressed state

**Specifications:**
| Component | Properties |
|-----------|-----------|
| PowerSource | capacity, current_speed |
| PowerConsumer | stress_impact, is_active |
| Shaft | stress_resistance |
| PowerNode | id, group_id |

**Algorithm:**
1. Detect connected components using BFS
2. Calculate total stress and capacity per group
3. Mark groups as overstressed if `stress > capacity`
4. Propagate speed and active state to machines

**Technical Implementation:**
- `src/gameplay/power.rs`: Graph-based power network
- FixedUpdate schedule for deterministic simulation
- BFS for connected component detection

---

### [P3] Multiblock Structures
**Priority:** P3 (Future - Advanced Features)
**Status:** ✅ Implemented

**User Story:**
As a player, I want to build large multi-block structures that form when placed correctly, so that I can create advanced production facilities.

**Acceptance Criteria:**
- **GIVEN** I place blocks matching a multiblock pattern
- **WHEN** the structure is validated
- **THEN** a MultiblockFormedEvent is fired
- **AND** the master block controls the entire structure
- **AND** breaking any block destroys the structure

**Specifications:**
```rust
pub struct MultiblockPattern {
    pub id: String,
    pub name: String,
    pub size: [i32; 3],
    pub blocks: HashMap<String, String>, // "x,y,z" -> block_id
    pub master_offset: [i32; 3],
}
```

**Events:**
- `MultiblockFormedEvent`: Structure successfully formed
- `MultiblockBrokenEvent`: Structure destroyed
- `ValidateStructureEvent`: Manual validation request

**Technical Implementation:**
- `src/gameplay/multiblock.rs`: Pattern matching and validation
- Master/Slave component system
- Automatic integrity checking

---

## Requirements

### Functional Requirements

**[MUST] Core Gameplay**
- Player movement with WASD + Mouse look
- Machine placement with orientation control
- Item transport along conveyors
- Recipe-based crafting in assemblers
- Real-time simulation at 20 TPS

**[MUST] Data-Driven Architecture**
- YAML-based block definitions (`assets/data/blocks/`)
- YAML-based recipe definitions (`assets/data/recipes/`)
- Hot-reload support for content changes

**[MUST] UI Systems**
- Machine interaction UI (right-click)
- Inventory display with real-time updates
- Recipe selection interface

**[SHOULD] Advanced Features**
- Power network with stress/speed mechanics
- Multiblock structure validation
- Visual feedback for machine states

**[COULD] Future Enhancements**
- Lua/Python scripting for automation
- Blueprint system
- Multiplayer synchronization

### Key Entities

**Player**
- Position, yaw, pitch
- Movement speed (walk/sprint)
- Camera offset (+1.8y)

**Machine Types**
- **Conveyor**: Transports items at 1.0 units/s
- **Miner**: Generates raw_ore infinitely at 1.0 items/s
- **Assembler**: Crafts items according to recipes

**Items**
- `raw_ore`: Base resource from miners
- `ingot`: Processed from raw_ore

**Power Components**
- PowerNode, PowerSource, PowerConsumer, Shaft
- Network groups with stress calculation

---

## Success Criteria

### User Metrics
- Players can build a functional ore-to-ingot production line within 5 minutes
- Conveyor system handles 100+ items without performance degradation
- Machine placement feels intuitive and responsive (< 100ms feedback)

### System Performance
- Maintain 60 FPS with 1000+ items in transit
- Simulation runs at stable 20 TPS
- UI updates in real-time (< 16ms per frame)

### Satisfaction Metrics
- Automation feels rewarding and predictable
- Power network mechanics provide meaningful challenge
- Building is satisfying with clear visual feedback

### Business Impact
- Foundation for future content (blueprints, scripting, multiplayer)
- Modding-friendly architecture supports community contributions
- Scalable to large factories (10,000+ machines)

---

## Edge Cases & Error Handling

### Conveyor System
- **Full Belt**: Items wait at source until space available
- **Blocked Output**: Items queue at assembler output
- **Loop Paths**: Items continue circulating (no deadlock detection yet)

### Machine Placement
- **Invalid Location**: Placement rejected with visual feedback
- **Overlapping Machines**: Grid prevents overlap
- **Boundary Conditions**: Placement limited to loaded chunks

### Power Network
- **Overstressed Network**: All consumers deactivate gracefully
- **Disconnected Machines**: Form separate network groups
- **Network Changes**: Recalculated on machine add/remove

### Multiblock Structures
- **Partial Placement**: No effect until complete pattern matched
- **Structure Broken**: All slave blocks removed, master becomes regular block
- **Rotation Mismatch**: Validation fails if orientation incorrect

---

## Implementation Notes

**Completed Phases:**
- ✅ Phase 1: Core Engine and Mod Foundation
- ✅ Phase 2: Logic and Logistics Simulation
- ✅ Phase 3: Power and Multiblock Systems

**Next Phases:**
- ⏳ Phase 4: Scripting and Advanced Automation
- ⏳ Phase 5: Optimization and Distribution

**Technical Decisions:**
- No physics engine for items (data-driven movement)
- ECS architecture for all game logic
- Fixed timestep for deterministic simulation
- Greedy meshing for voxel rendering

**Testing Coverage:**
- 13 tests passing (inventory, power, multiblock, machines)
- 70%+ coverage on core systems
- Integration tests for conveyor → assembler flow

---

## Related Documents

- `.specify/memory/constitution.md` - Project principles and standards
- `API_REFERENCE.md` - Complete API documentation
- `.specify/memory/changelog.md` - Development history

---

*This specification is based on the implemented GAME_SPEC.md and represents the current state of the game as of Phase 3 completion.*
