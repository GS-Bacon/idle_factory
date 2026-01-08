//! Blueprint system for saving and loading building patterns

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::components::Direction;
use crate::core::ItemId;

/// Direction for serialization (separate from gameplay Direction to maintain clean separation)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlueprintDirection {
    North,
    South,
    East,
    West,
}

impl From<Direction> for BlueprintDirection {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => BlueprintDirection::North,
            Direction::South => BlueprintDirection::South,
            Direction::East => BlueprintDirection::East,
            Direction::West => BlueprintDirection::West,
        }
    }
}

impl From<BlueprintDirection> for Direction {
    fn from(dir: BlueprintDirection) -> Self {
        match dir {
            BlueprintDirection::North => Direction::North,
            BlueprintDirection::South => Direction::South,
            BlueprintDirection::East => Direction::East,
            BlueprintDirection::West => Direction::West,
        }
    }
}

/// A block within a blueprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintBlock {
    /// Offset from the blueprint origin
    pub offset: IVec3,
    /// Block type (stored as string ID for serialization)
    pub item_id: ItemId,
    /// Rotation (0-3: 0, 90, 180, 270 degrees)
    pub rotation: u8,
    /// Machine direction (if this is a machine)
    pub direction: Option<BlueprintDirection>,
}

/// Blueprint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    /// Blueprint name
    pub name: String,
    /// Size (bounding box)
    pub size: IVec3,
    /// List of blocks
    pub blocks: Vec<BlueprintBlock>,
    /// Creation timestamp
    pub created_at: Option<f64>,
    /// Description
    pub description: Option<String>,
}

impl Blueprint {
    /// Create a new empty blueprint
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: IVec3::ZERO,
            blocks: Vec::new(),
            created_at: None,
            description: None,
        }
    }

    /// Add a block to the blueprint
    pub fn add_block(&mut self, block: BlueprintBlock) {
        // Update size to encompass the new block
        self.size.x = self.size.x.max(block.offset.x + 1);
        self.size.y = self.size.y.max(block.offset.y + 1);
        self.size.z = self.size.z.max(block.offset.z + 1);
        self.blocks.push(block);
    }

    /// Get the number of blocks in this blueprint
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Calculate required items to place this blueprint
    pub fn required_items(&self) -> HashMap<ItemId, u32> {
        let mut items = HashMap::new();
        for block in &self.blocks {
            *items.entry(block.item_id).or_insert(0) += 1;
        }
        items
    }
}

/// Blueprint library resource - stores all saved blueprints
#[derive(Resource, Debug, Default)]
pub struct BlueprintLibrary {
    /// Stored blueprints
    pub blueprints: Vec<Blueprint>,
}

impl BlueprintLibrary {
    /// Create a new empty library
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a blueprint to the library
    pub fn add(&mut self, blueprint: Blueprint) {
        self.blueprints.push(blueprint);
    }

    /// Find a blueprint by name
    pub fn find_by_name(&self, name: &str) -> Option<&Blueprint> {
        self.blueprints.iter().find(|b| b.name == name)
    }

    /// Get the number of blueprints in the library
    pub fn count(&self) -> usize {
        self.blueprints.len()
    }

    /// Remove a blueprint by index
    pub fn remove(&mut self, index: usize) -> Option<Blueprint> {
        if index < self.blueprints.len() {
            Some(self.blueprints.remove(index))
        } else {
            None
        }
    }
}

/// Blueprint placement preview state
#[derive(Resource, Debug, Default)]
pub struct BlueprintPreview {
    /// Currently selected blueprint index
    pub selected: Option<usize>,
    /// Placement position
    pub position: IVec3,
    /// Rotation (0-3: 0, 90, 180, 270 degrees)
    pub rotation: u8,
    /// Whether the preview is active
    pub active: bool,
}

impl BlueprintPreview {
    /// Rotate the preview 90 degrees clockwise
    pub fn rotate_cw(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    /// Rotate the preview 90 degrees counter-clockwise
    pub fn rotate_ccw(&mut self) {
        self.rotation = (self.rotation + 3) % 4;
    }

    /// Clear the preview selection
    pub fn clear(&mut self) {
        self.selected = None;
        self.active = false;
    }
}

/// Blueprint plugin
pub struct BlueprintPlugin;

impl Plugin for BlueprintPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueprintLibrary>()
            .init_resource::<BlueprintPreview>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

    #[test]
    fn test_blueprint_new() {
        let bp = Blueprint::new("Test Factory");
        assert_eq!(bp.name, "Test Factory");
        assert_eq!(bp.blocks.len(), 0);
        assert_eq!(bp.size, IVec3::ZERO);
    }

    #[test]
    fn test_blueprint_add_block() {
        let mut bp = Blueprint::new("Test");
        bp.add_block(BlueprintBlock {
            offset: IVec3::new(2, 0, 3),
            item_id: items::miner_block(),
            rotation: 0,
            direction: Some(BlueprintDirection::North),
        });

        assert_eq!(bp.block_count(), 1);
        assert_eq!(bp.size, IVec3::new(3, 1, 4));
    }

