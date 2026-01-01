//! Machine components: Miner, Conveyor, Furnace, Crusher

use crate::BlockType;
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

    /// Calculate the shape this conveyor should have based on adjacent conveyors.
    ///
    /// New auto-connect logic (2026-01-01):
    /// 1. Check inputs: which neighbors output to this conveyor
    /// 2. Check "waiting": which neighbors can receive input from this conveyor
    /// 3. Determine shape based on input count and waiting count
    ///
    /// Returns (shape, output_direction)
    pub fn calculate_shape<'a>(
        &self,
        adjacent_conveyors: impl Iterator<Item = &'a Conveyor>,
    ) -> ConveyorShape {
        let (shape, _output_dir) = self.calculate_shape_and_output(adjacent_conveyors);
        shape
    }

    /// Calculate shape and output direction based on adjacent conveyors.
    /// Returns (shape, output_direction)
    pub fn calculate_shape_and_output<'a>(
        &self,
        adjacent_conveyors: impl Iterator<Item = &'a Conveyor>,
    ) -> (ConveyorShape, Direction) {
        // Collect adjacent conveyors into a vec for multiple passes
        let neighbors: Vec<&Conveyor> = adjacent_conveyors.collect();

        let back_pos = self.position - self.direction.to_ivec3();
        let left_pos = self.position + self.direction.left().to_ivec3();
        let right_pos = self.position + self.direction.right().to_ivec3();
        let front_pos = self.position + self.direction.to_ivec3();

        // Check inputs: which neighbors output to this conveyor
        let mut has_back_input = false;
        let mut has_left_input = false;
        let mut has_right_input = false;
        let mut has_front_input = false;

        for conv in &neighbors {
            // Check if this conveyor outputs to our position
            let outputs_to_us = conv.position + conv.direction.to_ivec3() == self.position;
            if !outputs_to_us {
                continue;
            }

            if conv.position == back_pos {
                has_back_input = true;
            } else if conv.position == left_pos {
                has_left_input = true;
            } else if conv.position == right_pos {
                has_right_input = true;
            } else if conv.position == front_pos {
                has_front_input = true;
            }
        }

        // Check "waiting": which neighbors can receive input from this conveyor
        // A neighbor is "waiting" if it can receive from our position (back, left, or right)
        // and is not already outputting to us
        let left_waiting = !has_left_input
            && neighbors.iter().any(|c| {
                c.position == left_pos && Self::can_receive_from_static(c, self.position)
            });
        let right_waiting = !has_right_input
            && neighbors.iter().any(|c| {
                c.position == right_pos && Self::can_receive_from_static(c, self.position)
            });
        let front_waiting = !has_front_input
            && neighbors.iter().any(|c| {
                c.position == front_pos && Self::can_receive_from_static(c, self.position)
            });

        let input_count =
            [has_back_input, has_left_input, has_right_input, has_front_input].iter().filter(|&&b| b).count();
        let wait_count = [left_waiting, right_waiting, front_waiting].iter().filter(|&&b| b).count();

        // Input 2+: TJunction (merge)
        if input_count >= 2 {
            // Determine output based on waiting
            let output_dir = if front_waiting {
                self.direction
            } else if right_waiting {
                self.direction.right()
            } else if left_waiting {
                self.direction.left()
            } else {
                self.direction // default: front
            };
            return (ConveyorShape::TJunction, output_dir);
        }

        // Input 1
        if input_count == 1 {
            // Back input
            if has_back_input {
                // 2+ waiting -> Splitter
                if wait_count >= 2 {
                    return (ConveyorShape::Splitter, self.direction);
                }
                // Right waiting only -> CornerRight
                if right_waiting && !front_waiting {
                    return (ConveyorShape::CornerRight, self.direction.right());
                }
                // Left waiting only -> CornerLeft
                if left_waiting && !front_waiting {
                    return (ConveyorShape::CornerLeft, self.direction.left());
                }
                // Default: Straight
                return (ConveyorShape::Straight, self.direction);
            }

            // Left input
            if has_left_input {
                // Front + right waiting -> Splitter
                if front_waiting && right_waiting {
                    return (ConveyorShape::Splitter, self.direction);
                }
                // Right waiting only -> CornerRight (left in, right out)
                if right_waiting && !front_waiting {
                    return (ConveyorShape::CornerRight, self.direction.right());
                }
                // Default: CornerLeft (left in, front out)
                return (ConveyorShape::CornerLeft, self.direction);
            }

            // Right input
            if has_right_input {
                // Front + left waiting -> Splitter
                if front_waiting && left_waiting {
                    return (ConveyorShape::Splitter, self.direction);
                }
                // Left waiting only -> CornerLeft (right in, left out)
                if left_waiting && !front_waiting {
                    return (ConveyorShape::CornerLeft, self.direction.left());
                }
                // Default: CornerRight (right in, front out)
                return (ConveyorShape::CornerRight, self.direction);
            }

            // Front input (head-on) -> Straight
            if has_front_input {
                return (ConveyorShape::Straight, self.direction);
            }
        }

        // Input 0: Straight
        (ConveyorShape::Straight, self.direction)
    }

    /// Check if a conveyor can receive input from a given position (static version)
    fn can_receive_from_static(conv: &Conveyor, from_pos: IVec3) -> bool {
        // A conveyor can receive from back, left, or right (not front)
        let back_pos = conv.position - conv.direction.to_ivec3();
        let left_pos = conv.position + conv.direction.left().to_ivec3();
        let right_pos = conv.position + conv.direction.right().to_ivec3();
        from_pos == back_pos || from_pos == left_pos || from_pos == right_pos
    }
}

