//! Network components for multiplayer support
//!
//! This module provides types for entity identification across network boundaries.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network-unique identifier for entities
///
/// Used to reference entities across network boundaries in multiplayer.
/// Each entity that needs network synchronization should have a NetworkId component.
#[derive(Component, Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct NetworkId(pub u64);

impl NetworkId {
    /// Create a new NetworkId from a raw value
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw u64 value
    pub fn get(&self) -> u64 {
        self.0
    }
}

/// Bidirectional mapping between NetworkId and Entity
///
/// This resource maintains the relationship between network IDs and local entities,
/// enabling efficient lookup in both directions.
#[derive(Resource, Default)]
pub struct EntityMap {
    network_to_entity: HashMap<NetworkId, Entity>,
    entity_to_network: HashMap<Entity, NetworkId>,
}

impl EntityMap {
    /// Create a new empty EntityMap
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the Entity for a given NetworkId
    pub fn get_entity(&self, network_id: NetworkId) -> Option<Entity> {
        self.network_to_entity.get(&network_id).copied()
    }

    /// Get the NetworkId for a given Entity
    pub fn get_network_id(&self, entity: Entity) -> Option<NetworkId> {
        self.entity_to_network.get(&entity).copied()
    }

    /// Register a bidirectional mapping between Entity and NetworkId
    ///
    /// If the entity or network_id was already registered, the old mapping is removed.
    pub fn register(&mut self, entity: Entity, network_id: NetworkId) {
        // Remove any existing mappings
        if let Some(old_network_id) = self.entity_to_network.remove(&entity) {
            self.network_to_entity.remove(&old_network_id);
        }
        if let Some(old_entity) = self.network_to_entity.remove(&network_id) {
            self.entity_to_network.remove(&old_entity);
        }

        // Insert new mapping
        self.network_to_entity.insert(network_id, entity);
        self.entity_to_network.insert(entity, network_id);
    }

    /// Unregister an entity and its associated NetworkId
    pub fn unregister(&mut self, entity: Entity) {
        if let Some(network_id) = self.entity_to_network.remove(&entity) {
            self.network_to_entity.remove(&network_id);
        }
    }

    /// Unregister by NetworkId
    pub fn unregister_by_network_id(&mut self, network_id: NetworkId) {
        if let Some(entity) = self.network_to_entity.remove(&network_id) {
            self.entity_to_network.remove(&entity);
        }
    }

    /// Check if an entity is registered
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entity_to_network.contains_key(&entity)
    }

    /// Check if a network ID is registered
    pub fn contains_network_id(&self, network_id: NetworkId) -> bool {
        self.network_to_entity.contains_key(&network_id)
    }

    /// Get the number of registered mappings
    pub fn len(&self) -> usize {
        self.entity_to_network.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.entity_to_network.is_empty()
    }

    /// Clear all mappings
    pub fn clear(&mut self) {
        self.network_to_entity.clear();
        self.entity_to_network.clear();
    }
}

/// Generator for unique NetworkIds
///
/// This resource generates monotonically increasing NetworkIds.
/// In a multiplayer context, the server would typically own this resource.
#[derive(Resource, Default)]
pub struct NetworkIdGenerator {
    next_id: u64,
}

impl NetworkIdGenerator {
    /// Create a new generator starting from 0
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new generator starting from a specific value
    ///
    /// Useful for resuming from saved state or for client-side ID ranges.
    pub fn starting_from(start: u64) -> Self {
        Self { next_id: start }
    }