    #[test]
    fn test_blueprint_size_calculation() {
        let mut bp = Blueprint::new("Test");
        bp.add_block(BlueprintBlock {
            offset: IVec3::new(0, 0, 0),
            item_id: items::conveyor_block(),
            rotation: 0,
            direction: None,
        });
        bp.add_block(BlueprintBlock {
            offset: IVec3::new(5, 2, 3),
            item_id: items::conveyor_block(),
            rotation: 0,
            direction: None,
        });

        assert_eq!(bp.size, IVec3::new(6, 3, 4));
    }

    #[test]
    fn test_required_items() {
        let mut bp = Blueprint::new("Test");
        bp.add_block(BlueprintBlock {
            offset: IVec3::ZERO,
            item_id: items::conveyor_block(),
            rotation: 0,
            direction: None,
        });
        bp.add_block(BlueprintBlock {
            offset: IVec3::new(1, 0, 0),
            item_id: items::conveyor_block(),
            rotation: 0,
            direction: None,
        });
        bp.add_block(BlueprintBlock {
            offset: IVec3::new(2, 0, 0),
            item_id: items::miner_block(),
            rotation: 0,
            direction: Some(BlueprintDirection::North),
        });

        let items_map = bp.required_items();
        assert_eq!(items_map.get(&items::conveyor_block()), Some(&2));
        assert_eq!(items_map.get(&items::miner_block()), Some(&1));
    }

    #[test]
    fn test_library() {
        let mut lib = BlueprintLibrary::new();
        lib.add(Blueprint::new("Factory 1"));
        lib.add(Blueprint::new("Factory 2"));

        assert_eq!(lib.count(), 2);
        assert!(lib.find_by_name("Factory 1").is_some());
        assert!(lib.find_by_name("Unknown").is_none());
    }

    #[test]
    fn test_library_remove() {
        let mut lib = BlueprintLibrary::new();
        lib.add(Blueprint::new("Factory 1"));
        lib.add(Blueprint::new("Factory 2"));

        let removed = lib.remove(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "Factory 1");
        assert_eq!(lib.count(), 1);
        assert!(lib.find_by_name("Factory 1").is_none());
        assert!(lib.find_by_name("Factory 2").is_some());
    }

    #[test]
    fn test_preview_rotation() {
        let mut preview = BlueprintPreview::default();
        assert_eq!(preview.rotation, 0);

        preview.rotate_cw();
        assert_eq!(preview.rotation, 1);

        preview.rotate_cw();
        preview.rotate_cw();
        preview.rotate_cw();
        assert_eq!(preview.rotation, 0); // Wraps around

        preview.rotate_ccw();
        assert_eq!(preview.rotation, 3);
    }

    #[test]
    fn test_direction_conversion() {
        // Test Direction -> BlueprintDirection
        assert_eq!(
            BlueprintDirection::from(Direction::North),
            BlueprintDirection::North
        );
        assert_eq!(
            BlueprintDirection::from(Direction::South),
            BlueprintDirection::South
        );
        assert_eq!(
            BlueprintDirection::from(Direction::East),
            BlueprintDirection::East
        );
        assert_eq!(
            BlueprintDirection::from(Direction::West),
            BlueprintDirection::West
        );

        // Test BlueprintDirection -> Direction
        assert_eq!(Direction::from(BlueprintDirection::North), Direction::North);
        assert_eq!(Direction::from(BlueprintDirection::South), Direction::South);
        assert_eq!(Direction::from(BlueprintDirection::East), Direction::East);
        assert_eq!(Direction::from(BlueprintDirection::West), Direction::West);
    }

    #[test]
    fn test_blueprint_serialization() {
        let mut bp = Blueprint::new("Test Serialization");
        bp.description = Some("A test blueprint".to_string());
        bp.created_at = Some(1234567890.0);
        bp.add_block(BlueprintBlock {
            offset: IVec3::new(1, 2, 3),
            item_id: items::furnace_block(),
            rotation: 2,
            direction: Some(BlueprintDirection::East),
        });

        // Serialize to JSON
        let json = serde_json::to_string(&bp).expect("Failed to serialize");

        // Deserialize back
        let bp2: Blueprint = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(bp2.name, "Test Serialization");
        assert_eq!(bp2.description, Some("A test blueprint".to_string()));
        assert_eq!(bp2.created_at, Some(1234567890.0));
        assert_eq!(bp2.blocks.len(), 1);
        assert_eq!(bp2.blocks[0].offset, IVec3::new(1, 2, 3));
        assert_eq!(bp2.blocks[0].item_id, items::furnace_block());
        assert_eq!(bp2.blocks[0].rotation, 2);
        assert_eq!(bp2.blocks[0].direction, Some(BlueprintDirection::East));
    }
}