/// Marker for conveyor's visual model child entity (for model swapping)
#[derive(Component)]
pub struct ConveyorVisual;

/// Marker for conveyor item visual
#[derive(Component)]
pub struct ConveyorItemVisual;

/// Miner component - automatically mines blocks below
#[derive(Component)]
pub struct Miner {
    /// World position of this miner
    pub position: IVec3,
    /// Mining progress (0.0-1.0)
    pub progress: f32,
    /// Buffer of mined items (block type, count)
    pub buffer: Option<(BlockType, u32)>,
}

impl Default for Miner {
    fn default() -> Self {
        Self {
            position: IVec3::ZERO,
            progress: 0.0,
            buffer: None,
        }
    }
}

/// Furnace component for smelting
#[derive(Component, Default)]
pub struct Furnace {
    /// Fuel slot (coal)
    pub fuel: u32,
    /// Input slot - stores ore type and count
    pub input_type: Option<BlockType>,
    pub input_count: u32,
    /// Output slot - stores ingot type and count
    pub output_type: Option<BlockType>,
    pub output_count: u32,
    /// Smelting progress (0.0-1.0)
    pub progress: f32,
}

impl Furnace {
    /// Get smelt output for an ore type
    pub fn get_smelt_output(ore: BlockType) -> Option<BlockType> {
        match ore {
            BlockType::IronOre => Some(BlockType::IronIngot),
            BlockType::CopperOre => Some(BlockType::CopperIngot),
            _ => None,
        }
    }

    /// Check if this ore type can be added to input
    pub fn can_add_input(&self, ore: BlockType) -> bool {
        const MAX_MACHINE_STACK: u32 = 64;
        let type_ok = self.input_type.is_none() || self.input_type == Some(ore);
        let count_ok = self.input_count < MAX_MACHINE_STACK;
        type_ok && count_ok
    }
}

/// Crusher component - doubles ore output
#[derive(Component)]
pub struct Crusher {
    /// World position of this crusher
    pub position: IVec3,
    /// Input ore type and count
    pub input_type: Option<BlockType>,
    pub input_count: u32,
    /// Output ore type and count (doubled)
    pub output_type: Option<BlockType>,
    pub output_count: u32,
    /// Processing progress (0.0-1.0)
    pub progress: f32,
}

impl Crusher {
    /// Check if this ore can be crushed
    pub fn can_crush(ore: BlockType) -> bool {
        matches!(ore, BlockType::IronOre | BlockType::CopperOre)
    }
}

/// Resource to hold loaded 3D model handles for machines and conveyors
#[derive(Resource, Default)]
pub struct MachineModels {
    /// Conveyor models by shape
    pub conveyor_straight: Option<Handle<Scene>>,
    pub conveyor_corner_left: Option<Handle<Scene>>,
    pub conveyor_corner_right: Option<Handle<Scene>>,
    pub conveyor_t_junction: Option<Handle<Scene>>,
    pub conveyor_splitter: Option<Handle<Scene>>,
    /// Machine models
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
}

