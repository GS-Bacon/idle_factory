//! Network segment component
//!
//! A segment represents a connected group of network nodes that share resources.

use super::types::{NetworkTypeId, SegmentId};
use bevy::prelude::*;
use std::collections::HashMap;

/// Network segment component
///
/// Represents a connected group of nodes sharing the same network type.
/// Resources propagate instantly within a segment.
#[derive(Component, Clone, Debug)]
pub struct NetworkSegment {
    /// Unique segment identifier
    pub id: SegmentId,
    /// Network type (power, fluid, signal, etc.)
    pub network_type: NetworkTypeId,

    // === Power/General fields ===
    /// Total supply (production) per tick
    pub supply: f32,
    /// Total demand (consumption) per tick
    pub demand: f32,
    /// Current satisfaction ratio (0.0 - 1.0)
    pub satisfaction: f32,

    // === Fluid/Gas fields ===
    /// Total capacity (for storage networks)
    pub capacity: f32,
    /// Current amount stored
    pub amount: f32,

    // === Signal fields ===
    /// Signal strength (0-15 for discrete signals)
    pub signal_strength: u8,

    // === Node tracking ===
    /// All entities in this segment
    pub nodes: Vec<Entity>,
    /// Position to entity mapping (for fast lookup)
    pub node_positions: HashMap<IVec3, Entity>,
}

impl NetworkSegment {
    /// Create a new segment
    pub fn new(id: SegmentId, network_type: NetworkTypeId) -> Self {
        Self {
            id,
            network_type,
            supply: 0.0,
            demand: 0.0,
            satisfaction: 1.0,
            capacity: 0.0,
            amount: 0.0,
            signal_strength: 0,
            nodes: Vec::new(),
            node_positions: HashMap::new(),
        }
    }

    /// Add a node to this segment
    pub fn add_node(&mut self, entity: Entity, position: IVec3) {
        if !self.nodes.contains(&entity) {
            self.nodes.push(entity);
            self.node_positions.insert(position, entity);
        }
    }

    /// Remove a node from this segment
    pub fn remove_node(&mut self, entity: Entity, position: IVec3) {
        self.nodes.retain(|&e| e != entity);
        self.node_positions.remove(&position);
    }

    /// Check if this segment contains a node at the given position
    pub fn contains_position(&self, position: IVec3) -> bool {
        self.node_positions.contains_key(&position)
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if segment is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get fill ratio for fluid/gas segments
    pub fn fill_ratio(&self) -> f32 {
        if self.capacity > 0.0 {
            (self.amount / self.capacity).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Reset statistics for recalculation
    pub fn reset_stats(&mut self) {
        self.supply = 0.0;
        self.demand = 0.0;
        self.capacity = 0.0;
    }
}

impl Default for NetworkSegment {
    fn default() -> Self {
        Self {
            id: SegmentId::new(0),
            network_type: NetworkTypeId::new(0),
            supply: 0.0,
            demand: 0.0,
            satisfaction: 1.0,
            capacity: 0.0,
            amount: 0.0,
            signal_strength: 0,
            nodes: Vec::new(),
            node_positions: HashMap::new(),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_add_remove_node() {
        let mut segment = NetworkSegment::new(SegmentId::new(1), NetworkTypeId::new(1));

        let entity = Entity::from_raw(42);
        let pos = IVec3::new(10, 5, 20);

        segment.add_node(entity, pos);
        assert_eq!(segment.node_count(), 1);
        assert!(segment.contains_position(pos));

        segment.remove_node(entity, pos);
        assert_eq!(segment.node_count(), 0);
        assert!(!segment.contains_position(pos));
    }

    #[test]
    fn test_segment_fill_ratio() {
        let mut segment = NetworkSegment::default();

        // Empty capacity
        assert_eq!(segment.fill_ratio(), 0.0);

        // Partial fill
        segment.capacity = 1000.0;
        segment.amount = 500.0;
        assert!((segment.fill_ratio() - 0.5).abs() < 0.001);

        // Overfill clamped
        segment.amount = 1500.0;
        assert_eq!(segment.fill_ratio(), 1.0);
    }
}
