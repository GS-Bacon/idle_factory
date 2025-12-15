use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::gameplay::grid::{SimulationGrid, Direction};
use crate::gameplay::building::MachinePlacedEvent;

// ============================================================================
// Multiblock Pattern Definition
// ============================================================================

/// Defines a pattern for a multiblock structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MultiblockPattern {
    /// Unique identifier for this pattern
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Size of the pattern (x, y, z)
    pub size: [i32; 3],
    /// Block requirements at each position relative to master
    /// Key: "x,y,z" offset string, Value: required block ID
    pub blocks: HashMap<String, String>,
    /// Position of the master block (controller) relative to pattern origin
    pub master_offset: [i32; 3],
}

impl MultiblockPattern {
    /// Get required block at offset
    pub fn get_required_block(&self, offset: IVec3) -> Option<&String> {
        let key = format!("{},{},{}", offset.x, offset.y, offset.z);
        self.blocks.get(&key)
    }

    /// Get all offsets in this pattern
    pub fn get_all_offsets(&self) -> Vec<IVec3> {
        self.blocks.keys()
            .filter_map(|key| {
                let parts: Vec<i32> = key.split(',')
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() == 3 {
                    Some(IVec3::new(parts[0], parts[1], parts[2]))
                } else {
                    None
                }
            })
            .collect()
    }
}

// ============================================================================
// Multiblock Registry
// ============================================================================

/// Registry for all multiblock patterns
#[derive(Resource, Default)]
pub struct MultiblockRegistry {
    pub patterns: HashMap<String, MultiblockPattern>,
}

// ============================================================================
// Structure Validator
// ============================================================================

/// Result of structure validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub pattern_id: Option<String>,
    pub master_pos: Option<IVec3>,
    pub missing_blocks: Vec<(IVec3, String)>,
    pub slave_positions: Vec<IVec3>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            is_valid: false,
            pattern_id: None,
            master_pos: None,
            missing_blocks: Vec::new(),
            slave_positions: Vec::new(),
        }
    }
}

/// Validates multiblock structures in the world
pub struct StructureValidator;

impl StructureValidator {
    /// Validate a structure starting from a potential master position
    pub fn validate_at(
        pos: IVec3,
        pattern: &MultiblockPattern,
        grid: &SimulationGrid,
        _direction: Direction,
    ) -> ValidationResult {
        let mut result = ValidationResult::default();
        result.master_pos = Some(pos);
        result.pattern_id = Some(pattern.id.clone());

        let master_offset = IVec3::new(
            pattern.master_offset[0],
            pattern.master_offset[1],
            pattern.master_offset[2],
        );

        // Calculate pattern origin from master position
        let origin = pos - master_offset;

        let mut all_valid = true;
        let mut slave_positions = Vec::new();

        for offset in pattern.get_all_offsets() {
            let world_pos = origin + offset;
            let required_block = pattern.get_required_block(offset);

            if let Some(required) = required_block {
                let has_block = grid.machines.get(&world_pos)
                    .map(|m| &m.id == required)
                    .unwrap_or(false);

                if !has_block {
                    all_valid = false;
                    result.missing_blocks.push((world_pos, required.clone()));
                } else if world_pos != pos {
                    slave_positions.push(world_pos);
                }
            }
        }

        result.is_valid = all_valid;
        result.slave_positions = slave_positions;
        result
    }

    /// Try to find a valid pattern at the given position
    pub fn find_valid_pattern(
        pos: IVec3,
        registry: &MultiblockRegistry,
        grid: &SimulationGrid,
    ) -> Option<ValidationResult> {
        for pattern in registry.patterns.values() {
            let result = Self::validate_at(pos, pattern, grid, Direction::North);
            if result.is_valid {
                return Some(result);
            }
        }
        None
    }
}

// ============================================================================
// Master/Slave Components
// ============================================================================

/// Component marking a block as the master (controller) of a multiblock structure
#[derive(Component, Debug, Clone)]
pub struct MultiblockMaster {
    pub pattern_id: String,
    pub slave_positions: Vec<IVec3>,
    pub is_formed: bool,
}

/// Component marking a block as a slave (part) of a multiblock structure
#[derive(Component, Debug, Clone)]
pub struct MultiblockSlave {
    pub master_pos: IVec3,
}

/// Resource tracking all formed multiblock structures
#[derive(Resource, Default)]
pub struct FormedMultiblocks {
    /// Map from master position to structure info
    pub structures: HashMap<IVec3, MultiblockInfo>,
}

#[derive(Debug, Clone)]
pub struct MultiblockInfo {
    pub pattern_id: String,
    pub master_pos: IVec3,
    pub slave_positions: Vec<IVec3>,
    pub formed_at: f64,
}

