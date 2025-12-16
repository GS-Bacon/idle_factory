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

### [P2] Player Inventory System
**Priority:** P2 (Important - Core Gameplay)
**Status:** ✅ Implemented

**User Story:**
As a player, I want to manage my items through an intuitive inventory UI with equipment slots and hotbar access, so that I can organize resources and equipment efficiently.

**Acceptance Criteria:**
- **GIVEN** I press the E key
- **WHEN** the inventory UI opens
- **THEN** I see my equipment slots, main inventory (8x3 grid), and hotbar (9 slots)
- **AND** I can drag and drop items between slots
- **AND** player movement and camera controls are disabled
- **AND** the cursor is automatically released

**Specifications:**
| Component | Details |
|-----------|---------|
| Equipment Slots | 5 slots (Head, Chest, Legs, Feet, Tool) |
| Main Inventory | 24 slots (8 columns × 3 rows) |
| Hotbar | 9 slots (slots 24-32) |
| Slot Size | 54×54 pixels |
| Max Stack Size | 999 items |

**Controls:**
| Action | Input | Behavior |
|--------|-------|----------|
| Open Inventory | E | Show player inventory UI |
| Close Inventory | E / Esc | Return to gameplay |
| Drag Item | Left Click + Drag | Move/swap items |
| Sort Inventory | Sort Button | Organize by item ID |
| Delete Item | Drag to Trash | Remove item |

**Technical Implementation:**
- `src/gameplay/inventory.rs`: Inventory data structures (ItemRegistry, PlayerInventory, EquipmentSlots)
- `src/ui/inventory_ui.rs`: UI rendering and interaction systems
- Minecraft-style layout with consistent 54×54px slot sizing
- Equipment panel (left), main inventory (center), crafting list (right)
- Hotbar HUD at screen bottom when inventory closed

---

### [P2] Creative Mode System
**Priority:** P2 (Important - Testing & Development)
**Status:** ✅ Implemented

**User Story:**
As a developer/player, I want a creative mode with unlimited items and building freedom, so that I can test designs and build without resource constraints.

**Acceptance Criteria:**
- **GIVEN** the game is in Creative mode (default for testing)
- **WHEN** I press E to open inventory
- **THEN** I see an item catalog grid with all available items
- **AND** I can toggle between item catalog and normal inventory
- **AND** the hotbar remains visible even when inventory is open

**Specifications:**
| Feature | Details |
|---------|---------|
| Default Mode | Creative (for testing phase) |
| Item Catalog | Grid display of all items from ItemRegistry |
| Toggle Button | 54×54px square button with ⇄ icon |
| Hotbar Visibility | Always visible in Creative mode |
| Layout | Item catalog OR inventory at center (same position toggle) |

**Controls:**
| Action | Input | Behavior |
|--------|-------|----------|
| Toggle View | Toggle Button (⇄) | Switch between item catalog and inventory |
| Switch Mode | /gamemode <survival\|creative> | Change game mode |

**Technical Implementation:**
- `src/gameplay/commands.rs`: GameMode enum (Survival/Creative)
- `src/ui/inventory_ui.rs`: CreativeItemList component, spawn_creative_item_grid
- Visibility-based toggling (Visibility::Hidden/Visible)
- Conditional hotbar spawning based on GameMode
- No side-by-side layout (single-position toggle design)

---

### [P2] Settings and Configuration UI
**Priority:** P2 (Important - User Experience)
**Status:** ✅ Implemented

**User Story:**
As a player, I want to adjust game settings (FPS, mouse sensitivity, visual options) through an accessible UI with immediate feedback, so that I can customize my experience without restarting.

**Acceptance Criteria:**
- **GIVEN** I press Esc key
- **WHEN** the settings button appears
- **THEN** I can click to open settings panel
- **AND** I can adjust FPS target (30/60/120/144/240)
- **AND** I can adjust mouse sensitivity (0.001-0.01)
- **AND** I can toggle Enable Highlight and Enable UI Blur
- **AND** settings apply immediately without Apply button
- **AND** pressing Esc twice returns to gameplay with cursor auto-grab

**Specifications:**
| Setting | Type | Range | Default |
|---------|------|-------|---------|
| Target FPS | Integer | 30-240 | 60 |
| Mouse Sensitivity | Float | 0.001-0.01 | 0.003 |
| Enable Highlight | Boolean | ON/OFF | ON |
| Enable UI Blur | Boolean | ON/OFF | OFF |

**Controls:**
| Action | Input | Behavior |
|--------|-------|----------|
| Open Settings Menu | Esc (1st press) | Show settings button |
| Show Settings Panel | Click Settings Button | Open settings UI |
| Return to Gameplay | Esc (2nd press) | Close UI, auto-grab cursor |
| Adjust FPS | +/- Buttons | Change target FPS |
| Adjust Sensitivity | +/- Buttons | Change mouse sensitivity (±0.001) |
| Toggle Features | Square Toggle Buttons (54×54px) | ON (green) / OFF (red) |

**Technical Implementation:**
- `src/ui/settings_ui.rs`: Settings UI state machine (Closed/ButtonVisible/SettingsOpen)
- `src/core/config.rs`: GameConfig resource for settings storage
- Immediate application (no Apply button)
- Component markers: FpsIncreaseButton, FpsDecreaseButton, SensitivityButtons, ToggleButtons
- Player control blocking when settings open (via SettingsUiState)

---

### [P1] Hotbar HUD System
**Priority:** P1 (Critical - Core Gameplay)
**Status:** ✅ Implemented

**User Story:**
As a player, I want to see my active hotbar items at the bottom of the screen with a clear selection indicator, so that I know what I'm currently holding and can quickly switch items.

**Acceptance Criteria:**
- **GIVEN** the inventory UI is closed
- **WHEN** I'm in gameplay mode
- **THEN** the hotbar (9 slots) is displayed at screen bottom
- **AND** the currently selected slot is highlighted
- **AND** the active item name is displayed below the hotbar
- **AND** the hotbar updates in real-time when items change

**Specifications:**
| Component | Details |
|-----------|---------|
| Slots Displayed | 9 (slots 24-32 from player inventory) |
| Position | Bottom center of screen |
| Slot Size | 54×54 pixels |
| Background | Semi-transparent dark background |
| Highlight | Yellow border (3px) on selected slot |

**Controls:**
| Action | Input | Behavior |
|--------|-------|----------|
| Select Slot | 1-9 Keys | Change active hotbar slot |
| Scroll Selection | Mouse Wheel | Cycle through hotbar slots |

**Technical Implementation:**
- `src/ui/inventory_ui.rs`: spawn_hotbar_hud, update_hotbar_hud systems
- Conditional spawning based on GameMode (always visible in Creative, hidden when inventory open in Survival)
- Real-time synchronization with PlayerInventory resource
- SelectedHotbarSlot component for highlight tracking

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
- Player inventory UI with drag-and-drop (E key)
- Equipment slots and hotbar management
- Creative mode item catalog with toggle
- Settings UI with immediate application
- Hotbar HUD with selection highlight
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
- 15 tests passing (inventory, power, multiblock, machines, commands, UI)
- 70%+ coverage on core systems
- Integration tests for conveyor → assembler flow
- UI state management tests for inventory and settings

---

## Related Documents

- `.specify/memory/constitution.md` - Project principles and standards
- `API_REFERENCE.md` - Complete API documentation
- `.specify/memory/changelog.md` - Development history

---

*This specification is based on the implemented GAME_SPEC.md and represents the current state of the game as of Phase 3 completion.*
