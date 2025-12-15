# API Reference

## Module Structure
```
src/
├── main.rs          # Entry point
├── lib.rs           # GamePlugin
├── core/
│   ├── mod.rs       # CorePlugin
│   ├── config.rs    # GameConfig, ConfigPlugin
│   ├── input.rs     # KeyBindings
│   ├── registry.rs  # BlockRegistry, RecipeRegistry, RegistryPlugin
│   └── debug.rs     # DebugPlugin
├── rendering/
│   ├── mod.rs       # RenderingPlugin
│   ├── chunk.rs     # Chunk (CHUNK_SIZE=32)
│   ├── meshing.rs   # MeshDirty, ChunkMaterialHandle
│   ├── models.rs    # MeshBuilder, BlockVisual
│   └── voxel_loader.rs # VoxelAssets
├── gameplay/
│   ├── mod.rs       # GameplayPlugin
│   ├── grid.rs      # SimulationGrid, Direction, ItemSlot, Machine, MachineInstance
│   ├── building.rs  # BuildTool, MachinePlacedEvent, handle_building
│   ├── player.rs    # Player, spawn_player, look_player, move_player, grab_cursor
│   ├── items.rs     # VisualItem, update_visual_items
│   ├── interaction.rs # PlayerInteractEvent, InteractionPlugin
│   ├── power.rs     # PowerPlugin, PowerNode, PowerSource, PowerConsumer, Shaft
│   ├── multiblock.rs # MultiblockPlugin, MultiblockPattern, StructureValidator
│   └── machines/
│       ├── mod.rs       # register_machines
│       ├── conveyor.rs  # Conveyor, tick_conveyors
│       ├── miner.rs     # Miner, tick_miners
│       ├── assembler.rs # Assembler, tick_assemblers
│       ├── render.rs    # VisualMachine, update_machine_visuals
│       └── debug.rs     # draw_machine_io_markers
├── ui/
│   ├── mod.rs       # UiPlugin
│   ├── hud.rs       # spawn_crosshair
│   └── machine_ui.rs # MachineUiPlugin, MachineUiState
└── network/
    ├── mod.rs       # NetworkPlugin (stub)
    ├── client.rs    # client_network_system (stub)
    ├── server.rs    # server_network_system (stub)
    └── messages.rs  # ClientMessage, ServerMessage
```

---

## Core Module

### config.rs
```rust
#[derive(Resource)]
pub struct GameConfig {
    pub mouse_sensitivity: f32,    // default: 0.003
    pub walk_speed: f32,           // default: 5.0
    pub run_speed: f32,            // default: 10.0
    pub enable_highlight: bool,    // default: true
    pub max_items_per_conveyor: usize, // default: 4
}

pub struct ConfigPlugin; // init_resource::<GameConfig>()
```

### input.rs
```rust
#[derive(Resource)]
pub struct KeyBindings {
    pub forward: KeyCode,   // W
    pub backward: KeyCode,  // S
    pub left: KeyCode,      // A
    pub right: KeyCode,     // D
    pub jump: KeyCode,      // Space
    pub descend: KeyCode,   // ShiftLeft
    pub sprint: KeyCode,    // ControlLeft
}

pub struct InputPlugin; // init_resource::<KeyBindings>()
```

### registry.rs
```rust
#[derive(Resource, Default)]
pub struct BlockRegistry {
    pub map: HashMap<String, BlockProperty>,
}

pub struct BlockProperty {
    pub name: String,
    pub is_solid: bool,
    pub texture: String,
    pub collision_box: [f32; 6], // [min_x, min_y, min_z, max_x, max_y, max_z]
}

#[derive(Resource, Default)]
pub struct RecipeRegistry {
    pub map: HashMap<String, RecipeDefinition>,
}

pub struct RecipeDefinition {
    pub id: String,
    pub name: String,
    pub inputs: Vec<RecipeInput>,
    pub outputs: Vec<RecipeInput>,
    pub craft_time: f32,
}

pub struct RecipeInput {
    pub item: String,
    pub count: u32,
}

pub struct RegistryPlugin; // Startup: load_blocks, load_recipes
// Loads from: assets/data/blocks/core.yaml, assets/data/recipes/vanilla.yaml
```

