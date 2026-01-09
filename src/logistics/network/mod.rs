//! Resource network infrastructure (power, fluid, gas, signal)
//!
//! This module implements a Factorio Fluids 2.0-style segment-based network system
//! that supports electricity, liquids, gases, and signals with full Mod extensibility.
//!
//! # Design
//!
//! - **Segment-based**: Connected nodes form a segment with instant propagation
//! - **No flow simulation**: Resources equalize instantly within a segment
//! - **Mod extensible**: Custom network types via TOML/WebSocket API
//!
//! # Network Types
//!
//! | Type | Value | Storage | Distribution |
//! |------|-------|---------|--------------|
//! | Power | f32 (W) | Battery | Priority-based |
//! | Fluid | f32 (mB) | Tank | Fill-level equalization |
//! | Gas | f32 (mB) | Tank | Fill-level equalization |
//! | Signal | u8 (0-15) | None | Max strength propagation |

pub mod conduit;
pub mod detector;
pub mod distribution;
pub mod node;
pub mod registry;
pub mod segment;
pub mod types;
pub mod virtual_link;

pub use conduit::*;
pub use detector::*;
pub use distribution::*;
pub use node::*;
pub use registry::*;
pub use segment::*;
pub use types::*;
pub use virtual_link::*;

use bevy::prelude::*;

/// Network infrastructure plugin
///
/// Registers all network-related resources, events, and systems.
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<SegmentRegistry>()
            .init_resource::<VirtualLinkRegistry>()
            .init_resource::<NetworkTypeRegistry>()
            // Events
            .add_event::<SegmentFormed>()
            .add_event::<SegmentBroken>()
            .add_event::<PowerShortage>()
            .add_event::<VirtualLinkAdded>()
            .add_event::<VirtualLinkRemoved>()
            .add_event::<NetworkBlockPlaced>()
            .add_event::<NetworkBlockRemoved>()
            // Systems (FixedUpdate for deterministic simulation)
            .add_systems(
                FixedUpdate,
                (
                    detect_segments,
                    distribute_power,
                    distribute_fluid,
                    propagate_signal,
                )
                    .chain(),
            );
    }
}

// =============================================================================
// Events
// =============================================================================

/// Fired when a new segment is formed
#[derive(Event, Debug)]
pub struct SegmentFormed {
    pub segment_id: SegmentId,
    pub network_type: NetworkTypeId,
}

/// Fired when a segment is broken (split or destroyed)
#[derive(Event, Debug)]
pub struct SegmentBroken {
    pub segment_id: SegmentId,
    /// New segments created from the split (empty if destroyed)
    pub new_segments: Vec<SegmentId>,
}

/// Fired when power demand exceeds supply
#[derive(Event, Debug)]
pub struct PowerShortage {
    pub segment_id: SegmentId,
    pub supply: f32,
    pub demand: f32,
}

/// Fired when a virtual link is added
#[derive(Event, Debug)]
pub struct VirtualLinkAdded {
    pub link_id: VirtualLinkId,
}

/// Fired when a virtual link is removed
#[derive(Event, Debug)]
pub struct VirtualLinkRemoved {
    pub link_id: VirtualLinkId,
}