// ============================================================================
// Events
// ============================================================================

/// Event fired when a multiblock structure is formed
#[derive(Event)]
pub struct MultiblockFormedEvent {
    pub master_pos: IVec3,
    pub pattern_id: String,
    pub slave_positions: Vec<IVec3>,
}

/// Event fired when a multiblock structure is broken
#[derive(Event)]
pub struct MultiblockBrokenEvent {
    pub master_pos: IVec3,
    pub pattern_id: String,
}

/// Event to request structure validation at a position
#[derive(Event)]
pub struct ValidateStructureEvent {
    pub pos: IVec3,
}

// ============================================================================
// Systems
// ============================================================================

/// System to check for multiblock formation when a machine is placed
pub fn check_multiblock_formation(
    mut placed_events: EventReader<MachinePlacedEvent>,
    mut validate_events: EventWriter<ValidateStructureEvent>,
) {
    for event in placed_events.read() {
        // Check this position and surrounding positions
        for dx in -2..=2 {
            for dy in -2..=2 {
                for dz in -2..=2 {
                    validate_events.send(ValidateStructureEvent {
                        pos: event.pos + IVec3::new(dx, dy, dz),
                    });
                }
            }
        }
    }
}

/// System to validate structures when requested
pub fn validate_structures(
    mut validate_events: EventReader<ValidateStructureEvent>,
    registry: Res<MultiblockRegistry>,
    grid: Res<SimulationGrid>,
    mut formed_multiblocks: ResMut<FormedMultiblocks>,
    mut formed_events: EventWriter<MultiblockFormedEvent>,
    time: Res<Time>,
) {
    let mut checked: HashSet<IVec3> = HashSet::new();

    for event in validate_events.read() {
        if checked.contains(&event.pos) { continue; }
        checked.insert(event.pos);

        // Skip if already formed at this position
        if formed_multiblocks.structures.contains_key(&event.pos) { continue; }

        if let Some(result) = StructureValidator::find_valid_pattern(
            event.pos,
            &registry,
            &grid,
        ) {
            if result.is_valid {
                let info = MultiblockInfo {
                    pattern_id: result.pattern_id.clone().unwrap(),
                    master_pos: event.pos,
                    slave_positions: result.slave_positions.clone(),
                    formed_at: time.elapsed_secs_f64(),
                };

                formed_multiblocks.structures.insert(event.pos, info);

                formed_events.send(MultiblockFormedEvent {
                    master_pos: event.pos,
                    pattern_id: result.pattern_id.unwrap(),
                    slave_positions: result.slave_positions,
                });

                info!("Multiblock formed at {:?}", event.pos);
            }
        }
    }
}

/// System to check if formed multiblocks are still valid
pub fn check_multiblock_integrity(
    registry: Res<MultiblockRegistry>,
    grid: Res<SimulationGrid>,
    mut formed_multiblocks: ResMut<FormedMultiblocks>,
    mut broken_events: EventWriter<MultiblockBrokenEvent>,
) {
    let mut to_remove = Vec::new();

    for (master_pos, info) in formed_multiblocks.structures.iter() {
        if let Some(pattern) = registry.patterns.get(&info.pattern_id) {
            let result = StructureValidator::validate_at(
                *master_pos,
                pattern,
                &grid,
                Direction::North,
            );

            if !result.is_valid {
                to_remove.push(*master_pos);
                broken_events.send(MultiblockBrokenEvent {
                    master_pos: *master_pos,
                    pattern_id: info.pattern_id.clone(),
                });
            }
        }
    }

    for pos in to_remove {
        formed_multiblocks.structures.remove(&pos);
        info!("Multiblock broken at {:?}", pos);
    }
}

// ============================================================================
// Plugin
// ============================================================================

pub struct MultiblockPlugin;

impl Plugin for MultiblockPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MultiblockRegistry>()
            .init_resource::<FormedMultiblocks>()
            .add_event::<MultiblockFormedEvent>()
            .add_event::<MultiblockBrokenEvent>()
            .add_event::<ValidateStructureEvent>()
            .add_systems(Startup, load_multiblock_patterns)
            .add_systems(Update, (
                check_multiblock_formation,
                validate_structures,
                check_multiblock_integrity,
            ));
    }
}

