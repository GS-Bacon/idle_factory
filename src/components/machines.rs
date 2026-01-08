//! Machine components: Miner, Conveyor, Furnace, Crusher

use crate::constants::*;
use crate::core::ItemId;
use crate::game_spec::{find_recipe_by_id, MachineType};
use crate::BlockType;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Direction for conveyor belts
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Direction {
    North, // -Z
    South, // +Z
    East,  // +X
    West,  // -X
}

impl Direction {
    pub fn to_ivec3(self) -> IVec3 {
        match self {
            Direction::North => IVec3::new(0, 0, -1),
            Direction::South => IVec3::new(0, 0, 1),
            Direction::East => IVec3::new(1, 0, 0),
            Direction::West => IVec3::new(-1, 0, 0),
        }
    }

    pub fn to_rotation(self) -> Quat {
        match self {
            Direction::North => Quat::from_rotation_y(0.0),
            Direction::South => Quat::from_rotation_y(PI),
            Direction::East => Quat::from_rotation_y(-PI / 2.0),
            Direction::West => Quat::from_rotation_y(PI / 2.0),
        }
    }

    /// Rotate 90 degrees clockwise (when viewed from above)
    pub fn rotate_cw(self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    /// Get the direction to the left (counterclockwise)
    pub fn left(self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::East => Direction::North,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
        }
    }

    /// Get the direction to the right (clockwise)
    pub fn right(self) -> Self {
        self.rotate_cw()
    }

    /// Get the opposite direction
    pub fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

/// Single item on a conveyor
#[derive(Clone)]
pub struct ConveyorItem {
    pub block_type: BlockType,
    /// Position on conveyor (0.0 = entry, 1.0 = exit)
    pub progress: f32,
    /// Visual entity for this item
    pub visual_entity: Option<Entity>,
    /// Lateral offset for side-merge animation (-0.5 to 0.5, 0 = centered)
    pub lateral_offset: f32,
}

impl ConveyorItem {
    /// Create a new conveyor item from ItemId
    /// Note: Mod items fallback to Stone for rendering but preserve the ItemId
    pub fn new(item_id: ItemId, progress: f32) -> Self {
        // Try to convert to BlockType, fallback to Stone for Mod items
        let block_type = item_id.try_into().unwrap_or(BlockType::Stone);
        Self {
            block_type,
            progress,
            visual_entity: None,
            lateral_offset: 0.0,
        }
    }

    /// Create a new conveyor item from BlockType
    pub fn from_block_type(block_type: BlockType, progress: f32) -> Self {
        Self {
            block_type,
            progress,
            visual_entity: None,
            lateral_offset: 0.0,
        }
    }

    /// Get the item as ItemId (preferred API)
    pub fn item_id(&self) -> ItemId {
        self.block_type.into()
    }

    /// Get BlockType for rendering (always valid, never panics)
    pub fn block_type_for_render(&self) -> BlockType {
        self.block_type
    }

    /// Set the item type from ItemId
    /// Note: Mod items fallback to Stone for rendering
    pub fn set_item_id(&mut self, item_id: ItemId) {
        self.block_type = item_id.try_into().unwrap_or(BlockType::Stone);
    }
}

/// Conveyor shape based on input connections
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ConveyorShape {
    #[default]
    Straight,
    CornerLeft,  // Input from left side
    CornerRight, // Input from right side
    TJunction,   // Input from both sides
    Splitter,    // Output to front, left, and right (3-way split)
}

/// Conveyor belt component - moves items in a direction
#[derive(Component)]
pub struct Conveyor {
    /// World position of this conveyor
    pub position: IVec3,
    /// Direction items move (primary facing direction)
    pub direction: Direction,
    /// Actual output direction (may differ from direction for corners)
    pub output_direction: Direction,
    /// Items on this conveyor (sorted by progress, max CONVEYOR_MAX_ITEMS)
    pub items: Vec<ConveyorItem>,
    /// Index for round-robin output (splitter mode)
    pub last_output_index: usize,
    /// Index for alternating input (zipper mode)
    pub last_input_source: usize,
    /// Current shape (updated based on adjacent conveyors)
    pub shape: ConveyorShape,
}

impl Conveyor {
    /// Check if conveyor can accept a new item at the given position
    pub fn can_accept_item(&self, at_progress: f32) -> bool {
        if self.items.len() >= CONVEYOR_MAX_ITEMS {
            return false;
        }
        // Check spacing with existing items
        for item in &self.items {
            if (item.progress - at_progress).abs() < CONVEYOR_ITEM_SPACING {
                return false;
            }
        }
        true
    }

