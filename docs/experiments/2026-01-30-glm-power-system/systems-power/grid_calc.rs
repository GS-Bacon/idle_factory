//! Grid calculation system using Union-Find
//!
//! This module detects connected power networks and calculates power balance.

use bevy::prelude::*;

use crate::components::power::{
    PowerConsumer, PowerGrid, PowerGridChangeType, PowerGrids, PowerProducer, PowerWire,
};
use crate::components::{Block, Machine};
use crate::events::game_events::PowerGridChanged;
use crate::events::GuardedMessageWriter;
use std::collections::HashMap;

/// Cached state for optimization
#[derive(Resource, Default, Clone)]
pub struct PowerGridCache {
    pub last_entity_hash: u64,
    pub last_calculation_time: f64,
}

/// Union-Find data structure for grid calculation
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    /// Create a new Union-Find structure
    pub fn new(size: usize) -> Self {
        let parent = (0..size).collect();
        let rank = vec![0; size];
        Self { parent, rank }
    }

    /// Find with path compression
    pub fn find(&mut self, mut x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    /// Union by rank
    pub fn union(&mut self, x: usize, y: usize) {
        let x_root = self.find(x);
        let y_root = self.find(y);

        if x_root == y_root {
            return;
        }

        if self.rank[x_root] < self.rank[y_root] {
            self.parent[x_root] = y_root;
        } else if self.rank[x_root] > self.rank[y_root] {
            self.parent[y_root] = x_root;
        } else {
            self.parent[y_root] = x_root;
            self.rank[x_root] += 1;
        }
    }
}

