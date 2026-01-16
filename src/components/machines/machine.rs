//! Machine component, MachineBundle, MachineSlot, MachineSlots and helper functions

use bevy::prelude::*;

use crate::core::ItemId;
use crate::game_spec::{find_recipe, MachineSpec, MachineType, UiSlotType};

use super::Direction;

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
    /// Create a new MachineBundle from spec.
    /// Uses bottom-center position (Y=0) for VOX model compatibility.
    pub fn new(spec: &'static MachineSpec, position: IVec3, facing: Direction) -> Self {
        // VOX models have origin at bottom center, so use Y=0 offset
        let world_pos = Vec3::new(
            position.x as f32 + 0.5,
            position.y as f32, // Bottom of block (Y=0 offset)
            position.z as f32 + 0.5,
        );
        Self {
            machine: Machine::new(spec, position, facing),
            transform: Transform::from_translation(world_pos).with_rotation(facing.to_rotation()),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }

    /// Create a new MachineBundle with block-center position (Y=0.5).
    /// Use this for fallback cube meshes that have center origin.
    pub fn new_centered(spec: &'static MachineSpec, position: IVec3, facing: Direction) -> Self {
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

/// Machine inventory slot storing ItemId directly to support Mod items
#[derive(Clone, Debug, Default)]
pub struct MachineSlot {
    /// The item stored in this slot (ItemId preserves mod items)
    pub item_id: Option<ItemId>,
    pub count: u32,
}

impl MachineSlot {
    pub const fn empty() -> Self {
        Self {
            item_id: None,
            count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.item_id.is_none() || self.count == 0
    }

    /// Add items using ItemId (primary API)
    pub fn add_id(&mut self, item: ItemId, amount: u32) -> u32 {
        if self.item_id.is_none() || self.item_id == Some(item) {
            self.item_id = Some(item);
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
            self.item_id = None;
        }
        taken
    }

    pub fn clear(&mut self) {
        self.item_id = None;
        self.count = 0;
    }

    /// Get item as ItemId
    pub fn get_item_id(&self) -> Option<ItemId> {
        self.item_id
    }

    /// Set item type from ItemId
    pub fn set_item_id(&mut self, item: ItemId) {
        self.item_id = Some(item);
    }

    /// Check if slot contains a specific ItemId
    pub fn contains_id(&self, item: ItemId) -> bool {
        self.item_id == Some(item)
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
// Helper functions for recipe lookup
// =============================================================================

/// Get smelt output by ItemId
pub fn get_smelt_output_by_id(ore: ItemId) -> Option<ItemId> {
    find_recipe(MachineType::Furnace, ore)
        .and_then(|recipe| recipe.outputs.first())
        .map(|output| output.item)
}

/// Check if item can be crushed by ItemId
pub fn can_crush_by_id(ore: ItemId) -> bool {
    find_recipe(MachineType::Crusher, ore).is_some()
}

/// Get crush output by ItemId
pub fn get_crush_output_by_id(ore: ItemId) -> Option<(ItemId, u32)> {
    find_recipe(MachineType::Crusher, ore)
        .and_then(|recipe| recipe.outputs.first())
        .map(|output| (output.item, output.count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{items, Id, ItemCategory, StringInterner};

    /// Create a fake Mod item ID that doesn't exist in base items
    fn create_mod_item_id() -> ItemId {
        let mut interner = StringInterner::new();
        Id::<ItemCategory>::from_string("mymod:super_ingot", &mut interner)
    }

    #[test]
    fn test_machine_slot_with_base_item_no_panic() {
        let mut slot = MachineSlot::empty();
        slot.add_id(items::iron_ore(), 10);

        assert_eq!(slot.get_item_id(), Some(items::iron_ore()));
        assert_eq!(slot.count, 10);
    }

    #[test]
    fn test_machine_slot_with_mod_item_no_panic() {
        let mod_item_id = create_mod_item_id();
        let mut slot = MachineSlot::empty();
        slot.add_id(mod_item_id, 5);

        // get_item_id should return the mod item
        assert_eq!(slot.get_item_id(), Some(mod_item_id));
        assert_eq!(slot.count, 5);
    }
}
