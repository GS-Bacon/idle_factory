//! Logistics infrastructure (conveyors, pipes, power networks)
//!
//! This module contains logistics-related systems that are separate from
//! machine processing. Conveyors are treated as infrastructure rather than
//! machines because they have unique requirements:
//! - Continuous item movement (not discrete processing)
//! - Zipper merge at junctions
//! - Corner and splitter shape handling
//! - Round-robin output distribution
//!
//! Resource networks (power, fluid, signal) use a segment-based approach:
//! - Connected nodes form a segment
//! - Resources propagate instantly within a segment
//! - Mod extensible via TOML/WebSocket API

pub mod conveyor;
pub mod network;

pub use conveyor::*;
pub use network::*;