/// Load multiblock patterns from YAML files
fn load_multiblock_patterns(mut registry: ResMut<MultiblockRegistry>) {
    // Example: Register a simple 2x1x2 blast furnace pattern
    let blast_furnace = MultiblockPattern {
        id: "blast_furnace".to_string(),
        name: "Blast Furnace".to_string(),
        size: [2, 3, 2],
        blocks: {
            let mut blocks = HashMap::new();
            // Bottom layer (y=0)
            blocks.insert("0,0,0".to_string(), "furnace_brick".to_string());
            blocks.insert("1,0,0".to_string(), "furnace_brick".to_string());
            blocks.insert("0,0,1".to_string(), "furnace_brick".to_string());
            blocks.insert("1,0,1".to_string(), "furnace_brick".to_string());
            // Middle layer (y=1) - has controller
            blocks.insert("0,1,0".to_string(), "furnace_brick".to_string());
            blocks.insert("1,1,0".to_string(), "blast_furnace_controller".to_string());
            blocks.insert("0,1,1".to_string(), "furnace_brick".to_string());
            blocks.insert("1,1,1".to_string(), "furnace_brick".to_string());
            // Top layer (y=2)
            blocks.insert("0,2,0".to_string(), "furnace_brick".to_string());
            blocks.insert("1,2,0".to_string(), "furnace_brick".to_string());
            blocks.insert("0,2,1".to_string(), "furnace_brick".to_string());
            blocks.insert("1,2,1".to_string(), "furnace_brick".to_string());
            blocks
        },
        master_offset: [1, 1, 0], // Controller is at (1,1,0) relative to origin
    };

    registry.patterns.insert(blast_furnace.id.clone(), blast_furnace);

    info!("Loaded {} multiblock patterns", registry.patterns.len());
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::grid::MachineInstance;
    use crate::gameplay::machines::assembler::Assembler;

    fn create_test_pattern() -> MultiblockPattern {
        MultiblockPattern {
            id: "test_pattern".to_string(),
            name: "Test Pattern".to_string(),
            size: [2, 1, 1],
            blocks: {
                let mut blocks = HashMap::new();
                blocks.insert("0,0,0".to_string(), "block_a".to_string());
                blocks.insert("1,0,0".to_string(), "block_b".to_string());
                blocks
            },
            master_offset: [0, 0, 0],
        }
    }

    #[test]
    fn test_pattern_get_offsets() {
        let pattern = create_test_pattern();
        let offsets = pattern.get_all_offsets();
        assert_eq!(offsets.len(), 2);
        assert!(offsets.contains(&IVec3::new(0, 0, 0)));
        assert!(offsets.contains(&IVec3::new(1, 0, 0)));
    }

    #[test]
    fn test_structure_validator_invalid() {
        let pattern = create_test_pattern();
        let grid = SimulationGrid::default();

        let result = StructureValidator::validate_at(
            IVec3::ZERO,
            &pattern,
            &grid,
            Direction::North,
        );

        assert!(!result.is_valid);
        assert_eq!(result.missing_blocks.len(), 2);
    }

    #[test]
    fn test_structure_validator_valid() {
        let pattern = create_test_pattern();
        let mut grid = SimulationGrid::default();

        // Place required blocks
        grid.machines.insert(IVec3::new(0, 0, 0), MachineInstance {
            id: "block_a".to_string(),
            orientation: Direction::North,
            machine_type: crate::gameplay::grid::Machine::Assembler(Assembler::default()),
            power_node: None,
        });
        grid.machines.insert(IVec3::new(1, 0, 0), MachineInstance {
            id: "block_b".to_string(),
            orientation: Direction::North,
            machine_type: crate::gameplay::grid::Machine::Assembler(Assembler::default()),
            power_node: None,
        });

        let result = StructureValidator::validate_at(
            IVec3::ZERO,
            &pattern,
            &grid,
            Direction::North,
        );

        assert!(result.is_valid);
        assert!(result.missing_blocks.is_empty());
        assert_eq!(result.slave_positions.len(), 1);
        assert!(result.slave_positions.contains(&IVec3::new(1, 0, 0)));
    }

    #[test]
    fn test_find_valid_pattern() {
        let mut registry = MultiblockRegistry::default();
        registry.patterns.insert("test_pattern".to_string(), create_test_pattern());

        let mut grid = SimulationGrid::default();
        grid.machines.insert(IVec3::new(0, 0, 0), MachineInstance {
            id: "block_a".to_string(),
            orientation: Direction::North,
            machine_type: crate::gameplay::grid::Machine::Assembler(Assembler::default()),
            power_node: None,
        });
        grid.machines.insert(IVec3::new(1, 0, 0), MachineInstance {
            id: "block_b".to_string(),
            orientation: Direction::North,
            machine_type: crate::gameplay::grid::Machine::Assembler(Assembler::default()),
            power_node: None,
        });

        let result = StructureValidator::find_valid_pattern(IVec3::ZERO, &registry, &grid);
        assert!(result.is_some());
        assert!(result.unwrap().is_valid);
    }
}
