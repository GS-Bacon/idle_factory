//! Conveyor components: Conveyor, ConveyorItem, ConveyorShape, ConveyorVisual, ConveyorItemVisual

use crate::constants::*;
use crate::core::ItemId;
use bevy::prelude::*;

use super::Direction;

/// Single item on a conveyor
/// Stores ItemId directly to support Mod items without data loss
#[derive(Clone)]
pub struct ConveyorItem {
    /// The item being transported (ItemId preserves mod items)
    pub item_id: ItemId,
    /// Position on conveyor (0.0 = entry, 1.0 = exit)
    pub progress: f32,
    /// Previous progress for interpolation (set before each FixedUpdate tick)
    pub previous_progress: f32,
    /// Visual entity for this item
    pub visual_entity: Option<Entity>,
    /// Lateral offset for side-merge animation (-0.5 to 0.5, 0 = centered)
    pub lateral_offset: f32,
    /// Previous lateral offset for interpolation
    pub previous_lateral_offset: f32,
}

impl ConveyorItem {
    /// Create a new conveyor item from ItemId
    pub fn new(item_id: ItemId, progress: f32) -> Self {
        Self {
            item_id,
            progress,
            previous_progress: progress,
            visual_entity: None,
            lateral_offset: 0.0,
            previous_lateral_offset: 0.0,
        }
    }

    /// Get the item as ItemId (preferred API)
    pub fn get_item_id(&self) -> ItemId {
        self.item_id
    }

    /// Set the item type from ItemId
    pub fn set_item_id(&mut self, item_id: ItemId) {
        self.item_id = item_id;
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
    fn test_conveyor_item_with_base_item_no_panic() {
        // Base item should work normally
        let item = ConveyorItem::new(items::iron_ore(), 0.5);
        assert_eq!(item.get_item_id(), items::iron_ore());
    }

    #[test]
    fn test_conveyor_item_with_mod_item_no_panic() {
        // Mod item should NOT panic
        let mod_item_id = create_mod_item_id();
        let item = ConveyorItem::new(mod_item_id, 0.5);

        // get_item_id should return the mod item
        assert_eq!(item.get_item_id(), mod_item_id);
        // color() should return fallback gray
        assert_eq!(
            item.get_item_id().color(),
            bevy::prelude::Color::srgb(0.5, 0.5, 0.5)
        );
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
        // Should preserve the mod item ID
        assert_eq!(conveyor.items[0].get_item_id(), mod_item_id);
    }
}