impl MachineModels {
    /// Get conveyor model handle for a given shape
    pub fn get_conveyor_model(&self, shape: ConveyorShape) -> Option<Handle<Scene>> {
        match shape {
            ConveyorShape::Straight => self.conveyor_straight.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_conveyor(pos: IVec3, dir: Direction) -> Conveyor {
        Conveyor {
            position: pos,
            direction: dir,
            items: Vec::new(),
            last_output_index: 0,
            last_input_source: 0,
            shape: ConveyorShape::Straight,
        }
    }

    #[test]
    fn test_conveyor_shape_straight_when_no_input() {
        let target = make_conveyor(IVec3::new(5, 0, 5), Direction::North);
        let others: Vec<Conveyor> = vec![];
        assert_eq!(target.calculate_shape(others.iter()), ConveyorShape::Straight);
    }

    #[test]
    fn test_conveyor_shape_straight_when_back_input() {
        // Target at (5,0,5) facing North
        // Back is (5,0,6), needs to face North to feed into target
        let target = make_conveyor(IVec3::new(5, 0, 5), Direction::North);
        let back = make_conveyor(IVec3::new(5, 0, 6), Direction::North);
        let others = vec![back];
        assert_eq!(target.calculate_shape(others.iter()), ConveyorShape::Straight);
    }

    #[test]
    fn test_conveyor_shape_corner_left_when_left_input() {
        // Target at (5,0,5) facing North (-Z direction)
        // Left of North is West (-X)
        // So left position is (4,0,5), and it needs to face East (+X) to feed into target
        let target = make_conveyor(IVec3::new(5, 0, 5), Direction::North);
        let left = make_conveyor(IVec3::new(4, 0, 5), Direction::East);
        let others = vec![left];
        assert_eq!(target.calculate_shape(others.iter()), ConveyorShape::CornerLeft);
    }

    #[test]
    fn test_conveyor_shape_corner_right_when_right_input() {
        // Target at (5,0,5) facing North (-Z direction)
        // Right of North is East (+X)
        // So right position is (6,0,5), and it needs to face West (-X) to feed into target
        let target = make_conveyor(IVec3::new(5, 0, 5), Direction::North);
        let right = make_conveyor(IVec3::new(6, 0, 5), Direction::West);
        let others = vec![right];
        assert_eq!(target.calculate_shape(others.iter()), ConveyorShape::CornerRight);
    }

    #[test]
    fn test_conveyor_shape_t_junction_when_both_sides_input() {
        // Target at (5,0,5) facing North
        // Left feeds from (4,0,5) facing East
        // Right feeds from (6,0,5) facing West
        let target = make_conveyor(IVec3::new(5, 0, 5), Direction::North);
        let left = make_conveyor(IVec3::new(4, 0, 5), Direction::East);
        let right = make_conveyor(IVec3::new(6, 0, 5), Direction::West);
        let others = vec![left, right];
        assert_eq!(target.calculate_shape(others.iter()), ConveyorShape::TJunction);
    }

    #[test]
    fn test_conveyor_shape_t_junction_with_back_and_sides() {
        // T-junction should be selected even with back input
        let target = make_conveyor(IVec3::new(5, 0, 5), Direction::North);
        let back = make_conveyor(IVec3::new(5, 0, 6), Direction::North);
        let left = make_conveyor(IVec3::new(4, 0, 5), Direction::East);
        let right = make_conveyor(IVec3::new(6, 0, 5), Direction::West);
        let others = vec![back, left, right];
        assert_eq!(target.calculate_shape(others.iter()), ConveyorShape::TJunction);
    }

    #[test]
    fn test_conveyor_shape_ignores_non_feeding_conveyors() {
        // Adjacent conveyor that doesn't feed into target should be ignored
        let target = make_conveyor(IVec3::new(5, 0, 5), Direction::North);
        // Left conveyor facing South (away from target)
        let left_not_feeding = make_conveyor(IVec3::new(4, 0, 5), Direction::South);
        let others = vec![left_not_feeding];
        assert_eq!(target.calculate_shape(others.iter()), ConveyorShape::Straight);
    }

    #[test]
    fn test_conveyor_shape_splitter_auto_detect() {
        // Splitter is auto-detected when back input + 2+ waiting neighbors
        let target = make_conveyor(IVec3::new(5, 0, 5), Direction::North);
        // Back: conveyor feeding into target
        let back = make_conveyor(IVec3::new(5, 0, 6), Direction::North);
        // Left: conveyor waiting for input (its back is at target position)
        let left = make_conveyor(IVec3::new(4, 0, 5), Direction::West);
        // Right: conveyor waiting for input (its back is at target position)
        let right = make_conveyor(IVec3::new(6, 0, 5), Direction::East);
        let others = vec![back, left, right];
        assert_eq!(target.calculate_shape(others.iter()), ConveyorShape::Splitter);
    }

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
}
