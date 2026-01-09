//! Network type definitions
//!
//! Core types for the resource network system.

use crate::core::id::{Id, StringInterner};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Category Markers (for Id<T> phantom type)
// =============================================================================

/// Marker type for NetworkTypeId
#[derive(Clone, Copy)]
pub struct NetworkTypeCategory;

/// Marker type for SegmentId
#[derive(Clone, Copy)]
pub struct SegmentCategory;

// =============================================================================
// Type Aliases
// =============================================================================

/// Network type identifier (e.g., "base:power", "base:fluid")
pub type NetworkTypeId = Id<NetworkTypeCategory>;

/// Segment identifier (unique per segment)
pub type SegmentId = Id<SegmentCategory>;

// =============================================================================
// Network Type Specification
// =============================================================================

/// Network value type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum NetworkValueType {
    /// Floating point value (power in W, fluid in mB)
    Float,
    /// Discrete value (signal strength 0-15)
    Discrete,
}

/// Propagation type within a segment
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PropagationType {
    /// Instant propagation (power, signal)
    Instant,
    /// Segment-wide equalization (fluid)
    Segment,
    /// Distance-based decay (Mod use: heat, etc.)
    Distance,
}

/// Network type specification
///
/// Defines the behavior of a network type (power, fluid, signal, etc.)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkTypeSpec {
    /// Network type ID (e.g., "base:power")
    pub id: String,
    /// Display name
    pub name: String,
    /// Whether this network type supports storage (batteries, tanks)
    pub has_storage: bool,
    /// Value type (Float or Discrete)
    pub value_type: NetworkValueType,
    /// Propagation method
    pub propagation: PropagationType,
    /// Conduit compatibility group (e.g., "pipe" for fluid/gas sharing)
    #[serde(default)]
    pub conduit_group: Option<String>,
}

// =============================================================================
// Node Role
// =============================================================================

/// Role of a network node
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum NodeRole {
    /// Produces resources (generator, pump)
    Producer,
    /// Consumes resources (machine)
    Consumer,
    /// Stores resources (battery, tank)
    Storage,
    /// Transports resources (wire, pipe)
    Conduit,
}

// =============================================================================
// Network Type Registry
// =============================================================================

/// Registry for network types
///
/// Stores all registered network types (base + mods).
#[derive(Resource, Default)]
pub struct NetworkTypeRegistry {
    /// Registered network types by ID
    types: HashMap<u32, NetworkTypeSpec>,
    /// String interner for network type IDs
    interner: StringInterner,
    /// Pre-resolved base type IDs for fast access
    base_power: Option<NetworkTypeId>,
    base_fluid: Option<NetworkTypeId>,
    base_gas: Option<NetworkTypeId>,
    base_signal: Option<NetworkTypeId>,
}

impl NetworkTypeRegistry {
    /// Create a new registry with base types
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_base_types();
        registry
    }

    /// Register base network types
    fn register_base_types(&mut self) {
        // Power
        let power_id = self.register(NetworkTypeSpec {
            id: "base:power".to_string(),
            name: "電力".to_string(),
            has_storage: true,
            value_type: NetworkValueType::Float,
            propagation: PropagationType::Instant,
            conduit_group: None,
        });
        self.base_power = Some(power_id);

        // Fluid
        let fluid_id = self.register(NetworkTypeSpec {
            id: "base:fluid".to_string(),
            name: "液体".to_string(),
            has_storage: true,
            value_type: NetworkValueType::Float,
            propagation: PropagationType::Segment,
            conduit_group: Some("pipe".to_string()),
        });
        self.base_fluid = Some(fluid_id);

        // Gas
        let gas_id = self.register(NetworkTypeSpec {
            id: "base:gas".to_string(),
            name: "気体".to_string(),
            has_storage: true,
            value_type: NetworkValueType::Float,
            propagation: PropagationType::Segment,
            conduit_group: Some("pipe".to_string()),
        });
        self.base_gas = Some(gas_id);

        // Signal
        let signal_id = self.register(NetworkTypeSpec {
            id: "base:signal".to_string(),
            name: "信号".to_string(),
            has_storage: false,
            value_type: NetworkValueType::Discrete,
            propagation: PropagationType::Instant,
            conduit_group: None,
        });
        self.base_signal = Some(signal_id);
    }

    /// Register a network type
    pub fn register(&mut self, spec: NetworkTypeSpec) -> NetworkTypeId {
        let id = NetworkTypeId::from_string(&spec.id, &mut self.interner);
        self.types.insert(id.raw(), spec);
        id
    }

    /// Get a network type specification by ID
    pub fn get(&self, id: NetworkTypeId) -> Option<&NetworkTypeSpec> {
        self.types.get(&id.raw())
    }

    /// Get the base power network type ID
    pub fn power(&self) -> NetworkTypeId {
        self.base_power.expect("Base types not initialized")
    }

    /// Get the base fluid network type ID
    pub fn fluid(&self) -> NetworkTypeId {
        self.base_fluid.expect("Base types not initialized")
    }

    /// Get the base gas network type ID
    pub fn gas(&self) -> NetworkTypeId {
        self.base_gas.expect("Base types not initialized")
    }

    /// Get the base signal network type ID
    pub fn signal(&self) -> NetworkTypeId {
        self.base_signal.expect("Base types not initialized")
    }

    /// Get the string interner (for resolving IDs to strings)
    pub fn interner(&self) -> &StringInterner {
        &self.interner
    }

    /// Get mutable string interner
    pub fn interner_mut(&mut self) -> &mut StringInterner {
        &mut self.interner
    }

    /// List all registered network types
    pub fn list(&self) -> impl Iterator<Item = (&u32, &NetworkTypeSpec)> {
        self.types.iter()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_type_registry_base_types() {
        let registry = NetworkTypeRegistry::new();

        // Check base types are registered
        let power = registry.get(registry.power()).unwrap();
        assert_eq!(power.id, "base:power");
        assert_eq!(power.value_type, NetworkValueType::Float);

        let fluid = registry.get(registry.fluid()).unwrap();
        assert_eq!(fluid.id, "base:fluid");
        assert_eq!(fluid.conduit_group, Some("pipe".to_string()));

        let signal = registry.get(registry.signal()).unwrap();
        assert_eq!(signal.id, "base:signal");
        assert_eq!(signal.value_type, NetworkValueType::Discrete);
    }

    #[test]
    fn test_network_type_registry_custom() {
        let mut registry = NetworkTypeRegistry::new();

        // Register a custom network type
        let mana_id = registry.register(NetworkTypeSpec {
            id: "magic:mana".to_string(),
            name: "魔力".to_string(),
            has_storage: true,
            value_type: NetworkValueType::Float,
            propagation: PropagationType::Distance,
            conduit_group: Some("mana_crystal".to_string()),
        });

        let mana = registry.get(mana_id).unwrap();
        assert_eq!(mana.id, "magic:mana");
        assert_eq!(mana.propagation, PropagationType::Distance);
    }
}