/// Calculate power grids for all connected components
pub fn calculate_power_grids(
    mut power_grids: ResMut<PowerGrids>,
    mut power_producers: Query<(Entity, &PowerProducer)>,
    mut power_consumers: Query<(Entity, &mut PowerConsumer)>,
    mut power_wires: Query<(Entity, &mut PowerWire)>,
    blocks: Query<&Block>,
    machines: Query<&Machine>,
    mut grid_cache: ResMut<PowerGridCache>,
    mut grid_changed_writer: GuardedMessageWriter<PowerGridChanged>,
    time: Res<Time>,
) {
    // Optimization: Skip if no power entities exist
    let producer_count = power_producers.iter().count();
    let consumer_count = power_consumers.iter().count();
    let wire_count = power_wires.iter().count();

    if producer_count == 0 && consumer_count == 0 && wire_count == 0 {
        return;
    }

    // Calculate entity hash for change detection
    let current_time = time.elapsed_seconds();
    let mut entity_hash: u64 = 0;

    for (entity, _) in &power_producers {
        entity_hash = entity_hash.wrapping_add(entity.index() as u64);
    }
    for (entity, _) in &power_consumers {
        entity_hash = entity_hash.wrapping_add(entity.index() as u64);
    }
    for (entity, _) in &power_wires {
        entity_hash = entity_hash.wrapping_add(entity.index() as u64);
    }

    // Skip recalculation if no changes detected and not too old
    if entity_hash == grid_cache.last_entity_hash
        && (current_time - grid_cache.last_calculation_time) < 0.1
    {
        return;
    }

    // Update cache
    grid_cache.last_entity_hash = entity_hash;
    grid_cache.last_calculation_time = current_time;

    // Track previous grid count for split/merge detection
    let previous_grid_count = power_grids.grids.len();

    // Collect entities with positions
    let mut entities: Vec<(Entity, IVec3)> = Vec::new();
    for (entity, _) in &power_producers {
        if let Ok(machine) = machines.get(entity) {
            entities.push((*entity, machine.grid_position));
        }
    }
    for (entity, _) in &power_consumers {
        if let Ok(machine) = machines.get(entity) {
            entities.push((*entity, machine.grid_position));
        }
    }
    for (entity, wire) in &power_wires {
        if let Ok(block) = blocks.get(entity) {
            entities.push((*entity, block.position));
        }
    }

    if entities.is_empty() {
        return;
    }

    // Build position to entity mapping
    let position_map: HashMap<IVec3, Entity> = entities.iter().map(|(e, pos)| (*pos, *e)).collect();

    // Assign indices for Union-Find
    let entity_to_index: HashMap<Entity, usize> = entities
        .iter()
        .enumerate()
        .map(|(i, (e, _))| (*e, i))
        .collect();

    if entity_to_index.is_empty() {
        return;
    }

    let mut uf = UnionFind::new(entities.len());

    // Connect entities through wires
    // Wires connect adjacent power entities (generators, consumers, other wires)
    let adjacency_offsets = [
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 0, 0),
        IVec3::new(0, 1, 0),
        IVec3::new(0, -1, 0),
        IVec3::new(0, 0, 1),
        IVec3::new(0, 0, -1),
    ];

    for (wire_entity, wire_pos) in &power_wires {
        let Some(&wire_idx) = entity_to_index.get(wire_entity) else {
            continue;
        };

        // Find adjacent power entities
        for offset in &adjacency_offsets {
            let adjacent_pos = *wire_pos + offset;
            if let Some(&adjacent_entity) = position_map.get(&adjacent_pos) {
                if let Some(&adjacent_idx) = entity_to_index.get(&adjacent_entity) {
                    uf.union(wire_idx, adjacent_idx);
                }
            }
        }
    }
    let entities: Vec<Entity> = power_producers
        .iter()
        .map(|(e, _)| *e)
        .chain(power_consumers.iter().map(|(e, _)| *e))
        .chain(power_wires.iter().map(|(e, _)| *e))
        .collect();

    // Assign indices for Union-Find
    let entity_to_index: HashMap<Entity, usize> =
        entities.iter().enumerate().map(|(i, e)| (*e, i)).collect();

    if entity_to_index.is_empty() {
        return;
    }

    let mut uf = UnionFind::new(entities.len());

    // Connect entities through wires
    for (entity, wire) in &power_wires {
        if let (Some(&idx1), Some(&idx2)) = (
            entity_to_index.get(entity),
            entity_to_index.get(&Entity::from_raw(wire.grid_id as u32)),
        ) {
            uf.union(*idx1, *idx2);
        }
    }

    // Build grid map
    let mut grid_map: HashMap<usize, Vec<Entity>> = HashMap::new();
    for (entity, index) in &entity_to_index {
        let root = uf.find(*index);
        grid_map.entry(root).or_default().push(*entity);
    }

    // Calculate power for each grid
    let mut new_grids: HashMap<u64, PowerGrid> = HashMap::new();
    let mut grid_id = power_grids.next_id;

    // Detect grid splits and merges
    let current_grid_count = grid_map.len();
    let grid_split_detected = current_grid_count > previous_grid_count;
    let grid_merge_detected = current_grid_count < previous_grid_count;

    for (grid_index, entities) in grid_map {
        let mut total_generation: u32 = 0;
        let mut total_consumption: u32 = 0;
        let mut producers: Vec<Entity> = Vec::new();
        let mut consumers: Vec<Entity> = Vec::new();
        let mut wires: Vec<Entity> = Vec::new();

        for entity in entities {
            if power_producers.get(entity).is_some() {
                producers.push(entity);
                total_generation += power_producers.get(entity).unwrap().output_watts;
            }
            if power_consumers.get(entity).is_some() {
                consumers.push(entity);
                total_consumption += power_consumers.get(entity).unwrap().required_power as u32;
            }
            if power_wires.get(entity).is_some() {
                wires.push(entity);
            }
        }

        let grid = PowerGrid {
            id: grid_id,
            total_generation,
            total_consumption,
            producers,
            consumers,
            wires,
        };

        // Update consumer power states
        let has_power = grid.has_power();
        for consumer in &consumers {
            if let Ok(mut consumer_comps) = power_consumers.get_mut(*consumer) {
                consumer_comps.current_power = if has_power {
                    consumer_comps.required_power
                } else {
                    0.0
                };
            }
        }

        // Assign grid IDs to wires
        for wire in &wires {
            if let Ok(mut wire_comps) = power_wires.get_mut(*wire) {
                wire_comps.grid_id = grid.id;
            }
        }

        // Emit change event
        let change_type = if grid.consumers.is_empty() && grid.producers.is_empty() {
            PowerGridChangeType::GridCreated
        } else if grid_split_detected {
            let new_grid_ids: Vec<u64> = new_grids.keys().copied().collect();
            PowerGridChangeType::GridSplit { new_grid_ids }
        } else if grid_merge_detected && grid_id > 0 {
            PowerGridChangeType::GridMerged {
                merged_into_id: grid_id - 1,
            }
        } else {
            PowerGridChangeType::GridCreated
        };

        grid_changed_writer.send(PowerGridChanged {
            grid_id,
            change_type,
        });

        new_grids.insert(grid_id, grid);
        grid_id += 1;
    }

    power_grids.grids = new_grids;
    power_grids.next_id = grid_id;
}
