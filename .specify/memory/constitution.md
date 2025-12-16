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

*This constitution is a living document. Update it as the project evolves, but maintain consistency with established principles.*
