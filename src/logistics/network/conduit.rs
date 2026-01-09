//! Conduit components
//!
//! Components for network conduits (wires, pipes, signal cables).

use super::types::{NetworkTypeId, SegmentId};
use bevy::prelude::*;

// =============================================================================
// Wire (Power Conduit)
// =============================================================================

/// Wire component for power transmission
///
/// Wires connect power producers and consumers.
#[derive(Component, Clone, Debug)]
pub struct Wire {
    /// Position in the world
    pub position: IVec3,
    /// Connected segment (None if not connected)
    pub segment_id: Option<SegmentId>,
    /// Wire tier (affects capacity/efficiency in future)
    pub tier: u8,
}

impl Wire {
    /// Create a new wire at the given position
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            segment_id: None,
            tier: 1,
        }
    }

    /// Create a wire with a specific tier
    pub fn with_tier(position: IVec3, tier: u8) -> Self {
        Self {
            position,
            segment_id: None,
            tier,
        }
    }
}

// =============================================================================
// Pipe (Fluid/Gas Conduit)
// =============================================================================

/// Pipe component for fluid/gas transmission
///
/// Pipes connect fluid producers, consumers, and storage.
#[derive(Component, Clone, Debug)]
pub struct Pipe {
    /// Position in the world
    pub position: IVec3,
    /// Connected segment (None if not connected)
    pub segment_id: Option<SegmentId>,
    /// Network type (fluid or gas)
    pub network_type: NetworkTypeId,
    /// Pipe tier (affects flow rate in future)
    pub tier: u8,
    /// Internal capacity (mB)
    pub capacity: f32,
    /// Current amount in pipe
    pub amount: f32,
}

impl Pipe {
    /// Create a new pipe at the given position
    pub fn new(position: IVec3, network_type: NetworkTypeId) -> Self {
        Self {
            position,
            segment_id: None,
            network_type,
            tier: 1,
            capacity: 100.0, // 100mB default capacity per pipe segment
            amount: 0.0,
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
}

// =============================================================================
// Signal Wire (Signal Conduit)
// =============================================================================

/// Signal wire component for signal transmission
///
/// Signal wires propagate redstone-like signals.
#[derive(Component, Clone, Debug)]
pub struct SignalWire {
    /// Position in the world
    pub position: IVec3,
    /// Connected segment (None if not connected)
    pub segment_id: Option<SegmentId>,
    /// Current signal strength at this wire (0-15)
    pub strength: u8,
    /// Whether this wire has decay (Minecraft redstone style)
    pub has_decay: bool,
}

impl SignalWire {
    /// Create a new signal wire at the given position
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            segment_id: None,
            strength: 0,
            has_decay: false,
        }
    }

    /// Create a decay wire (signal strength decreases with distance)
    pub fn with_decay(position: IVec3) -> Self {
        Self {
            position,
            segment_id: None,
            strength: 0,
            has_decay: true,
        }
    }

    /// Check if signal is on
    pub fn is_on(&self) -> bool {
        self.strength > 0
    }
}

// =============================================================================
// Generic Conduit Component
// =============================================================================

/// Generic conduit marker for any network type
///
/// Used for Mod-defined network types that don't fit wire/pipe/signal.
#[derive(Component, Clone, Debug)]
pub struct GenericConduit {
    /// Position in the world
    pub position: IVec3,
    /// Network type
    pub network_type: NetworkTypeId,
    /// Connected segment
    pub segment_id: Option<SegmentId>,
    /// Conduit group (for compatibility checking)
    pub conduit_group: Option<String>,
}

impl GenericConduit {
    /// Create a new generic conduit
    pub fn new(position: IVec3, network_type: NetworkTypeId) -> Self {
        Self {
            position,
            network_type,
            segment_id: None,
            conduit_group: None,
        }
    }

    /// Create with a conduit group
    pub fn with_group(position: IVec3, network_type: NetworkTypeId, group: String) -> Self {
        Self {
            position,
            network_type,
            segment_id: None,
            conduit_group: Some(group),
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
    fn test_wire() {
        let wire = Wire::new(IVec3::new(10, 5, 20));
        assert_eq!(wire.position, IVec3::new(10, 5, 20));
        assert!(wire.segment_id.is_none());
        assert_eq!(wire.tier, 1);
    }

    #[test]
    fn test_pipe_fill_ratio() {
        let mut pipe = Pipe::new(IVec3::ZERO, NetworkTypeId::new(1));
        assert_eq!(pipe.fill_ratio(), 0.0);

        pipe.amount = 50.0;
        assert!((pipe.fill_ratio() - 0.5).abs() < 0.001);

        pipe.amount = 100.0;
        assert_eq!(pipe.fill_ratio(), 1.0);
    }

    #[test]
    fn test_signal_wire() {
        let mut wire = SignalWire::new(IVec3::ZERO);
        assert!(!wire.is_on());

        wire.strength = 15;
        assert!(wire.is_on());

        let decay_wire = SignalWire::with_decay(IVec3::ZERO);
        assert!(decay_wire.has_decay);
    }
}
