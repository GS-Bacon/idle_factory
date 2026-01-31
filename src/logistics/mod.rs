//! Logistics infrastructure (conveyors, inserters, pipes)
//!
//! This module contains logistics-related systems that are separate from
//! machine processing. Conveyors are treated as infrastructure rather than
//! machines because they have unique requirements:
//! - Continuous item movement (not discrete processing)
//! - Zipper merge at junctions
//! - Corner and splitter shape handling
//! - Round-robin output distribution

pub mod conveyor;
pub mod network_utils;

pub use conveyor::*;
pub use network_utils::*;