    /// Add an item at the specified progress position with optional visual and lateral offset
    pub fn add_item_with_visual(
        &mut self,
        item_id: ItemId,
        at_progress: f32,
        visual_entity: Option<Entity>,
        lateral_offset: f32,
    ) {
        let mut item = ConveyorItem::new(item_id, at_progress);
        item.visual_entity = visual_entity;
        item.lateral_offset = lateral_offset;
        self.items.push(item);
        // Sort by progress so we process items in order
        self.items.sort_by(|a, b| {
            a.progress
                .partial_cmp(&b.progress)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Add an item at the specified progress position (no visual, no lateral offset)
    pub fn add_item(&mut self, item_id: ItemId, at_progress: f32) {
        self.add_item_with_visual(item_id, at_progress, None, 0.0);
    }

    /// Add an item at the specified progress position from BlockType (legacy API)
    pub fn add_item_block_type(&mut self, block_type: BlockType, at_progress: f32) {
        self.add_item_with_visual(block_type.into(), at_progress, None, 0.0);
    }

    /// Check if conveyor can accept item at entry (progress = 0.0)
    #[allow(dead_code)]
    pub fn can_accept_at_entry(&self) -> bool {
        self.can_accept_item(0.0)
    }

    /// Check if this conveyor is facing away from the given position
    #[allow(dead_code)]
    pub fn is_facing_away_from(&self, from_pos: IVec3) -> bool {
        let expected_dir = from_pos - self.position;
        match self.direction {
            Direction::East => expected_dir.x < 0,
            Direction::West => expected_dir.x > 0,
            Direction::South => expected_dir.z < 0,
            Direction::North => expected_dir.z > 0,
        }
    }

    /// Calculate the join progress position for an item coming from a source position.
    pub fn get_join_progress(&self, from_pos: IVec3) -> Option<f32> {
        self.get_join_info(from_pos).map(|(p, _)| p)
    }

    /// Calculate join info (progress, lateral_offset) for an item coming from a source position.
    pub fn get_join_info(&self, from_pos: IVec3) -> Option<(f32, f32)> {
        let offset = self.position - from_pos;

        match self.direction {
            Direction::East => {
                if offset.x == 1 && offset.z == 0 {
                    Some((0.0, 0.0))
                } else if offset.x == 0 && offset.z == 1 {
                    Some((0.5, 0.5))
                } else if offset.x == 0 && offset.z == -1 {
                    Some((0.5, -0.5))
                } else {
                    None
                }
            }
            Direction::West => {
                if offset.x == -1 && offset.z == 0 {
                    Some((0.0, 0.0))
                } else if offset.x == 0 && offset.z == 1 {
                    Some((0.5, -0.5))
                } else if offset.x == 0 && offset.z == -1 {
                    Some((0.5, 0.5))
                } else {
                    None
                }
            }
            Direction::South => {
                if offset.z == 1 && offset.x == 0 {
                    Some((0.0, 0.0))
                } else if offset.z == 0 && offset.x == 1 {
                    Some((0.5, -0.5))
                } else if offset.z == 0 && offset.x == -1 {
                    Some((0.5, 0.5))
                } else {
                    None
                }
            }
            Direction::North => {
                if offset.z == -1 && offset.x == 0 {
                    Some((0.0, 0.0))
                } else if offset.z == 0 && offset.x == 1 {
                    Some((0.5, 0.5))
                } else if offset.z == 0 && offset.x == -1 {
                    Some((0.5, -0.5))
                } else {
                    None
                }
            }
        }
    }

    /// Get splitter output positions in round-robin order: [front, left, right]
    pub fn get_splitter_outputs(&self) -> [IVec3; 3] {
        let front = self.position + self.direction.to_ivec3();
        let left = self.position + self.direction.left().to_ivec3();
        let right = self.position + self.direction.right().to_ivec3();
        [front, left, right]
    }
}

/// Marker for conveyor's visual model child entity (for model swapping)
#[derive(Component)]
pub struct ConveyorVisual;

/// Marker for conveyor item visual
#[derive(Component)]
pub struct ConveyorItemVisual;

// =============================================================================
// Generic Machine Component (Phase C Data-Driven Design)
// =============================================================================

use crate::game_spec::MachineSpec;

// =============================================================================
// MachineBundle - Safe machine spawning (Phase D.0)
// =============================================================================

/// Bundle for spawning machines with all required components.
#[derive(Bundle)]
pub struct MachineBundle {
    pub machine: Machine,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl MachineBundle {
    /// Create a new MachineBundle from spec
    pub fn new(spec: &'static MachineSpec, position: IVec3, facing: Direction) -> Self {
        let world_pos = crate::utils::grid_to_world_center(position);
        Self {
            machine: Machine::new(spec, position, facing),
            transform: Transform::from_translation(world_pos).with_rotation(facing.to_rotation()),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

/// Slot storage for a machine (type + count)
#[derive(Clone, Debug, Default)]
pub struct MachineSlot {
    pub item_type: Option<BlockType>,
    pub count: u32,
}

impl MachineSlot {
    pub const fn empty() -> Self {
        Self {
            item_type: None,
            count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.item_type.is_none() || self.count == 0
    }

    pub fn add(&mut self, item: BlockType, amount: u32) -> u32 {
        if self.item_type.is_none() || self.item_type == Some(item) {
            self.item_type = Some(item);
            self.count += amount;
            amount
        } else {
            0 // Can't add different item type
        }
    }

    pub fn take(&mut self, amount: u32) -> u32 {
        let taken = amount.min(self.count);
        self.count -= taken;
        if self.count == 0 {
            self.item_type = None;
        }
        taken
    }

    pub fn clear(&mut self) {
        self.item_type = None;
        self.count = 0;
    }

    /// Get item type as ItemId (preferred API)
    pub fn item_id(&self) -> Option<ItemId> {
        self.item_type.map(|bt| bt.into())
    }

    /// Get BlockType for rendering (returns None for Mod items that don't have BlockType)
    pub fn block_type_for_render(&self) -> Option<BlockType> {
        self.item_type
    }

    /// Add items using ItemId (preferred API)
    /// Note: Mod items fallback to Stone for storage
    pub fn add_id(&mut self, item: ItemId, amount: u32) -> u32 {
        let block_type: BlockType = item.try_into().unwrap_or(BlockType::Stone);
        self.add(block_type, amount)
    }

    /// Set item type from ItemId
    /// Note: Mod items fallback to Stone for storage
    pub fn set_item_id(&mut self, item: ItemId) {
        self.item_type = Some(item.try_into().unwrap_or(BlockType::Stone));
    }

    /// Check if slot contains a specific ItemId
    pub fn contains_id(&self, item: ItemId) -> bool {
        self.item_type
            .map(|bt| bt == item.try_into().unwrap_or(BlockType::Stone))
            .unwrap_or(false)
    }
}

/// Generic machine slots container
#[derive(Clone, Debug)]
pub struct MachineSlots {
    /// Input slots (indexed by slot_id)
    pub inputs: Vec<MachineSlot>,
    /// Output slots (indexed by slot_id)
    pub outputs: Vec<MachineSlot>,
    /// Fuel count (for machines that require fuel)
    pub fuel: u32,
}

impl Default for MachineSlots {
    fn default() -> Self {
        Self {
            inputs: vec![MachineSlot::empty()],
            outputs: vec![MachineSlot::empty()],
            fuel: 0,
        }
    }
}

impl MachineSlots {
    /// Create slots based on MachineSpec
    pub fn from_spec(spec: &MachineSpec) -> Self {
        use crate::game_spec::UiSlotType;

        let mut max_input_id = 0u8;
        let mut max_output_id = 0u8;

        for slot_def in spec.ui_slots {
            match slot_def.slot_type {
                UiSlotType::Input => max_input_id = max_input_id.max(slot_def.slot_id + 1),
                UiSlotType::Output => max_output_id = max_output_id.max(slot_def.slot_id + 1),
                UiSlotType::Fuel => {} // Fuel is separate
            }
        }

        Self {
            inputs: vec![MachineSlot::empty(); max_input_id as usize],
            outputs: vec![MachineSlot::empty(); max_output_id as usize],
            fuel: 0,
        }
    }

    /// Get input slot by ID
    pub fn input(&self, slot_id: u8) -> Option<&MachineSlot> {
        self.inputs.get(slot_id as usize)
    }

    /// Get input slot by ID (mutable)
    pub fn input_mut(&mut self, slot_id: u8) -> Option<&mut MachineSlot> {
        self.inputs.get_mut(slot_id as usize)
    }

    /// Get output slot by ID
    pub fn output(&self, slot_id: u8) -> Option<&MachineSlot> {
        self.outputs.get(slot_id as usize)
    }

    /// Get output slot by ID (mutable)
    pub fn output_mut(&mut self, slot_id: u8) -> Option<&mut MachineSlot> {
        self.outputs.get_mut(slot_id as usize)
    }
}

/// Generic machine component - data-driven machine
#[derive(Component, Clone, Debug)]
pub struct Machine {
    /// Reference to machine spec
    pub spec: &'static MachineSpec,
    /// World position
    pub position: IVec3,
    /// Facing direction
    pub facing: Direction,
    /// Processing progress (0.0 - 1.0)
    pub progress: f32,
    /// Slot storage
    pub slots: MachineSlots,
    /// Tick counter (for timing/randomization)
    pub tick_count: u32,
}

impl Machine {
    /// Create a new machine from spec
    pub fn new(spec: &'static MachineSpec, position: IVec3, facing: Direction) -> Self {
        Self {
            spec,
            position,
            facing,
            progress: 0.0,
            slots: MachineSlots::from_spec(spec),
            tick_count: 0,
        }
    }

    /// Check if machine is currently processing
    pub fn is_processing(&self) -> bool {
        self.progress > 0.0
    }

    /// Get output position (facing direction + 1)
    pub fn output_position(&self) -> IVec3 {
        self.position + self.facing.to_ivec3()
    }

    /// Get input position (opposite of facing)
    pub fn input_position(&self) -> IVec3 {
        let back = match self.facing {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        };
        self.position + back.to_ivec3()
    }
}

// =============================================================================
// Legacy Machine Components - REMOVED
// =============================================================================
// Miner, Furnace, Crusher structs have been removed.
// Use the generic Machine component instead.
// See MachineSpec in game_spec/machines.rs for machine definitions.

// Helper functions moved to standalone functions for backward compatibility
// These use the recipe system as Single Source of Truth

/// Get smelt output for an ore type (uses recipe system)
pub fn get_smelt_output(ore: BlockType) -> Option<BlockType> {
    let item_id: ItemId = ore.into();
    find_recipe_by_id(MachineType::Furnace, item_id)
        .and_then(|recipe| recipe.outputs.first())
        .map(|output| output.item)
}

/// Check if this ore can be crushed (uses recipe system)
pub fn can_crush(ore: BlockType) -> bool {
    let item_id: ItemId = ore.into();
    find_recipe_by_id(MachineType::Crusher, item_id).is_some()
}

/// Get crush output for an ore type (uses recipe system)
pub fn get_crush_output(ore: BlockType) -> Option<(BlockType, u32)> {
    let item_id: ItemId = ore.into();
    find_recipe_by_id(MachineType::Crusher, item_id)
        .and_then(|recipe| recipe.outputs.first())
        .map(|output| (output.item, output.count))
}

/// Resource to hold loaded 3D model handles for machines and conveyors
#[derive(Resource, Default)]
pub struct MachineModels {
    /// Conveyor models by shape (glTF scenes)
    pub conveyor_straight: Option<Handle<Scene>>,
    pub conveyor_corner_left: Option<Handle<Scene>>,
    pub conveyor_corner_right: Option<Handle<Scene>>,
    pub conveyor_t_junction: Option<Handle<Scene>>,
    pub conveyor_splitter: Option<Handle<Scene>>,
    /// Machine models (glTF scenes)
    pub miner: Option<Handle<Scene>>,
    pub furnace: Option<Handle<Scene>>,
    pub crusher: Option<Handle<Scene>>,
    /// Item models (for conveyor display)
    pub item_iron_ore: Option<Handle<Scene>>,
    pub item_copper_ore: Option<Handle<Scene>>,
    pub item_coal: Option<Handle<Scene>>,
    pub item_stone: Option<Handle<Scene>>,
    pub item_iron_ingot: Option<Handle<Scene>>,
    pub item_copper_ingot: Option<Handle<Scene>>,
    /// Whether models are loaded (fallback to procedural if not)
    pub loaded: bool,
    /// VOX mesh handles (direct mesh, for hot reload)
    pub vox_miner: Option<Handle<Mesh>>,
    pub vox_conveyor_straight: Option<Handle<Mesh>>,
    /// Generation counter for hot reload (increment to trigger respawn)
    pub vox_generation: u32,
}

impl MachineModels {
    /// Get conveyor model handle for a given shape
    pub fn get_conveyor_model(&self, shape: ConveyorShape) -> Option<Handle<Scene>> {
        match shape {
            ConveyorShape::Straight => self.conveyor_straight.clone(),
            // No swap - logic correctly identifies turn direction
            ConveyorShape::CornerLeft => self.conveyor_corner_left.clone(),
            ConveyorShape::CornerRight => self.conveyor_corner_right.clone(),
            ConveyorShape::TJunction => self.conveyor_t_junction.clone(),
            ConveyorShape::Splitter => self.conveyor_splitter.clone(),
        }
    }

    /// Get item model handle for a given BlockType
    pub fn get_item_model(&self, block_type: crate::BlockType) -> Option<Handle<Scene>> {
        match block_type {
            crate::BlockType::IronOre => self.item_iron_ore.clone(),
            crate::BlockType::CopperOre => self.item_copper_ore.clone(),
            crate::BlockType::Coal => self.item_coal.clone(),
            crate::BlockType::Stone => self.item_stone.clone(),
            crate::BlockType::IronIngot => self.item_iron_ingot.clone(),
            crate::BlockType::CopperIngot => self.item_copper_ingot.clone(),
            _ => None, // Other block types don't have item models
        }
    }
}

// =============================================================================
// ECS Composition Components (Phase B architecture)
// =============================================================================

/// Input port definition for machines that accept items
#[derive(Clone, Debug)]
pub struct InputPort {
    /// Direction relative to machine facing (Back = behind machine)
    pub direction: PortDirection,
    /// Optional filter for accepted item types
    pub filter: Option<Vec<BlockType>>,
}

/// Output port definition for machines that eject items
#[derive(Clone, Debug)]
pub struct OutputPort {
    /// Direction relative to machine facing (Front = in front of machine)
    pub direction: PortDirection,
}

/// Port direction relative to machine facing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDirection {
    Front,
    Back,
    Left,
    Right,
}

/// Component for machines that accept items through input ports
#[derive(Component, Clone, Debug)]
pub struct ItemAcceptor {
    /// Input ports
    pub ports: Vec<InputPort>,
}

impl Default for ItemAcceptor {
    fn default() -> Self {
        Self {
            ports: vec![InputPort {
                direction: PortDirection::Back,
                filter: None,
            }],
        }
    }
}

/// Component for machines that eject items through output ports
#[derive(Component, Clone, Debug)]
pub struct ItemEjector {
    /// Output ports
    pub ports: Vec<OutputPort>,
}

impl Default for ItemEjector {
    fn default() -> Self {
        Self {
            ports: vec![OutputPort {
                direction: PortDirection::Front,
            }],
        }
    }
}

/// Component for machines that process recipes
#[derive(Component, Clone, Debug, Default)]
pub struct Crafter {
    /// Current recipe ID being processed (if any)
    pub recipe_id: Option<&'static str>,
    /// Processing progress (0.0 to 1.0)
    pub progress: f32,
    /// Speed multiplier (1.0 = normal speed)
    pub speed_multiplier: f32,
}

/// Component for machine's item inventory
#[derive(Component, Clone, Debug, Default)]
pub struct MachineInventory {
    /// Input slots
    pub input_slots: Vec<Option<(BlockType, u32)>>,
    /// Output slots
    pub output_slots: Vec<Option<(BlockType, u32)>>,
    /// Fuel slot (for furnaces)
    pub fuel_slot: Option<(BlockType, u32)>,
}

impl MachineInventory {
    /// Create with specified slot counts
    pub fn new(input_count: usize, output_count: usize, _has_fuel: bool) -> Self {
        // Note: fuel_slot is always initialized as None; has_fuel is reserved for future use
        Self {
            input_slots: vec![None; input_count],
            output_slots: vec![None; output_count],
            fuel_slot: None,
        }
    }
}

/// Component for machines that consume power (future use)
#[derive(Component, Clone, Debug, Default)]
pub struct PowerConsumer {
    /// Required power per tick
    pub required_power: f32,
    /// Currently available power
    pub current_power: f32,
}

// =============================================================================
// Machine Descriptor (for UI auto-generation)
// =============================================================================

/// Machine category for grouping in UI
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineCategory {
    /// Resource production (miner)
    Production,
    /// Item processing (furnace, crusher)
    Processing,
    /// Item logistics (conveyor, inserter)
    Logistics,
    /// Storage
    Storage,
}

/// Machine metadata for UI auto-generation
#[derive(Clone, Debug)]
pub struct MachineDescriptor {
    /// Unique machine ID
    pub id: &'static str,
    /// Display name (localized)
    pub display_name: &'static str,
    /// Category for UI grouping
    pub category: MachineCategory,
    /// Number of input slots
    pub input_slots: u8,
    /// Number of output slots
    pub output_slots: u8,
    /// Whether machine has a fuel slot
    pub has_fuel_slot: bool,
    /// Whether machine has recipe selection
    pub has_recipe_select: bool,
    /// Power consumption (None = no power required)
    pub power_consumption: Option<f32>,
}

impl MachineDescriptor {
    /// Miner descriptor
    pub const MINER: Self = Self {
        id: "miner",
        display_name: "採掘機",
        category: MachineCategory::Production,
        input_slots: 0,
        output_slots: 1,
        has_fuel_slot: false,
        has_recipe_select: false,
        power_consumption: None,
    };

    /// Furnace descriptor
    pub const FURNACE: Self = Self {
        id: "furnace",
        display_name: "精錬炉",
        category: MachineCategory::Processing,
        input_slots: 1,
        output_slots: 1,
        has_fuel_slot: true,
        has_recipe_select: false,
        power_consumption: None,
    };

    /// Crusher descriptor
    pub const CRUSHER: Self = Self {
        id: "crusher",
        display_name: "粉砕機",
        category: MachineCategory::Processing,
        input_slots: 1,
        output_slots: 1,
        has_fuel_slot: false,
        has_recipe_select: false,
        power_consumption: None,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_left_right() {
        assert_eq!(Direction::North.left(), Direction::West);
        assert_eq!(Direction::North.right(), Direction::East);
        assert_eq!(Direction::East.left(), Direction::North);
        assert_eq!(Direction::East.right(), Direction::South);
        assert_eq!(Direction::South.left(), Direction::East);
        assert_eq!(Direction::South.right(), Direction::West);
        assert_eq!(Direction::West.left(), Direction::South);
        assert_eq!(Direction::West.right(), Direction::North);
    }

    #[test]
    fn test_machine_inventory_new() {
        let inv = MachineInventory::new(2, 1, true);
        assert_eq!(inv.input_slots.len(), 2);
        assert_eq!(inv.output_slots.len(), 1);
    }

    #[test]
    fn test_machine_descriptor_constants() {
        assert_eq!(MachineDescriptor::MINER.id, "miner");
        assert_eq!(MachineDescriptor::FURNACE.has_fuel_slot, true);
        assert_eq!(MachineDescriptor::CRUSHER.input_slots, 1);
    }

    // =============================================================================
    // P.1: Mod item safety tests (no panic on unknown ItemId)
    // =============================================================================

    use crate::core::{items, Id, ItemCategory, StringInterner};

    /// Create a fake Mod item ID that doesn't exist in base items
    fn create_mod_item_id() -> ItemId {
        let mut interner = StringInterner::new();
        Id::<ItemCategory>::from_string("mymod:super_ingot", &mut interner)
    }

    #[test]
    fn test_conveyor_item_with_base_item_no_panic() {
        // Base item should work normally
        let item = ConveyorItem::new(items::iron_ore(), 0.5);
        assert_eq!(item.item_id(), items::iron_ore());
        assert_eq!(item.block_type_for_render(), BlockType::IronOre);
    }

    #[test]
    fn test_conveyor_item_with_mod_item_no_panic() {
        // Mod item should NOT panic - fallback to Stone for rendering
        let mod_item_id = create_mod_item_id();
        let item = ConveyorItem::new(mod_item_id, 0.5);

        // block_type_for_render should fallback to Stone (not panic)
        assert_eq!(item.block_type_for_render(), BlockType::Stone);
    }

    #[test]
    fn test_machine_slot_with_base_item_no_panic() {
        let mut slot = MachineSlot::empty();
        slot.add_id(items::iron_ore(), 10);

        assert_eq!(slot.item_id(), Some(items::iron_ore()));
        assert_eq!(slot.block_type_for_render(), Some(BlockType::IronOre));
        assert_eq!(slot.count, 10);
    }

    #[test]
    fn test_machine_slot_with_mod_item_no_panic() {
        let mod_item_id = create_mod_item_id();
        let mut slot = MachineSlot::empty();
        slot.add_id(mod_item_id, 5);

        // block_type_for_render should fallback to Stone (not panic)
        assert_eq!(slot.block_type_for_render(), Some(BlockType::Stone));
        assert_eq!(slot.count, 5);
    }

    #[test]
    fn test_conveyor_add_item_with_mod_item_no_panic() {
        let mod_item_id = create_mod_item_id();
        let mut conveyor = Conveyor {
            position: IVec3::ZERO,
            direction: Direction::East,
            output_direction: Direction::East,
            items: Vec::new(),
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::Straight,
        };

        // Adding Mod item should NOT panic
        conveyor.add_item(mod_item_id, 0.0);
        assert_eq!(conveyor.items.len(), 1);
        // Should fallback to Stone for rendering
        assert_eq!(conveyor.items[0].block_type_for_render(), BlockType::Stone);
    }
}
