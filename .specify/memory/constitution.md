# Project Constitution: Infinite Voxel Factory

## Overview
This constitution defines the foundational principles, coding standards, and architectural guidelines for the Infinite Voxel Factory project—a 3D voxel-based factory simulation game built with Rust and the Bevy game engine.

## Core Principles

### 1. Pure Factory Building Focus
- **No Survival Elements**: The game has no HP, hunger, or enemies
- **Automation First**: All features must support the core goal of building automated production chains
- **Peaceful Construction**: Players focus solely on engineering and optimization

### 2. Data-Driven Architecture
- **YAML-Based Configuration**: All game content (blocks, recipes, items) must be defined in external YAML files
- **Hot Reload Support**: Content changes should be reflected without rebuilding the game
- **Modding-Ready**: Architecture must support easy extension and modification by users

### 3. Technology Stack
- **Language**: Rust (stable)
- **Game Engine**: Bevy (latest stable version)
- **ECS-First**: Leverage Bevy's Entity Component System for all game logic
- **Deterministic Simulation**: Game logic must be reproducible and deterministic

## Code Quality Standards

### Naming Conventions
- **Modules**: snake_case (e.g., `gameplay/inventory.rs`)
- **Structs/Enums**: PascalCase (e.g., `PlayerInventory`, `InventoryUiState`)
- **Functions**: snake_case (e.g., `spawn_player`, `tick_conveyors`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `CHUNK_SIZE`, `SLOT_SIZE`)
- **Components**: Clear, descriptive names (e.g., `PowerNode`, `MachineInstance`)

### Documentation
- **Public API**: All public functions and types must have doc comments (`///`)
- **Module-Level**: Each module must have a file-level doc comment explaining its purpose
- **Complex Logic**: Non-obvious algorithms require inline comments explaining the "why"
- **Examples**: Complex systems should include usage examples in doc comments

### Error Handling
- **No Panics in Production**: Use `Result<T, E>` for fallible operations
- **Logging**: Use `info!`, `warn!`, `error!` macros appropriately
- **Graceful Degradation**: Systems should handle missing data without crashing

## Testing Standards

### Coverage Requirements
- **Minimum Coverage**: 70% for core gameplay systems
- **Critical Systems**: 90%+ coverage for power networks, inventory, crafting
- **UI Systems**: Visual testing where applicable

### Test Types
- **Unit Tests**: For isolated logic (inventory operations, recipes, validators)
- **Integration Tests**: For system interactions (conveyor → assembler → output)
- **Simulation Tests**: For deterministic gameplay scenarios

### Test-Driven Development
- **New Features**: Write failing tests first
- **Bug Fixes**: Add regression test before fixing
- **Refactoring**: Maintain passing tests throughout

## Architecture Standards

### ECS Best Practices
- **Components**: Pure data structures, no logic
- **Systems**: Single-responsibility functions operating on queries
- **Resources**: Global state only when necessary
- **Events**: For cross-system communication

### Performance Requirements
- **Target FPS**: 60 FPS minimum
- **Chunk Loading**: Async, non-blocking
- **Item Rendering**: Instanced rendering for 1000+ items
- **Memory**: Efficient data structures, avoid clones

### Module Organization
```
src/
├── core/        # Configuration, registry, input
├── rendering/   # Voxel rendering, meshing, models
├── gameplay/    # Game logic, machines, player
├── ui/          # All UI systems
└── network/     # Multiplayer (future)
```

### Dependency Rules
- **No Circular Dependencies**: Modules must form a DAG
- **Core Independence**: `core/` depends on nothing but Bevy
- **UI Isolation**: UI never directly modifies gameplay state (use events)

## User Experience Principles

### Controls
- **Keyboard First**: Primary controls via WASD + hotkeys
- **Mouse Support**: Camera control + UI interaction
- **Accessibility**: Rebindable keys, colorblind-friendly UI

### UI Design
- **Minecraft-Inspired**: Familiar slot-based inventories
- **Minimalist HUD**: Show only essential information
- **Tooltips**: Comprehensive information on hover

### Feedback
- **Visual Indicators**: Clear markers for machine state (active/idle/overstressed)
- **Sound Design**: Audio feedback for actions (placement, crafting)
- **Progression**: Clear goals and achievement milestones

## Technical Constraints

### Voxel System
- **Chunk Size**: 32×32×32 blocks
- **Greedy Meshing**: Required for performance
- **LOD**: Level-of-detail for distant chunks
- **Occlusion Culling**: Frustum + basic occlusion

### Power System
- **Kinetic Energy**: Rotation speed + stress mechanics
- **Network Groups**: BFS-based connected component detection
- **Overstress Behavior**: Graceful shutdown, no cascading failures
- **Deterministic**: Same input = same output, every time

### Multiplayer (Future)
- **Client-Server**: Authoritative server architecture
- **Fixed Timestep**: 20 TPS simulation
- **Client Prediction**: For smooth player movement
- **State Sync**: Delta compression for efficiency

## Security & Safety

### Memory Safety
- **Rust Guarantees**: Leverage ownership and borrowing
- **No Unsafe**: Only in critical performance paths, with justification
- **Dependency Audit**: Regular `cargo audit` checks

