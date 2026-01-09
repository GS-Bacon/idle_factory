//! Virtual link system
//!
//! Allows Mods to create wireless connections between network nodes.

use super::types::NetworkTypeId;
use crate::core::id::Id;
use bevy::prelude::*;
use std::collections::HashMap;

// =============================================================================
// Virtual Link ID
// =============================================================================

/// Marker type for VirtualLinkId
#[derive(Clone, Copy)]
pub struct VirtualLinkCategory;

/// Virtual link identifier
pub type VirtualLinkId = Id<VirtualLinkCategory>;

// =============================================================================
// Virtual Link
// =============================================================================

/// Virtual link connecting two positions wirelessly
///
/// Used by Mods to implement wireless power transmission, teleportation pipes, etc.
#[derive(Clone, Debug)]
pub struct VirtualLink {
    /// Unique link identifier
    pub id: VirtualLinkId,
    /// Source position
    pub from_pos: IVec3,
    /// Destination position
    pub to_pos: IVec3,
    /// Network type this link connects
    pub network_type: NetworkTypeId,
    /// Whether the link is bidirectional
    pub bidirectional: bool,
    /// Optional: Maximum distance for the link to work
    pub max_distance: Option<f32>,
    /// Optional: Efficiency loss over distance (0.0 - 1.0)
    pub efficiency: f32,
}

impl VirtualLink {
    /// Create a new virtual link
    pub fn new(
        id: VirtualLinkId,
        from_pos: IVec3,
        to_pos: IVec3,
        network_type: NetworkTypeId,
        bidirectional: bool,
    ) -> Self {
        Self {
            id,
            from_pos,
            to_pos,
            network_type,
            bidirectional,
            max_distance: None,
            efficiency: 1.0,
        }
    }

    /// Create with efficiency loss
    pub fn with_efficiency(mut self, efficiency: f32) -> Self {
        self.efficiency = efficiency.clamp(0.0, 1.0);
        self
    }

    /// Create with max distance
    pub fn with_max_distance(mut self, distance: f32) -> Self {
        self.max_distance = Some(distance);
        self
    }

    /// Check if the link is within range
    pub fn is_in_range(&self) -> bool {
        if let Some(max_dist) = self.max_distance {
            let dist = (self.from_pos.as_vec3() - self.to_pos.as_vec3()).length();
            dist <= max_dist
        } else {
            true // No distance limit
        }
    }

    /// Get the actual distance
    pub fn distance(&self) -> f32 {
        (self.from_pos.as_vec3() - self.to_pos.as_vec3()).length()
    }
}

// =============================================================================
// Virtual Link Registry
// =============================================================================

/// Registry for virtual links
///
/// Manages all virtual connections in the game.
#[derive(Resource, Default)]
pub struct VirtualLinkRegistry {
    /// All links by ID
    links: HashMap<u32, VirtualLink>,
    /// Links by source position (for fast lookup during flood fill)
    by_source: HashMap<IVec3, Vec<VirtualLinkId>>,
    /// Links by destination position (for bidirectional lookup)
    by_destination: HashMap<IVec3, Vec<VirtualLinkId>>,
    /// Next link ID
    next_id: u32,
}

impl VirtualLinkRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            links: HashMap::new(),
            by_source: HashMap::new(),
            by_destination: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a virtual link
    pub fn add_link(
        &mut self,
        from_pos: IVec3,
        to_pos: IVec3,
        network_type: NetworkTypeId,
        bidirectional: bool,
    ) -> VirtualLinkId {
        let id = VirtualLinkId::new(self.next_id);
        self.next_id += 1;

        let link = VirtualLink::new(id, from_pos, to_pos, network_type, bidirectional);
        self.links.insert(id.raw(), link);

        // Index by source
        self.by_source.entry(from_pos).or_default().push(id);

        // Index by destination (for bidirectional)
        if bidirectional {
            self.by_destination.entry(to_pos).or_default().push(id);
        }

        id
    }

    /// Remove a virtual link
    pub fn remove_link(&mut self, id: VirtualLinkId) -> Option<VirtualLink> {
        if let Some(link) = self.links.remove(&id.raw()) {
            // Remove from source index
            if let Some(list) = self.by_source.get_mut(&link.from_pos) {
                list.retain(|&lid| lid != id);
            }

            // Remove from destination index
            if link.bidirectional {
                if let Some(list) = self.by_destination.get_mut(&link.to_pos) {
                    list.retain(|&lid| lid != id);
                }
            }

            Some(link)
        } else {
            None
        }
    }

    /// Get a link by ID
    pub fn get(&self, id: VirtualLinkId) -> Option<&VirtualLink> {
        self.links.get(&id.raw())
    }

    /// Get all links from a position
    pub fn get_links_from(&self, pos: IVec3) -> Vec<&VirtualLink> {
        let mut result = Vec::new();

        // Direct links from this position
        if let Some(ids) = self.by_source.get(&pos) {
            for id in ids {
                if let Some(link) = self.links.get(&id.raw()) {
                    if link.is_in_range() {
                        result.push(link);
                    }
                }
            }
        }

        // Bidirectional links to this position
        if let Some(ids) = self.by_destination.get(&pos) {
            for id in ids {
                if let Some(link) = self.links.get(&id.raw()) {
                    if link.bidirectional && link.is_in_range() {
                        result.push(link);
                    }
                }
            }
        }

        result
    }

    /// Get connected positions from a position (for flood fill)
    pub fn get_connected_positions(&self, pos: IVec3, network_type: NetworkTypeId) -> Vec<IVec3> {
        self.get_links_from(pos)
            .into_iter()
            .filter(|link| link.network_type == network_type)
            .map(|link| {
                if link.from_pos == pos {
                    link.to_pos
                } else {
                    link.from_pos
                }
            })
            .collect()
    }

    /// Get all links
    pub fn iter(&self) -> impl Iterator<Item = &VirtualLink> {
        self.links.values()
    }

    /// Get link count
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_link_distance() {
        let link = VirtualLink::new(
            VirtualLinkId::new(1),
            IVec3::new(0, 0, 0),
            IVec3::new(10, 0, 0),
            NetworkTypeId::new(1),
            false,
        );

        assert!((link.distance() - 10.0).abs() < 0.001);
        assert!(link.is_in_range()); // No max distance

        let link_limited = link.with_max_distance(5.0);
        assert!(!link_limited.is_in_range()); // Beyond max distance
    }

    #[test]
    fn test_virtual_link_registry() {
        let mut registry = VirtualLinkRegistry::new();
        let network_type = NetworkTypeId::new(1);

        let pos_a = IVec3::new(0, 0, 0);
        let pos_b = IVec3::new(100, 0, 0);

        let id = registry.add_link(pos_a, pos_b, network_type, true);

        // Check link exists
        assert!(registry.get(id).is_some());
        assert_eq!(registry.len(), 1);

        // Check connected positions
        let connected = registry.get_connected_positions(pos_a, network_type);
        assert_eq!(connected.len(), 1);
        assert_eq!(connected[0], pos_b);

        // Bidirectional - check from other side
        let connected_back = registry.get_connected_positions(pos_b, network_type);
        assert_eq!(connected_back.len(), 1);
        assert_eq!(connected_back[0], pos_a);

        // Remove link
        registry.remove_link(id);
        assert!(registry.get(id).is_none());
        assert_eq!(registry.len(), 0);
    }
}
