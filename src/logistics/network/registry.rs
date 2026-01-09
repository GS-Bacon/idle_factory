//! Segment registry
//!
//! Manages all network segments in the game.

use super::segment::NetworkSegment;
use super::types::{NetworkTypeId, SegmentId};
use crate::core::id::StringInterner;
use bevy::prelude::*;
use std::collections::HashMap;

/// Segment registry resource
///
/// Tracks all network segments and provides operations for segment management.
#[derive(Resource, Default)]
pub struct SegmentRegistry {
    /// All segments by ID
    segments: HashMap<u32, NetworkSegment>,
    /// Next segment ID to allocate
    next_id: u32,
    /// String interner for segment IDs (for debugging)
    interner: StringInterner,
}

impl SegmentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            segments: HashMap::new(),
            next_id: 1,
            interner: StringInterner::new(),
        }
    }

    /// Create a new segment
    pub fn create_segment(&mut self, network_type: NetworkTypeId) -> SegmentId {
        let id = SegmentId::new(self.next_id);
        self.next_id += 1;

        let segment = NetworkSegment::new(id, network_type);
        self.segments.insert(id.raw(), segment);

        id
    }

    /// Get a segment by ID
    pub fn get(&self, id: SegmentId) -> Option<&NetworkSegment> {
        self.segments.get(&id.raw())
    }

    /// Get a mutable segment by ID
    pub fn get_mut(&mut self, id: SegmentId) -> Option<&mut NetworkSegment> {
        self.segments.get_mut(&id.raw())
    }

    /// Remove a segment
    pub fn remove(&mut self, id: SegmentId) -> Option<NetworkSegment> {
        self.segments.remove(&id.raw())
    }

    /// Merge two segments into one
    ///
    /// Moves all nodes from segment B into segment A, then removes B.
    /// Returns the merged segment ID (A).
    pub fn merge_segments(&mut self, a: SegmentId, b: SegmentId) -> SegmentId {
        if a == b {
            return a;
        }

        // Get segment B's nodes
        let b_nodes: Vec<(Entity, IVec3)> = if let Some(seg_b) = self.segments.get(&b.raw()) {
            seg_b
                .node_positions
                .iter()
                .map(|(&pos, &entity)| (entity, pos))
                .collect()
        } else {
            return a;
        };

        // Move nodes to segment A
        if let Some(seg_a) = self.segments.get_mut(&a.raw()) {
            for (entity, pos) in b_nodes {
                seg_a.add_node(entity, pos);
            }
        }

        // Remove segment B
        self.segments.remove(&b.raw());

        a
    }

    /// Split a segment (called when a node is removed and connectivity is broken)
    ///
    /// Takes a list of node groups that should become separate segments.
    /// Returns the new segment IDs.
    pub fn split_segment(
        &mut self,
        original_id: SegmentId,
        groups: Vec<Vec<(Entity, IVec3)>>,
    ) -> Vec<SegmentId> {
        let network_type = match self.segments.get(&original_id.raw()) {
            Some(seg) => seg.network_type,
            None => return Vec::new(),
        };

        // Remove the original segment
        self.segments.remove(&original_id.raw());

        // Create new segments for each group
        let mut new_ids = Vec::new();
        for group in groups {
            if group.is_empty() {
                continue;
            }

            let new_id = self.create_segment(network_type);
            if let Some(seg) = self.segments.get_mut(&new_id.raw()) {
                for (entity, pos) in group {
                    seg.add_node(entity, pos);
                }
            }
            new_ids.push(new_id);
        }

        new_ids
    }

    /// List all segments
    pub fn iter(&self) -> impl Iterator<Item = &NetworkSegment> {
        self.segments.values()
    }

    /// List all segments (mutable)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut NetworkSegment> {
        self.segments.values_mut()
    }

    /// Get segments of a specific network type
    pub fn segments_by_type(&self, network_type: NetworkTypeId) -> Vec<&NetworkSegment> {
        self.segments
            .values()
            .filter(|s| s.network_type == network_type)
            .collect()
    }

    /// Get segment count
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the string interner
    pub fn interner(&self) -> &StringInterner {
        &self.interner
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_segment() {
        let mut registry = SegmentRegistry::new();
        let network_type = NetworkTypeId::new(1);

        let id1 = registry.create_segment(network_type);
        let id2 = registry.create_segment(network_type);

        assert_ne!(id1, id2);
        assert_eq!(registry.len(), 2);

        let seg1 = registry.get(id1).unwrap();
        assert_eq!(seg1.network_type, network_type);
    }

    #[test]
    fn test_merge_segments() {
        let mut registry = SegmentRegistry::new();
        let network_type = NetworkTypeId::new(1);

        let id_a = registry.create_segment(network_type);
        let id_b = registry.create_segment(network_type);

        // Add nodes to each segment
        let entity_a = Entity::from_raw(1);
        let entity_b = Entity::from_raw(2);
        let pos_a = IVec3::new(0, 0, 0);
        let pos_b = IVec3::new(1, 0, 0);

        registry.get_mut(id_a).unwrap().add_node(entity_a, pos_a);
        registry.get_mut(id_b).unwrap().add_node(entity_b, pos_b);

        // Merge
        let merged_id = registry.merge_segments(id_a, id_b);

        assert_eq!(merged_id, id_a);
        assert_eq!(registry.len(), 1);

        let merged = registry.get(merged_id).unwrap();
        assert_eq!(merged.node_count(), 2);
        assert!(merged.contains_position(pos_a));
        assert!(merged.contains_position(pos_b));
    }

    #[test]
    fn test_split_segment() {
        let mut registry = SegmentRegistry::new();
        let network_type = NetworkTypeId::new(1);

        let original_id = registry.create_segment(network_type);

        // Add some nodes
        let entities: Vec<Entity> = (0..4).map(|i| Entity::from_raw(i)).collect();
        let positions: Vec<IVec3> = (0..4).map(|i| IVec3::new(i, 0, 0)).collect();

        {
            let seg = registry.get_mut(original_id).unwrap();
            for (entity, pos) in entities.iter().zip(positions.iter()) {
                seg.add_node(*entity, *pos);
            }
        }

        // Split into two groups
        let groups = vec![
            vec![(entities[0], positions[0]), (entities[1], positions[1])],
            vec![(entities[2], positions[2]), (entities[3], positions[3])],
        ];

        let new_ids = registry.split_segment(original_id, groups);

        assert_eq!(new_ids.len(), 2);
        assert_eq!(registry.len(), 2);

        // Check each new segment has 2 nodes
        for id in new_ids {
            let seg = registry.get(id).unwrap();
            assert_eq!(seg.node_count(), 2);
        }
    }
}