### debug.rs
```rust
#[derive(Resource)]
pub struct DebugState {
    pub enabled: bool, // F3 toggle
}

pub struct DebugPlugin; // FPS display, compass
```

---

## Gameplay Module

### grid.rs
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    #[default] North, // Z-
    South,            // Z+
    East,             // X+
    West,             // X-
}

impl Direction {
    pub fn to_ivec3(&self) -> IVec3;
    pub fn opposite(&self) -> Self;
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemSlot {
    pub item_id: String,
    pub count: u32,
    pub progress: f32,           // 0.0-1.0 for conveyor position
    pub unique_id: u64,
    pub from_direction: Option<Direction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Machine {
    Conveyor(Conveyor),
    Miner(Miner),
    Assembler(Assembler),
}

#[derive(Clone, Debug, Component, PartialEq)]
pub struct MachineInstance {
    pub id: String,              // Block ID
    pub orientation: Direction,  // Front direction
    pub machine_type: Machine,
    pub power_node: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct SimulationGrid {
    pub machines: HashMap<IVec3, MachineInstance>,
}
```

### building.rs
```rust
#[derive(Resource, Default)]
pub struct BuildTool {
    pub active_block_id: String,
    pub orientation: Direction,
}

#[derive(Event)]
pub struct MachinePlacedEvent {
    pub pos: IVec3,
    pub machine_id: String,
}

pub fn handle_building(/*...*/) // Keys: 1=conveyor, 2=miner, 3=assembler
// Raycast, place on left-click, interact on right-click
```

### player.rs
```rust
#[derive(Component)]
pub struct Player {
    pub yaw: f32,   // Y-axis rotation
    pub pitch: f32, // X-axis rotation
}

pub fn spawn_player(commands: Commands);       // Position (0, 5, 0), Camera at +1.8y
pub fn look_player(/*...*/);                   // Mouse look
pub fn move_player(/*...*/);                   // WASD + Space/Shift + Ctrl sprint
pub fn grab_cursor(/*...*/);                   // Left-click lock, Escape unlock
```

### interaction.rs
```rust
#[derive(Event)]
pub struct PlayerInteractEvent {
    pub grid_pos: IVec3,
    pub mouse_button: MouseButton,
}

pub struct InteractionPlugin; // add_event::<PlayerInteractEvent>()
```

### power.rs
```rust
#[derive(Component)]
pub struct PowerNode {
    pub id: u32,
    pub group_id: Option<u32>,
}

#[derive(Component)]
pub struct PowerSource {
    pub capacity: f32,
    pub current_speed: f32,
}

#[derive(Component)]
pub struct PowerConsumer {
    pub stress_impact: f32,
    pub is_active: bool,
    pub current_speed_received: f32,
}

#[derive(Component)]
pub struct Shaft {
    pub stress_resistance: f32,
}

#[derive(Resource, Default)]
pub struct PowerNetworkGraph {
    pub adjacencies: HashMap<u32, HashSet<u32>>,
    pub node_entity_map: HashMap<u32, Entity>,
    pub next_node_id: u32,
}

#[derive(Resource, Default)]
pub struct PowerNetworkGroups {
    pub groups: HashMap<u32, NetworkGroup>,
    pub next_group_id: u32,
}

pub struct NetworkGroup {
    pub nodes: HashSet<u32>,
    pub total_stress_demand: f32,
    pub total_source_capacity: f32,
    pub is_overstressed: bool,
    pub ideal_speed: f32,
}

pub struct PowerPlugin; // FixedUpdate systems
// Systems: spawn_power_node_system, update_power_graph_system,
//          detect_network_groups_system, calculate_power_states_system
```

### multiblock.rs
```rust
pub struct MultiblockPattern {
    pub id: String,
    pub name: String,
    pub size: [i32; 3],
    pub blocks: HashMap<String, String>, // "x,y,z" -> block_id
    pub master_offset: [i32; 3],
}

impl MultiblockPattern {
    pub fn get_required_block(&self, offset: IVec3) -> Option<&String>;
    pub fn get_all_offsets(&self) -> Vec<IVec3>;
}

#[derive(Resource, Default)]
pub struct MultiblockRegistry {
    pub patterns: HashMap<String, MultiblockPattern>,
}

pub struct ValidationResult {
    pub is_valid: bool,
    pub pattern_id: Option<String>,
    pub master_pos: Option<IVec3>,
    pub missing_blocks: Vec<(IVec3, String)>,
    pub slave_positions: Vec<IVec3>,
}

pub struct StructureValidator;
impl StructureValidator {
    pub fn validate_at(pos: IVec3, pattern: &MultiblockPattern, grid: &SimulationGrid, direction: Direction) -> ValidationResult;
    pub fn find_valid_pattern(pos: IVec3, registry: &MultiblockRegistry, grid: &SimulationGrid) -> Option<ValidationResult>;
}

#[derive(Component)]
pub struct MultiblockMaster {
    pub pattern_id: String,
    pub slave_positions: Vec<IVec3>,
    pub is_formed: bool,
}

#[derive(Component)]
pub struct MultiblockSlave {
    pub master_pos: IVec3,
}

#[derive(Resource, Default)]
pub struct FormedMultiblocks {
    pub structures: HashMap<IVec3, MultiblockInfo>,
}

pub struct MultiblockInfo {
    pub pattern_id: String,
    pub master_pos: IVec3,
    pub slave_positions: Vec<IVec3>,
    pub formed_at: f64,
}

// Events
#[derive(Event)] pub struct MultiblockFormedEvent { pub master_pos: IVec3, pub pattern_id: String, pub slave_positions: Vec<IVec3> }
#[derive(Event)] pub struct MultiblockBrokenEvent { pub master_pos: IVec3, pub pattern_id: String }
#[derive(Event)] pub struct ValidateStructureEvent { pub pos: IVec3 }

pub struct MultiblockPlugin;
// Systems: check_multiblock_formation, validate_structures, check_multiblock_integrity
```

---

## Machines Module

### conveyor.rs
```rust
#[derive(Component, Default)]
pub struct Conveyor {
    pub inventory: Vec<ItemSlot>,
}

const CONVEYOR_SPEED: f32 = 1.0;

pub fn draw_conveyor_guides(/*...*/);           // Disabled
pub fn handle_conveyor_interaction(/*...*/);    // Right-click adds raw_ore (debug)
pub fn tick_conveyors(/*...*/);                 // Move items, transfer to next machine
```

### miner.rs
```rust
#[derive(Component, Default)]
pub struct Miner {
    pub progress: f32,
}

const MINING_SPEED: f32 = 1.0; // 1 item/second

pub fn tick_miners(/*...*/); // Generate raw_ore, output to front conveyor
```

### assembler.rs
```rust
#[derive(Component, Default)]
pub struct Assembler {
    pub input_inventory: Vec<ItemSlot>,
    pub output_inventory: Vec<ItemSlot>,
    pub active_recipe: Option<String>,
    pub crafting_progress: f32,
}

pub fn tick_assemblers(/*...*/);
// Receives items from front (opposite of orientation)
// Outputs to back (orientation direction)
// Processes recipe when inputs available
```

### render.rs
```rust
#[derive(Component)]
pub struct VisualMachine {
    pub grid_pos: IVec3,
}

pub fn update_machine_visuals(/*...*/); // Sync visual entities with SimulationGrid
```

### debug.rs
```rust
pub fn draw_machine_io_markers(/*...*/); // Gizmos for machine input/output
```

---

## UI Module

### hud.rs
```rust
pub fn spawn_crosshair(commands: Commands); // White crosshair at screen center
```

### machine_ui.rs
```rust
#[derive(States, Default)]
pub enum MachineUiState {
    #[default] Closed,
    Open,
}

#[derive(Resource, Default)]
pub struct OpenMachineUi {
    pub target_pos: Option<IVec3>,
}

// Marker components
#[derive(Component)] pub struct MachineUiRoot;
#[derive(Component)] pub struct RecipeButton { pub recipe_id: String }
#[derive(Component)] pub struct CloseButton;
#[derive(Component)] pub struct InventoryDisplay { pub slot_type: InventorySlotType }

pub enum InventorySlotType { Input, Output }

// Events
#[derive(Event)] pub struct OpenMachineUiEvent { pub pos: IVec3 }
#[derive(Event)] pub struct CloseMachineUiEvent;
#[derive(Event)] pub struct SetRecipeEvent { pub pos: IVec3, pub recipe_id: String }

pub struct MachineUiPlugin;
// OnEnter(Open): spawn_machine_ui
// OnExit(Open): despawn_machine_ui
// Update: handle_machine_interaction, handle_open/close_ui_event,
//         handle_recipe_button_click, handle_close_button_click,
//         update_inventory_display, handle_escape_key, apply_recipe_event
```

---

## Rendering Module

### chunk.rs
```rust
pub const CHUNK_SIZE: usize = 32;

#[derive(Component)]
pub struct Chunk {
    blocks: Vec<Option<String>>, // CHUNK_SIZE^3
}

impl Chunk {
    pub fn new() -> Self;
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&String>;
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, id: &str);
    pub fn index(x: usize, y: usize, z: usize) -> usize;
}
```

### meshing.rs
```rust
#[derive(Component)]
pub struct MeshDirty; // Marker for chunks needing remesh

#[derive(Component)]
pub struct ChunkMaterialHandle(pub Handle<StandardMaterial>);

pub fn update_chunk_mesh(/*...*/); // Greedy meshing
```

### models.rs
```rust
pub enum BlockVisual {
    Cube,
    Custom(Handle<Mesh>),
    None,
}

pub fn get_block_visual(/*...*/) -> BlockVisual;

pub struct MeshBuilder { pub mesh: Mesh }
impl MeshBuilder {
    pub fn new() -> Self;
    pub fn add_quad(/*...*/);
    pub fn build(self) -> Mesh;
}
```

### voxel_loader.rs
```rust
#[derive(Resource, Default)]
pub struct VoxelAssets {
    pub meshes: HashMap<String, Handle<Mesh>>,
}

pub fn load_vox_assets(/*...*/); // Loads .vox files from assets/models/
```

---

## Network Module (Stub)

### messages.rs
```rust
pub struct PlayerInput {
    pub forward: bool, pub back: bool, pub left: bool, pub right: bool,
}

pub enum ClientMessage {
    JoinRequest { player_name: String },
    PlayerInput { tick: u32, inputs: PlayerInput },
    PlaceBlock { pos: IVec3, block_id: String, orientation: Direction },
}

pub enum ServerMessage {
    Welcome { player_id: u64, game_state: InitialGameState },
    GameStateUpdate { tick: u32, delta: GameStateDelta },
    PlayerConnected { player_id: u64, player_name: String },
    PlayerDisconnected { player_id: u64 },
    Error { message: String },
}

pub struct InitialGameState {
    pub grid: HashMap<IVec3, MachineInstance>,
}

pub struct GameStateDelta {
    pub updated_machines: Vec<(IVec3, MachineInstance)>,
    pub removed_machines: Vec<IVec3>,
}
```

---

## Plugin Hierarchy
```
GamePlugin (lib.rs)
├── CorePlugin
│   ├── ConfigPlugin
│   ├── InputPlugin
│   ├── RegistryPlugin
│   └── DebugPlugin
├── RenderingPlugin
├── GameplayPlugin
│   ├── InteractionPlugin
│   ├── PowerPlugin
│   ├── MultiblockPlugin
│   └── register_machines()
├── UiPlugin
│   └── MachineUiPlugin
└── NetworkPlugin (stub)
```

---

## System Schedule

### Startup
- spawn_player
- load_blocks, load_recipes
- load_vox_assets
- spawn_crosshair
- load_multiblock_patterns

### Update
- handle_building
- move_player, look_player, grab_cursor
- tick_conveyors, tick_miners, tick_assemblers
- handle_conveyor_interaction
- update_machine_visuals
- draw_machine_io_markers
- update_visual_items
- update_chunk_mesh
- check_multiblock_formation, validate_structures, check_multiblock_integrity
- Machine UI systems

### FixedUpdate
- Power systems: spawn_power_node_system, update_power_graph_system,
  detect_network_groups_system, calculate_power_states_system

---

## Data Files
- `assets/data/blocks/core.yaml` - Block definitions
- `assets/data/recipes/vanilla.yaml` - Recipe definitions
- `assets/models/*.vox` - Voxel models

## Tests
- power.rs: 2 tests (grouping, overstress)
- multiblock.rs: 4 tests (pattern offsets, validation)
- machine_ui.rs: 2 tests (state transitions, recipe setting)
- assembler.rs: 1 test (full cycle)