### Data Validation
- **YAML Parsing**: Fail gracefully on malformed data
- **Save Files**: Version checking and migration support
- **User Input**: Sanitize all external data

## Development Workflow

### Version Control
- **Branch Strategy**: Feature branches from `master`
- **Commit Messages**: Japanese, descriptive, with context
- **PR Requirements**: Tests pass, no compiler warnings

### Continuous Integration
- **Automated Tests**: All tests run on every commit
- **Clippy**: No warnings allowed
- **Format**: `cargo fmt` enforced

### Documentation
- **API Reference**: Auto-generated from doc comments
- **Roadmap**: Maintained in `PROJECT_ROADMAP.md`
- **Work Log**: Updated in `WORK_LOG.md`

## Success Metrics

### Performance
- **Frame Time**: < 16.67ms (60 FPS)
- **Build Time**: < 2 minutes (debug)
- **Test Suite**: < 10 seconds

### Quality
- **Zero Compiler Warnings**: Clean builds required
- **Test Coverage**: Trending upward
- **Documentation**: All public APIs documented

### Player Experience
- **Intuitive**: New features follow established patterns
- **Stable**: No crashes or data loss
- **Fun**: Automation feels rewarding and satisfying

---

## Project Phases and Roadmap

### Phase 1: Core Engine and Mod Foundation ✅ COMPLETE
**Goal:** Establish data-driven architecture and rendering foundation

**Completed Features:**
- Asset Server with hot-reload support
- YAML loader for blocks, items, and recipes
- Dynamic texture atlas generation
- Chunk system (32×32×32 blocks)
- Greedy meshing for voxel rendering
- Custom shader support (.wgsl)
- Multiplayer foundation (headless server, replication, client prediction)

### Phase 2: Logic and Logistics Simulation ✅ COMPLETE
**Goal:** Implement functional factory simulation

**Completed Features:**
- Fixed timestep simulation (20 TPS)
- Deterministic logic for multiplayer
- Grid-based machine placement (SimulationGrid)
- Item entity optimization with instanced rendering
- Inventory system with stacking
- Visual item system for conveyor belts
- Debug overlay (F3) and debug mode

### Phase 3: Power and Multiblock Systems ✅ COMPLETE
**Goal:** Advanced mechanics and complex structures

**Completed Features:**
- Power network with stress/speed mechanics
  - Graph-based network detection (BFS)
  - Stress calculation and propagation
  - Overstress detection and handling
- Multiblock structure validation
  - Pattern matching with rotation support
  - Master/Slave component system
  - Automatic integrity checking
- GUI framework for machine interaction
  - Recipe selection UI
  - Real-time inventory display
  - Event-driven state management

### Phase 4: Advanced Automation and Scripting ✅ COMPLETE
**Goal:** Enable player-created logic and automation

**Completed Features:**
- Lua VM integration (mlua)
  - Lua 5.4 with vendored build
  - Sandbox API (os, io, load, require disabled)
  - ScriptEngine and ScriptRegistry resources
  - Programmable component for machines
  - Built-in functions (print, clamp, lerp)
- Signal system
  - Wire-based signal transmission (SignalNetwork)
  - SignalEmitter/SignalReceiver components
  - Logic gates (AND, OR, NOT, XOR, NAND, NOR)
  - Numerical processors (Add, Subtract, Multiply, Divide, Compare, Equal)
  - BFS-based network group detection

**Technical Details:**
- Thread-safe Lua VM (Arc<Mutex<Lua>>)
- SignalValue enum for type-safe signal passing
- 12 new tests (5 scripting + 7 signals)

### Phase 5: Optimization and Distribution ✅ COMPLETE
**Goal:** Performance optimization and modding support

**Completed Features:**
- Multithreading
  - AsyncComputeTaskPool for parallel chunk generation
  - ChunkLoadQueue for non-blocking terrain generation
  - Task completion polling with block_on/poll_once
- LOD (Level of Detail)
  - ChunkLod component (Full, Medium, Low, Icon)
  - Distance-based LOD updates
  - Automatic chunk unloading for memory management
  - Configurable LOD distances via LodSettings
- Modding SDK
  - ModManifest YAML schema with dependencies
  - ModRegistry for mod management
  - Automatic mod discovery from `mods/` directory
  - Dependency resolution and load order

**Technical Details:**
- 6 new tests (3 optimization + 3 modding)
- Total: 40 tests passing

---

## Future Vision

### Ultimate Goals
- **Space Station Completion**: Players deliver resources to build a visible orbital station
- **Blueprint System**: Save and share factory designs
- **Creative/Survival Modes**: Different gameplay experiences
- **Community Mods**: Thriving modding ecosystem

### Design Philosophy for New Features
- **Always Data-Driven**: New content defined in YAML/external files
- **Player Empowerment**: Tools, not hand-holding
- **Emergent Complexity**: Simple rules, complex possibilities
- **No Artificial Limits**: Let players build massive factories

---

*This constitution is a living document. Update it as the project evolves, but maintain consistency with established principles.*