    /// Generate the next unique NetworkId
    pub fn generate(&mut self) -> NetworkId {
        let id = NetworkId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Peek at the next ID without consuming it
    pub fn peek(&self) -> NetworkId {
        NetworkId(self.next_id)
    }

    /// Get the current counter value
    pub fn current_value(&self) -> u64 {
        self.next_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_id_creation() {
        let id = NetworkId::new(42);
        assert_eq!(id.get(), 42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_network_id_equality() {
        let id1 = NetworkId(100);
        let id2 = NetworkId(100);
        let id3 = NetworkId(200);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_network_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(NetworkId(1));
        set.insert(NetworkId(2));
        set.insert(NetworkId(1)); // duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&NetworkId(1)));
        assert!(set.contains(&NetworkId(2)));
    }

    #[test]
    fn test_network_id_generator() {
        let mut gen = NetworkIdGenerator::new();

        assert_eq!(gen.peek(), NetworkId(0));
        assert_eq!(gen.generate(), NetworkId(0));
        assert_eq!(gen.generate(), NetworkId(1));
        assert_eq!(gen.generate(), NetworkId(2));
        assert_eq!(gen.current_value(), 3);
    }

    #[test]
    fn test_network_id_generator_starting_from() {
        let mut gen = NetworkIdGenerator::starting_from(1000);

        assert_eq!(gen.generate(), NetworkId(1000));
        assert_eq!(gen.generate(), NetworkId(1001));
    }

    #[test]
    fn test_entity_map_register_and_get() {
        let mut map = EntityMap::new();
        let entity = Entity::from_raw(42);
        let network_id = NetworkId(100);

        map.register(entity, network_id);

        assert_eq!(map.get_entity(network_id), Some(entity));
        assert_eq!(map.get_network_id(entity), Some(network_id));
    }

    #[test]
    fn test_entity_map_unregister() {
        let mut map = EntityMap::new();
        let entity = Entity::from_raw(42);
        let network_id = NetworkId(100);

        map.register(entity, network_id);
        assert!(map.contains_entity(entity));
        assert!(map.contains_network_id(network_id));

        map.unregister(entity);
        assert!(!map.contains_entity(entity));
        assert!(!map.contains_network_id(network_id));
        assert_eq!(map.get_entity(network_id), None);
        assert_eq!(map.get_network_id(entity), None);
    }

    #[test]
    fn test_entity_map_unregister_by_network_id() {
        let mut map = EntityMap::new();
        let entity = Entity::from_raw(42);
        let network_id = NetworkId(100);

        map.register(entity, network_id);
        map.unregister_by_network_id(network_id);

        assert!(!map.contains_entity(entity));
        assert!(!map.contains_network_id(network_id));
    }

    #[test]
    fn test_entity_map_reregister_entity() {
        let mut map = EntityMap::new();
        let entity = Entity::from_raw(42);
        let network_id1 = NetworkId(100);
        let network_id2 = NetworkId(200);

        map.register(entity, network_id1);
        map.register(entity, network_id2);

        // Old mapping should be removed
        assert!(!map.contains_network_id(network_id1));
        // New mapping should exist
        assert_eq!(map.get_network_id(entity), Some(network_id2));
        assert_eq!(map.get_entity(network_id2), Some(entity));
    }

    #[test]
    fn test_entity_map_reregister_network_id() {
        let mut map = EntityMap::new();
        let entity1 = Entity::from_raw(42);
        let entity2 = Entity::from_raw(43);
        let network_id = NetworkId(100);

        map.register(entity1, network_id);
        map.register(entity2, network_id);

        // Old mapping should be removed
        assert!(!map.contains_entity(entity1));
        // New mapping should exist
        assert_eq!(map.get_entity(network_id), Some(entity2));
        assert_eq!(map.get_network_id(entity2), Some(network_id));
    }

    #[test]
    fn test_entity_map_len_and_empty() {
        let mut map = EntityMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        map.register(Entity::from_raw(1), NetworkId(1));
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);

        map.register(Entity::from_raw(2), NetworkId(2));
        assert_eq!(map.len(), 2);

        map.unregister(Entity::from_raw(1));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_entity_map_clear() {
        let mut map = EntityMap::new();
        map.register(Entity::from_raw(1), NetworkId(1));
        map.register(Entity::from_raw(2), NetworkId(2));

        map.clear();

        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_network_id_serialization() {
        let id = NetworkId(12345);
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: NetworkId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}
