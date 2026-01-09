//! Network node components
//!
//! Components that attach to entities to make them part of a network.

use super::types::{NetworkTypeId, NodeRole, SegmentId};
use crate::core::FluidId;
use crate::game_spec::machines::PortSide;
use bevy::prelude::*;

// =============================================================================
// Network Port (Generic)
// =============================================================================

/// Network port component
///
/// Attaches to an entity to make it part of a network.
/// An entity can have multiple ports (e.g., a machine with power input and fluid output).
#[derive(Component, Clone, Debug)]
pub struct NetworkPort {
    /// Which side of the block this port is on
    pub side: PortSide,
    /// Network type (power, fluid, signal, etc.)
    pub network_type: NetworkTypeId,
    /// Role in the network
    pub role: NodeRole,
    /// Which segment this port belongs to (None if not connected)
    pub segment_id: Option<SegmentId>,
}

impl NetworkPort {
    /// Create a new network port
    pub fn new(side: PortSide, network_type: NetworkTypeId, role: NodeRole) -> Self {
        Self {
            side,
            network_type,
            role,
            segment_id: None,
        }
    }

    /// Check if this port is connected to a segment
    pub fn is_connected(&self) -> bool {
        self.segment_id.is_some()
    }
}

// =============================================================================
// Power Node
// =============================================================================

/// Power node component
///
/// Adds power-specific data to a network port.
#[derive(Component, Clone, Debug)]
pub struct PowerNode {
    /// Power in watts (positive for producers, represents demand for consumers)
    pub power_watts: f32,
    /// Current satisfaction ratio (0.0 - 1.0)
    /// For producers: always 1.0
    /// For consumers: how much of demand is met
    pub satisfaction: f32,
    /// Priority for power distribution (-10 to 10, higher = served first)
    pub priority: i8,
}

impl PowerNode {
    /// Create a power producer (generator)
    pub fn producer(power_watts: f32) -> Self {
        Self {
            power_watts,
            satisfaction: 1.0,
            priority: 0,
        }
    }

    /// Create a power consumer (machine)
    pub fn consumer(power_watts: f32) -> Self {
        Self {
            power_watts,
            satisfaction: 0.0,
            priority: 0,
        }
    }

    /// Create a power consumer with priority
    pub fn consumer_with_priority(power_watts: f32, priority: i8) -> Self {
        Self {
            power_watts,
            satisfaction: 0.0,
            priority,
        }
    }

    /// Check if this node has sufficient power
    pub fn has_power(&self) -> bool {
        self.satisfaction >= 0.1
    }

    /// Get effective power multiplier based on satisfaction
    pub fn power_multiplier(&self) -> f32 {
        self.satisfaction
    }
}

impl Default for PowerNode {
    fn default() -> Self {
        Self::consumer(0.0)
    }
}

// =============================================================================
// Fluid Node
// =============================================================================

/// Fluid node component
///
/// Adds fluid-specific data to a network port.
#[derive(Component, Clone, Debug)]
pub struct FluidNode {
    /// Type of fluid (None if empty/any)
    pub fluid_type: Option<FluidId>,
    /// Storage capacity in mB (millibuckets)
    pub capacity: f32,
    /// Current amount stored
    pub amount: f32,
    /// Flow rate in mB/tick
    pub flow_rate: f32,
}

impl FluidNode {
    /// Create a fluid storage (tank)
    pub fn storage(capacity: f32) -> Self {
        Self {
            fluid_type: None,
            capacity,
            amount: 0.0,
            flow_rate: 1000.0, // Default flow rate
        }
    }

    /// Create a fluid producer (pump)
    pub fn producer(flow_rate: f32) -> Self {
        Self {
            fluid_type: None,
            capacity: 0.0,
            amount: 0.0,
            flow_rate,
        }
    }

    /// Create a fluid consumer (machine)
    pub fn consumer(flow_rate: f32) -> Self {
        Self {
            fluid_type: None,
            capacity: 0.0,
            amount: 0.0,
            flow_rate,
        }
    }

    /// Get fill ratio
    pub fn fill_ratio(&self) -> f32 {
        if self.capacity > 0.0 {
            (self.amount / self.capacity).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.amount <= 0.0
    }

    /// Check if full
    pub fn is_full(&self) -> bool {
        self.capacity > 0.0 && self.amount >= self.capacity
    }
}

impl Default for FluidNode {
    fn default() -> Self {
        Self::storage(1000.0)
    }
}

// =============================================================================
// Signal Node
// =============================================================================

/// Signal node component
///
/// Adds signal-specific data to a network port.
#[derive(Component, Clone, Debug)]
pub struct SignalNode {
    /// Signal strength (0-15)
    pub strength: u8,
    /// Decay per block (0 for no decay, 1 for Minecraft-style)
    pub decay_per_block: u8,
}

impl SignalNode {
    /// Create a signal source (lever, button)
    pub fn source(strength: u8) -> Self {
        Self {
            strength: strength.min(15),
            decay_per_block: 0,
        }
    }

    /// Create a signal receiver (lamp, piston)
    pub fn receiver() -> Self {
        Self {
            strength: 0,
            decay_per_block: 0,
        }
    }

    /// Create a signal conduit (wire)
    pub fn conduit() -> Self {
        Self {
            strength: 0,
            decay_per_block: 0,
        }
    }

    /// Create a decay wire (Minecraft-style redstone)
    pub fn decay_wire(decay_rate: u8) -> Self {
        Self {
            strength: 0,
            decay_per_block: decay_rate,
        }
    }

    /// Check if signal is on
    pub fn is_on(&self) -> bool {
        self.strength > 0
    }

    /// Get analog strength (0.0 - 1.0)
    pub fn analog_strength(&self) -> f32 {
        self.strength as f32 / 15.0
    }
}

impl Default for SignalNode {
    fn default() -> Self {
        Self::receiver()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_node() {
        let producer = PowerNode::producer(100.0);
        assert_eq!(producer.satisfaction, 1.0);
        assert!(producer.has_power());

        let mut consumer = PowerNode::consumer(50.0);
        assert_eq!(consumer.satisfaction, 0.0);
        assert!(!consumer.has_power());

        consumer.satisfaction = 0.5;
        assert!(consumer.has_power());
        assert_eq!(consumer.power_multiplier(), 0.5);
    }

    #[test]
    fn test_fluid_node() {
        let mut storage = FluidNode::storage(1000.0);
        assert!(storage.is_empty());
        assert!(!storage.is_full());

        storage.amount = 500.0;
        assert!((storage.fill_ratio() - 0.5).abs() < 0.001);

        storage.amount = 1000.0;
        assert!(storage.is_full());
    }

    #[test]
    fn test_signal_node() {
        let source = SignalNode::source(15);
        assert!(source.is_on());
        assert_eq!(source.analog_strength(), 1.0);

        let receiver = SignalNode::receiver();
        assert!(!receiver.is_on());
        assert_eq!(receiver.analog_strength(), 0.0);
    }
}
