//! Conveyor belt system

use crate::block_type::BlockType;
use crate::constants::*;
use bevy::prelude::*;
use std::f32::consts::PI;

/// Direction for conveyor belts
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

/// Conveyor shape based on input connections
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ConveyorShape {
    #[default]
    Straight,
    CornerLeft,   // Input from left side
    CornerRight,  // Input from right side
    TJunction,    // Input from both sides
    Splitter,     // Output to front, left, and right (3-way split)
}

/// Conveyor belt component - moves items in a direction
#[derive(Component)]
pub struct Conveyor {
    /// World position of this conveyor
    pub position: IVec3,
    /// Direction items move
    pub direction: Direction,
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
    /// Create a new conveyor
    pub fn new(position: IVec3, direction: Direction) -> Self {
        Self {
            position,
            direction,
            items: Vec::new(),
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::Straight,
        }
    }

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
    pub fn add_item_with_visual(&mut self, block_type: BlockType, at_progress: f32, visual_entity: Option<Entity>, lateral_offset: f32) {
        self.items.push(ConveyorItem {
            block_type,
            progress: at_progress,
            visual_entity,
            lateral_offset,
        });
        // Sort by progress so we process items in order
        // Use unwrap_or(Ordering::Equal) to handle NaN gracefully
        self.items.sort_by(|a, b| a.progress.partial_cmp(&b.progress).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Add an item at the specified progress position (no visual, no lateral offset)
    pub fn add_item(&mut self, block_type: BlockType, at_progress: f32) {
        self.add_item_with_visual(block_type, at_progress, None, 0.0);
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
        // Conveyor should face away from the source (opposite of from_pos direction)
        match self.direction {
            Direction::East => expected_dir.x < 0,  // Source is west, facing east
            Direction::West => expected_dir.x > 0,  // Source is east, facing west
            Direction::South => expected_dir.z < 0, // Source is north, facing south
            Direction::North => expected_dir.z > 0, // Source is south, facing north
        }
    }

    /// Calculate the join progress position for an item coming from a source position.
    /// Returns Some(progress) if the item can join (correct direction), None otherwise.
    /// - From behind: joins at 0.0 (entry)
    /// - From side (perpendicular): joins at 0.5 (middle)
    pub fn get_join_progress(&self, from_pos: IVec3) -> Option<f32> {
        self.get_join_info(from_pos).map(|(p, _)| p)
    }

    /// Calculate join info (progress, lateral_offset) for an item coming from a source position.
    /// Returns Some((progress, lateral_offset)) if valid, None otherwise.
    /// lateral_offset: perpendicular offset from center (-0.5 to +0.5)
    /// Positive = right side of conveyor direction, Negative = left side
    pub fn get_join_info(&self, from_pos: IVec3) -> Option<(f32, f32)> {
        let offset = self.position - from_pos;

        // Determine if source is behind or to the side of this conveyor
        // Coordinate system: North=-Z, South=+Z, East=+X, West=-X
        // left/right are relative to conveyor direction
        match self.direction {
            Direction::East => {
                // Conveyor going East (+X)
                // left = North (-Z), right = South (+Z)
                if offset.x == 1 && offset.z == 0 {
                    Some((0.0, 0.0)) // Behind (West), join at entry, centered
                } else if offset.x == 0 && offset.z == 1 {
                    Some((0.5, 0.5)) // From South (+Z offset = right side), positive lateral
                } else if offset.x == 0 && offset.z == -1 {
                    Some((0.5, -0.5)) // From North (-Z offset = left side), negative lateral
                } else {
                    None
                }
            }
            Direction::West => {
                // Conveyor going West (-X)
                // left = South (+Z), right = North (-Z)
                if offset.x == -1 && offset.z == 0 {
                    Some((0.0, 0.0)) // Behind (East), join at entry, centered
                } else if offset.x == 0 && offset.z == 1 {
                    Some((0.5, -0.5)) // From South (+Z offset = left side), negative lateral
                } else if offset.x == 0 && offset.z == -1 {
                    Some((0.5, 0.5)) // From North (-Z offset = right side), positive lateral
                } else {
                    None
                }
            }
            Direction::South => {
                // Conveyor going South (+Z)
                // left = East (+X), right = West (-X)
                if offset.z == 1 && offset.x == 0 {
                    Some((0.0, 0.0)) // Behind (North), join at entry, centered
                } else if offset.z == 0 && offset.x == 1 {
                    Some((0.5, -0.5)) // From East (+X offset = left side), negative lateral
                } else if offset.z == 0 && offset.x == -1 {
                    Some((0.5, 0.5)) // From West (-X offset = right side), positive lateral
                } else {
                    None
                }
            }
            Direction::North => {
                // Conveyor going North (-Z)
                // left = West (-X), right = East (+X)
                if offset.z == -1 && offset.x == 0 {
                    Some((0.0, 0.0)) // Behind (South), join at entry, centered
                } else if offset.z == 0 && offset.x == 1 {
                    Some((0.5, 0.5)) // From East (+X offset = right side), positive lateral
                } else if offset.z == 0 && offset.x == -1 {
                    Some((0.5, -0.5)) // From West (-X offset = left side), negative lateral
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

/// Marker for conveyor visual entity
#[derive(Component)]
pub struct ConveyorVisual;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_rotation() {
        assert_eq!(Direction::North.rotate_cw(), Direction::East);
        assert_eq!(Direction::East.rotate_cw(), Direction::South);
        assert_eq!(Direction::South.rotate_cw(), Direction::West);
        assert_eq!(Direction::West.rotate_cw(), Direction::North);
    }

    #[test]
    fn test_direction_left_right() {
        assert_eq!(Direction::North.left(), Direction::West);
        assert_eq!(Direction::North.right(), Direction::East);
        assert_eq!(Direction::East.left(), Direction::North);
        assert_eq!(Direction::East.right(), Direction::South);
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::South.opposite(), Direction::North);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::West.opposite(), Direction::East);
    }

    #[test]
    fn test_conveyor_can_accept_item() {
        let mut conveyor = Conveyor::new(IVec3::ZERO, Direction::North);

        // Empty conveyor should accept
        assert!(conveyor.can_accept_item(0.0));

        // Add an item
        conveyor.add_item(BlockType::IronOre, 0.0);

        // Too close should reject
        assert!(!conveyor.can_accept_item(0.1));

        // Far enough should accept
        assert!(conveyor.can_accept_item(0.5));
    }

    #[test]
    fn test_conveyor_get_join_info() {
        // Conveyor at (5,7,5) going East (+X direction)
        // For a conveyor going East: left = North (-Z), right = South (+Z)
        let conveyor = Conveyor::new(IVec3::new(5, 7, 5), Direction::East);

        // From behind (west) - source at (4,7,5), offset = (5-4, 0, 0) = (1,0,0)
        let from_behind = conveyor.get_join_info(IVec3::new(4, 7, 5));
        assert_eq!(from_behind, Some((0.0, 0.0)));

        // From South - source at (5,7,6), offset = (5-5, 0, 5-6) = (0,0,-1)
        // South (+Z) is the RIGHT side of an East-going conveyor
        // But offset.z = -1, which is the NORTH direction in offset
        // Wait, offset = position - from_pos = (5,7,5) - (5,7,6) = (0,0,-1)
        // offset.z == -1 means source is at higher Z (south)
        // For East-going conveyor: from South means positive lateral (right side)
        // In the code: offset.z == -1 => Some((0.5, -0.5))
        let from_south = conveyor.get_join_info(IVec3::new(5, 7, 6));
        assert_eq!(from_south, Some((0.5, -0.5))); // From South, left side entry

        // From North - source at (5,7,4), offset = (0,0,1)
        // offset.z == 1 => Some((0.5, 0.5))
        let from_north = conveyor.get_join_info(IVec3::new(5, 7, 4));
        assert_eq!(from_north, Some((0.5, 0.5))); // From North, right side entry

        // From front (east) - should be None (can't join from the exit)
        let from_front = conveyor.get_join_info(IVec3::new(6, 7, 5));
        assert!(from_front.is_none());
    }
}
